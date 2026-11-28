package com.velmora.stepbyte.velmora.player

import android.app.PendingIntent
import android.app.PictureInPictureParams
import android.app.RemoteAction
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.ActivityInfo
import android.content.pm.PackageManager
import android.content.res.Configuration
import android.graphics.drawable.Icon
import android.os.Build
import android.os.Bundle
import android.util.Rational
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.mutableStateOf
import androidx.core.content.ContextCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.Lifecycle
import com.velmora.stepbyte.R
import com.velmora.stepbyte.ui.theme.velmoraTheme

/**
 * Velmora's video player: full screen, landscape by default, with
 * picture-in-picture when you leave mid-video. The playback itself lives in
 * [VideoSession], so it survives this screen rotating or going to PiP.
 */
class VideoPlayerActivity : ComponentActivity() {
  /** Read by the screen: hide the controls in the PiP window. */
  val inPip = mutableStateOf(false)
  private var playing = false
  private var aspect = Rational(16, 9)

  private val actions =
    object : BroadcastReceiver() {
      override fun onReceive(context: Context, intent: Intent) {
        val p = VideoSession.player ?: return
        when (intent.getIntExtra(EXTRA, 0)) {
          PLAY_PAUSE -> if (p.isPlaying) p.pause() else p.play()
          BACK -> p.seekBack()
          FORWARD -> p.seekForward()
        }
      }
    }

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    if (VideoSession.ui.value.title == null) {
      finish()
      return
    }
    WindowCompat.setDecorFitsSystemWindows(window, false)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
      window.attributes.layoutInDisplayCutoutMode = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
    }
    window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
    immersive()
    if (savedInstanceState == null) requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
    ContextCompat.registerReceiver(this, actions, IntentFilter(ACTION), ContextCompat.RECEIVER_NOT_EXPORTED)

    setContent {
      velmoraTheme(darkTheme = true, pureBlack = true) {
        VideoPlayerScreen(activity = this, onClose = ::closePlayer)
      }
    }
  }

  fun immersive() {
    WindowInsetsControllerCompat(window, window.decorView).apply {
      hide(WindowInsetsCompat.Type.systemBars())
      systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
    }
  }

  /** Portrait and landscape, by the button. */
  fun toggleRotation() {
    val landscape = resources.configuration.orientation == Configuration.ORIENTATION_LANDSCAPE
    requestedOrientation = if (landscape) ActivityInfo.SCREEN_ORIENTATION_SENSOR_PORTRAIT else ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
  }

  /** The screen's brightness, 0..1, or the system's (negative). */
  fun setBrightness(v: Float) {
    window.attributes = window.attributes.apply { screenBrightness = v }
  }

  fun brightness(): Float = window.attributes.screenBrightness

  fun closePlayer() {
    VideoSession.close()
    finish()
  }

  // ── Picture-in-picture ──

  private val pipSupported: Boolean
    get() = Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && packageManager.hasSystemFeature(PackageManager.FEATURE_PICTURE_IN_PICTURE)

  /** The screen tells the activity what's playing, so PiP has the right shape and buttons. */
  fun updatePip(isPlaying: Boolean, width: Int, height: Int) {
    playing = isPlaying
    if (width > 0 && height > 0) {
      // Android refuses ratios outside 1:2.39 to 2.39:1.
      val r = (width.toFloat() / height).coerceIn(1f / 2.39f, 2.39f)
      aspect = Rational((r * 1000).toInt(), 1000)
    }
    if (pipSupported) runCatching { setPictureInPictureParams(pipParams()) }
  }

  private fun pipParams(): PictureInPictureParams {
    val b = PictureInPictureParams.Builder().setAspectRatio(aspect).setActions(pipActions())
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
      b.setAutoEnterEnabled(playing)
      b.setSeamlessResizeEnabled(true)
    }
    return b.build()
  }

  private fun pipActions(): List<RemoteAction> {
    fun action(code: Int, icon: Int, label: String) =
      RemoteAction(
        Icon.createWithResource(this, icon),
        label,
        label,
        PendingIntent.getBroadcast(this, code, Intent(ACTION).setPackage(packageName).putExtra(EXTRA, code), PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT),
      )
    return listOf(
      action(BACK, R.drawable.ic_replay_10, "Back 10 seconds"),
      if (playing) action(PLAY_PAUSE, R.drawable.pause, "Pause") else action(PLAY_PAUSE, R.drawable.play, "Play"),
      action(FORWARD, R.drawable.ic_forward_10, "Forward 10 seconds"),
    )
  }

  fun enterPip(): Boolean {
    if (!pipSupported) return false
    return runCatching { enterPictureInPictureMode(pipParams()) }.getOrDefault(false)
  }

  override fun onUserLeaveHint() {
    super.onUserLeaveHint()
    // Android 12+ enters by itself (setAutoEnterEnabled).
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S && playing) enterPip()
  }

  override fun onPictureInPictureModeChanged(isInPictureInPictureMode: Boolean, newConfig: Configuration) {
    super.onPictureInPictureModeChanged(isInPictureInPictureMode, newConfig)
    inPip.value = isInPictureInPictureMode
    // The PiP window was closed (not expanded): stop watching.
    if (!isInPictureInPictureMode && !lifecycle.currentState.isAtLeast(Lifecycle.State.STARTED)) {
      if (!PlayerPrefs.background(this)) closePlayer()
    }
    if (!isInPictureInPictureMode) immersive()
  }

  override fun onWindowFocusChanged(hasFocus: Boolean) {
    super.onWindowFocusChanged(hasFocus)
    if (hasFocus) immersive()
  }

  override fun onStop() {
    super.onStop()
    val inPipNow = Build.VERSION.SDK_INT >= Build.VERSION_CODES.N && isInPictureInPictureMode
    if (!inPipNow && !isFinishing && !PlayerPrefs.background(this)) VideoSession.player?.pause()
  }

  override fun onDestroy() {
    runCatching { unregisterReceiver(actions) }
    // Leaving for good stops the video, unless it's playing on purpose in the background.
    if (isFinishing && VideoSession.ui.value.title != null) {
      val keep = PlayerPrefs.background(this) && VideoSession.player?.isPlaying == true
      if (!keep) VideoSession.close()
    }
    super.onDestroy()
  }

  companion object {
    private const val ACTION = "com.velmora.stepbyte.VIDEO_PIP"
    private const val EXTRA = "what"
    private const val PLAY_PAUSE = 1
    private const val BACK = 2
    private const val FORWARD = 3
  }
}
