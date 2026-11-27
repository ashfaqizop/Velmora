package com.velmora.stepbyte.ui.screens

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.runtime.Immutable
import com.velmora.stepbyte.R

@Immutable
sealed class Screens(
  @StringRes val titleId: Int,
  @DrawableRes val iconIdInactive: Int,
  @DrawableRes val iconIdActive: Int,
  val route: String,
) {
  object Home :
    Screens(
      titleId = R.string.home,
      iconIdInactive = R.drawable.home_outlined,
      iconIdActive = R.drawable.home_filled,
      route = "home",
    )

  object Podcast :
    Screens(
      titleId = R.string.podcast,
      iconIdInactive = R.drawable.radio,
      iconIdActive = R.drawable.radio,
      route = "podcasts",
    )

  object Search :
    Screens(
      titleId = R.string.search,
      iconIdInactive = R.drawable.search,
      iconIdActive = R.drawable.search,
      route = "search_input",
    )

  object ListenTogether :
    Screens(
      titleId = R.string.together,
      iconIdInactive = R.drawable.group_outlined,
      iconIdActive = R.drawable.group_filled,
      route = "listen_together",
    )

  object Library :
    Screens(
      titleId = R.string.filter_library,
      iconIdInactive = R.drawable.library_music_outlined,
      iconIdActive = R.drawable.library_music_filled,
      route = "library",
    )

  object Generate :
    Screens(
      titleId = R.string.playlists,
      iconIdInactive = R.drawable.playlist_play,
      iconIdActive = R.drawable.playlist_play,
      route = "generate",
    )

  // Velmora (video) mode tabs. Their routes live in the "velmora" graph.
  object VelmoraHome :
    Screens(
      titleId = R.string.velmora_tab_home,
      iconIdInactive = R.drawable.home_outlined,
      iconIdActive = R.drawable.home_filled,
      route = "v/home",
    )

  object VelmoraBrowse :
    Screens(
      titleId = R.string.velmora_tab_browse,
      iconIdInactive = R.drawable.explore_outlined,
      iconIdActive = R.drawable.explore_filled,
      route = "v/browse",
    )

  object VelmoraTheatre :
    Screens(
      titleId = R.string.velmora_tab_theatre,
      iconIdInactive = R.drawable.theaters_outlined,
      iconIdActive = R.drawable.theaters_filled,
      route = "v/theatre",
    )

  object VelmoraSearch :
    Screens(
      titleId = R.string.velmora_tab_search,
      iconIdInactive = R.drawable.search,
      iconIdActive = R.drawable.search,
      route = "v/search",
    )

  companion object {
    val MainScreens = listOf(Home, Podcast, Search, Library)
    val VelmoraScreens = listOf(VelmoraHome, VelmoraBrowse, VelmoraTheatre, VelmoraSearch)
  }
}
