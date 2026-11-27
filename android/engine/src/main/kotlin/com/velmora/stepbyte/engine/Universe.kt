package com.velmora.stepbyte.engine

import android.content.Context
import android.os.Environment
import com.velmora.stepbyte.engine.ffi.EngineConfig
import com.velmora.stepbyte.engine.ffi.EngineException
import com.velmora.stepbyte.engine.ffi.EventSink
import com.velmora.stepbyte.engine.ffi.VelmoraEngine
import com.velmora.stepbyte.engine.ffi.engineVersion
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

/**
 * Velmora's Rust engine (UniverseAPI), one per process.
 *
 * Get it with [Universe.get]; the first call starts the engine and its
 * loopback stream server off the main thread. Every call is a suspend
 * function that runs in Rust; a cancelled coroutine stops waiting while the
 * engine finishes (and caches) what it started. Failures throw [EngineError].
 */
class Universe private constructor(private val ffi: VelmoraEngine) {

  /** `http://127.0.0.1:{port}`, the base of every playable URL. */
  val streamBase: String = ffi.streamBase()

  /** Whether TMDB keys were built in (otherwise metadata comes from Cinemeta). */
  val hasTmdb: Boolean
    get() = ffi.hasTmdb()

  /** Pull the remote config (mirrors, kill switches). The engine keeps its current one on failure. */
  suspend fun refreshConfig() = call { ffi.refreshConfig() }

  /** The home screen's rows: trending movies, series, anime... */
  suspend fun home(): List<Rail> = call {
    parse(ffi.home()).jsonArray.mapNotNull { el ->
      val o = el as? JsonObject ?: return@mapNotNull null
      Rail(
        id = o.string("id") ?: "",
        title = o.string("title") ?: "",
        items = titles(o["items"] as? JsonArray ?: JsonArray(emptyList())),
      )
    }
  }

  /** Merged results across providers. */
  suspend fun search(query: String): List<Title> = call { titles(parse(ffi.search(query)).jsonArray) }

  /** One batch per provider as each answers; completes when all have. */
  fun searchStream(query: String): Flow<SearchBatch> = callbackFlow {
    val sink =
      object : EventSink {
        override fun onEvent(json: String) {
          val o = parse(json).jsonObject
          trySend(
            SearchBatch(
              provider = o.string("provider") ?: "?",
              items = titles(o["items"]?.jsonArray ?: JsonArray(emptyList())),
              error = o.string("error"),
            ),
          )
        }
      }
    launch {
      try {
        call { ffi.searchStream(query, sink) }
        close()
      } catch (e: Throwable) {
        close(e)
      }
    }
    awaitClose()
  }

  /** Title page data, including seasons and episodes for series. */
  suspend fun detail(title: Title): TitleDetail = call {
    val o = parse(ffi.detail(title.json["origin"].toString())).jsonObject
    TitleDetail(title = toTitle(o["summary"]!!.jsonObject), info = json.decodeFromJsonElement(TitleDetailInfo.serializer(), o))
  }

  /** Ranked, probed sources for a movie or one episode. */
  suspend fun sources(title: Title, target: Target, dub: Boolean = false): SourcesReport = call {
    report(parse(ffi.sources(title.json.toString(), target.toJson().toString(), mode(dub))).jsonObject)
  }

  /** [sources], reporting each provider as it finishes and then [SourcesUpdate.Done]. */
  fun sourcesWatch(title: Title, target: Target, dub: Boolean = false): Flow<SourcesUpdate> =
    callbackFlow {
      val sink =
        object : EventSink {
          override fun onEvent(json: String) {
            val o = parse(json).jsonObject
            val update =
              when (o.string("kind")) {
                "plan" ->
                  SourcesUpdate.Plan(o["providers"]?.jsonArray?.mapNotNull { it.jsonPrimitive.contentOrNull } ?: emptyList())
                "batch" ->
                  SourcesUpdate.Batch(
                    provider = o.string("provider") ?: "?",
                    sources = sourceList(o["sources"]),
                    error = o.string("error"),
                    cached = o["cached"]?.jsonPrimitive?.booleanOrNull ?: false,
                  )
                else -> null
              }
            if (update != null) trySend(update)
          }
        }
      launch {
        try {
          val done =
            call {
              report(parse(ffi.sourcesWatch(title.json.toString(), target.toJson().toString(), mode(dub), sink)).jsonObject)
            }
          send(SourcesUpdate.Done(done))
          close()
        } catch (e: Throwable) {
          close(e)
        }
      }
      awaitClose()
    }

  /** A cached source didn't play: look them up afresh next time. */
  suspend fun forgetSources(title: Title, target: Target, dub: Boolean = false) = call {
    ffi.forgetSources(title.json.toString(), target.toJson().toString(), mode(dub))
  }

