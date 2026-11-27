package com.velmora.stepbyte.innertube.models.response

import com.velmora.stepbyte.innertube.models.Button
import com.velmora.stepbyte.innertube.models.Continuation
import com.velmora.stepbyte.innertube.models.GridRenderer
import com.velmora.stepbyte.innertube.models.Menu
import com.velmora.stepbyte.innertube.models.MusicDetailHeaderRenderer
import com.velmora.stepbyte.innertube.models.MusicEditablePlaylistDetailHeaderRenderer
import com.velmora.stepbyte.innertube.models.MusicShelfRenderer
import com.velmora.stepbyte.innertube.models.ResponseContext
import com.velmora.stepbyte.innertube.models.Runs
import com.velmora.stepbyte.innertube.models.SectionListRenderer
import com.velmora.stepbyte.innertube.models.SubscriptionButton
import com.velmora.stepbyte.innertube.models.Tabs
import com.velmora.stepbyte.innertube.models.ThumbnailRenderer
import kotlinx.serialization.Serializable

@Serializable
data class BrowseResponse(
  val contents: Contents?,
  val continuationContents: ContinuationContents?,
  val onResponseReceivedActions: List<ResponseAction>?,
  val header: Header?,
  val microformat: Microformat?,
  val responseContext: ResponseContext,
  val background: ThumbnailRenderer?,
) {
  @Serializable
  data class Contents(
    val singleColumnBrowseResultsRenderer: Tabs?,
    val sectionListRenderer: SectionListRenderer?,
    val twoColumnBrowseResultsRenderer: TwoColumnBrowseResultsRenderer?,
  )

  @Serializable
  data class TwoColumnBrowseResultsRenderer(
    val tabs: List<Tabs.Tab?>?,
    val secondaryContents: SecondaryContents?,
  )

  @Serializable
  data class SecondaryContents(
    val sectionListRenderer: SectionListRenderer?,
  )

  @Serializable
  data class ContinuationContents(
    val sectionListContinuation: SectionListContinuation?,
    val musicPlaylistShelfContinuation: MusicPlaylistShelfContinuation?,
    val gridContinuation: GridContinuation?,
    val musicShelfContinuation: MusicShelfRenderer?,
  ) {
    @Serializable
    data class SectionListContinuation(
      val contents: List<SectionListRenderer.Content> = emptyList(),
      val continuations: List<Continuation>?,
    )

    @Serializable
    data class MusicPlaylistShelfContinuation(
      val contents: List<MusicShelfRenderer.Content> = emptyList(),
      val continuations: List<Continuation>?,
    )

    @Serializable
    data class GridContinuation(
      val items: List<GridRenderer.Item> = emptyList(),
      val continuations: List<Continuation>?,
    )
  }

  @Serializable
  data class ResponseAction(
    val appendContinuationItemsAction: ContinuationItems?,
  ) {
    @Serializable
    data class ContinuationItems(
      val continuationItems: List<MusicShelfRenderer.Content>?,
    )
  }

  @Serializable
  data class Header(
    val musicImmersiveHeaderRenderer: MusicImmersiveHeaderRenderer?,
    val musicDetailHeaderRenderer: MusicDetailHeaderRenderer?,
    val musicEditablePlaylistDetailHeaderRenderer: MusicEditablePlaylistDetailHeaderRenderer?,
    val musicVisualHeaderRenderer: MusicVisualHeaderRenderer?,
    val musicHeaderRenderer: MusicHeaderRenderer?,
  ) {
    @Serializable
    data class MusicImmersiveHeaderRenderer(
      val title: Runs,
      val description: Runs?,
      val thumbnail: ThumbnailRenderer?,
      val playButton: Button?,
      val startRadioButton: Button?,
      val subscriptionButton: SubscriptionButton?,
      val menu: Menu,
      val subscriptionButton2: SubscriptionButton2?,
      val monthlyListenerCount: Runs? = null,
    ) {
      @Serializable
      data class SubscriptionButton2(
        val subscribeButtonRenderer: SubscribeButtonRenderer?,
      ) {
        @Serializable
        data class SubscribeButtonRenderer(
          val subscriberCountWithSubscribeText: Runs?,
        )
      }
    }

    @Serializable
    data class MusicVisualHeaderRenderer(
      val title: Runs,
      val foregroundThumbnail: ThumbnailRenderer,
      val thumbnail: ThumbnailRenderer?,
    )

    @Serializable
    data class Buttons(
      val menuRenderer: Menu.MenuRenderer?,
    )

    @Serializable
    data class MusicHeaderRenderer(
      val buttons: List<Buttons>?,
      val title: Runs?,
      val thumbnail: MusicThumbnailRenderer?,
      val subtitle: Runs?,
      val secondSubtitle: Runs?,
      val straplineTextOne: Runs?,
      val straplineThumbnail: MusicThumbnailRenderer?,
    )

    @Serializable
    data class MusicThumbnail(
      val url: String?,
    )

    @Serializable
    data class MusicThumbnailRenderer(
      val musicThumbnailRenderer: MusicThumbnailRenderer,
      val thumbnails: List<MusicThumbnail>?,
    )
  }

  @Serializable
  data class Microformat(
    val microformatDataRenderer: MicroformatDataRenderer?,
  ) {
    @Serializable
    data class MicroformatDataRenderer(
      val urlCanonical: String?,
    )
  }
}
