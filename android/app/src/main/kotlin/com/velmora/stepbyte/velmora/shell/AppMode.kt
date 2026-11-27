package com.velmora.stepbyte.velmora.shell

import androidx.compose.runtime.staticCompositionLocalOf
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.navigation.NavController
import com.velmora.stepbyte.ui.screens.Screens

/**
 * Velmora has two modes: Velmora (movies, series, anime, live TV, sports,
 * Theatre) and Music (Echo's YouTube Music app). Each mode is its own nested
 * navigation graph, so switching saves one mode's back stack and restores the
 * other's: you come back exactly where you left.
 */
enum class AppMode {
  VELMORA,
  MUSIC;

  val graph: String
    get() = if (this == VELMORA) VELMORA_GRAPH else MUSIC_GRAPH

  /** The mode's first tab, where its tab bar pops back to. */
  val homeRoute: String
    get() = if (this == VELMORA) Screens.VelmoraHome.route else Screens.Home.route

  val other: AppMode
    get() = if (this == VELMORA) MUSIC else VELMORA

  companion object {
    fun from(value: String?): AppMode = entries.firstOrNull { it.name == value } ?: VELMORA
  }
}

const val VELMORA_GRAPH = "velmora"
const val MUSIC_GRAPH = "music"

/** Velmora-mode routes that aren't tabs. */
object VelmoraRoutes {
  const val DOWNLOADS = "v/downloads"
}

/** The mode the app was last in; it reopens in the same one. */
val AppModeKey = stringPreferencesKey("velmora_app_mode")

/** Set once the switch panel's one-time "drag me" hint has played. */
val ModeSwitchHintSeenKey = booleanPreferencesKey("velmora_mode_switch_hint_seen")

/** Asks the shell to switch modes (from search, the wordmark, anywhere). */
val LocalSwitchMode = staticCompositionLocalOf<(AppMode) -> Unit> { {} }

/** The mode currently showing. */
val LocalAppMode = staticCompositionLocalOf { AppMode.VELMORA }

/**
 * Move to [to]'s graph, saving the current mode's back stack and restoring the
 * one [to] had. Same pattern as bottom-navigation tabs with nested graphs.
 */
fun NavController.switchMode(to: AppMode) {
  navigate(to.graph) {
    popUpTo(graph.id) { saveState = true }
    launchSingleTop = true
    restoreState = true
  }
}

/** Words that, typed into search, offer a jump to the other mode. */
fun modeFromQuery(query: String): AppMode? =
  when (query.trim().lowercase()) {
    "music", "musics", "song", "songs", "yt music", "youtube music" -> AppMode.MUSIC
    "movie", "movies", "velmora", "film", "films", "series", "anime", "tv" -> AppMode.VELMORA
    else -> null
  }
