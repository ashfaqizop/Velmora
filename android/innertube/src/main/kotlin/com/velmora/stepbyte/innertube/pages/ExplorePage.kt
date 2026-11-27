package com.velmora.stepbyte.innertube.pages

import com.velmora.stepbyte.innertube.models.AlbumItem

data class ExplorePage(
  val newReleaseAlbums: List<AlbumItem>,
  val moodAndGenres: List<MoodAndGenres.Item>,
)
