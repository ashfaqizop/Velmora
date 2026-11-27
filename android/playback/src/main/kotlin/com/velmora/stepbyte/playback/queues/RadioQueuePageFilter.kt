package com.velmora.stepbyte.playback.queues

import com.velmora.stepbyte.innertube.models.SongItem

internal class RadioQueuePageFilter {
  private val seenSongIds = mutableSetOf<String>()

  fun reset() = seenSongIds.clear()

  fun filter(songs: List<SongItem>): List<SongItem> = songs.filter { seenSongIds.add(it.id) }
}
