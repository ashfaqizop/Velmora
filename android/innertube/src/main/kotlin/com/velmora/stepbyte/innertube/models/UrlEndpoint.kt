package com.velmora.stepbyte.innertube.models

import kotlinx.serialization.Serializable

@Serializable
data class UrlEndpoint(
  val url: String? = null,
  val target: String? = null,
)
