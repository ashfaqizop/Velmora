package com.velmora.stepbyte.velmora.privacy

import android.content.Context
import android.util.Log
import com.velmora.stepbyte.BuildConfig
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.velmora.account.Supa
import java.time.Instant
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

/**
 * Anonymous usage counts, the same as desktop (app/src/lib/analytics.svelte.ts):
 * which titles and sites get played, which player, what quality, which songs,
 * and the hour plays start (Theatre puts screenings at busy hours). Counted on
 * the phone, sent once a day as plain totals for finished days. No ID, no
 * account, nothing that says who. On by default; Settings > Velmora > Privacy
 * switches it off and forgets what was counted. Server: site/api/_lib/stats.js.
 */
object UsageStats {
  private const val TAG = "VelmoraStats"
  private const val PREFS = "velmora_stats"
  private val json = Json { ignoreUnknownKeys = true }
  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
  private val lock = Mutex()
  private val TITLE = Regex("^(movie|tv)/\\d{1,9}$")
  private val SONG = Regex("^[A-Za-z0-9_-]{11}$")
  private val WORD = Regex("^[a-z0-9_]{2,24}$")

  @Serializable
  private data class Day(
    val titles: MutableMap<String, Int> = mutableMapOf(),
    val sources: MutableMap<String, Int> = mutableMapOf(),
    val engines: MutableMap<String, Int> = mutableMapOf(),
    val quality: MutableMap<String, Int> = mutableMapOf(),
    val music: MutableMap<String, Int> = mutableMapOf(),
    val hours: MutableMap<String, Int> = mutableMapOf(),
    var app: String = "",
  )

  private lateinit var app: Context
  private val _enabled = MutableStateFlow(true)
  val enabled: StateFlow<Boolean> = _enabled.asStateFlow()

  private val _noticeSeen = MutableStateFlow(true)
  /** False until the one-time "this is on, and where to turn it off" card is dismissed. */
  val noticeSeen: StateFlow<Boolean> = _noticeSeen.asStateFlow()

  private fun prefs() = app.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

  /** Call once at startup. Sends finished days now and every six hours. */
  fun init(context: Context) {
    if (::app.isInitialized) return
    app = context.applicationContext
    _enabled.value = prefs().getBoolean("on", true)
    _noticeSeen.value = prefs().getBoolean("notice", false)
    scope.launch {
      delay(20_000)
      while (true) {
        flush()
        delay(6 * 3_600_000L)
      }
    }
  }

  fun setEnabled(on: Boolean) {
    _enabled.value = on
    // Off means off: what was counted but not sent goes too.
    val e = prefs().edit().putBoolean("on", on)
    if (!on) e.remove("pending")
    e.apply()
  }

  fun noticeDone() {
    _noticeSeen.value = true
    prefs().edit().putBoolean("notice", true).apply()
  }

  /** TMDB's own name for a title ("movie/603", "tv/1396"), or null when we can't tell. */
  fun titleKey(t: Title): String? {
    val o = t.value.origin
    if (o.provider == "tmdb" && TITLE.matches(o.id)) return o.id
    val id = (t.json["ids"] as? JsonObject)?.get("tmdb")?.jsonPrimitive?.intOrNull ?: return null
    return "${if (t.value.kind == "movie") "movie" else "tv"}/$id"
  }

  fun qualityBucket(resolution: Int?): String =
    when {
      resolution == null || resolution <= 0 -> "other"
      resolution >= 2160 -> "2160"
      resolution >= 1440 -> "1440"
      resolution >= 1080 -> "1080"
      resolution >= 720 -> "720"
      resolution >= 480 -> "480"
      else -> "other"
    }

  /** A minute into a title: count the title, the site, the player and the quality. */
  fun watched(title: Title, provider: String?, resolution: Int?) {
    if (!::app.isInitialized || !_enabled.value) return
    val key = titleKey(title)
    update { d ->
      key?.let { d.titles.bump(it) }
      provider?.lowercase()?.takeIf { WORD.matches(it) }?.let { d.sources.bump(it) }
      d.engines.bump("exo")
      d.quality.bump(qualityBucket(resolution))
      d.hours.bump("%02d".format(Instant.now().atZone(ZoneOffset.UTC).hour))
    }
  }

