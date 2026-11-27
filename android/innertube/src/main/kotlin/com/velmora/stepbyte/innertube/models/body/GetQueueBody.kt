package com.velmora.stepbyte.innertube.models.body

import com.velmora.stepbyte.innertube.models.Context
import kotlinx.serialization.Serializable

@Serializable
data class GetQueueBody(
  val context: Context,
  val videoIds: List<String>?,
  val playlistId: String?,
)
