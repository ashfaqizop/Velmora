package com.velmora.stepbyte.velmora.player

import android.content.Context
import android.media.AudioManager
import android.view.ViewGroup
import androidx.activity.compose.BackHandler
import androidx.annotation.OptIn
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.AspectRatio
import androidx.compose.material.icons.rounded.Brightness6
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material.icons.rounded.FastForward
import androidx.compose.material.icons.rounded.FastRewind
import androidx.compose.material.icons.rounded.Forward10
import androidx.compose.material.icons.rounded.Favorite
import androidx.compose.material.icons.rounded.FavoriteBorder
import androidx.compose.material.icons.rounded.Forum
import androidx.compose.material.icons.rounded.Lock
import androidx.compose.material.icons.rounded.LockOpen
import androidx.compose.material.icons.rounded.Pause
import androidx.compose.material.icons.rounded.PictureInPictureAlt
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Replay
import androidx.compose.material.icons.rounded.Replay10
import androidx.compose.material.icons.rounded.ScreenRotation
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.material.icons.rounded.SignalCellularAlt
import androidx.compose.material.icons.rounded.SkipNext
import androidx.compose.material.icons.rounded.Subtitles
import androidx.compose.material.icons.rounded.VideoLibrary
import androidx.compose.material.icons.rounded.VolumeUp
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.C
import androidx.media3.common.Player
import androidx.media3.common.VideoSize
import androidx.media3.common.util.UnstableApi
import androidx.media3.ui.AspectRatioFrameLayout
import androidx.media3.ui.CaptionStyleCompat
import androidx.media3.ui.PlayerView
import coil3.compose.AsyncImage
import com.velmora.stepbyte.engine.Target
import com.velmora.stepbyte.velmora.shell.Brand
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

private val NoSites = kotlinx.coroutines.flow.MutableStateFlow(emptyList<FoundSite>())
private val NoSources = kotlinx.coroutines.flow.MutableStateFlow(emptyList<com.velmora.stepbyte.engine.Source>())

/** Which sheet is open over the video. */
enum class Sheet { None, Settings, Sources, Subtitles, Audio, Quality, Speed, Episodes }

private val RESIZE = listOf(AspectRatioFrameLayout.RESIZE_MODE_FIT to "Fit", AspectRatioFrameLayout.RESIZE_MODE_ZOOM to "Zoom", AspectRatioFrameLayout.RESIZE_MODE_FILL to "Stretch")

/** The player's live numbers, polled a few times a second while the screen is up. */
class Clock {
  var position by mutableLongStateOf(0L)
  var duration by mutableLongStateOf(0L)
  var buffered by mutableLongStateOf(0L)
  var playing by mutableStateOf(false)
  var buffering by mutableStateOf(false)
  var firstFrame by mutableStateOf(false)
  var videoSize by mutableStateOf(VideoSize.UNKNOWN)
}

