package com.velmora.stepbyte.velmora.shell

import android.os.Build
import android.provider.Settings
import android.view.HapticFeedbackConstants
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.datastore.preferences.core.edit
import coil3.compose.AsyncImage
import com.velmora.stepbyte.LocalDatabase
import com.velmora.stepbyte.R
import com.velmora.stepbyte.utils.dataStore
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.cos
import kotlin.math.floor
import kotlin.math.hypot
import kotlin.math.max
import kotlin.math.roundToInt
import kotlin.math.sin
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch

// The mode switch. A frosted panel drops from the top with one track split
// into two worlds: Velmora on the left (a drifting film strip of posters,
// blue) and Music on the right (album covers and a live waveform, cream).
// Dragging the glass thumb towards the other world pulls it in behind the
// thumb: posters flip into album covers (or back), the film strip bends into
// a sound wave, the colour flows across, and the phone ticks every 10%.
// Past 60% it commits and the new mode opens with a circular reveal from the
// thumb. Letting go early springs back.

/** Did the user turn animations off (Developer options or accessibility)? */
@Composable
fun rememberReducedMotion(): Boolean {
  val context = LocalContext.current
  return remember {
    Settings.Global.getFloat(context.contentResolver, Settings.Global.ANIMATOR_DURATION_SCALE, 1f) == 0f
  }
}

/** How far the thumb must travel before the switch happens. */
private const val COMMIT_AT = 0.6f

@Composable
fun ModeSwitchPanel(
  visible: Boolean,
  current: AppMode,
  onDismiss: () -> Unit,
  /** Switch to [to]; [center] is the thumb's centre in root coordinates, for the reveal. */
  onCommit: (to: AppMode, center: Offset) -> Unit,
) {
  BackHandler(enabled = visible, onBack = onDismiss)

  val scrim by animateFloatAsState(if (visible) 0.55f else 0f, tween(300), label = "scrim")
  if (scrim > 0.001f) {
    Box(
      Modifier.fillMaxSize()
        .background(Color.Black.copy(alpha = scrim))
        .clickable(interactionSource = remember { MutableInteractionSource() }, indication = null, onClick = onDismiss),
    )
  }

  AnimatedVisibility(
    visible = visible,
    enter =
      slideInVertically(spring(dampingRatio = 0.78f, stiffness = Spring.StiffnessMediumLow)) { -it } +
        fadeIn(tween(200)),
    exit = slideOutVertically(tween(260, easing = FastOutSlowInEasing)) { -it } + fadeOut(tween(200)),
  ) {
    Box(
      Modifier.fillMaxWidth()
        .shadow(24.dp, RoundedCornerShape(bottomStart = 36.dp, bottomEnd = 36.dp))
        .clip(RoundedCornerShape(bottomStart = 36.dp, bottomEnd = 36.dp))
        .background(MaterialTheme.colorScheme.surfaceContainerHigh.copy(alpha = 0.97f))
        // Swallow taps so they don't reach the scrim.
        .clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) {},
    ) {
      Column(
        Modifier.windowInsetsPadding(WindowInsets.statusBars).padding(start = 16.dp, end = 16.dp, top = 12.dp, bottom = 14.dp),
      ) {
        Row(verticalAlignment = Alignment.Bottom) {
          Text(
            if (current == AppMode.VELMORA) "Velmora" else "Music",
            fontFamily = Brand.Serif,
            fontSize = 34.sp,
            color = MaterialTheme.colorScheme.onSurface,
          )
          Text(
            "  you're here",
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(bottom = 7.dp),
          )
        }
        Text(
          if (current == AppMode.VELMORA) "Drag the thumb right to switch to Music" else "Drag the thumb left to switch to Velmora",
          style = MaterialTheme.typography.bodyMedium,
          color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(14.dp))
        if (visible) SwitchTrack(current = current, onCommit = onCommit)
        Spacer(Modifier.height(10.dp))
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
          WorldLabel("Velmora", "Movies · series · anime", active = current == AppMode.VELMORA, alignEnd = false)
          WorldLabel("Music", "Songs · albums · podcasts", active = current == AppMode.MUSIC, alignEnd = true)
        }
        Spacer(Modifier.height(10.dp))
        Box(
          Modifier.align(Alignment.CenterHorizontally)
            .size(width = 36.dp, height = 4.dp)
            .clip(CircleShape)
            .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)),
        )
      }
    }
  }
}

