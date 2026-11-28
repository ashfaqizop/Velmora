package com.velmora.stepbyte.velmora.player

import android.content.Context
import android.net.ConnectivityManager
import com.velmora.stepbyte.engine.Source
import com.velmora.stepbyte.engine.SourcesReport
import com.velmora.stepbyte.engine.SourcesUpdate
import com.velmora.stepbyte.engine.Target
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.engine.TitleDetail
import com.velmora.stepbyte.engine.TitleSummary
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.engine.Wire
import com.velmora.stepbyte.velmora.library.VelmoraLibrary
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive

/** Player settings that only live on this phone. */
object PlayerPrefs {
  private const val FILE = "velmora_player"

  enum class MobileData(val label: String, val detail: String) {
    Ask("Ask before torrents", "Streams play; torrents ask first"),
    Allow("Always allow", "Mobile data is used like Wi-Fi"),
    NoTorrents("No torrents", "Only direct streams on mobile data"),
  }

  private fun p(c: Context) = c.applicationContext.getSharedPreferences(FILE, Context.MODE_PRIVATE)

  fun mobileData(c: Context) = runCatching { MobileData.valueOf(p(c).getString("mobile_data", null) ?: "Ask") }.getOrDefault(MobileData.Ask)

  fun setMobileData(c: Context, v: MobileData) = p(c).edit().putString("mobile_data", v.name).apply()

  /** Cap picked sources at 720p on mobile data. */
  fun saveData(c: Context) = p(c).getBoolean("save_data", true)

  fun setSaveData(c: Context, v: Boolean) = p(c).edit().putBoolean("save_data", v).apply()

  fun background(c: Context) = p(c).getBoolean("background", false)

  fun setBackground(c: Context, v: Boolean) = p(c).edit().putBoolean("background", v).apply()

  fun autoNext(c: Context) = p(c).getBoolean("auto_next", true)

  fun setAutoNext(c: Context, v: Boolean) = p(c).edit().putBoolean("auto_next", v).apply()

  /** Anime: dubbed rather than subtitled. */
  fun dub(c: Context) = p(c).getBoolean("dub", false)

  fun setDub(c: Context, v: Boolean) = p(c).edit().putBoolean("dub", v).apply()

  fun subtitleScale(c: Context) = p(c).getFloat("sub_scale", 1f)

  fun setSubtitleScale(c: Context, v: Float) = p(c).edit().putFloat("sub_scale", v).apply()

  fun metered(c: Context): Boolean =
    runCatching { (c.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager).isActiveNetworkMetered }.getOrDefault(false)
}

/** One site's answer while sources are being found. */
data class FoundSite(val provider: String, val state: State, val count: Int = 0, val cached: Boolean = false) {
  enum class State { Looking, Found, None, Failed }
}

/** One lookup: a title, a movie or an episode, sub or dub. */
class Finding internal constructor(val title: Title, val target: Target, val dub: Boolean) {
  private val _sites = MutableStateFlow<List<FoundSite>>(emptyList())
  val sites: StateFlow<List<FoundSite>> = _sites.asStateFlow()

  /** Every source so far, best first within each site, in arrival order. */
  private val _sources = MutableStateFlow<List<Source>>(emptyList())
  val sources: StateFlow<List<Source>> = _sources.asStateFlow()

  private val _report = MutableStateFlow<SourcesReport?>(null)
  val report: StateFlow<SourcesReport?> = _report.asStateFlow()

  private val _error = MutableStateFlow<String?>(null)
  val error: StateFlow<String?> = _error.asStateFlow()

  internal val result = CompletableDeferred<SourcesReport>()
  val started = System.currentTimeMillis()

  val done: Boolean
    get() = result.isCompleted