@OptIn(UnstableApi::class)
@Composable
fun VideoPlayerScreen(activity: VideoPlayerActivity, onClose: () -> Unit) {
  val context = LocalContext.current
  val ui by VideoSession.ui.collectAsState()
  val inPip by activity.inPip
  val clock = remember { Clock() }
  var sheet by remember { mutableStateOf(Sheet.None) }
  var showControls by remember { mutableStateOf(true) }
  var locked by remember { mutableStateOf(false) }
  var resize by remember { mutableIntStateOf(0) }
  var flash by remember { mutableStateOf<String?>(null) }
  var zoom by remember { mutableFloatStateOf(1f) }
  var chatLayout by remember { mutableStateOf(ChatLayout.Overlay) }
  val player = VideoSession.player
  val show = ui.theatre
  val landscape = androidx.compose.ui.platform.LocalConfiguration.current.orientation == android.content.res.Configuration.ORIENTATION_LANDSCAPE
  // The chat beside the film (or under it, upright) takes room from the video.
  val sidebar = show != null && chatLayout == ChatLayout.Sidebar && !inPip && !ui.held && ui.phase != Phase.Ended
  val videoPad = if (!sidebar) Modifier else if (landscape) Modifier.padding(end = 340.dp) else Modifier.padding(bottom = 300.dp)

  // Live numbers and the events that matter to the screen.
  DisposableEffect(player) {
    val p = player
    val l =
      object : Player.Listener {
        override fun onIsPlayingChanged(isPlaying: Boolean) {
          clock.playing = isPlaying
        }

        override fun onPlaybackStateChanged(state: Int) {
          clock.buffering = state == Player.STATE_BUFFERING
        }

        override fun onRenderedFirstFrame() {
          clock.firstFrame = true
        }

        override fun onVideoSizeChanged(videoSize: VideoSize) {
          clock.videoSize = videoSize
        }
      }
    p?.addListener(l)
    onDispose { p?.removeListener(l) }
  }
  LaunchedEffect(player) {
    while (isActive) {
      val p = VideoSession.player
      if (p != null) {
        clock.position = p.currentPosition
        clock.duration = p.duration.takeIf { it != C.TIME_UNSET && it > 0 } ?: 0L
        clock.buffered = p.bufferedPosition
        clock.playing = p.isPlaying
        clock.buffering = p.playbackState == Player.STATE_BUFFERING
      }
      delay(250)
    }
  }
  // A new source starts without a picture.
  LaunchedEffect(ui.current) { clock.firstFrame = false }
  LaunchedEffect(clock.playing, clock.videoSize) { activity.updatePip(clock.playing, clock.videoSize.width, clock.videoSize.height) }
  // Controls hide themselves while playing.
  LaunchedEffect(showControls, clock.playing, sheet) {
    if (showControls && clock.playing && sheet == Sheet.None) {
      delay(3_500)
      showControls = false
    }
  }
  LaunchedEffect(flash) {
    if (flash != null) {
      delay(900)
      flash = null
    }
  }

  BackHandler {
    when {
      sheet != Sheet.None -> sheet = Sheet.None
      locked -> showControls = true
      else -> onClose()
    }
  }

  Box(Modifier.fillMaxSize().background(Color.Black)) {
    // The picture.
    AndroidView(
      factory = { ctx ->
        PlayerView(ctx).apply {
          useController = false
          layoutParams = ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT)
          setShutterBackgroundColor(android.graphics.Color.BLACK)
          setKeepContentOnPlayerReset(true)
          subtitleView?.apply {
            setStyle(CaptionStyleCompat(android.graphics.Color.WHITE, android.graphics.Color.TRANSPARENT, android.graphics.Color.TRANSPARENT, CaptionStyleCompat.EDGE_TYPE_OUTLINE, android.graphics.Color.BLACK, null))
            setFractionalTextSize(0.0533f * PlayerPrefs.subtitleScale(ctx))
            setApplyEmbeddedStyles(false)
          }
        }
      },
      update = { v ->
        if (v.player !== VideoSession.player) v.player = VideoSession.player
        v.resizeMode = RESIZE[resize].first
        v.subtitleView?.setFractionalTextSize(0.0533f * PlayerPrefs.subtitleScale(v.context))
      },
      modifier = Modifier.fillMaxSize().then(videoPad).graphicsLayer {
        scaleX = zoom
        scaleY = zoom
      },
    )

    val watching = ui.phase == Phase.Playing && clock.firstFrame

    if (!inPip && watching && !(show != null && ui.held)) {
      Gestures(
        clock = clock,
        activity = activity,
        locked = locked,
        onTap = { showControls = !showControls },
        onZoom = { z ->
          zoom = z
          if (z > 1.04f) flash = "Zoom ${(z * 100).toInt()}%"
        },
        onFlash = { flash = it },
      )
    }

    // Spinner while a playing video waits for data.
    if (watching && clock.buffering) {
      BufferingBadge(ui, Modifier.align(Alignment.Center))
    }

    if (!watching && !inPip && show == null) {
      StatusScreen(ui, onClose = onClose, onSheet = { sheet = it })
    }

    if (!inPip && watching && !(show != null && (ui.held || ui.phase == Phase.Ended))) {
      AnimatedVisibility(visible = showControls && !locked, enter = fadeIn(tween(180)), exit = fadeOut(tween(260)), modifier = videoPad) {
        Controls(
          ui = ui,
          clock = clock,
          onBack = onClose,
          onSheet = { sheet = it },
          onLock = {
            locked = true
            showControls = false
            flash = "Locked. Tap the lock to unlock"
          },
          onResize = {
            resize = (resize + 1) % RESIZE.size
            zoom = 1f
            flash = RESIZE[resize].second
          },
          onRotate = { activity.toggleRotation() },
          onPip = { activity.enterPip() },
          onInteract = { showControls = true },
          onChat =
            if (show == null) null
            else {
              {
                chatLayout = ChatLayout.entries[(chatLayout.ordinal + 1) % ChatLayout.entries.size]
                flash = chatLayout.label
              }
            },
          chatLabel = if (chatLayout == ChatLayout.Hidden) "Show chat" else "Chat",
        )
      }
      AnimatedVisibility(visible = locked && showControls, enter = fadeIn(), exit = fadeOut(), modifier = Modifier.align(Alignment.CenterStart)) {
        Surface(onClick = {
          locked = false
          showControls = true
        }, shape = CircleShape, color = Color.Black.copy(alpha = 0.5f), modifier = Modifier.padding(24.dp).size(56.dp)) {
          Box(contentAlignment = Alignment.Center) { Icon(Icons.Rounded.LockOpen, contentDescription = "Unlock", tint = Color.White) }
        }
      }
      // Locked: a tap just shows the unlock button for a moment.
      if (locked) {
        LaunchedEffect(showControls) {
          if (showControls) {
            delay(2_500)
            showControls = false
          }
        }
      }
    }

    // Theatre: reactions, the behind pill and the chat over the film; the
    // lobby before it (lifting away at the start), the end card after.
    if (show != null && !inPip) {
      if (watching && !ui.held && ui.phase != Phase.Ended) {
        FloatingReactions(videoPad)
        BehindPill(ui, Modifier.align(Alignment.TopCenter).padding(top = 72.dp))
        if (chatLayout == ChatLayout.Overlay) {
          ChatPanel(show, clock, inLobby = false, overlay = true, modifier = Modifier.align(Alignment.BottomStart).padding(bottom = 120.dp).width(340.dp).height(200.dp))
        }
        if (sidebar) {
          ChatPanel(
            show,
            clock,
            inLobby = false,
            modifier =
              (if (landscape) Modifier.align(Alignment.CenterEnd).width(340.dp).fillMaxHeight() else Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(300.dp))
                .background(Color(0xF2101318)),
          )
        }
      }
      AnimatedVisibility(
        visible = ui.held || (!watching && ui.phase != Phase.Ended),
        enter = fadeIn(tween(300)),
        exit = slideOutVertically(tween(650)) { -it } + fadeOut(tween(650)),
      ) {
        TheatreLobby(ui, show, clock, onClose = onClose, onSources = { sheet = Sheet.Sources })
      }
      if (ui.phase == Phase.Ended) TheatreEnd(show, clock, onClose = onClose)
    }

    // A short label in the middle: "Zoom", "1.5x", "Locked".
    AnimatedVisibility(visible = flash != null, enter = fadeIn() + scaleIn(initialScale = 0.9f), exit = fadeOut(), modifier = Modifier.align(Alignment.TopCenter).padding(top = 48.dp)) {
      Surface(shape = RoundedCornerShape(50), color = Color.Black.copy(alpha = 0.6f)) {
        Text(flash ?: "", color = Color.White, style = MaterialTheme.typography.labelLarge, modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp))
      }
    }

    // Next episode, near the end.
    val next = ui.next
    if (!inPip && next != null && ui.phase == Phase.Playing && show == null && !ui.live) {
      NextEpisodeCard(
        label = "S${next.season} · E${next.number}" + (next.title?.takeIf { it.isNotBlank() }?.let { " · $it" } ?: ""),
        thumbnail = next.thumbnail ?: ui.title?.value?.backdrop,
        ended = VideoSession.player?.playbackState == Player.STATE_ENDED,
        onPlay = { VideoSession.playNext() },
        onDismiss = { VideoSession.dismissNext() },
        modifier = Modifier.align(Alignment.BottomEnd),
      )
    }

    if (!inPip) {
      PlayerSheets(sheet = sheet, ui = ui, onSheet = { sheet = it }, onFlash = { flash = it })
    }
  }
}

