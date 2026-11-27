package com.velmora.stepbyte.innertube.models.comment

import com.velmora.stepbyte.innertube.models.NavigationEndpoint
import com.velmora.stepbyte.innertube.models.Runs
import com.velmora.stepbyte.innertube.models.Thumbnails
import kotlinx.serialization.Serializable

@Serializable
data class CommentThreadRenderer(
  val comment: Comment? = null,
  val commentViewModel: CommentViewModelWrapper? = null,
  val replies: Replies? = null,
) {
  @Serializable
  data class CommentViewModelWrapper(
    val commentViewModel: CommentViewModel? = null,
  ) {
    @Serializable
    data class CommentViewModel(
      val commentId: String? = null,
      val commentKey: String? = null,
    )
  }

  @Serializable
  data class Comment(
    val commentRenderer: CommentRenderer? = null,
  )

  @Serializable
  data class Replies(
    val commentRepliesRenderer: CommentRepliesRenderer? = null,
  ) {
    @Serializable
    data class CommentRepliesRenderer(
      val contents: List<Content>? = null,
      val viewReplies: com.velmora.stepbyte.innertube.models.Button? = null,
      val hideReplies: com.velmora.stepbyte.innertube.models.Button? = null,
    ) {
      @Serializable
      data class Content(
        val continuationItemRenderer: ContinuationItemRenderer? = null,
      )
    }
  }
}

@Serializable
data class CommentRenderer(
  val authorText: Runs? = null,
  val authorThumbnail: Thumbnails? = null,
  val contentText: Runs? = null,
  val publishedTimeText: Runs? = null,
  val authorEndpoint: NavigationEndpoint? = null,
  val commentId: String? = null,
  val voteCount: Runs? = null,
  val voteStatus: String? = null,
  val replyCount: Int? = null,
)

@Serializable
data class ContinuationItemRenderer(
  val trigger: String? = null,
  val continuationEndpoint: ContinuationEndpoint? = null,
  val button: com.velmora.stepbyte.innertube.models.Button? = null,
) {
  @Serializable
  data class ContinuationEndpoint(
    val clickTrackingParams: String? = null,
    val continuationCommand: ContinuationCommand? = null,
  ) {
    @Serializable
    data class ContinuationCommand(
      val token: String? = null,
    )
  }
}
