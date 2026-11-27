package com.velmora.stepbyte.models

import com.velmora.stepbyte.innertube.models.YTItem

data class ItemsPage(
  val items: List<YTItem>,
  val continuation: String?,
)
