package com.velmora.stepbyte.velmora.account

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.velmora.stepbyte.R
import com.velmora.stepbyte.velmora.shell.Brand

/** Whether Echo's saved YouTube cookie is a signed-in one. */
fun isYouTubeMusicSignedIn(cookie: String): Boolean = cookie.isNotEmpty() && "SAPISID" in cookie

/**
 * Music needs a YouTube Music account, like desktop. Shown over Music mode
 * until you sign in (Echo's own sign-in page), or go back to Velmora.
 */
@Composable
fun YouTubeMusicGate(onSignIn: () -> Unit, onBackToVelmora: () -> Unit) {
  BackHandler(onBack = onBackToVelmora)
  Box(
    Modifier.fillMaxSize()
      .background(Brush.verticalGradient(listOf(Color(0xFF2A1A05), Brand.Night)))
      .pointerInput(Unit) { awaitPointerEventScope { while (true) awaitPointerEvent() } }
      .windowInsetsPadding(WindowInsets.systemBars),
    contentAlignment = Alignment.Center,
  ) {
    Column(
      Modifier.padding(horizontal = 28.dp),
      horizontalAlignment = Alignment.CenterHorizontally,
      verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
      Box(
        Modifier.size(104.dp).clip(RoundedCornerShape(34.dp)).background(Brush.linearGradient(listOf(Brand.Amber, Brand.Cream))),
        contentAlignment = Alignment.Center,
      ) {
        Icon(painterResource(R.drawable.music_note), contentDescription = null, tint = Color(0xFF3A2200), modifier = Modifier.size(52.dp))
      }
      Spacer(Modifier.height(6.dp))
      Text("Music", fontFamily = Brand.Serif, fontStyle = FontStyle.Italic, fontSize = 56.sp, color = Brand.Cream)
      Text(
        "Music plays from your YouTube Music account: your library, playlists, likes and mixes. Sign in once and it stays signed in.",
        style = MaterialTheme.typography.bodyLarge,
        color = Color.White.copy(alpha = 0.8f),
        textAlign = TextAlign.Center,
      )
      Spacer(Modifier.height(10.dp))
      Button(
        onClick = onSignIn,
        colors = ButtonDefaults.buttonColors(containerColor = Brand.Cream, contentColor = Color(0xFF3A2200)),
        modifier = Modifier.fillMaxWidth().height(56.dp),
      ) {
        Text("Sign in to YouTube Music", style = MaterialTheme.typography.titleMedium)
      }
      TextButton(onClick = onBackToVelmora) { Text("Back to Velmora", color = Color.White.copy(alpha = 0.8f)) }
    }
  }
}
