package com.velmora.stepbyte.velmora.social

import android.net.Uri
import com.velmora.stepbyte.BuildConfig
import kotlinx.coroutines.flow.MutableStateFlow

/**
 * Links that open the app, from anywhere: the website's "Open in Velmora"
 * page, Velmora Social, shared addresses and velmora:// links.
 *
 *   velmora://title/movie/603          a title page
 *   velmora://music/<videoId>          a song, in Music mode
 *   velmora://profile/<username>       a Social profile
 *   velmora://theatre/<id>             a screening
 *   https://getvelmora.dpdns.org/open/?to=title/movie/603   (same list)
 *   https://social.getvelmora.dpdns.org/@name, /title/…, /list/…, /theatre …
 *
 * Test builds use velmora-debug:// so both can be installed side by side.
 */
object VelmoraLinks {
  sealed interface Target {
    /** A TMDB title, "movie/603" or "tv/1396". */
    data class Title(val key: String) : Target

    data class Song(val videoId: String) : Target

    /** A page of Velmora Social, by its path. */
    data class Social(val path: String) : Target

    data class Screening(val id: String) : Target
  }

  /** The last link opened, for the shell to act on (then set back to null). */
  val requests = MutableStateFlow<Target?>(null)

  private val TITLE = Regex("^(movie|tv)/\\d{1,9}$")
  private val SONG = Regex("^[A-Za-z0-9_-]{11}$")
  private val USER = Regex("^[A-Za-z0-9_]{3,20}$")
  private val ID = Regex("^[0-9a-f-]{36}$")

  /** What [uri] asks for, or null when it isn't one of ours. */
  fun parse(uri: Uri?): Target? {
    uri ?: return null
    val scheme = uri.scheme?.lowercase()
    if (scheme == BuildConfig.VELMORA_SCHEME || scheme == "velmora") {
      if (uri.host == "auth") return null
      return parseTo(listOfNotNull(uri.host).plus(uri.pathSegments).joinToString("/"))
    }
    if (scheme != "https" && scheme != "http") return null
    val path = uri.path.orEmpty().trimEnd('/').ifEmpty { "/" }
    return when (uri.host?.lowercase()) {
      "getvelmora.dpdns.org", "www.getvelmora.dpdns.org" ->
        if (path == "/open" || path.startsWith("/open/")) uri.getQueryParameter("to")?.let(::parseTo) else null
      SocialLinks.HOST ->
        when {
          // A sign-in coming back belongs to the browser that started it.
          uri.getQueryParameter("vcode") != null -> null
          SocialLinks.isSocialPath(path) -> Target.Social(path)
          else -> null
        }
      else -> null
    }
  }

  /** The `to=` form: "title/movie/603", "music/<id>", "profile/<name>", "theatre/<id>". */
  fun parseTo(to: String): Target? {
    val clean = to.trim().trim('/')
    val kind = clean.substringBefore('/')
    val rest = clean.substringAfter('/', "")
    return when (kind) {
      "title" -> rest.takeIf { TITLE.matches(it) }?.let { Target.Title(it) }
      "music" -> rest.takeIf { SONG.matches(it) }?.let { Target.Song(it) }
      "profile" -> rest.takeIf { USER.matches(it) }?.let { Target.Social("/@$it") }
      "theatre" -> rest.takeIf { ID.matches(it) }?.let { Target.Screening(it) }
      else -> null
    }
  }
}
