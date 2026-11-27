package com.velmora.stepbyte.velmora.account

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage

/**
 * A Velmora profile picture: the Google or Discord photo, or the profile's
 * animal colour with the first letter of the name on it.
 */
@Composable
fun AccountAvatar(profile: Profile, size: Dp = 40.dp, modifier: Modifier = Modifier) {
  Box(
    modifier = modifier.size(size).clip(CircleShape).background(Avatars.color(profile.avatar)),
    contentAlignment = Alignment.Center,
  ) {
    if (Avatars.isPhoto(profile.avatar)) {
      AsyncImage(
        model = profile.avatar,
        contentDescription = profile.displayName,
        contentScale = ContentScale.Crop,
        modifier = Modifier.size(size),
      )
    } else {
      Text(
        profile.displayName.trim().take(1).uppercase().ifEmpty { "V" },
        color = Color.White,
        fontWeight = FontWeight.Bold,
        fontSize = (size.value * 0.42f).sp,
      )
    }
  }
}