  internal suspend fun run(context: Context) {
    try {
      Universe.get(context).sourcesWatch(title, target, dub).collect { u ->
        when (u) {
          is SourcesUpdate.Plan -> _sites.value = u.providers.map { FoundSite(it, FoundSite.State.Looking) }
          is SourcesUpdate.Batch -> {
            val site =
              FoundSite(
                provider = u.provider,
                state =
                  when {
                    u.error != null -> FoundSite.State.Failed
                    u.sources.isNotEmpty() -> FoundSite.State.Found
                    else -> FoundSite.State.None
                  },
                count = u.sources.size,
                cached = u.cached,
              )
            _sites.update { list -> if (list.any { it.provider == u.provider }) list.map { if (it.provider == u.provider) site else it } else list + site }
            if (u.sources.isNotEmpty()) _sources.update { it + u.sources }
          }
          is SourcesUpdate.Done -> {
            _report.value = u.report
            _sources.value = u.report.sources
            _sites.update { list -> list.map { if (it.state == FoundSite.State.Looking) it.copy(state = FoundSite.State.None) else it } }
            result.complete(u.report)
          }
        }
      }
      if (!result.isCompleted) result.complete(SourcesReport(_sources.value, emptyList()))
    } catch (e: kotlinx.coroutines.CancellationException) {
      result.cancel(e)
      throw e
    } catch (e: Throwable) {
      _error.value = e.message ?: e.javaClass.simpleName
      result.completeExceptionally(e)
    }
  }
}

/**
 * Finding sources ahead, like desktop: the title page starts the lookup for
 * whatever Play would play, so pressing Play takes a result that's ready (or
 * joins the one still running) instead of starting from nothing. The engine
 * caches each site's answer as well, so coming back is quick.
 */
object SourceFinder {
  private const val REUSE_MS = 10 * 60 * 1000L
  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
  private val found = LinkedHashMap<String, Finding>()

  private fun key(title: Title, target: Target, dub: Boolean): String {
    val imdb = (title.json["ids"] as? JsonObject)?.get("imdb")?.jsonPrimitive?.contentOrNull
    val t = when (target) {
      Target.Movie -> "movie"
      is Target.Episode -> "s${target.season}e${target.episode}"
    }
    return "${imdb ?: VelmoraLibrary.key(title)}|$t|${if (dub) "dub" else "sub"}"
  }

  @Synchronized
  fun peek(title: Title, target: Target, dub: Boolean): Finding? {
    val f = found[key(title, target, dub)] ?: return null
    return if (f.done && System.currentTimeMillis() - f.started > REUSE_MS) null else f
  }

  /** Start (or join) the lookup. */
  @Synchronized
  fun find(context: Context, title: Title, target: Target, dub: Boolean): Finding {
    peek(title, target, dub)?.let {
      return it
    }
    val k = key(title, target, dub)
    val f = Finding(title, target, dub)
    found[k] = f
    if (found.size > 30) found.remove(found.keys.first())
    val app = context.applicationContext
    scope.launch {
      f.run(app)
      if (f.result.isCancelled || f.error.value != null) synchronized(this@SourceFinder) { if (found[k] === f) found.remove(k) }
    }
    return f
  }

  /** A source didn't play: the next lookup starts fresh. */
  fun forget(context: Context, title: Title, target: Target, dub: Boolean) {
    synchronized(this) { found.remove(key(title, target, dub)) }
    val app = context.applicationContext
    scope.launch { runCatching { Universe.get(app).forgetSources(title, target, dub) } }
  }

  /**
   * The title to look sources up for: the one opened, with what its details
   * know added (IMDb and other ids, every site it's on). The title page and
   * the player both use this, so they share one lookup.
   */
  fun lookupTitle(title: Title, detail: TitleDetail?): Title {
    if (detail == null) return title
    val mine = title.json
    val theirs = detail.title.json
    val ids = JsonObject(((theirs["ids"] as? JsonObject).orEmpty()) + ((mine["ids"] as? JsonObject).orEmpty()))
    val provider = title.value.origin.provider
    val also = mutableListOf<JsonElement>()
    val seen = mutableSetOf(provider)
    fun addRef(el: JsonElement?) {
      val o = el as? JsonObject ?: return
      val p = o["provider"]?.jsonPrimitive?.contentOrNull ?: return
      if (seen.add(p)) also += o
    }
    (mine["also_on"] as? JsonArray)?.forEach(::addRef)
    addRef(theirs["origin"])
    (theirs["also_on"] as? JsonArray)?.forEach(::addRef)
    val merged = buildMap<String, JsonElement> {
      putAll(mine)
      put("ids", ids)
      put("also_on", JsonArray(also))
      if (mine["year"] == null && theirs["year"] != null) put("year", theirs["year"]!!)
    }
    val o = JsonObject(merged)
    return runCatching { Wire(Universe.json.decodeFromJsonElement(TitleSummary.serializer(), o), o) }.getOrDefault(title)
  }
}

// ── Labels for sources, shared by the player's picker and the title page ──

