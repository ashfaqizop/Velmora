package com.velmora.stepbyte.velmora.shell

import android.os.Build
import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.velmora.stepbyte.R
import com.velmora.stepbyte.constants.LiquidGlassGlobalEnabledKey
import com.velmora.stepbyte.utils.rememberPreference

/**
 * Velmora's two looks: Material You (Material 3 Expressive, wallpaper colours
 * on Android 12+) and Liquid Glass (Echo's frosted, refracting surfaces).
 * Liquid Glass needs blur from Android 12; earlier phones get frosted panels
 * without blur, and the card says so.
 */
@Composable
fun LookPicker() {
  val (glass, setGlass) = rememberPreference(LiquidGlassGlobalEnabledKey, defaultValue = false)
  val blurSupported = Build.VERSION.SDK_INT >= Build.VERSION_CODES.S

  Column(Modifier.fillMaxWidth()) {
    Text(
      "Look",
      style = MaterialTheme.typography.labelLarge,
      color = MaterialTheme.colorScheme.primary,
      modifier = Modifier.padding(start = 12.dp, bottom = 8.dp),
    )
    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
      LookCard(
        title = "Material You",
        detail = if (blurSupported) "Your wallpaper's colours" else "Velmora's colours",
        selected = !glass,
        onClick = { setGlass(false) },
        modifier = Modifier.weight(1f),
      ) { MaterialPreview() }
      LookCard(
        title = "Liquid Glass",
        detail = if (blurSupported) "Frosted, see-through bars" else "Simplified on this phone",
        selected = glass,
        onClick = { setGlass(true) },
        modifier = Modifier.weight(1f),
      ) { GlassPreview() }
    }
  }
}

@Composable
private fun LookCard(
  title: String,
  detail: String,
  selected: Boolean,
  onClick: () -> Unit,
  modifier: Modifier = Modifier,
  preview: @Composable () -> Unit,
) {
  val border by
    animateColorAsState(
      if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant,
      label = "lookBorder",
    )
  Surface(
    onClick = onClick,
    shape = RoundedCornerShape(24.dp),
    color = MaterialTheme.colorScheme.surfaceContainer,
    modifier =
      modifier
        .border(if (selected) 2.dp else 1.dp, border, RoundedCornerShape(24.dp))
        .semantics {
          role = Role.RadioButton
          this.selected = selected
        },
  ) {
    Column(Modifier.padding(10.dp)) {
      Box(
        Modifier.fillMaxWidth().height(110.dp).clip(RoundedCornerShape(16.dp)),
      ) {
        preview()
        if (selected) {
          Box(
            Modifier.align(Alignment.TopEnd)
              .padding(6.dp)
              .size(24.dp)
              .clip(CircleShape)
              .background(MaterialTheme.colorScheme.primary),
            contentAlignment = Alignment.Center,
          ) {
            Icon(
              painterResource(R.drawable.check),
              contentDescription = null,
              tint = MaterialTheme.colorScheme.onPrimary,
              modifier = Modifier.size(16.dp),
            )
          }
        }
      }
      Spacer(Modifier.height(10.dp))
      Text(title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
      Text(detail, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
  }
}

/** Tonal, rounded blocks: Material 3 Expressive in miniature. */
@Composable
private fun MaterialPreview() {
  val c = MaterialTheme.colorScheme
  Box(Modifier.fillMaxWidth().height(110.dp).background(c.surfaceContainerHigh)) {
    Box(Modifier.padding(10.dp).size(46.dp).clip(RoundedCornerShape(16.dp)).background(c.primaryContainer))
    Box(Modifier.offset(64.dp, 12.dp).size(70.dp, 12.dp).clip(CircleShape).background(c.onSurface.copy(alpha = 0.6f)))
    Box(Modifier.offset(64.dp, 32.dp).size(46.dp, 10.dp).clip(CircleShape).background(c.onSurfaceVariant.copy(alpha = 0.5f)))
    Box(
      Modifier.align(Alignment.BottomCenter)
        .padding(10.dp)
        .fillMaxWidth()
        .height(30.dp)
        .clip(RoundedCornerShape(50))
        .background(c.secondaryContainer),
    ) {
      Box(Modifier.align(Alignment.CenterStart).padding(start = 8.dp).size(40.dp, 18.dp).clip(CircleShape).background(c.primary))
    }
  }
}

/** Light shining through frosted, bordered panes over a colourful scene. */
@Composable
private fun GlassPreview() {
  Box(
    Modifier.fillMaxWidth()
      .height(110.dp)
      .background(Brush.linearGradient(listOf(Brand.Blue, Brand.Violet, Brand.Amber))),
  ) {
    Box(Modifier.offset(14.dp, 10.dp).size(54.dp).clip(CircleShape).background(Brand.Cream.copy(alpha = 0.9f)))
    Box(Modifier.offset(70.dp, 30.dp).size(40.dp).clip(CircleShape).background(Brand.BlueDeep.copy(alpha = 0.8f)))
    Box(
      Modifier.align(Alignment.BottomCenter)
        .padding(10.dp)
        .fillMaxWidth()
        .height(30.dp)
        .clip(RoundedCornerShape(50))
        .background(Color.White.copy(alpha = 0.28f))
        .border(1.dp, Color.White.copy(alpha = 0.7f), RoundedCornerShape(50)),
    ) {
      Box(
        Modifier.align(Alignment.CenterStart)
          .padding(start = 8.dp)
          .size(40.dp, 18.dp)
          .clip(CircleShape)
          .background(Color.White.copy(alpha = 0.55f)),
      )
    }
  }
}
