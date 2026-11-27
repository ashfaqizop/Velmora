package com.velmora.stepbyte.innertube.pages

import com.velmora.stepbyte.innertube.models.YTItem

data class LibraryContinuationPage(
  val items: List<YTItem>,
  val continuation: String?,
)
