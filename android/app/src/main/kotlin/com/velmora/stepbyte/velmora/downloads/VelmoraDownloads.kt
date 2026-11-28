package com.velmora.stepbyte.velmora.downloads

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.os.StatFs
import android.util.Log
import com.velmora.stepbyte.engine.DownloadStatus
import com.velmora.stepbyte.engine.Source
import com.velmora.stepbyte.engine.Target
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.engine.TitleSummary
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.engine.Wire
import com.velmora.stepbyte.velmora.library.VelmoraLibrary
import com.velmora.stepbyte.velmora.player.PlayerPrefs
import com.velmora.stepbyte.velmora.player.SourceFinder
import com.velmora.stepbyte.velmora.screens.DetailCache
import java.io.File
import java.util.UUID
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject

/** How good a download should be. Smaller is faster and lighter on storage. */
enum class DownloadQuality(val label: String, val max: Int) {
  Best("Best", Int.MAX_VALUE),
  Full("1080p", 1080),
  High("720p", 720),
  Saver("480p", 480),
}

/** Where a download is, as the Downloads page shows it. */
enum class DownloadPhase { Finding, WaitingWifi, Queued, Running, Paused, Done, Failed }

/** One saved download: a movie or an episode, from finding a source to the file. */
@Serializable
data class DownloadRecord(
  val id: String,
  val title: JsonObject,
  val season: Int? = null,
  val episode: Int? = null,
  val episodeName: String? = null,
  val quality: DownloadQuality = DownloadQuality.Full,
  val addedAt: Long,
  val source: JsonObject? = null,
  /** Sources that failed for this one, so a retry takes the next. */
  val tried: List<String> = emptyList(),
  val phase: DownloadPhase = DownloadPhase.Finding,
  val path: String? = null,
  val size: Long = 0,
  val error: String? = null,
  /** Paused by hand (not by the Wi-Fi rule). */
  val held: Boolean = false,
) {
  val target: Target
    get() = if (season != null && episode != null) Target.Episode(season, episode) else Target.Movie
}

/** A record with its live progress. */
data class DownloadItem(
  val record: DownloadRecord,
  val titleValue: TitleSummary,
  val downloaded: Long = 0,
  val total: Long? = null,
  val speed: Long = 0,
) {
  val fraction: Float
    get() = total?.takeIf { it > 0 }?.let { (downloaded.toDouble() / it).toFloat().coerceIn(0f, 1f) } ?: 0f

  val title: Title
    get() = Wire(titleValue, record.title)
}

/**
 * Velmora's downloads on this phone. The engine does the transfer and seals
 * each file into a `.vlm` vault file; this keeps the list (so a download
 * survives the app being closed: it starts again from its `.part` file), picks
 * a source of the wanted quality, falls over to the next when one fails, waits
 * for Wi-Fi when that's the rule, checks free space, and knows which titles
 * and episodes are on the phone for offline play.
 */
