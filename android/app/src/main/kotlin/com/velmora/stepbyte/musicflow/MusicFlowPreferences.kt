package com.velmora.stepbyte.musicflow

import androidx.datastore.preferences.core.Preferences
import com.velmora.stepbyte.constants.CanvasThumbnailAnimationKey
import com.velmora.stepbyte.constants.DataSaverEnabledKey
import com.velmora.stepbyte.constants.MusicFlowAnimatedBackgroundKey
import com.velmora.stepbyte.constants.MusicFlowAutoNextKey
import com.velmora.stepbyte.constants.MusicFlowAutoplayKey
import com.velmora.stepbyte.constants.MusicFlowCellularPrefetchKey
import com.velmora.stepbyte.constants.MusicFlowEnabledKey
import com.velmora.stepbyte.constants.MusicFlowHideRecentKey
import com.velmora.stepbyte.constants.MusicFlowLyricsKey

data class MusicFlowPreferences(
  val enabled: Boolean = true,
  val autoplay: Boolean = true,
  val autoNext: Boolean = true,
  val animatedBackground: Boolean = false,
  val lyrics: Boolean = true,
  val cellularPrefetch: Boolean = true,
  val hideRecent: Boolean = true,
  val dataSaver: Boolean = false,
) {
  companion object {
    fun from(preferences: Preferences) =
      MusicFlowPreferences(
        enabled = preferences[MusicFlowEnabledKey] ?: true,
        autoplay = preferences[MusicFlowAutoplayKey] ?: true,
        autoNext = preferences[MusicFlowAutoNextKey] ?: true,
        animatedBackground =
          preferences[MusicFlowAnimatedBackgroundKey]
            ?: preferences[CanvasThumbnailAnimationKey]
            ?: false,
        lyrics = preferences[MusicFlowLyricsKey] ?: true,
        cellularPrefetch = preferences[MusicFlowCellularPrefetchKey] ?: true,
        hideRecent = preferences[MusicFlowHideRecentKey] ?: true,
        dataSaver = preferences[DataSaverEnabledKey] ?: false,
      )
  }
}
