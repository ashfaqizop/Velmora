package com.velmora.stepbyte.velmora.downloads

import android.widget.Toast
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Download
import androidx.compose.material.icons.rounded.DownloadDone
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material.icons.rounded.Lock
import androidx.compose.material.icons.rounded.Pause
import androidx.compose.material.icons.rounded.VideoLibrary
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.FilledTonalIconButton
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.velmora.stepbyte.engine.Season
import com.velmora.stepbyte.engine.Target
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.velmora.shell.Brand

/** The download state of one movie or episode, for its button. */
@Composable
fun rememberDownload(title: Title, target: Target): DownloadItem? {
  val items by VelmoraDownloads.items.collectAsState()
  val key = remember(title, target) { VelmoraDownloads.keyOf(title, target) }
  return remember(items, key) { VelmoraDownloads.itemFor(title, target) }
}

/** The icon inside a download button: arrow, ring with progress, or done. */
@Composable
fun DownloadGlyph(item: DownloadItem?, size: androidx.compose.ui.unit.Dp = 24.dp) {
  val phase = item?.record?.phase
  AnimatedContent(
    targetState = phase,
    transitionSpec = { (scaleIn(spring(dampingRatio = Spring.DampingRatioMediumBouncy), initialScale = 0.6f) + fadeIn()) togetherWith (scaleOut(tween(120), targetScale = 0.6f) + fadeOut(tween(120))) },
    label = "download",
  ) { p ->
    Box(Modifier.size(size + 8.dp), contentAlignment = Alignment.Center) {
      when (p) {
        null -> Icon(Icons.Rounded.Download, contentDescription = "Download", modifier = Modifier.size(size))
        DownloadPhase.Done -> Icon(Icons.Rounded.DownloadDone, contentDescription = "Downloaded", tint = Brand.Violet, modifier = Modifier.size(size))
        DownloadPhase.Failed -> Icon(Icons.Rounded.ErrorOutline, contentDescription = "Download failed", tint = MaterialTheme.colorScheme.error, modifier = Modifier.size(size))
        DownloadPhase.Paused, DownloadPhase.WaitingWifi -> {
          CircularProgressIndicator(progress = { item?.fraction ?: 0f }, modifier = Modifier.size(size + 6.dp), strokeWidth = 2.5.dp, color = MaterialTheme.colorScheme.outline)
          Icon(Icons.Rounded.Pause, contentDescription = "Paused", modifier = Modifier.size(size * 0.6f))
        }
        else -> {
          val fraction = item?.fraction ?: 0f
          if (p == DownloadPhase.Running && fraction > 0f) {
            CircularProgressIndicator(progress = { fraction }, modifier = Modifier.size(size + 6.dp), strokeWidth = 2.5.dp, color = Brand.Blue)
          } else {
            CircularProgressIndicator(modifier = Modifier.size(size + 6.dp), strokeWidth = 2.5.dp, color = Brand.Blue)
          }
          Icon(Icons.Rounded.Download, contentDescription = "Downloading", modifier = Modifier.size(size * 0.6f))
        }
      }
    }
  }
}

/** The title page's download button: opens the sheet, or Downloads once it's going. */
@Composable
fun DownloadButton(title: Title, target: Target, seasons: List<Season>, onOpenDownloads: () -> Unit, modifier: Modifier = Modifier) {
  val item = rememberDownload(title, target)
  var sheet by remember { mutableStateOf(false) }
  FilledTonalIconButton(
    onClick = { if (item == null || item.record.phase == DownloadPhase.Failed) sheet = true else onOpenDownloads() },
    modifier = modifier.size(56.dp),
    shape = RoundedCornerShape(18.dp),
  ) {
    DownloadGlyph(item)
  }
  if (sheet) DownloadSheet(title, target, seasons, onDismiss = { sheet = false }, onOpenDownloads = onOpenDownloads)
}

/** An episode row's small download button: one tap starts it at the usual quality. */
@Composable
fun EpisodeDownloadButton(title: Title, season: Int, episode: Int, episodeName: String?, onOpenDownloads: () -> Unit) {
  val context = LocalContext.current
  val haptics = LocalHapticFeedback.current
  val target = remember(season, episode) { Target.Episode(season, episode) }
  val item = rememberDownload(title, target)
  IconButton(onClick = {
    if (item == null || item.record.phase == DownloadPhase.Failed) {
      haptics.performHapticFeedback(HapticFeedbackType.LongPress)
      val msg = VelmoraDownloads.add(context, title, target, episodeName = episodeName)
      Toast.makeText(context, msg ?: "Downloading S$season E$episode", Toast.LENGTH_SHORT).show()
    } else {
      onOpenDownloads()
    }
  }) {
    DownloadGlyph(item, size = 20.dp)
  }
}