object SourceLabels {
  private val providers =
    mapOf(
      "hd_hub" to "HDHub",
      "zoko_anime" to "ZokoAnime",
      "iptv_org" to "Live TV",
      "tmdb" to "TMDB",
      "anilist" to "AniList",
      "cinemeta" to "Cinemeta",
    )

  fun provider(id: String): String =
    providers[id] ?: id.split('_').joinToString(" ") { part -> part.replaceFirstChar { it.uppercase() } }

  /** "HDHub · HubCloud", leaving out junk host names. */
  fun title(s: Source): String {
    val v = s.value
    val name = provider(v.provider)
    val host = v.host.replace(Regex("[^\\p{L}\\p{N} ._-]"), "").replace(Regex("\\s+"), " ").trim()
    val junk =
      host.length < 3 ||
        host.equals(name, ignoreCase = true) ||
        Regex("(\\p{L})\\1{2,}").containsMatchIn(host) ||
        Regex("^(player|play|server|stream|watch|video|link|source)\\s*\\d*$", RegexOption.IGNORE_CASE).matches(host)
    return if (junk) name else "$name · $host"
  }

  /** Quality first, then codec, HDR, audio, size, seeds. */
  fun chips(s: Source): List<String> {
    val v = s.value
    val t = v.tags
    val size = v.sizeBytes ?: t.sizeBytes
    return listOfNotNull(
      t.resolution?.let { if (it >= 2160) "4K" else "${it}p" },
      when (t.codec) {
        "h265" -> "HEVC"
        null -> null
        else -> t.codec?.uppercase()
      },
      t.hdr?.takeIf { it != "sdr" }?.let { if (it == "dolby_vision") "Dolby Vision" else "HDR" },
      if (t.multiAudio) "Multi audio" else null,
      size?.let(::bytes),
      v.torrent?.seeders?.let { "$it seeds" },
    )
  }

  fun bytes(n: Long): String =
    when {
      n >= 1_073_741_824L -> "%.1f GB".format(n / 1_073_741_824.0)
      n >= 1_048_576L -> "${n / 1_048_576} MB"
      else -> "${n / 1024} KB"
    }

  fun language(code: String): String {
    val c = code.trim().lowercase()
    val names =
      mapOf(
        "en" to "English", "eng" to "English", "hi" to "Hindi", "hin" to "Hindi", "ja" to "Japanese", "jpn" to "Japanese",
        "ko" to "Korean", "kor" to "Korean", "fr" to "French", "fre" to "French", "fra" to "French", "es" to "Spanish",
        "spa" to "Spanish", "de" to "German", "ger" to "German", "deu" to "German", "ta" to "Tamil", "tam" to "Tamil",
        "te" to "Telugu", "tel" to "Telugu", "ml" to "Malayalam", "kn" to "Kannada", "bn" to "Bengali", "zh" to "Chinese",
        "it" to "Italian", "pt" to "Portuguese", "ru" to "Russian", "ar" to "Arabic", "tr" to "Turkish", "id" to "Indonesian",
      )
    names[c]?.let {
      return it
    }
    names[c.take(2)]?.let { if (c.length <= 3 || c.startsWith(c.take(2) + "-")) return it }
    return runCatching { java.util.Locale(c).getDisplayLanguage(java.util.Locale.ENGLISH).takeIf { it.isNotBlank() && it.lowercase() != c } }.getOrNull()
      ?: if (code.length <= 3) code.uppercase() else code.replaceFirstChar { it.uppercase() }
  }

  /** "Hindi, English", or "Multi audio". */
  fun audio(s: Source): String? {
    val v = s.value
    val raw = v.audio.map { it.lang ?: it.label }.ifEmpty { v.tags.audio.ifEmpty { v.tags.languages } }
    val names = raw.filter { it.isNotBlank() }.map(::language).distinct()
    return when {
      names.size > 3 -> names.take(2).joinToString(", ") + " +${names.size - 2}"
      names.isNotEmpty() -> names.joinToString(", ")
      v.tags.multiAudio -> "Multi audio"
      else -> null
    }
  }

  val Source.key: String
    get() = "${value.provider}|${value.host}|${value.url}"
}

private fun JsonObject?.orEmpty(): Map<String, JsonElement> = this ?: emptyMap()

@Suppress("unused")
private fun str(v: String) = JsonPrimitive(v)