// ── Gestures ───────────────────────────────────────────────────────────

/**
 * Tap shows the controls; double-tap a side skips 10 s (more taps add up);
 * drag sideways to seek; drag up and down on the left for brightness and on
 * the right for volume; pinch to zoom in on the picture.
 */
@Composable
private fun Gestures(clock: Clock, activity: VideoPlayerActivity, locked: Boolean, onTap: () -> Unit, onZoom: (Float) -> Unit, onFlash: (String) -> Unit) {
  val context = LocalContext.current
  val haptics = LocalHapticFeedback.current
  val scope = rememberCoroutineScope()
  val audio = remember { context.getSystemService(Context.AUDIO_SERVICE) as AudioManager }
  var skip by remember { mutableIntStateOf(0) }
  var skipSide by remember { mutableIntStateOf(0) }
  var skipJob by remember { mutableStateOf<Job?>(null) }
  var seekPreview by remember { mutableStateOf<Long?>(null) }
  var level by remember { mutableStateOf<Pair<ImageVector, Float>?>(null) }
  var zoom by remember { mutableFloatStateOf(1f) }

  BoxWithConstraints(Modifier.fillMaxSize()) {
    val width = constraints.maxWidth.toFloat()
    val height = constraints.maxHeight.toFloat()
    Box(
      Modifier.fillMaxSize()
        .pointerInput(locked) {
          detectTapGestures(
            onTap = { onTap() },
            onDoubleTap = { at ->
              if (locked) return@detectTapGestures
              val side = if (at.x < width / 2) -1 else 1
              val p = VideoSession.player ?: return@detectTapGestures
              if (side != skipSide) skip = 0
              skipSide = side
              skip += 10
              VideoSession.seekTo((p.currentPosition + side * 10_000L).coerceIn(0L, p.duration.coerceAtLeast(0L)))
              haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
              skipJob?.cancel()
              skipJob =
                scope.launch {
                  delay(700)
                  skip = 0
                  skipSide = 0
                }
            },
          )
        }
        .pointerInput(locked) {
          if (locked) return@pointerInput
          var axis = 0 // 0 undecided, 1 horizontal, 2 vertical
          var startPos = 0L
          var startLevel = 0f
          var leftSide = true
          var dx = 0f
          var dy = 0f
          detectDragGestures(
            onDragStart = { at ->
              axis = 0
              dx = 0f
              dy = 0f
              leftSide = at.x < width / 2
              startPos = VideoSession.player?.currentPosition ?: 0L
            },
            onDragEnd = {
              seekPreview?.let { VideoSession.seekTo(it) }
              seekPreview = null
              level = null
            },
            onDragCancel = {
              seekPreview = null
              level = null
            },
          ) { change, amount ->
            change.consume()
            dx += amount.x
            dy += amount.y
            if (axis == 0 && (kotlin.math.abs(dx) > 24 || kotlin.math.abs(dy) > 24)) {
              axis = if (kotlin.math.abs(dx) > kotlin.math.abs(dy)) 1 else 2
              startLevel =
                if (leftSide) {
                  activity.brightness().takeIf { it >= 0 } ?: 0.5f
                } else {
                  audio.getStreamVolume(AudioManager.STREAM_MUSIC).toFloat() / audio.getStreamMaxVolume(AudioManager.STREAM_MUSIC).coerceAtLeast(1)
                }
            }
            when (axis) {
              1 -> {
                // A full swipe across is 90 seconds.
                val dur = clock.duration
                if (dur > 0) seekPreview = (startPos + (dx / width * 90_000).toLong()).coerceIn(0L, dur)
              }
              2 -> {
                val v = (startLevel - dy / (height * 0.8f)).coerceIn(0f, 1f)
                if (leftSide) {
                  activity.setBrightness(v.coerceAtLeast(0.01f))
                  level = Icons.Rounded.Brightness6 to v
                } else {
                  val max = audio.getStreamMaxVolume(AudioManager.STREAM_MUSIC)
                  audio.setStreamVolume(AudioManager.STREAM_MUSIC, (v * max).toInt(), 0)
                  level = Icons.Rounded.VolumeUp to v
                }
              }
            }
          }
        }
        .pointerInput(locked) {
          if (locked) return@pointerInput
          // Two fingers only, so one-finger drags still seek and set levels.
          awaitEachGesture {
            awaitFirstDown(requireUnconsumed = false)
            do {
              val event = awaitPointerEvent()
              if (event.changes.count { it.pressed } >= 2) {
                val z = event.calculateZoom()
                if (z != 1f) {
                  zoom = (zoom * z).coerceIn(1f, 2.5f)
                  onZoom(zoom)
                  event.changes.forEach { it.consume() }
                }
              }
            } while (event.changes.any { it.pressed })
          }
        },
    ) {
      // Double-tap feedback: a soft half-circle on that side with the seconds.
      if (skip > 0) {
        Box(
          Modifier.align(if (skipSide < 0) Alignment.CenterStart else Alignment.CenterEnd)
            .fillMaxHeight()
            .width(with(androidx.compose.ui.platform.LocalDensity.current) { (width * 0.38f).toDp() })
            .clip(RoundedCornerShape(if (skipSide < 0) 0 else 50, if (skipSide < 0) 50 else 0, if (skipSide < 0) 50 else 0, if (skipSide < 0) 0 else 50))
            .background(Color.White.copy(alpha = 0.12f)),
          contentAlignment = Alignment.Center,
        ) {
          Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Icon(if (skipSide < 0) Icons.Rounded.FastRewind else Icons.Rounded.FastForward, contentDescription = null, tint = Color.White, modifier = Modifier.size(40.dp))
            Text("${skip}s", color = Color.White, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
          }
        }
      }
      seekPreview?.let { at ->
        Surface(shape = RoundedCornerShape(20.dp), color = Color.Black.copy(alpha = 0.65f), modifier = Modifier.align(Alignment.Center)) {
          Column(Modifier.padding(horizontal = 22.dp, vertical = 14.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            Text(time(at), color = Color.White, style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
            val diff = (at - clock.position) / 1000
            Text((if (diff >= 0) "+" else "−") + time(kotlin.math.abs(diff) * 1000), color = Color.White.copy(alpha = 0.7f), style = MaterialTheme.typography.labelLarge)
          }
        }
      }
      level?.let { (icon, v) ->
        LevelPill(icon, v, Modifier.align(Alignment.Center))
      }
    }
  }
}

@Composable
private fun LevelPill(icon: ImageVector, value: Float, modifier: Modifier) {
  Surface(shape = RoundedCornerShape(24.dp), color = Color.Black.copy(alpha = 0.65f), modifier = modifier) {
    Row(Modifier.padding(horizontal = 18.dp, vertical = 12.dp), verticalAlignment = Alignment.CenterVertically) {
      Icon(icon, contentDescription = null, tint = Color.White)
      Spacer(Modifier.width(12.dp))
      Box(Modifier.width(140.dp).height(6.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.25f))) {
        Box(Modifier.fillMaxWidth(value).fillMaxHeight().clip(CircleShape).background(Color.White))
      }
      Spacer(Modifier.width(12.dp))
      Text("${(value * 100).toInt()}", color = Color.White, style = MaterialTheme.typography.labelLarge, modifier = Modifier.widthIn(min = 28.dp))
    }
  }
}

// ── Controls ───────────────────────────────────────────────────────────

@Composable
private fun Controls(
  ui: PlaybackUi,
  clock: Clock,
  onBack: () -> Unit,
  onSheet: (Sheet) -> Unit,
  onLock: () -> Unit,
  onResize: () -> Unit,
  onRotate: () -> Unit,
  onPip: () -> Unit,
  onInteract: () -> Unit,
  onChat: (() -> Unit)? = null,
  chatLabel: String = "Chat",
) {
  val title = ui.title?.value
  Box(
    Modifier.fillMaxSize()
      .background(Brush.verticalGradient(0f to Color.Black.copy(alpha = 0.7f), 0.25f to Color.Transparent, 0.65f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.8f)))
      .windowInsetsPadding(WindowInsets.displayCutout),
  ) {
    // Top: back, name, episode and source, then the tools.
    Row(Modifier.fillMaxWidth().padding(horizontal = 8.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
      IconButton(onClick = onBack) { Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = "Back", tint = Color.White) }
      Column(Modifier.weight(1f).padding(horizontal = 4.dp)) {
        Text(title?.name ?: "", color = Color.White, fontFamily = Brand.Serif, fontSize = 24.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        val sub = listOfNotNull(VideoSession.episodeLabel(ui.target), ui.current?.let { SourceLabels.title(it) }).joinToString("  ·  ")
        if (sub.isNotEmpty()) Text(sub, color = Color.White.copy(alpha = 0.7f), style = MaterialTheme.typography.labelMedium, maxLines = 1, overflow = TextOverflow.Ellipsis)
      }
      TopTool(Icons.Rounded.Subtitles, "Subtitles") { onSheet(Sheet.Subtitles) }
      TopTool(Icons.Rounded.PictureInPictureAlt, "Picture-in-picture", onPip)
      TopTool(Icons.Rounded.Settings, "Settings") { onSheet(Sheet.Settings) }
    }

    // Middle: back 10, play or pause, forward 10.
    Row(Modifier.align(Alignment.Center), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(36.dp)) {
      if (!ui.live) {
        RoundTool(Icons.Rounded.Replay10, "Back 10 seconds", 52.dp) {
          VideoSession.seekTo((VideoSession.player?.currentPosition ?: 0L) - 10_000L)
          onInteract()
        }
      }
      PlayPause(clock.playing) {
        VideoSession.player?.let { if (it.isPlaying) it.pause() else it.play() }
        onInteract()
      }
      if (!ui.live) {
        RoundTool(Icons.Rounded.Forward10, "Forward 10 seconds", 52.dp) {
          VideoSession.seekTo((VideoSession.player?.currentPosition ?: 0L) + 10_000L)
          onInteract()
        }
      }
    }

    // Bottom: the bar, then the tools.
    Column(Modifier.align(Alignment.BottomCenter).fillMaxWidth().padding(horizontal = 16.dp, vertical = 10.dp)) {
      if (ui.live) {
        LiveRow()
      } else {
        Row(verticalAlignment = Alignment.CenterVertically) {
          Text(time(clock.position), color = Color.White, style = MaterialTheme.typography.labelLarge)
          SeekBar(clock, Modifier.weight(1f).padding(horizontal = 12.dp), onInteract)
          Text(if (clock.duration > 0) "−" + time(clock.duration - clock.position) else "--:--", color = Color.White, style = MaterialTheme.typography.labelLarge)
        }
      }
      Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        BottomTool(Icons.Rounded.Lock, "Lock", onLock)
        BottomTool(Icons.Rounded.AspectRatio, "Fit", onResize)
        BottomTool(Icons.Rounded.ScreenRotation, "Rotate", onRotate)
        Spacer(Modifier.weight(1f))
        if (onChat != null) BottomTool(Icons.Rounded.Forum, chatLabel, onChat)
        ui.channel?.let { ch -> FavoriteTool(ch) }
        BottomTool(Icons.Rounded.SignalCellularAlt, if (ui.live) "Streams" else "Sources") { onSheet(Sheet.Sources) }
        if (ui.theatre == null && !ui.live && ui.title?.value?.episodic == true && ui.seasons.isNotEmpty()) {
          BottomTool(Icons.Rounded.VideoLibrary, "Episodes") { onSheet(Sheet.Episodes) }
        }
        if (ui.theatre == null && !ui.live && VideoSession.nextEpisode() != null) {
          BottomTool(Icons.Rounded.SkipNext, "Next") { VideoSession.playNext() }
        }
      }
    }
  }
}

