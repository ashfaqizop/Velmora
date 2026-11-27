package com.velmora.stepbyte.innertube.models.body

import com.velmora.stepbyte.innertube.models.Context
import kotlinx.serialization.Serializable

@Serializable
data class FeedbackBody(
  val context: Context,
  val feedbackTokens: List<String>,
  val isFeedbackTokenUnencrypted: Boolean = false,
  val shouldMerge: Boolean = false,
)
