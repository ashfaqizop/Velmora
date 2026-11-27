package com.velmora.stepbyte.innertube.models.response

import kotlinx.serialization.Serializable

@Serializable
data class EditPlaylistResponse(
  val newHeader: BrowseResponse.Header?,
)
