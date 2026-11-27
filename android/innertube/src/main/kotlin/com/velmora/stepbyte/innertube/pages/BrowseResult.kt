package com.velmora.stepbyte.innertube.pages

import com.velmora.stepbyte.innertube.models.YTItem
import com.velmora.stepbyte.innertube.models.filterBlockedArtists
import com.velmora.stepbyte.innertube.models.filterDislikedSongs
import com.velmora.stepbyte.innertube.models.filterExplicit
import com.velmora.stepbyte.innertube.models.filterVideoSongs
import com.velmora.stepbyte.innertube.models.filterYoutubeShorts

data class BrowseResult(
  val title: String?,
  val items: List<Item>,
) {
  data class Item(
    val title: String?,
    val items: List<YTItem>,
  )

  fun filterBlockedArtists() =
    copy(
      items =
        items.mapNotNull {
          it.copy(
            items =
              it.items.filterBlockedArtists().ifEmpty {
                return@mapNotNull null
              },
          )
        },
    )

  fun filterDislikedSongs() =
    copy(
      items =
        items.mapNotNull {
          it.copy(
            items =
              it.items.filterDislikedSongs().ifEmpty {
                return@mapNotNull null
              },
          )
        },
    )

  fun filterExplicit(enabled: Boolean = true) =
    if (enabled) {
      copy(
        items =
          items.mapNotNull {
            it.copy(
              items =
                it.items.filterExplicit().ifEmpty {
                  return@mapNotNull null
                },
            )
          },
      )
    } else {
      this
    }

  fun filterVideoSongs(disableVideos: Boolean = false) =
    if (disableVideos) {
      copy(
        items =
          items.mapNotNull {
            it.copy(
              items =
                it.items.filterVideoSongs(true).ifEmpty {
                  return@mapNotNull null
                },
            )
          },
      )
    } else {
      this
    }

  fun filterYoutubeShorts(enabled: Boolean = false) =
    if (enabled) {
      copy(
        items =
          items.mapNotNull {
            it.copy(
              items =
                it.items.filterYoutubeShorts(true).ifEmpty {
                  return@mapNotNull null
                },
            )
          },
      )
    } else {
      this
    }
}
