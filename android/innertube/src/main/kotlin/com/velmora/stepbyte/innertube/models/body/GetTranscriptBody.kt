package com.velmora.stepbyte.innertube.models.body

import com.velmora.stepbyte.innertube.models.Context
import kotlinx.serialization.Serializable

@Serializable
data class GetTranscriptBody(
  val context: Context,
  val params: String,
)