@Composable
private fun WorldLabel(name: String, detail: String, active: Boolean, alignEnd: Boolean) {
  Column(horizontalAlignment = if (alignEnd) Alignment.End else Alignment.Start) {
    Text(
      name,
      style = MaterialTheme.typography.titleSmall,
      fontWeight = if (active) FontWeight.Bold else FontWeight.Medium,
      color = if (active) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface,
    )
    Text(detail, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
  }
}

/** 0 = Velmora world, 1 = Music world, smooth across a soft edge. */
private fun musicness(x: Float, thumbX: Float, mid: Float, edge: Float, current: AppMode): Float {
  fun step(v: Float) = (v / edge + 0.5f).coerceIn(0f, 1f)
  // The world each spot shows at rest: left half Velmora, right half Music.
  val home = step(x - mid)
  return if (current == AppMode.VELMORA) {
    // Music spreads in behind the thumb as it moves right.
    max(home, step(thumbX - x))
  } else {
    // Velmora spreads in behind the thumb as it moves left.
    minOf(home, 1f - step(x - thumbX))
  }
}

@Composable
private fun SwitchTrack(current: AppMode, onCommit: (AppMode, Offset) -> Unit) {
  val context = LocalContext.current
  val view = LocalView.current
  val density = LocalDensity.current
  val scope = rememberCoroutineScope()
  val reduceMotion = rememberReducedMotion()

  val database = LocalDatabase.current
  val topSongs by remember(database) { database.topSongs(12) }.collectAsState(initial = emptyList())
  val covers = remember(topSongs) { topSongs.mapNotNull { it.song.thumbnailUrl }.distinct() }
  val posterFeed by HomeFeed.rails.collectAsState()
  val posters = remember(posterFeed) { HomeFeed.posters() }
  LaunchedEffect(Unit) { HomeFeed.load(context) }

  val rest = if (current == AppMode.VELMORA) 0f else 1f
  val target = current.other
  val pos = remember { Animatable(rest) }
  var committed by remember { mutableStateOf(false) }
  var dragging by remember { mutableStateOf(false) }
  var lastTick by remember { mutableIntStateOf((rest * 10).roundToInt()) }
  var trackOrigin by remember { mutableStateOf(Offset.Zero) }

  // Time for the drifting strip and the waveform.
  val clock = rememberInfiniteTransition(label = "switchClock")
  val drift by
    clock.animateFloat(
      initialValue = 0f,
      targetValue = 1f,
      animationSpec = infiniteRepeatable(tween(16000, easing = LinearEasing), RepeatMode.Restart),
      label = "drift",
    )
  val wave by
    clock.animateFloat(
      initialValue = 0f,
      targetValue = (2 * PI).toFloat(),
      animationSpec = infiniteRepeatable(tween(1400, easing = LinearEasing), RepeatMode.Restart),
      label = "wave",
    )
  val t = if (reduceMotion) 0f else drift
  val phase = if (reduceMotion) 0f else wave

  val thumbScale by animateFloatAsState(if (dragging) 1.12f else 1f, spring(dampingRatio = 0.5f), label = "thumbScale")

  // One-time hint: the thumb leans towards the other world and back.
  LaunchedEffect(Unit) {
    val seen = context.dataStore.data.first()[ModeSwitchHintSeenKey] == true
    if (!seen && !reduceMotion) {
      delay(550)
      val lean = if (current == AppMode.VELMORA) 0.22f else 0.78f
      pos.animateTo(lean, spring(dampingRatio = 0.6f, stiffness = Spring.StiffnessLow))
      pos.animateTo(rest, spring(dampingRatio = 0.45f, stiffness = Spring.StiffnessLow))
      context.dataStore.edit { it[ModeSwitchHintSeenKey] = true }
    }
  }

  fun tick(value: Float) {
    val step = (value * 10).toInt()
    if (step != lastTick) {
      lastTick = step
      view.performHapticFeedback(HapticFeedbackConstants.CLOCK_TICK)
    }
  }

  BoxWithConstraints(
    Modifier.fillMaxWidth()
      .height(172.dp)
      .clip(RoundedCornerShape(34.dp))
      .onGloballyPositioned { trackOrigin = it.positionInRoot() },
  ) {
    val w = constraints.maxWidth.toFloat()
    val h = constraints.maxHeight.toFloat()
    val pad = with(density) { 8.dp.toPx() }
    val thumbPx = with(density) { 72.dp.toPx() }
    val travel = (w - thumbPx - 2 * pad).coerceAtLeast(1f)
    val mid = w / 2f
    val edge = with(density) { 56.dp.toPx() }
    val m = pos.value
    val thumbX = pad + thumbPx / 2f + m * travel
    val thumbCenter = { trackOrigin + Offset(thumbX, h / 2f) }

    fun commit() {
      if (committed) return
      committed = true
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
        view.performHapticFeedback(HapticFeedbackConstants.CONFIRM)
      } else {
        view.performHapticFeedback(HapticFeedbackConstants.LONG_PRESS)
      }
      scope.launch {
        val end = if (target == AppMode.MUSIC) 1f else 0f
        pos.animateTo(end, tween(if (reduceMotion) 1 else 180, easing = FastOutSlowInEasing))
        onCommit(target, trackOrigin + Offset(pad + thumbPx / 2f + end * travel, h / 2f))
      }
    }

    fun passed(value: Float) = if (target == AppMode.MUSIC) value >= COMMIT_AT else value <= 1f - COMMIT_AT

    // ── Background: the two worlds' colours, flowing behind the thumb ──
    Canvas(Modifier.fillMaxSize()) {
      val steps = 48
      val sliceW = size.width / steps
      for (i in 0 until steps) {
        val x = i * sliceW + sliceW / 2
        val k = musicness(x, thumbX, mid, edge, current)
        val top = lerp(lerp(Brand.BlueDeep, Brand.Blue, i / steps.toFloat()), lerp(Brand.Amber, Brand.Cream, i / steps.toFloat()), k)
        val bottom = lerp(Color(0xFF101A44), Color(0xFFE59A3A), k)
        drawRect(
          brush = Brush.verticalGradient(listOf(top, bottom), startY = 0f, endY = size.height),
          topLeft = Offset(i * sliceW, 0f),
          size = Size(sliceW + 1f, size.height),
        )
      }
      // Live waveform along the bottom, only where Music has arrived.
      val bars = 40
      val barW = size.width / bars
      for (i in 0 until bars) {
        val x = i * barW + barW / 2
        val k = musicness(x, thumbX, mid, edge, current)
        if (k < 0.02f) continue
        val amp = 0.25f + 0.75f * abs(sin(x / size.width * 9f + phase) * cos(x / size.width * 3.7f - phase * 0.6f))
        val bh = amp * 26.dp.toPx() * k
        drawRoundRect(
          color = Color.White.copy(alpha = 0.55f * k),
          topLeft = Offset(x - barW * 0.28f, size.height - 12.dp.toPx() - bh),
          size = Size(barW * 0.56f, bh),
          cornerRadius = CornerRadius(barW, barW),
        )
      }
    }

    // ── The strip: posters that turn into album covers and bend into a wave ──
    val tileW = with(density) { 58.dp.toPx() }
    val tileH = with(density) { 84.dp.toPx() }
    val gap = with(density) { 12.dp.toPx() }
    val step = tileW + gap
    val count = (w / step).toInt() + 3
    val stripY = h / 2f - tileH / 2f - with(density) { 6.dp.toPx() }
    val amplitude = with(density) { 18.dp.toPx() }
    for (i in 0 until count) {
      val raw = i * step - t * step * count
      val x = raw - floor(raw / (step * count)) * (step * count) - step
      val center = x + tileW / 2f
      val k = musicness(center, thumbX, mid, edge, current)
      val angle = center / w * 3f * PI.toFloat() + phase
      val dy = sin(angle) * amplitude * k
      val rot = cos(angle) * 14f * k
      val poster = posters.getOrNull(i % max(posters.size, 1))
      val cover = covers.getOrNull(i % max(covers.size, 1))
      Box(
        Modifier.offset { IntOffset(x.roundToInt(), (stripY + dy).roundToInt()) }
          .size(with(density) { tileW.toDp() }, with(density) { tileH.toDp() })
          .graphicsLayer {
            rotationZ = rot
            // Album covers are square: the tile shrinks towards a square as it turns.
            scaleY = 1f - 0.26f * k
            shape = RoundedCornerShape((10 + 8 * k).dp)
            clip = true
            alpha = 0.92f
          }
          .background(Color.White.copy(alpha = 0.12f)),
      ) {
        if (poster != null) {
          AsyncImage(
            model = poster,
            contentDescription = null,
            contentScale = ContentScale.Crop,
            modifier = Modifier.fillMaxSize().graphicsLayer { alpha = 1f - k },
          )
        }
        if (cover != null) {
          AsyncImage(
            model = cover,
            contentDescription = null,
            contentScale = ContentScale.Crop,
            modifier = Modifier.fillMaxSize().graphicsLayer { alpha = k },
          )
        }
        // Film sprocket holes, fading out as the tile becomes a cover.
        Box(
          Modifier.fillMaxSize().drawWithContent {
            drawContent()
            val a = 1f - k
            if (a > 0.02f) {
              val hole = 5.dp.toPx()
              var y = 6.dp.toPx()
              while (y < size.height - hole) {
                drawRoundRect(Color.Black.copy(alpha = 0.45f * a), Offset(3.dp.toPx(), y), Size(hole * 0.8f, hole), CornerRadius(1.5f.dp.toPx()))
                drawRoundRect(Color.Black.copy(alpha = 0.45f * a), Offset(size.width - 3.dp.toPx() - hole * 0.8f, y), Size(hole * 0.8f, hole), CornerRadius(1.5f.dp.toPx()))
                y += hole * 2f
              }
            }
          },
        )
      }
    }

    // ── Commit marker: a faint line at 60% of the way ──
    val markX = pad + thumbPx / 2f + (if (target == AppMode.MUSIC) COMMIT_AT else 1f - COMMIT_AT) * travel
    Box(
      Modifier.offset { IntOffset(markX.roundToInt() - 1, (h * 0.18f).roundToInt()) }
        .size(2.dp, with(density) { (h * 0.64f).toDp() })
        .background(Color.White.copy(alpha = 0.35f), CircleShape),
    )

    // ── The glass thumb ──
    Box(
      Modifier.offset { IntOffset((thumbX - thumbPx / 2f).roundToInt(), (h / 2f - thumbPx / 2f).roundToInt()) }
        .size(72.dp)
        .graphicsLayer {
          scaleX = thumbScale
          scaleY = thumbScale
        }
        .shadow(16.dp, CircleShape, ambientColor = Color.Black, spotColor = Color.Black)
        .clip(CircleShape)
        .background(
          Brush.radialGradient(
            listOf(Color.White.copy(alpha = 0.55f), Color.White.copy(alpha = 0.22f)),
          ),
        )
        .border(1.5.dp, Color.White.copy(alpha = 0.75f), CircleShape),
      contentAlignment = Alignment.Center,
    ) {
      val k = m
      Icon(
        painterResource(R.drawable.movie),
        contentDescription = null,
        tint = Brand.BlueDeep,
        modifier = Modifier.size(30.dp).graphicsLayer {
          alpha = 1f - k
          rotationZ = k * 180f
          scaleX = 1f - 0.4f * k
          scaleY = 1f - 0.4f * k
        },
      )
      Icon(
        painterResource(R.drawable.music_note),
        contentDescription = null,
        tint = Color(0xFF6B3D00),
        modifier = Modifier.size(30.dp).graphicsLayer {
          alpha = k
          rotationZ = (k - 1f) * 180f
          scaleX = 0.6f + 0.4f * k
          scaleY = 0.6f + 0.4f * k
        },
      )
    }

    // ── Input: drag the thumb (or anywhere on the track), or tap the other side ──
    val switchLabel = if (target == AppMode.MUSIC) "Switch to Music" else "Switch to Velmora"
    Box(
      Modifier.fillMaxSize()
        .semantics {
          contentDescription = "Mode switch"
          stateDescription = if (current == AppMode.VELMORA) "Velmora" else "Music"
          onClick(label = switchLabel) {
            commit()
            true
          }
          customActions = listOf(CustomAccessibilityAction(switchLabel) { commit(); true })
        }
        .pointerInputDrag(
          key = current,
          onStart = { dragging = true },
          onDrag = { dx ->
            if (!committed) {
              val next = (pos.value + dx / travel).coerceIn(0f, 1f)
              scope.launch { pos.snapTo(next) }
              tick(next)
              if (passed(next)) commit()
            }
          },
          onEnd = {
            dragging = false
            if (!committed) {
              scope.launch {
                // Spring back with a little overshoot: the fun part of letting go early.
                pos.animateTo(rest, spring(dampingRatio = 0.42f, stiffness = Spring.StiffnessMediumLow))
              }
            }
          },
          onTap = { x ->
            val onOtherSide = if (target == AppMode.MUSIC) x > mid else x < mid
            if (onOtherSide && !committed) {
              scope.launch {
                val end = if (target == AppMode.MUSIC) 1f else 0f
                pos.animateTo(end, tween(if (reduceMotion) 1 else 420, easing = FastOutSlowInEasing)) { tick(value) }
                commit()
              }
            }
          },
        ),
    )
  }
}

