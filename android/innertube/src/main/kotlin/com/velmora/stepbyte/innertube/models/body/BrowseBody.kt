package com.velmora.stepbyte.innertube.models.body

import com.velmora.stepbyte.innertube.models.Context
import kotlinx.serialization.Serializable

@Serializable
data class BrowseBody(
  val context: Context,
  val browseId: String?,
  val params: String?,
  val continuation: String?,
)