/** Live TV: the LIVE mark, and Go live when the stream has drifted back. */
@OptIn(UnstableApi::class)
@Composable
private fun LiveRow() {
  var offset by remember { mutableLongStateOf(0L) }
  LaunchedEffect(Unit) {
    while (true) {
      val p = VideoSession.player
      offset = if (p != null && p.isCurrentMediaItemLive) p.currentLiveOffset.takeIf { it != C.TIME_UNSET } ?: 0L else 0L
      delay(1_000)
    }
  }
  val behind = offset > 25_000
  Row(Modifier.padding(vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
    Surface(shape = RoundedCornerShape(50), color = if (behind) Color.White.copy(alpha = 0.2f) else Color(0xFFE53935)) {
      Text("LIVE", color = Color.White, style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.Bold, modifier = Modifier.padding(horizontal = 10.dp, vertical = 3.dp))
    }
    if (behind) {
      Spacer(Modifier.width(10.dp))
      TextButton(onClick = { VideoSession.goLive() }, colors = ButtonDefaults.textButtonColors(contentColor = Color.White)) { Text("Go live · ${time(offset)} behind") }
    }
  }
}

/** Live TV: keep this channel in favourites. */
@Composable
private fun FavoriteTool(ch: com.velmora.stepbyte.engine.TvChannel) {
  val context = LocalContext.current
  val favorites by LiveTv.favorites.collectAsState()
  LaunchedEffect(Unit) { LiveTv.load(context) }
  val on = favorites.any { it.id == ch.id }
  TextButton(onClick = { LiveTv.toggleFavorite(context, ch) }, colors = ButtonDefaults.textButtonColors(contentColor = Color.White)) {
    Icon(if (on) Icons.Rounded.Favorite else Icons.Rounded.FavoriteBorder, contentDescription = null, tint = if (on) Color(0xFFFF6B8B) else Color.White, modifier = Modifier.size(20.dp))
    Spacer(Modifier.width(6.dp))
    Text(if (on) "Favourite" else "Add favourite", style = MaterialTheme.typography.labelLarge)
  }
}

@Composable
private fun TopTool(icon: ImageVector, label: String, onClick: () -> Unit) {
  IconButton(onClick = onClick) { Icon(icon, contentDescription = label, tint = Color.White) }
}

@Composable
private fun BottomTool(icon: ImageVector, label: String, onClick: () -> Unit) {
  TextButton(onClick = onClick, colors = ButtonDefaults.textButtonColors(contentColor = Color.White)) {
    Icon(icon, contentDescription = null, modifier = Modifier.size(20.dp))
    Spacer(Modifier.width(6.dp))
    Text(label, style = MaterialTheme.typography.labelLarge)
  }
}

@Composable
private fun RoundTool(icon: ImageVector, label: String, size: androidx.compose.ui.unit.Dp, onClick: () -> Unit) {
  Surface(onClick = onClick, shape = CircleShape, color = Color.Black.copy(alpha = 0.35f), modifier = Modifier.size(size)) {
    Box(contentAlignment = Alignment.Center) { Icon(icon, contentDescription = label, tint = Color.White, modifier = Modifier.size(size * 0.55f)) }
  }
}

/** The big button: the icon pops as it changes, and the shape squares off while paused. */
@Composable
private fun PlayPause(playing: Boolean, onClick: () -> Unit) {
  val corner by animateFloatAsState(if (playing) 50f else 30f, spring(dampingRatio = Spring.DampingRatioMediumBouncy), label = "corner")
  Surface(onClick = onClick, shape = RoundedCornerShape(corner.toInt()), color = Color.White, modifier = Modifier.size(76.dp)) {
    Box(contentAlignment = Alignment.Center) {
      AnimatedContent(
        targetState = playing,
        transitionSpec = { (scaleIn(spring(dampingRatio = Spring.DampingRatioMediumBouncy), initialScale = 0.5f) + fadeIn()) togetherWith (scaleOut(targetScale = 0.5f) + fadeOut(tween(90))) },
        label = "playpause",
      ) { on ->
        Icon(if (on) Icons.Rounded.Pause else Icons.Rounded.PlayArrow, contentDescription = if (on) "Pause" else "Play", tint = Color.Black, modifier = Modifier.size(42.dp))
      }
    }
  }
}

/** The seek bar: buffered and played, a thumb that grows while held, and the time it would land on. */
@Composable
private fun SeekBar(clock: Clock, modifier: Modifier, onInteract: () -> Unit) {
  var dragging by remember { mutableStateOf<Float?>(null) }
  val thumb by animateFloatAsState(if (dragging != null) 9f else 6f, label = "thumb")
  val dur = clock.duration
  val played = dragging ?: if (dur > 0) clock.position.toFloat() / dur else 0f
  val buffered = if (dur > 0) clock.buffered.toFloat() / dur else 0f
  Box(modifier.height(36.dp), contentAlignment = Alignment.Center) {
    Canvas(
      Modifier.fillMaxWidth()
        .height(36.dp)
        .pointerInput(dur) {
          detectTapGestures { at ->
            if (dur > 0) VideoSession.seekTo((at.x / size.width * dur).toLong())
            onInteract()
          }
        }
        .pointerInput(dur) {
          detectHorizontalDragGestures(
            onDragStart = { at -> dragging = (at.x / size.width).coerceIn(0f, 1f) },
            onDragEnd = {
              dragging?.let { f -> if (dur > 0) VideoSession.seekTo((f * dur).toLong()) }
              dragging = null
            },
            onDragCancel = { dragging = null },
          ) { change, amount ->
            change.consume()
            dragging = ((dragging ?: 0f) + amount / size.width).coerceIn(0f, 1f)
            onInteract()
          }
        },
    ) {
      val h = 4.dp.toPx()
      val y = size.height / 2
      val r = CornerRadius(h / 2, h / 2)
      drawRoundRect(Color.White.copy(alpha = 0.22f), Offset(0f, y - h / 2), Size(size.width, h), r)
      drawRoundRect(Color.White.copy(alpha = 0.4f), Offset(0f, y - h / 2), Size(size.width * buffered.coerceIn(0f, 1f), h), r)
      drawRoundRect(Brand.Blue, Offset(0f, y - h / 2), Size(size.width * played.coerceIn(0f, 1f), h), r)
      drawCircle(Color.White, thumb.dp.toPx(), Offset(size.width * played.coerceIn(0f, 1f), y))
    }
    dragging?.let { f ->
      if (dur > 0) {
        Surface(shape = RoundedCornerShape(10.dp), color = Color.Black.copy(alpha = 0.7f), modifier = Modifier.align(Alignment.TopCenter).graphicsLayer { translationY = -36.dp.toPx() }) {
          Text(time((f * dur).toLong()), color = Color.White, style = MaterialTheme.typography.labelLarge, modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp))
        }
      }
    }
  }
}