/** Horizontal drag plus tap on one box. */
private fun Modifier.pointerInputDrag(
  key: Any,
  onStart: () -> Unit,
  onDrag: (Float) -> Unit,
  onEnd: () -> Unit,
  onTap: (Float) -> Unit,
): Modifier =
  this.pointerInput(key) {
      detectHorizontalDragGestures(
        onDragStart = { onStart() },
        onDragEnd = onEnd,
        onDragCancel = onEnd,
        onHorizontalDrag = { change, dx ->
          change.consume()
          onDrag(dx)
        },
      )
    }
    .pointerInput(key to "tap") { detectTapGestures(onTap = { onTap(it.x) }) }

// ── The reveal ─────────────────────────────────────────────────────────

/** Drives the full-screen circular reveal that plays when the mode changes. */
@Stable
class ModeRevealState {
  var center by mutableStateOf(Offset.Zero)
    private set
  var target by mutableStateOf(AppMode.VELMORA)
    private set
  var active by mutableStateOf(false)
    private set
  val grow = Animatable(0f)
  val alpha = Animatable(0f)

  /** Cover the screen from [from], call [switch] while covered, then uncover. */
  suspend fun play(from: Offset, to: AppMode, reduceMotion: Boolean, switch: () -> Unit) {
    center = from
    target = to
    active = true
    if (reduceMotion) {
      grow.snapTo(1f)
      alpha.snapTo(0f)
      alpha.animateTo(1f, tween(140))
    } else {
      alpha.snapTo(1f)
      grow.snapTo(0f)
      grow.animateTo(1f, tween(560, easing = FastOutSlowInEasing))
    }
    switch()
    delay(180)
    alpha.animateTo(0f, tween(if (reduceMotion) 160 else 380))
    active = false
  }
}