  /** A song heard properly (the play-history rule). */
  fun listened(videoId: String) {
    if (!::app.isInitialized || !_enabled.value || !SONG.matches(videoId)) return
    update { d -> d.music.bump(videoId) }
  }

  /** Exactly what would be sent, for Settings > Privacy. */
  fun preview(): String {
    if (!::app.isInitialized) return "Nothing is waiting to be sent."
    val pending = read()
    if (pending.isEmpty()) return "Nothing is waiting to be sent."
    val pretty = Json { prettyPrint = true }
    return pretty.encodeToString(JsonArray.serializer(), JsonArray(pending.toSortedMap().map { (day, d) -> batch(day, d) }))
  }

  private fun MutableMap<String, Int>.bump(k: String) {
    this[k] = ((this[k] ?: 0) + 1).coerceAtMost(200)
  }

  private fun update(change: (Day) -> Unit) {
    scope.launch {
      lock.withLock {
        val pending = read()
        val today = utcDay(System.currentTimeMillis())
        val d = pending.getOrPut(today) { Day() }
        d.app = BuildConfig.VERSION_NAME
        change(d)
        write(pending)
      }
    }
  }

  /** Send finished days; today's counts wait until the day is over. */
  suspend fun flush() {
    if (!::app.isInitialized || !_enabled.value) return
    lock.withLock {
      val pending = read()
      if (pending.isEmpty()) return
      val now = System.currentTimeMillis()
      val today = utcDay(now)
      val oldest = utcDay(now - 8 * 86_400_000L)
      var changed = false
      for ((day, d) in pending.toMap()) {
        if (day >= today) continue
        if (day < oldest) {
          pending.remove(day)
          changed = true
          continue
        }
        val sent =
          runCatching {
              val r = Supa.request("POST", "${Supa.SITE}/api/stats/report", body = batch(day, d).toString(), supabase = false)
              r.code == 200 || r.code == 400
            }
            .getOrDefault(false)
        if (sent) {
          pending.remove(day)
          changed = true
        } else {
          Log.d(TAG, "report for $day not sent; trying later")
          break
        }
      }
      if (changed) write(pending)
    }
  }

  private fun batch(day: String, d: Day): JsonObject = buildJsonObject {
    put("v", 1)
    put("day", day)
    put("app", d.app.ifBlank { BuildConfig.VERSION_NAME })
    put("titles", buildJsonArray { d.titles.keys.forEach { add(buildJsonObject { put("k", it) }) } })
    put("sources", counts(d.sources))
    put("engines", counts(d.engines))
    put("quality", counts(d.quality))
    put("music", buildJsonArray { d.music.keys.forEach { add(buildJsonObject { put("k", it) }) } })
    put("hours", counts(d.hours))
  }

  private fun counts(m: Map<String, Int>) = buildJsonArray {
    m.forEach { (k, n) ->
      add(
        buildJsonObject {
          put("k", k)
          put("n", n)
        },
      )
    }
  }

  private fun read(): MutableMap<String, Day> =
    runCatching {
        val text = prefs().getString("pending", null) ?: return mutableMapOf()
        val o = json.parseToJsonElement(text).jsonObject
        o.mapValues { json.decodeFromJsonElement(Day.serializer(), it.value) }.toMutableMap()
      }
      .getOrDefault(mutableMapOf())

  private fun write(p: Map<String, Day>) {
    val o = JsonObject(p.mapValues { json.encodeToJsonElement(Day.serializer(), it.value) })
    prefs().edit().putString("pending", o.toString()).apply()
  }

  private fun utcDay(ms: Long): String = DateTimeFormatter.ISO_LOCAL_DATE.format(Instant.ofEpochMilli(ms).atZone(ZoneOffset.UTC))
}