fun time(ms: Long): String {
  val s = (ms / 1000).coerceAtLeast(0)
  val h = s / 3600
  val m = (s % 3600) / 60
  val sec = s % 60
  return if (h > 0) "%d:%02d:%02d".format(h, m, sec) else "%d:%02d".format(m, sec)
}

// ── Before the picture: finding, opening, asking, failing ──────────────

@kotlin.OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun StatusScreen(ui: PlaybackUi, onClose: () -> Unit, onSheet: (Sheet) -> Unit) {
  val title = ui.title?.value
  val finding = ui.finding
  val sites by (finding?.sites ?: NoSites).collectAsState()
  val found by (finding?.sources ?: NoSources).collectAsState()

  Box(Modifier.fillMaxSize()) {
    AsyncImage(
      model = title?.backdrop ?: title?.poster,
      contentDescription = null,
      contentScale = ContentScale.Crop,
      modifier = Modifier.fillMaxSize().graphicsLayer { alpha = 0.35f },
    )
    Box(Modifier.fillMaxSize().background(Brush.radialGradient(listOf(Color.Black.copy(alpha = 0.3f), Color.Black.copy(alpha = 0.92f)))))
    IconButton(onClick = onClose, modifier = Modifier.padding(8.dp).windowInsetsPadding(WindowInsets.displayCutout)) {
      Icon(Icons.Rounded.Close, contentDescription = "Close", tint = Color.White)
    }
    Column(
      Modifier.align(Alignment.Center).padding(horizontal = 32.dp).widthIn(max = 560.dp),
      horizontalAlignment = Alignment.CenterHorizontally,
      verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
      Text(title?.name ?: "", color = Color.White, fontFamily = Brand.Serif, fontSize = 34.sp, lineHeight = 36.sp, textAlign = TextAlign.Center, maxLines = 2, overflow = TextOverflow.Ellipsis)
      VideoSession.episodeLabel(ui.target)?.let { Text(it, color = Color.White.copy(alpha = 0.7f), style = MaterialTheme.typography.bodyMedium, textAlign = TextAlign.Center) }

      AnimatedContent(targetState = ui.phase, transitionSpec = { (fadeIn(tween(220)) + slideInVertically { it / 8 }) togetherWith (fadeOut(tween(120)) + slideOutVertically { -it / 8 }) }, label = "status") { phase ->
        Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(12.dp)) {
          when (phase) {
            Phase.Finding -> {
              LoadingIndicator(modifier = Modifier.size(64.dp), color = Brand.Blue)
              val answered = sites.count { it.state != FoundSite.State.Looking }
              Text(
                if (sites.isEmpty()) "Asking every site…" else "Looking on ${sites.size} sites · $answered answered · ${found.size} sources",
                color = Color.White,
                style = MaterialTheme.typography.bodyMedium,
              )
              SiteChips(sites)
              if (found.isNotEmpty()) {
                FilledTonalButton(onClick = { VideoSession.playNow() }) {
                  Icon(Icons.Rounded.PlayArrow, contentDescription = null)
                  Spacer(Modifier.width(6.dp))
                  Text("Play the best so far")
                }
              }
            }
            Phase.Opening,
            Phase.Playing -> {
              LoadingIndicator(modifier = Modifier.size(64.dp), color = Brand.Blue)
              val src = ui.current
              Text(
                when {
                  ui.message != null -> ui.message
                  src?.value?.isTorrent == true -> "Joining the torrent swarm…"
                  else -> "Starting ${src?.let { SourceLabels.title(it) } ?: "the video"}…"
                },
                color = Color.White,
                style = MaterialTheme.typography.bodyMedium,
                textAlign = TextAlign.Center,
              )
              src?.let { s ->
                val chips = SourceLabels.chips(s)
                if (chips.isNotEmpty()) Text(chips.joinToString("  ·  "), color = Color.White.copy(alpha = 0.6f), style = MaterialTheme.typography.labelMedium)
              }
              ui.torrent?.let { t -> TorrentLine(t) }
              TextButton(onClick = { onSheet(Sheet.Sources) }) { Text("Choose another source", color = Color.White) }
            }
            Phase.AskData -> {
              Icon(Icons.Rounded.SignalCellularAlt, contentDescription = null, tint = Brand.Amber, modifier = Modifier.size(48.dp))
              Text("You're on mobile data", color = Color.White, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
              Text(
                "The best source is a torrent, which can use a lot of data${ui.current?.value?.sizeBytes?.let { " (up to ${SourceLabels.bytes(it)})" } ?: ""}.",
                color = Color.White.copy(alpha = 0.8f),
                style = MaterialTheme.typography.bodyMedium,
                textAlign = TextAlign.Center,
              )
              Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedButton(onClick = { VideoSession.answerData(false) }) { Text("Use a direct stream", color = Color.White) }
                Button(onClick = { VideoSession.answerData(true) }, colors = ButtonDefaults.buttonColors(containerColor = Brand.Blue)) { Text("Stream anyway") }
              }
            }
            Phase.Failed -> {
              Icon(Icons.Rounded.ErrorOutline, contentDescription = null, tint = MaterialTheme.colorScheme.error, modifier = Modifier.size(48.dp))
              Text(ui.message ?: "This one won't play right now.", color = Color.White, style = MaterialTheme.typography.bodyLarge, textAlign = TextAlign.Center)
              Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                if (ui.sources.isNotEmpty()) OutlinedButton(onClick = { onSheet(Sheet.Sources) }) { Text("Sources", color = Color.White) }
                if (ui.title?.value?.kind == "anime") {
                  OutlinedButton(onClick = { VideoSession.setDub(!ui.dub) }) { Text(if (ui.dub) "Try subbed" else "Try dubbed", color = Color.White) }
                }
                Button(onClick = onClose, colors = ButtonDefaults.buttonColors(containerColor = Brand.Blue)) { Text("Back") }
              }
            }
            Phase.Ended -> {
              Icon(Icons.Rounded.Replay, contentDescription = null, tint = Color.White, modifier = Modifier.size(48.dp))
              Text("That's the end", color = Color.White, style = MaterialTheme.typography.titleLarge)
              Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedButton(onClick = {
                  VideoSession.seekTo(0)
                  VideoSession.player?.play()
                }) { Text("Watch again", color = Color.White) }
                Button(onClick = onClose, colors = ButtonDefaults.buttonColors(containerColor = Brand.Blue)) { Text("Done") }
              }
            }
          }
        }
      }
    }
  }
}

