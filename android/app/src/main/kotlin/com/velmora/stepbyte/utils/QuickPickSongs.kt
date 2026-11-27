package com.velmora.stepbyte.utils

import com.velmora.stepbyte.innertube.models.filterBlockedArtists
import com.velmora.stepbyte.innertube.models.filterExplicit
import com.velmora.stepbyte.innertube.models.filterVideoSongs
import com.velmora.stepbyte.db.entities.Song

fun filterQuickPickSongs(
  songs: List<Song>,
  excludedSongIds: Set<String>,
  blockedArtists: Set<String>,
  hideExplicit: Boolean,
  hideVideoSongs: Boolean,
): List<Song> {
  val visible = songs.filterNot { it.song.hideFromQuickPicks || it.id in excludedSongIds }
  val allowedIds =
    visible
      .map { it.toCarouselSong() }
      .filterBlockedArtists(blockedArtists)
      .filterExplicit(hideExplicit)
      .filterVideoSongs(hideVideoSongs)
      .mapTo(mutableSetOf()) { it.id }
  return visible.filter { it.id in allowedIds }.distinctBy { it.id }
}
