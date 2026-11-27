package com.velmora.stepbyte.innertube.models.body

import com.velmora.stepbyte.innertube.models.Context
import kotlinx.serialization.Serializable

@Serializable
data class SearchBody(
  val context: Context,
  val query: String?,
  val params: String?,
)