@Composable
fun rememberModeRevealState(): ModeRevealState = remember { ModeRevealState() }

@Composable
fun ModeReveal(state: ModeRevealState) {
  if (!state.active) return
  val music = state.target == AppMode.MUSIC
  val inner = if (music) Brand.Cream else Brand.Blue
  val outer = if (music) Brand.Amber else Brand.BlueDeep
  val grow = state.grow.value
  val alpha = state.alpha.value
  Box(
    Modifier.fillMaxSize()
      .graphicsLayer { this.alpha = alpha }
      .drawBehind {
        val c = state.center
        val far =
          listOf(Offset.Zero, Offset(size.width, 0f), Offset(0f, size.height), Offset(size.width, size.height))
            .maxOf { hypot(it.x - c.x, it.y - c.y) }
        val r = far * grow
        if (r > 0f) {
          drawCircle(brush = Brush.radialGradient(listOf(inner, outer), center = c, radius = max(r, 1f)), radius = r, center = c)
        }
      },
    contentAlignment = Alignment.Center,
  ) {
    // The mode's name appears in the middle as the colour fills the screen.
    val textAlpha = ((grow - 0.45f) / 0.4f).coerceIn(0f, 1f)
    Column(
      horizontalAlignment = Alignment.CenterHorizontally,
      modifier =
        Modifier.graphicsLayer {
          this.alpha = textAlpha
          scaleX = 0.9f + 0.1f * textAlpha
          scaleY = 0.9f + 0.1f * textAlpha
        },
    ) {
      Icon(
        painterResource(if (music) R.drawable.music_note else R.drawable.movie),
        contentDescription = null,
        tint = if (music) Color(0xFF6B3D00) else Color.White,
        modifier = Modifier.size(44.dp),
      )
      Spacer(Modifier.height(8.dp))
      Text(
        if (music) "Music" else "Velmora",
        fontFamily = Brand.Serif,
        fontStyle = if (music) FontStyle.Italic else FontStyle.Normal,
        fontSize = 64.sp,
        color = if (music) Color(0xFF3A2200) else Color.White,
      )
    }
  }
}

