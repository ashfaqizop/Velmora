package com.velmora.stepbyte.velmora.player

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.util.Log
import androidx.annotation.OptIn
import androidx.media3.common.util.UnstableApi
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService

/**
 * The video's media session: lock screen and notification controls, and
 * playback that carries on with the screen off when "Keep playing in the
 * background" is on. Media3 shows the notification and holds the foreground
 * state while the video plays; the player itself is [VideoSession]'s.
 */
@OptIn(UnstableApi::class)
class VideoService : MediaSessionService() {
  private var session: MediaSession? = null

  override fun onCreate() {
    super.onCreate()
    val player = VideoSession.player ?: return
    val open =
      PendingIntent.getActivity(
        this,
        7,
        Intent(this, VideoPlayerActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
      )
    session = MediaSession.Builder(this, player).setId("velmora-video").setSessionActivity(open).build()
  }

  override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? = session

  override fun onTaskRemoved(rootIntent: Intent?) {
    // Swiped away from recents: stop, unless it's playing on purpose in the background.
    val p = VideoSession.player
    if (p == null || !p.playWhenReady || !PlayerPrefs.background(this)) {
      VideoSession.close()
      stopSelf()
    }
  }

  override fun onDestroy() {
    session?.run {
      release()
      session = null
    }
    super.onDestroy()
  }

  companion object {
    fun start(context: Context) {
      runCatching { context.startService(Intent(context, VideoService::class.java)) }.onFailure { Log.w("VelmoraPlayer", "media session service didn't start", it) }
    }

    fun stop(context: Context) {
      runCatching { context.stopService(Intent(context, VideoService::class.java)) }
    }
  }
}