/** Quality, Wi-Fi, and for a series this episode or the whole season. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DownloadSheet(title: Title, target: Target, seasons: List<Season>, onDismiss: () -> Unit, onOpenDownloads: () -> Unit) {
  val context = LocalContext.current
  val state = rememberModalBottomSheetState(skipPartiallyExpanded = true)
  var quality by remember { mutableStateOf(VelmoraDownloads.defaultQuality()) }
  var wifi by remember { mutableStateOf(VelmoraDownloads.wifiOnly()) }
  val ep = target as? Target.Episode
  val season = ep?.let { e -> seasons.firstOrNull { it.number == e.season } }
  val free = remember { VelmoraDownloads.freeBytes() }

  fun done(msg: String) {
    Toast.makeText(context, msg, Toast.LENGTH_SHORT).show()
    onDismiss()
  }

  ModalBottomSheet(onDismissRequest = onDismiss, sheetState = state) {
    Column(Modifier.fillMaxWidth().padding(horizontal = 22.dp).padding(bottom = 28.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
      Text("Download", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
      Text(
        title.value.name + (ep?.let { " · S${it.season} E${it.episode}" } ?: ""),
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
      )
      Text("Quality", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.SemiBold)
      Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        DownloadQuality.entries.forEach { q ->
          FilterChip(selected = quality == q, onClick = { quality = q }, label = { Text(q.label) })
        }
      }
      Text(
        when (quality) {
          DownloadQuality.Best -> "The best source there is, up to 4K. Can be many GB."
          DownloadQuality.Full -> "Sharp on any phone and tablet. About 2 GB a film."
          DownloadQuality.High -> "Good on a phone and quick to download. About 1 GB an hour."
          DownloadQuality.Saver -> "Smallest. Fine for a phone screen."
        } + if (free > 0) " ${VelmoraDownloads.human(free)} free." else "",
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
      )
      Row(verticalAlignment = Alignment.CenterVertically) {
        Column(Modifier.weight(1f)) {
          Text("Wi-Fi only", style = MaterialTheme.typography.bodyLarge)
          Text("Waits for Wi-Fi when you're on mobile data", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Switch(checked = wifi, onCheckedChange = {
          wifi = it
          VelmoraDownloads.setWifiOnly(it)
        })
      }
      Row(verticalAlignment = Alignment.CenterVertically) {
        Icon(Icons.Rounded.Lock, contentDescription = null, tint = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.size(14.dp))
        Spacer(Modifier.width(6.dp))
        Text("Saved encrypted. It plays offline, only in Velmora.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
      }
      Spacer(Modifier.height(4.dp))
      Button(
        onClick = {
          VelmoraDownloads.setDefaultQuality(quality)
          val name = season?.episodes?.firstOrNull { it.number == ep?.episode }?.title
          val msg = VelmoraDownloads.add(context, title, target, quality, name)
          done(msg ?: if (ep != null) "Downloading S${ep.season} E${ep.episode}" else "Downloading ${title.value.name}")
        },
        modifier = Modifier.fillMaxWidth().height(54.dp),
        shape = RoundedCornerShape(18.dp),
        colors = ButtonDefaults.buttonColors(containerColor = Brand.Blue),
      ) {
        Icon(Icons.Rounded.Download, contentDescription = null)
        Spacer(Modifier.width(8.dp))
        Text(if (ep != null) "This episode" else "Download", style = MaterialTheme.typography.titleMedium)
      }
      if (season != null && season.episodes.size > 1) {
        FilledTonalButton(
          onClick = {
            VelmoraDownloads.setDefaultQuality(quality)
            val n = VelmoraDownloads.addSeason(context, title, season, quality)
            done(if (n == 0) "That season is already on its way" else "Downloading $n episodes of season ${season.number}")
          },
          modifier = Modifier.fillMaxWidth().height(54.dp),
          shape = RoundedCornerShape(18.dp),
        ) {
          Icon(Icons.Rounded.VideoLibrary, contentDescription = null)
          Spacer(Modifier.width(8.dp))
          Text("All of season ${season.number} · ${season.episodes.size} episodes", style = MaterialTheme.typography.titleMedium)
        }
      }
    }
  }
}