object VelmoraDownloads {
  private const val TAG = "VelmoraDownloads"
  private val json = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
    encodeDefaults = true
  }
  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
  private val lock = Mutex()

  private lateinit var app: Context
  private lateinit var file: File

  private val _items = MutableStateFlow<List<DownloadItem>>(emptyList())
  val items: StateFlow<List<DownloadItem>> = _items.asStateFlow()

  private val _metered = MutableStateFlow(false)
  val metered: StateFlow<Boolean> = _metered.asStateFlow()

  @Volatile private var records: List<DownloadRecord> = emptyList()
  /** Record id to the engine's job id, for this run. */
  private val engineIds = mutableMapOf<String, Long>()
  private var pump: Job? = null

  // ── Settings ──

  private fun prefs() = app.getSharedPreferences("velmora_downloads", Context.MODE_PRIVATE)

  fun wifiOnly(): Boolean = prefs().getBoolean("wifi_only", true)

  fun setWifiOnly(v: Boolean) {
    prefs().edit().putBoolean("wifi_only", v).apply()
    kick()
  }

  fun defaultQuality(): DownloadQuality = runCatching { DownloadQuality.valueOf(prefs().getString("quality", null) ?: "Full") }.getOrDefault(DownloadQuality.Full)

  fun setDefaultQuality(q: DownloadQuality) = prefs().edit().putString("quality", q.name).apply()

  // ── Starting up ──

  fun init(context: Context) {
    if (::app.isInitialized) return
    app = context.applicationContext
    file = File(app.filesDir, "velmora/downloads.json")
    records = runCatching { json.decodeFromString<List<DownloadRecord>>(file.readText()) }.getOrDefault(emptyList())
    // Nothing is running yet in this process.
    records = records.map { if (it.phase == DownloadPhase.Running || it.phase == DownloadPhase.Queued) it.copy(phase = DownloadPhase.Queued) else it }
    _metered.value = PlayerPrefs.metered(app)
    publish()
    watchNetwork()
    if (records.any { it.phase !in listOf(DownloadPhase.Done, DownloadPhase.Failed, DownloadPhase.Paused) }) kick()
  }

  private fun watchNetwork() {
    val cm = app.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager ?: return
    runCatching {
      cm.registerDefaultNetworkCallback(
        object : ConnectivityManager.NetworkCallback() {
          override fun onCapabilitiesChanged(network: Network, caps: NetworkCapabilities) {
            val metered = !caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_METERED)
            if (metered != _metered.value) {
              _metered.value = metered
              kick()
            }
          }

          override fun onLost(network: Network) {
            kick()
          }
        },
      )
    }
  }

  // ── Adding ──

  /** Download [title] ([target] for an episode). Returns a message for a snackbar when it can't. */
  fun add(context: Context, title: Title, target: Target = Target.Movie, quality: DownloadQuality = defaultQuality(), episodeName: String? = null): String? {
    init(context)
    val key = keyOf(title, target)
    val existing = records.firstOrNull { keyOf(it) == key }
    if (existing != null && existing.phase != DownloadPhase.Failed) {
      return if (existing.phase == DownloadPhase.Done) "Already downloaded" else "Already downloading"
    }
    val ep = target as? Target.Episode
    val record =
      DownloadRecord(
        id = existing?.id ?: UUID.randomUUID().toString(),
        title = title.json,
        season = ep?.season,
        episode = ep?.episode,
        episodeName = episodeName,
        quality = quality,
        addedAt = System.currentTimeMillis(),
      )
    records = records.filterNot { it.id == record.id } + record
    save()
    publish()
    kick()
    return null
  }

  /** Every episode of a season that isn't on the phone yet. Returns how many were added. */
  fun addSeason(context: Context, title: Title, season: com.velmora.stepbyte.engine.Season, quality: DownloadQuality = defaultQuality()): Int {
    var n = 0
    season.episodes.sortedBy { it.number }.forEach { e ->
      if (add(context, title, Target.Episode(e.season, e.number), quality, e.title) == null) n++
    }
    return n
  }

  fun pause(id: String) {
    update(id) { it.copy(held = true, phase = DownloadPhase.Paused) }
    engineIds[id]?.let { eid -> scope.launch { runCatching { Universe.get(app).downloadPause(eid) } } }
  }

  fun resume(id: String) {
    update(id) { it.copy(held = false, phase = if (it.phase == DownloadPhase.Failed) DownloadPhase.Finding else DownloadPhase.Queued, error = null) }
    kick()
  }

  /** Try again with the next source. */
  fun retry(id: String) {
    update(id) { it.copy(held = false, phase = DownloadPhase.Finding, source = null, error = null) }
    engineIds.remove(id)
    kick()
  }

  /** Stop and forget it; a finished file is deleted. */
  fun remove(id: String) {
    val r = records.firstOrNull { it.id == id } ?: return
    val eid = engineIds.remove(id)
    records = records.filterNot { it.id == id }
    save()
    publish()
    scope.launch {
      val engine = runCatching { Universe.get(app) }.getOrNull() ?: return@launch
      if (eid != null) runCatching { engine.downloadCancel(eid) }
      r.path?.let { p -> runCatching { engine.deleteLocal(p) }.onFailure { File(p).delete() } }
    }
  }

  // ── What's on the phone ──

  fun keyOf(title: Title, target: Target): String =
    VelmoraLibrary.key(title) + when (target) {
      Target.Movie -> ""
      is Target.Episode -> "|s${target.season}e${target.episode}"
    }

  private fun keyOf(r: DownloadRecord): String {
    val origin = r.title["origin"] as? JsonObject
    val p = origin?.get("provider")?.toString()?.trim('"') ?: ""
    val id = origin?.get("id")?.toString()?.trim('"') ?: ""
    return "$p:$id" + if (r.season != null && r.episode != null) "|s${r.season}e${r.episode}" else ""
  }

  /** The finished file for [title] at [target], if it's still there. */
  fun localFile(title: Title, target: Target): String? {
    if (!::app.isInitialized) return null
    val key = keyOf(title, target)
    return records.firstOrNull { keyOf(it) == key && it.phase == DownloadPhase.Done }?.path?.takeIf { File(it).isFile }
  }

  /** The download (any phase) for [title] at [target]. */
  fun itemFor(title: Title, target: Target): DownloadItem? {
    val key = keyOf(title, target)
    return _items.value.firstOrNull { keyOf(it.record) == key }
  }

  /** The folder the engine saves into (the same one `Universe` gives it). */
  fun folder(): File =
    app.getExternalFilesDir(android.os.Environment.DIRECTORY_MOVIES)?.let { File(it, "Velmora") } ?: File(app.filesDir, "downloads")

  fun freeBytes(): Long = runCatching { StatFs(folder().apply { mkdirs() }.path).availableBytes }.getOrDefault(-1L)

  // ── The work ──

  private fun kick() {
    if (!::app.isInitialized) return
    DownloadService.start(app)
    if (pump?.isActive == true) return
    pump = scope.launch { loop() }
  }

  /** Finds sources, starts jobs, follows progress, until nothing is left to do. */
  private suspend fun loop() {
    val engine =
      try {
        Universe.get(app)
      } catch (e: CancellationException) {
        throw e
      } catch (e: Throwable) {
        Log.w(TAG, "engine didn't start", e)
        return
      }
    while (scope.isActive) {
      val wifiBlocked = wifiOnly() && _metered.value
      // Pause what's running if the rule says so.
      for (r in records) {
        val eid = engineIds[r.id] ?: continue
        if (wifiBlocked && r.phase == DownloadPhase.Running) {
          runCatching { engine.downloadPause(eid) }
          update(r.id) { it.copy(phase = DownloadPhase.WaitingWifi) }
        }
      }
      // Start the next ones: one source lookup at a time, the engine queues the rest.
      for (r in records.toList()) {
        if (r.held) continue
        when {
          wifiBlocked && r.phase in listOf(DownloadPhase.Finding, DownloadPhase.Queued, DownloadPhase.WaitingWifi) ->
            if (r.phase != DownloadPhase.WaitingWifi) update(r.id) { it.copy(phase = DownloadPhase.WaitingWifi) }
          r.phase == DownloadPhase.WaitingWifi || r.phase == DownloadPhase.Queued || r.phase == DownloadPhase.Finding -> {
            val eid = engineIds[r.id]
            when {
              // Paused by the Wi-Fi rule: carry on.
              eid != null && r.phase == DownloadPhase.WaitingWifi -> {
                runCatching { engine.downloadResume(eid) }
                update(r.id) { it.copy(phase = DownloadPhase.Queued) }
              }
              // Already with the engine.
              eid != null -> {}
              else -> start(r)
            }
          }
          r.phase == DownloadPhase.Paused && !r.held -> {}
        }
      }
      // Progress from the engine.
      val statuses = runCatching { engine.downloads() }.getOrDefault(emptyList())
      follow(statuses)
      val busy = records.any { it.phase in listOf(DownloadPhase.Finding, DownloadPhase.Queued, DownloadPhase.Running) && !it.held }
      val waiting = records.any { it.phase == DownloadPhase.WaitingWifi }
      if (!busy && !waiting) break
      delay(if (busy) 1_000 else 5_000)
    }
    publish()
    DownloadService.refresh(app)
  }

  /** Look up sources and hand the best fitting one to the engine. */
  private suspend fun start(r: DownloadRecord) {
    val title = toTitle(r.title) ?: return fail(r.id, "This download's title couldn't be read.")
    update(r.id) { it.copy(phase = DownloadPhase.Finding) }
    publish()
    val source =
      r.source?.let { s -> runCatching { Wire(Universe.json.decodeFromJsonElement(com.velmora.stepbyte.engine.SourceInfo.serializer(), s), s) }.getOrNull() }
        ?: pick(title, r)
        ?: return fail(r.id, "No downloadable source has this ${if (r.season != null) "episode" else "one"} yet. Try another quality or later.")
    // Room for it?
    val size = source.value.sizeBytes ?: source.value.tags.sizeBytes
    val free = freeBytes()
    if (size != null && free in 0 until size + 200L * 1024 * 1024) {
      return fail(r.id, "Not enough space: it needs ${human(size)} and the phone has ${human(free)} free.")
    }
    val name = buildString {
      append(title.value.name)
      if (r.season != null && r.episode != null) append(" S%02dE%02d".format(r.season, r.episode)) else title.value.year?.let { append(" ($it)") }
    }
    try {
      val eid = Universe.get(app).downloadStart(source, name, r.target)
      engineIds[r.id] = eid
      update(r.id) { it.copy(phase = DownloadPhase.Queued, source = source.json, error = null) }
    } catch (e: CancellationException) {
      throw e
    } catch (e: Throwable) {
      update(r.id) { it.copy(tried = it.tried + keyOfSource(source), source = null) }
      fail(r.id, e.message ?: "The download didn't start.")
    }
  }

  private fun keyOfSource(s: Source) = "${s.value.provider}|${s.value.host}|${s.value.url}"

  /** The best ranked source that can be saved, at or under the wanted quality. */
  private suspend fun pick(title: Title, r: DownloadRecord): Source? {
    val detail = DetailCache.get(VelmoraLibrary.key(title))
    val lookup = SourceFinder.lookupTitle(title, detail)
    val dub = title.value.kind == "anime" && PlayerPrefs.dub(app)
    val report = runCatching { SourceFinder.find(app, lookup, r.target, dub).result.await() }.getOrNull() ?: return null
    val usable =
      report.sources.filter {
        it.value.format != "dash" && it.value.format != "unknown" && keyOfSource(it) !in r.tried &&
          // Pack torrents need the episode to pick the file.
          (!it.value.isPack || r.season != null || it.value.isTorrent)
      }
    val fits = usable.filter { (it.value.tags.resolution ?: 720) <= r.quality.max }
    // Direct files first: they resume cleanly and need no peers.
    val ordered = (fits.ifEmpty { usable }).sortedWith(compareBy<Source> { if (it.value.isTorrent) 1 else 0 }.thenByDescending { it.value.tags.resolution ?: 0 })
    return ordered.firstOrNull()
  }

  /** Match engine statuses to records. */
  private fun follow(statuses: List<DownloadStatus>) {
    val byEngine = statuses.associateBy { it.id }
    val live = mutableMapOf<String, DownloadStatus>()
    for ((rid, eid) in engineIds.toMap()) {
      val s = byEngine[eid] ?: continue
      live[rid] = s
      val r = records.firstOrNull { it.id == rid } ?: continue
      when (s.state.state) {
        "running", "extracting" -> if (r.phase != DownloadPhase.Running) update(rid) { it.copy(phase = DownloadPhase.Running) }
        "queued" -> if (r.phase != DownloadPhase.Queued) update(rid) { it.copy(phase = DownloadPhase.Queued) }
        "paused" -> if (r.phase == DownloadPhase.Running) update(rid) { it.copy(phase = if (r.held) DownloadPhase.Paused else DownloadPhase.WaitingWifi) }
        "completed" -> {
          val path = s.state.path
          engineIds.remove(rid)
          update(rid) { it.copy(phase = DownloadPhase.Done, path = path, size = path?.let { p -> File(p).length() } ?: s.downloaded, error = null) }
          DownloadService.notifyDone(app, toTitle(r.title)?.value?.name ?: s.title, r.season, r.episode)
        }
        "failed" -> {
          engineIds.remove(rid)
          // Next source, up to three tries.
          val src = r.source
          if (src != null && r.tried.size < 2) {
            update(rid) { it.copy(tried = it.tried + sourceKey(src), source = null, phase = DownloadPhase.Finding) }
          } else {
            fail(rid, s.state.error ?: "The download failed.")
          }
        }
        "cancelled" -> engineIds.remove(rid)
      }
    }
    publish(live)
  }

  private fun sourceKey(o: JsonObject): String {
    fun s(k: String) = o[k]?.toString()?.trim('"') ?: ""
    return "${s("provider")}|${s("host")}|${s("url")}"
  }

  private fun fail(id: String, message: String) {
    update(id) { it.copy(phase = DownloadPhase.Failed, error = message) }
  }

  // ── State ──

  private fun update(id: String, f: (DownloadRecord) -> DownloadRecord) {
    records = records.map { if (it.id == id) f(it) else it }
    save()
    publish()
  }

  private var lastLive: Map<String, DownloadStatus> = emptyMap()

  private fun publish(live: Map<String, DownloadStatus> = lastLive) {
    lastLive = live
    _items.value =
      records
        .sortedByDescending { it.addedAt }
        .mapNotNull { r ->
          val t = toTitle(r.title) ?: return@mapNotNull null
          val s = live[r.id]
          DownloadItem(r, t.value, downloaded = s?.downloaded ?: if (r.phase == DownloadPhase.Done) r.size else 0, total = s?.total ?: if (r.phase == DownloadPhase.Done) r.size else null, speed = s?.bytesPerSec ?: 0)
        }
    DownloadService.refresh(app)
  }

  private fun save() {
    val snapshot = records
    scope.launch {
      lock.withLock {
        runCatching {
          file.parentFile?.mkdirs()
          val tmp = File(file.path + ".tmp")
          tmp.writeText(json.encodeToString(kotlinx.serialization.builtins.ListSerializer(DownloadRecord.serializer()), snapshot))
          tmp.renameTo(file)
        }
          .onFailure { Log.w(TAG, "couldn't save downloads", it) }
      }
    }
  }

  private fun toTitle(o: JsonObject): Title? =
    runCatching { Wire(Universe.json.decodeFromJsonElement(TitleSummary.serializer(), o), o) }.getOrNull()

  fun human(n: Long): String =
    when {
      n >= 1_073_741_824L -> "%.1f GB".format(n / 1_073_741_824.0)
      n >= 1_048_576L -> "${n / 1_048_576} MB"
      else -> "${(n / 1024).coerceAtLeast(0)} KB"
    }
}