/** Every site as a small chip: looking, found (with count), none, failed. */
@Composable
private fun SiteChips(sites: List<FoundSite>) {
  FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterHorizontally), verticalArrangement = Arrangement.spacedBy(6.dp)) {
    sites.forEach { s ->
      val (bg, fg) =
        when (s.state) {
          FoundSite.State.Found -> Brand.Blue.copy(alpha = 0.9f) to Color.White
          FoundSite.State.Looking -> Color.White.copy(alpha = 0.12f) to Color.White
          FoundSite.State.None -> Color.White.copy(alpha = 0.06f) to Color.White.copy(alpha = 0.45f)
          FoundSite.State.Failed -> Color(0x33E53935) to Color(0xFFFF8A80)
        }
      Surface(shape = RoundedCornerShape(50), color = bg) {
        Row(Modifier.padding(horizontal = 10.dp, vertical = 5.dp), verticalAlignment = Alignment.CenterVertically) {
          if (s.state == FoundSite.State.Looking) {
            CircularProgressIndicator(modifier = Modifier.size(10.dp), strokeWidth = 1.5.dp, color = Color.White)
            Spacer(Modifier.width(6.dp))
          }
          Text(
            SourceLabels.provider(s.provider) + if (s.state == FoundSite.State.Found) " · ${s.count}" else "",
            color = fg,
            style = MaterialTheme.typography.labelMedium,
          )
        }
      }
    }
  }
}

