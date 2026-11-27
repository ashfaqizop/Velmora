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
    val recs = (o["extra"] as? JsonObject)?.get("recommendations") as? JsonArray
    TitleDetail(
      title = toTitle(o["summary"]!!.jsonObject),
      info = json.decodeFromJsonElement(TitleDetailInfo.serializer(), o),
      recommendations = recs?.let { titles(it) } ?: emptyList(),
    )
  }

  // ── Browse ──

  /**
   * Titles from a TMDB list endpoint: `/discover/movie`, `/trending/tv/week`,
   * `/movie/603/recommendations`. [kind] is `movie` or `tv` for lists whose
   * items don't say which they are.
   */
  suspend fun tmdbList(path: String, query: Map<String, String> = emptyMap(), kind: String = "movie", field: String = "results"): List<Title> =
    call {
      val q = JsonObject(query.mapValues { kotlinx.serialization.json.JsonPrimitive(it.value) })
      titles(parse(ffi.tmdbList(path, q.toString(), field, kind)).jsonArray)
    }

  /** A read-only TMDB call, as TMDB's own JSON (catalog paths only). */
  suspend fun tmdbRaw(path: String, query: Map<String, String> = emptyMap()): JsonObject = call {
    val q = JsonObject(query.mapValues { kotlinx.serialization.json.JsonPrimitive(it.value) })
    parse(ffi.tmdbRaw(path, q.toString())).jsonObject
  }

  /** Summaries for TMDB ids like `movie/603` or `tv/1396`. */
  suspend fun tmdbTitles(ids: List<String>): List<Title> = call {
    titles(parse(ffi.tmdbTitles(JsonArray(ids.map { kotlinx.serialization.json.JsonPrimitive(it) }).toString())).jsonArray)
  }

  /** Anime from AniList filters: search, genres, sort, year, season, status, format. */
  suspend fun anilistFind(filters: JsonObject): List<Title> = call { titles(parse(ffi.anilistFind(filters.toString())).jsonArray) }

  // ── Live TV ──

  suspend fun tvChannels(query: TvQuery): TvPage = call {
    json.decodeFromString(TvPage.serializer(), ffi.tvChannels(withDefaults.encodeToString(TvQuery.serializer(), query)))
  }

  suspend fun tvCategories(): List<Facet> = call { facets(ffi.tvCategories()) }

  suspend fun tvCountries(): List<Facet> = call { facets(ffi.tvCountries()) }

  // ── Sports ──

  suspend fun sportsLive(): List<Match> = call { matches(ffi.sportsLive()) }

  suspend fun sportsUpcoming(days: Int = 3): List<Match> = call { matches(ffi.sportsUpcoming(days.toUInt())) }

  suspend fun sportsResults(days: Int = 2): List<Match> = call { matches(ffi.sportsResults(days.toUInt())) }

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

    // TvQuery's limit must cross even at its default; the engine reads 0 as none.
    private val withDefaults = Json {
      encodeDefaults = true
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
      fun config(prefs: String?) =
        EngineConfig(
          cacheDir = cache.absolutePath,
          downloadDir = downloads.absolutePath,
          vaultKeyPath = key.absolutePath,
          preferencesJson = prefs,
        )
      val saved = savedPreferences(context)
      return try {
        mapErrors { VelmoraEngine(config(saved)) }
      } catch (e: EngineError) {
        // Saved preferences the engine can't read must not stop it from starting.
        if (!e.badInput || saved == null) throw e
        prefs(context).edit().remove(PREFS_KEY).apply()
        mapErrors { VelmoraEngine(config(null)) }
      }
    }

    private const val PREFS_FILE = "velmora_engine"
    private const val PREFS_KEY = "preferences"

    private fun prefs(context: Context) = context.applicationContext.getSharedPreferences(PREFS_FILE, Context.MODE_PRIVATE)

    /** The engine preferences saved on this phone (desktop's Preferences JSON), if any. */
    fun savedPreferences(context: Context): String? = prefs(context).getString(PREFS_KEY, null)

    /**
     * Merge [patch] (some Preferences fields) into the engine's preferences:
     * right away if the engine is running, and on every later start.
     */
    suspend fun updatePreferences(context: Context, patch: JsonObject) =
      withContext(Dispatchers.IO) {
        val running = instance
        val base =
          running?.let { runCatching { parse(it.ffi.preferences()).jsonObject }.getOrNull() }
            ?: savedPreferences(context)?.let { runCatching { parse(it).jsonObject }.getOrNull() }
            ?: JsonObject(emptyMap())
        val merged = JsonObject(base + patch).toString()
        prefs(context).edit().putString(PREFS_KEY, merged).apply()
        running?.let { mapErrors { it.ffi.setPreferences(merged) } }
        Unit
      }

    private fun parse(text: String): JsonElement = json.parseToJsonElement(text)

    private fun mode(dub: Boolean) = if (dub) "dub" else "sub"

    private fun JsonObject.string(key: String): String? = this[key]?.jsonPrimitive?.contentOrNull

    private fun toTitle(o: JsonObject): Title = Wire(json.decodeFromJsonElement(TitleSummary.serializer(), o), o)

    private fun titles(array: JsonArray): List<Title> = array.mapNotNull { el ->
      runCatching { toTitle(el.jsonObject) }.getOrNull()
    }

    private fun facets(text: String): List<Facet> =
      parse(text).jsonArray.mapNotNull { runCatching { json.decodeFromJsonElement(Facet.serializer(), it) }.getOrNull() }

    // One odd fixture must not hide the rest.
    private fun matches(text: String): List<Match> =
      parse(text).jsonArray.mapNotNull { runCatching { json.decodeFromJsonElement(Match.serializer(), it) }.getOrNull() }

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
