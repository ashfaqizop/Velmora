package com.velmora.stepbyte.innertube.pages

import com.velmora.stepbyte.metadata.cleaner.ApiText
import com.velmora.stepbyte.innertube.models.AlbumItem
import com.velmora.stepbyte.innertube.models.MusicTwoRowItemRenderer

data class LibraryAlbumsPage(
  val albums: List<AlbumItem>,
  val continuation: String?,
) {
  companion object {
    fun fromMusicTwoRowItemRenderer(renderer: MusicTwoRowItemRenderer): AlbumItem? {
      return AlbumItem(
        browseId = renderer.navigationEndpoint.browseEndpoint?.browseId ?: return null,
        playlistId =
          renderer.thumbnailOverlay
            ?.musicItemThumbnailOverlayRenderer
            ?.content
            ?.musicPlayButtonRenderer
            ?.playNavigationEndpoint
            ?.watchPlaylistEndpoint
            ?.playlistId ?: return null,
        title = renderer.title.runs?.firstOrNull()?.text?.let(ApiText::clean) ?: return null,
        artists = null,
        year = renderer.subtitle?.runs?.lastOrNull()?.text?.toIntOrNull(),
        thumbnail =
          renderer.thumbnailRenderer.musicThumbnailRenderer?.getThumbnailUrl() ?: return null,
        explicit =
          renderer.subtitleBadges?.find {
            it.musicInlineBadgeRenderer?.icon?.iconType == "MUSIC_EXPLICIT_BADGE"
          } != null,
      )
    }
  }
}
