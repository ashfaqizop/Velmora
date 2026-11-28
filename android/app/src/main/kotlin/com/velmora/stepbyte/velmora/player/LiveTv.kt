package com.velmora.stepbyte.velmora.player

import android.content.Context
import com.velmora.stepbyte.engine.TvChannel
import com.velmora.stepbyte.engine.Universe
import java.util.Locale
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.ListSerializer

/** A channel remembered on this phone: a favourite or one watched lately. */
@Serializable data class SavedChannel(val id: String, val name: String, val logo: String? = null, val country: String? = null)

/**
 * Live TV on this phone: favourite channels, the ones watched lately, and the
 * country "where to watch" asks about. All local, like desktop's.
 */
object LiveTv {
  private const val FILE = "velmora_live"
  private val listJson = ListSerializer(SavedChannel.serializer())

  private val _favorites = MutableStateFlow<List<SavedChannel>>(emptyList())
  val favorites: StateFlow<List<SavedChannel>> = _favorites.asStateFlow()

  private val _recent = MutableStateFlow<List<SavedChannel>>(emptyList())
  val recent: StateFlow<List<SavedChannel>> = _recent.asStateFlow()

  private var loaded = false

  private fun prefs(c: Context) = c.applicationContext.getSharedPreferences(FILE, Context.MODE_PRIVATE)

  fun load(c: Context) {
    if (loaded) return
    loaded = true
    _favorites.value = read(c, "favorites")
    _recent.value = read(c, "recent")
  }

  private fun read(c: Context, key: String): List<SavedChannel> =
    runCatching { Universe.json.decodeFromString(listJson, prefs(c).getString(key, null) ?: "[]") }.getOrDefault(emptyList())

  private fun write(c: Context, key: String, list: List<SavedChannel>) {
    prefs(c).edit().putString(key, Universe.json.encodeToString(listJson, list)).apply()
  }

  private fun TvChannel.saved() = SavedChannel(id, name, logo, country)

  fun isFavorite(id: String) = _favorites.value.any { it.id == id }

  /** Add or remove; returns whether it's a favourite now. */
  fun toggleFavorite(c: Context, ch: TvChannel): Boolean {
    load(c)
    val on = !isFavorite(ch.id)
    _favorites.value = if (on) _favorites.value + ch.saved() else _favorites.value.filterNot { it.id == ch.id }
    write(c, "favorites", _favorites.value)
    return on
  }

  /** Remember a channel that was just tuned in. */
  fun played(c: Context, ch: TvChannel) {
    load(c)
    _recent.value = (listOf(ch.saved()) + _recent.value.filterNot { it.id == ch.id }).take(12)
    write(c, "recent", _recent.value)
  }

  /** The country for "where to watch": chosen, else the phone's. */
  fun region(c: Context): String = prefs(c).getString("region", null) ?: Locale.getDefault().country.ifBlank { "US" }.uppercase()

  fun setRegion(c: Context, code: String) = prefs(c).edit().putString("region", code.uppercase()).apply()

  /** Play a remembered channel (it's looked up fresh: streams change). */
  suspend fun play(c: Context, id: String): Boolean {
    val ch = runCatching { Universe.get(c).tvChannel(id) }.getOrNull() ?: return false
    VideoSession.playChannel(c, ch)
    return true
  }
}
