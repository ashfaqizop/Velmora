package com.velmora.stepbyte.innertube.models

import kotlinx.serialization.Serializable

@Serializable
data class TasteSignals(
  val videoId: String,
  val genres: List<String> = emptyList(),
  val moodTags: List<String> = emptyList(),
)