  /** Register [source] with the stream server; torrents join the session first. */
  suspend fun open(source: Source, target: Target = Target.Movie): OpenedStream = call {
    val ep = target as? Target.Episode
    json.decodeFromString(
      OpenedStream.serializer(),
      ffi.openStream(source.json.toString(), ep?.season?.toUInt(), ep?.episode?.toUInt()),
    )
  }

  /** Forget a stream; a streaming torrent is removed with its pieces. */
  suspend fun closeStream(stream: OpenedStream) = call { ffi.closeStream(stream.id) }

  /** Download progress of a streaming torrent, or `null` if it isn't one. */
  suspend fun torrentStatus(stream: OpenedStream): TorrentProgress? = call {
    ffi.torrentStatus(stream.id)?.let { json.decodeFromString(TorrentProgress.serializer(), it) }
  }

  /** Check the stream server, vault, torrent session and network on this device. */
  suspend fun selfTest(): SelfTest = call { json.decodeFromString(SelfTest.serializer(), ffi.selfTest()) }

  /** The engine's preferences as JSON (the desktop `Preferences` shape). */
  fun preferencesJson(): String = ffi.preferences()

  fun setPreferencesJson(prefs: String) = ffi.setPreferences(prefs)

  companion object {
    /** Lenient: the engine may add fields before the app models them. */
    val json = Json {
      ignoreUnknownKeys = true
      explicitNulls = false
    }

    private val lock = Mutex()
    @Volatile private var instance: Universe? = null

    /** The bridge's version and CPU, e.g. `0.6.6 (aarch64)`. Loads the library. */
    fun version(): String = engineVersion()

    /** The running engine, started on first use. */
    suspend fun get(context: Context): Universe =
      instance
        ?: lock.withLock {
          instance
            ?: withContext(Dispatchers.IO) { Universe(start(context.applicationContext)) }.also {
              instance = it
            }
        }

    private fun start(context: Context): VelmoraEngine {
      val cache = File(context.cacheDir, "engine")
      val downloads =
        context.getExternalFilesDir(Environment.DIRECTORY_MOVIES)?.let { File(it, "Velmora") }
          ?: File(context.filesDir, "downloads")
      // Not in auto-backup: a restored key on another phone would open nothing
      // and hide that the real one is gone. Key backup is its own feature.
      val key = File(context.noBackupFilesDir, "vault/key.bin")
      return mapErrors {
        VelmoraEngine(
          EngineConfig(
            cacheDir = cache.absolutePath,
            downloadDir = downloads.absolutePath,
            vaultKeyPath = key.absolutePath,
            preferencesJson = null,
          ),
        )
      }
    }

    private fun parse(text: String): JsonElement = json.parseToJsonElement(text)

    private fun mode(dub: Boolean) = if (dub) "dub" else "sub"

    private fun JsonObject.string(key: String): String? = this[key]?.jsonPrimitive?.contentOrNull

    private fun toTitle(o: JsonObject): Title = Wire(json.decodeFromJsonElement(TitleSummary.serializer(), o), o)

    private fun titles(array: JsonArray): List<Title> = array.mapNotNull { el ->
      runCatching { toTitle(el.jsonObject) }.getOrNull()
    }

    private fun sourceList(el: JsonElement?): List<Source> =
      (el as? JsonArray)?.mapNotNull { s ->
        runCatching { Wire(json.decodeFromJsonElement(SourceInfo.serializer(), s), s.jsonObject) }.getOrNull()
      } ?: emptyList()

    private fun report(o: JsonObject) =
      SourcesReport(
        sources = sourceList(o["sources"]),
        errors =
          (o["errors"] as? JsonArray)?.mapNotNull {
            runCatching { json.decodeFromJsonElement(ProviderFailure.serializer(), it) }.getOrNull()
          } ?: emptyList(),
      )

    private inline fun <T> mapErrors(block: () -> T): T =
      try {
        block()
      } catch (e: EngineException) {
        throw EngineError.from(e)
      }

    private suspend inline fun <T> call(crossinline block: suspend () -> T): T =
      try {
        block()
      } catch (e: EngineException) {
        throw EngineError.from(e)
      }
  }
}

/** A failed engine call, with the engine's own message. */
class EngineError(message: String, val badInput: Boolean, cause: Throwable? = null) :
  Exception(message, cause) {
  companion object {
    fun from(e: EngineException): EngineError =
      when (e) {
        is EngineException.BadInput -> EngineError("Bad input: ${e.reason}", badInput = true, cause = e)
        is EngineException.Failed -> EngineError(e.reason, badInput = false, cause = e)
      }
  }
}
