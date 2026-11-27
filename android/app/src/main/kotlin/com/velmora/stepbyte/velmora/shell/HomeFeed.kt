package com.velmora.stepbyte.velmora.shell

import android.content.Context
import com.velmora.stepbyte.engine.Rail
import com.velmora.stepbyte.engine.Universe
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Home's rows from the engine, kept for the whole process: switching tabs or
 * modes doesn't fetch them again, and the mode-switch panel borrows their
 * posters for its film strip.
 */
object HomeFeed {
  private val _rails = MutableStateFlow<List<Rail>>(emptyList())
  val rails: StateFlow<List<Rail>> = _rails.asStateFlow()

  private val _loading = MutableStateFlow(false)
  val loading: StateFlow<Boolean> = _loading.asStateFlow()

  private val _error = MutableStateFlow<String?>(null)
  val error: StateFlow<String?> = _error.asStateFlow()

  private val lock = Mutex()

  /** Fetch the rows unless they're already here (or [force]). */
  suspend fun load(context: Context, force: Boolean = false) {
    if (!force && _rails.value.isNotEmpty()) return
    lock.withLock {
      if (!force && _rails.value.isNotEmpty()) return
      _loading.value = true
      _error.value = null
      try {
        _rails.value = Universe.get(context).home().filter { it.items.isNotEmpty() }
      } catch (e: CancellationException) {
        throw e
      } catch (e: Throwable) {
        _error.value = e.message?.takeIf { it.isNotBlank() } ?: e.javaClass.simpleName
      } finally {
        _loading.value = false
      }
    }
  }

  /** Poster URLs across the rows, for the switch panel. */
  fun posters(limit: Int = 16): List<String> =
    _rails.value.flatMap { it.items }.mapNotNull { it.value.poster }.distinct().take(limit)
}
