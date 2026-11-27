package com.velmora.stepbyte.innertube.models.response

import com.velmora.stepbyte.innertube.models.PlaylistPanelRenderer
import kotlinx.serialization.Serializable

@Serializable
data class GetQueueResponse(
  val queueDatas: List<QueueData>,
) {
  @Serializable
  data class QueueData(
    val content: PlaylistPanelRenderer.Content,
  )
}
