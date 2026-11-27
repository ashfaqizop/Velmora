package com.velmora.stepbyte.velmora.shell

import android.view.HapticFeedbackConstants
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.boundsInRoot
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onLongClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.velmora.stepbyte.R

/**
 * The "Velmora" wordmark on the left of the top bar, with an italic "Music"
 * after it in Music mode. Long-press it to switch modes straight away; the
 * reveal grows from the wordmark.
 */
@Composable
fun VelmoraWordmark(mode: AppMode, onLongPress: (center: Offset) -> Unit) {
  val view = LocalView.current
  var center by remember { mutableStateOf(Offset.Zero) }
  Row(
    verticalAlignment = Alignment.Bottom,
    modifier =
      Modifier.onGloballyPositioned { center = it.boundsInRoot().center }
        .semantics {
          contentDescription = if (mode == AppMode.MUSIC) "Velmora Music" else "Velmora"
          onLongClick(label = if (mode == AppMode.MUSIC) "Switch to Velmora" else "Switch to Music") {
            onLongPress(center)
            true
          }
        }
        .pointerInput(mode) {
          detectTapGestures(
            onLongPress = {
              view.performHapticFeedback(HapticFeedbackConstants.LONG_PRESS)
              onLongPress(center)
            },
          )
        },
  ) {
    Text("Velmora", fontFamily = Brand.Serif, fontSize = 30.sp, color = MaterialTheme.colorScheme.onSurface)
    AnimatedContent(
      targetState = mode,
      transitionSpec = {
        (slideInVertically { it / 2 } + fadeIn()) togetherWith (slideOutVertically { -it / 2 } + fadeOut())
      },
      label = "wordmarkMode",
    ) { m ->
      if (m == AppMode.MUSIC) {
        Text(
          " Music",
          fontFamily = Brand.Serif,
          fontStyle = FontStyle.Italic,
          fontSize = 30.sp,
          color = MaterialTheme.colorScheme.primary,
        )
      } else {
        Text("", fontSize = 30.sp)
      }
    }
  }
}

/** The right side of the top bar: avatar, Downloads, the mode switch. */
@Composable
fun ShellActions(
  mode: AppMode,
  accountImageUrl: String?,
  onAvatar: () -> Unit,
  onDownloads: () -> Unit,
  onSwitch: () -> Unit,
) {
  IconButton(onClick = onAvatar) { ShellAvatar(accountImageUrl) }
  IconButton(onClick = onDownloads) {
    Icon(
      painterResource(R.drawable.download),
      contentDescription = if (mode == AppMode.MUSIC) "Downloaded music" else "Downloads",
    )
  }
  ModeSwitchButton(mode = mode, onClick = onSwitch)
}

/** The avatar with a slowly turning Velmora-coloured ring. */
@Composable
private fun ShellAvatar(accountImageUrl: String?) {
  val ring = rememberInfiniteTransition(label = "avatarRing")
  val angle by
    ring.animateFloat(
      initialValue = 0f,
      targetValue = 360f,
      animationSpec = infiniteRepeatable(tween(12000, easing = LinearEasing), RepeatMode.Restart),
      label = "avatarRingAngle",
    )
  Box(modifier = Modifier.size(34.dp), contentAlignment = Alignment.Center) {
    Box(
      Modifier.fillMaxSize()
        .rotate(angle)
        .border(2.dp, Brush.sweepGradient(listOf(Brand.Blue, Brand.Violet, Brand.Cream, Brand.Amber, Brand.Blue)), CircleShape),
    )
    if (!accountImageUrl.isNullOrBlank()) {
      AsyncImage(
        model = accountImageUrl,
        contentDescription = "Account",
        contentScale = ContentScale.Crop,
        modifier = Modifier.size(25.dp).clip(CircleShape),
      )
    } else {
      Icon(
        painterResource(R.drawable.person),
        contentDescription = "Account",
        modifier = Modifier.size(19.dp),
        tint = MaterialTheme.colorScheme.onSurface,
      )
    }
  }
}

/**
 * The switch button: a small pill split blue and cream, the two worlds,
 * with the swap arrows on top. Opens the mode-switch panel.
 */
@Composable
fun ModeSwitchButton(mode: AppMode, onClick: () -> Unit) {
  val shimmer = rememberInfiniteTransition(label = "switchPill")
  val shift by
    shimmer.animateFloat(
      initialValue = 0f,
      targetValue = 1f,
      animationSpec = infiniteRepeatable(tween(3200, easing = LinearEasing), RepeatMode.Reverse),
      label = "switchPillShift",
    )
  val colors =
    if (mode == AppMode.VELMORA) listOf(Brand.Blue, Brand.Violet, Brand.Amber) else listOf(Brand.Amber, Brand.Cream, Brand.Blue)
  IconButton(onClick = onClick, modifier = Modifier.padding(end = 4.dp).width(56.dp)) {
    Box(
      Modifier.width(46.dp)
        .height(30.dp)
        .clip(RoundedCornerShape(50))
        .background(
          Brush.horizontalGradient(
            colors,
            startX = -60f + 60f * shift,
            endX = 120f + 60f * shift,
          ),
        )
        .semantics {
          role = Role.Button
          contentDescription = "Switch between Velmora and Music"
        },
      contentAlignment = Alignment.Center,
    ) {
      Icon(
        painterResource(R.drawable.swap_mode),
        contentDescription = null,
        tint = Color.White,
        modifier = Modifier.size(20.dp),
      )
    }
  }
}

@Suppress("unused") private val keepWeight = FontWeight.Normal
