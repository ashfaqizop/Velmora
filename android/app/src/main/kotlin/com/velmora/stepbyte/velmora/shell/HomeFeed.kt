package com.velmora.stepbyte.velmora.shell

import android.content.Context
import com.velmora.stepbyte.engine.Rail
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.engine.TitleSummary
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.engine.Wire
import java.io.File
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

/**
 * Home's rows from the engine, kept for the whole process: switching tabs or
 * modes doesn't fetch them again, and the mode-switch panel borrows their
 * posters for its film strip. The last good set is saved too, so Home opens
 * with posters offline or on a slow connection and refreshes underneath.
 */
object HomeFeed {
  private val _rails = MutableStateFlow<List<Rail>>(emptyList())
  val rails: StateFlow<List<Rail>> = _rails.asStateFlow()

  private val _loading = MutableStateFlow(false)
  val loading: StateFlow<Boolean> = _loading.asStateFlow()

  private val _error = MutableStateFlow<String?>(null)
  val error: StateFlow<String?> = _error.asStateFlow()

  /** Showing the saved rows because the fresh ones couldn't load. */
  private val _stale = MutableStateFlow(false)
  val stale: StateFlow<Boolean> = _stale.asStateFlow()

  private val lock = Mutex()
  private var fresh = false

  /** Fetch the rows unless they're already here (or [force]). */
  suspend fun load(context: Context, force: Boolean = false) {
    if (!force && fresh) return
    lock.withLock {
      if (!force && fresh) return
      if (_rails.value.isEmpty()) _rails.value = readCache(context)
      _loading.value = true
      _error.value = null
      try {
        val rails = Universe.get(context).home().filter { it.items.isNotEmpty() }
        if (rails.isEmpty()) throw IllegalStateException("None of the sources answered. Check your connection and try again.")
        _rails.value = rails
        _stale.value = false
        fresh = true
        writeCache(context, rails)
      } catch (e: CancellationException) {
        throw e
      } catch (e: Throwable) {
        _error.value = e.message?.takeIf { it.isNotBlank() } ?: e.javaClass.simpleName
        _stale.value = _rails.value.isNotEmpty()
      } finally {
        _loading.value = false
      }
    }
  }

  /** Every title across the rows, once each. */
  fun all(): List<Title> = _rails.value.flatMap { it.items }.distinctBy { "${it.value.origin.provider}:${it.value.origin.id}" }

  fun rail(id: String): Rail? = _rails.value.firstOrNull { it.id == id }

  /** Poster URLs across the rows, for the switch panel. */
  fun posters(limit: Int = 16): List<String> =
    _rails.value.flatMap { it.items }.mapNotNull { it.value.poster }.distinct().take(limit)

  /** Titles wide enough for the hero, a few from each trending row. */
  fun spotlight(): List<Title> {
    val picks = mutableListOf<Title>()
    for (id in listOf("trending_movies", "trending_series", "trending_anime")) {
      picks += rail(id)?.items?.filter { it.value.backdrop != null }?.take(2).orEmpty()
    }
    if (picks.size < 3) picks += _rails.value.flatMap { it.items }.filter { it.value.backdrop != null }.take(5)
    return picks.distinctBy { "${it.value.origin.provider}:${it.value.origin.id}" }.take(6)
  }

  // ── The saved copy ──

  private fun cacheFile(context: Context) = File(context.applicationContext.filesDir, "velmora/home.json")

  private suspend fun readCache(context: Context): List<Rail> =
    withContext(Dispatchers.IO) {
      runCatching {
          Universe.json.parseToJsonElement(cacheFile(context).readText()).jsonArray.mapNotNull { el ->
            val o = el as? JsonObject ?: return@mapNotNull null
            val items =
              (o["items"] as? JsonArray)?.mapNotNull { t ->
                runCatching { Wire(Universe.json.decodeFromJsonElement(TitleSummary.serializer(), t), t.jsonObject) }.getOrNull()
              } ?: emptyList()
            Rail(o["id"]?.jsonPrimitive?.contentOrNull ?: "", o["title"]?.jsonPrimitive?.contentOrNull ?: "", items)
          }
        }
        .getOrDefault(emptyList())
    }

  private suspend fun writeCache(context: Context, rails: List<Rail>) =
    withContext(Dispatchers.IO) {
      runCatching {
        val out = buildJsonArray {
          for (r in rails) {
            add(
              buildJsonObject {
                put("id", JsonPrimitive(r.id))
                put("title", JsonPrimitive(r.title))
                put("items", JsonArray(r.items.map { it.json }))
              },
            )
          }
        }
        val file = cacheFile(context)
        file.parentFile?.mkdirs()
        val tmp = File(file.path + ".tmp")
        tmp.writeText(out.toString())
        tmp.renameTo(file)
      }
      Unit
    }
}
