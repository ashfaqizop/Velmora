package com.velmora.stepbyte.models

import com.velmora.stepbyte.innertube.models.YTItem
import com.velmora.stepbyte.db.entities.LocalItem

data class SimilarRecommendation(
  val title: LocalItem,
  val items: List<YTItem>,
)