@Composable
private fun TorrentLine(t: com.velmora.stepbyte.engine.TorrentProgress) {
  Column(horizontalAlignment = Alignment.CenterHorizontally) {
    Box(Modifier.width(220.dp).height(4.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.2f))) {
      Box(Modifier.fillMaxWidth(t.fraction.coerceIn(0f, 1f)).fillMaxHeight().clip(CircleShape).background(Brand.Blue))
    }
    Text(
      "${SourceLabels.bytes(t.downloaded)} of ${SourceLabels.bytes(t.total)} from peers",
      color = Color.White.copy(alpha = 0.6f),
      style = MaterialTheme.typography.labelSmall,
      modifier = Modifier.padding(top = 4.dp),
    )
  }
}

@kotlin.OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun BufferingBadge(ui: PlaybackUi, modifier: Modifier) {
  Column(modifier, horizontalAlignment = Alignment.CenterHorizontally) {
    LoadingIndicator(modifier = Modifier.size(56.dp), color = Color.White)
    ui.message?.let { Text(it, color = Color.White, style = MaterialTheme.typography.labelLarge, modifier = Modifier.padding(top = 8.dp)) }
    ui.torrent?.takeIf { !it.finished }?.let { TorrentLine(it) }
  }
}

/** "Next episode" with a countdown ring; it plays by itself when the episode ends. */
@Composable
private fun NextEpisodeCard(label: String, thumbnail: String?, ended: Boolean, onPlay: () -> Unit, onDismiss: () -> Unit, modifier: Modifier) {
  var left by remember { mutableIntStateOf(8) }
  LaunchedEffect(ended) {
    if (!ended) return@LaunchedEffect
    left = 8
    while (left > 0) {
      delay(1_000)
      left--
    }
    onPlay()
  }
  val ring by animateFloatAsState(if (ended) left / 8f else 1f, tween(950), label = "ring")
  Surface(shape = RoundedCornerShape(24.dp), color = Color(0xE6161A24), modifier = modifier.padding(24.dp).width(320.dp)) {
    Row(Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
      Box(Modifier.width(112.dp).height(63.dp).clip(RoundedCornerShape(14.dp)).background(Color.White.copy(alpha = 0.1f))) {
        AsyncImage(model = thumbnail, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
      }
      Column(Modifier.weight(1f).padding(horizontal = 12.dp)) {
        Text("Next episode", color = Color.White.copy(alpha = 0.7f), style = MaterialTheme.typography.labelMedium)
        Text(label, color = Color.White, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
        Row {
          TextButton(onClick = onDismiss, contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 0.dp)) { Text("Not now", color = Color.White.copy(alpha = 0.7f)) }
        }
      }
      Box(contentAlignment = Alignment.Center) {
        CircularProgressIndicator(progress = { ring }, modifier = Modifier.size(52.dp), color = Brand.Blue, trackColor = Color.White.copy(alpha = 0.15f), strokeWidth = 3.dp)
        Surface(onClick = onPlay, shape = CircleShape, color = Color.White, modifier = Modifier.size(42.dp)) {
          Box(contentAlignment = Alignment.Center) {
            if (ended && left > 0) {
              Text("$left", color = Color.Black, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
            } else {
              Icon(Icons.Rounded.SkipNext, contentDescription = "Play next", tint = Color.Black)
            }
          }
        }
      }
    }
  }
}
