package com.velmora.stepbyte.innertube.pages

import com.velmora.stepbyte.innertube.models.SongItem

data class PlaylistContinuationPage(
  val songs: List<SongItem>,
  val continuation: String?,
)
