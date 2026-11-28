package com.velmora.stepbyte.velmora.downloads

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Download
import androidx.compose.material.icons.rounded.DownloadDone
import androidx.compose.material.icons.rounded.ExpandLess
import androidx.compose.material.icons.rounded.ExpandMore
import androidx.compose.material.icons.rounded.Lock
import androidx.compose.material.icons.rounded.Pause
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Wifi
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.LinearWavyProgressIndicator
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import coil3.compose.AsyncImage
import com.velmora.stepbyte.LocalPlayerAwareWindowInsets
import com.velmora.stepbyte.engine.Target
import com.velmora.stepbyte.ui.screens.Screens
import com.velmora.stepbyte.velmora.library.VelmoraLibrary
import com.velmora.stepbyte.velmora.player.VideoSession
import com.velmora.stepbyte.velmora.screens.EmptyState
import com.velmora.stepbyte.velmora.screens.openTitle
import com.velmora.stepbyte.velmora.screens.pressable
import com.velmora.stepbyte.velmora.shell.Brand

private val ACTIVE = listOf(DownloadPhase.Finding, DownloadPhase.WaitingWifi, DownloadPhase.Queued, DownloadPhase.Running, DownloadPhase.Paused, DownloadPhase.Failed)

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun DownloadsScreen(navController: NavController) {
  val context = LocalContext.current
  LaunchedEffect(Unit) { VelmoraDownloads.init(context) }
  val items by VelmoraDownloads.items.collectAsState()
  val metered by VelmoraDownloads.metered.collectAsState()
  val active = items.filter { it.record.phase in ACTIVE }
  val done = items.filter { it.record.phase == DownloadPhase.Done }
  // Movies alone, episodes under their show.
  val groups = remember(done) { done.groupBy { VelmoraLibrary.key(it.title) }.values.toList() }
  var free by remember { mutableLongStateOf(-1L) }
  LaunchedEffect(done.size) { free = VelmoraDownloads.freeBytes() }
  var confirm by remember { mutableStateOf<List<DownloadItem>?>(null) }
  val used = done.sumOf { it.record.size }

  LazyColumn(Modifier.fillMaxSize(), contentPadding = LocalPlayerAwareWindowInsets.current.asPaddingValues()) {
    item(key = "head") {
      Column(Modifier.padding(horizontal = 20.dp, vertical = 8.dp)) {
        Text("Downloads", fontFamily = Brand.Serif, fontSize = 44.sp, lineHeight = 46.sp)
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(top = 2.dp)) {
          Icon(Icons.Rounded.Lock, contentDescription = null, tint = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.size(14.dp))
          Spacer(Modifier.width(6.dp))
          Text("Saved encrypted, so they play only in Velmora, offline too.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        if (free >= 0) StorageBar(used, free)
      }
    }
    if (active.isNotEmpty()) {
      item(key = "active-h") { Heading(if (active.size == 1) "Downloading" else "Downloading · ${active.size}") }
      if (metered && VelmoraDownloads.wifiOnly() && active.any { it.record.phase == DownloadPhase.WaitingWifi }) {
        item(key = "wifi") { WifiNote() }
      }
      items(active, key = { "a" + it.record.id }) { d -> ActiveCard(d, Modifier.animateItem()) }
    }
    if (groups.isNotEmpty()) {
      item(key = "done-h") { Heading("On this phone") }
      items(groups, key = { "g" + VelmoraLibrary.key(it.first().title) }) { g ->
        if (g.size == 1 && g.first().record.season == null) {
          MovieRow(g.first(), onDelete = { confirm = g }, onOpen = { navController.openTitle(g.first().title) }, modifier = Modifier.animateItem())
        } else {
          ShowGroup(g, onDelete = { confirm = it }, onOpen = { navController.openTitle(g.first().title) }, modifier = Modifier.animateItem())
        }
      }
    }
    if (items.isEmpty()) {
      item(key = "empty") {
        EmptyState(
          Icons.Rounded.Download,
          "Nothing downloaded yet",
          "Tap the download button on a title page or next to an episode. Whole seasons work too.",
          action = "Find something",
          onAction = { navController.navigate(Screens.VelmoraBrowse.route) },
        )
      }
    }
    item(key = "settings") { SettingsCard() }
  }

  confirm?.let { list ->
    val size = list.sumOf { it.record.size }
    val name = list.first().titleValue.name + if (list.size == 1 && list.first().record.season != null) " · S${list.first().record.season} E${list.first().record.episode}" else ""
    AlertDialog(
      onDismissRequest = { confirm = null },
      title = { Text(if (list.size > 1) "Delete ${list.size} episodes?" else "Delete this download?") },
      text = { Text("$name. That frees ${VelmoraDownloads.human(size)}. You can download it again later.") },
      confirmButton = {
        TextButton(onClick = {
          list.forEach { VelmoraDownloads.remove(it.record.id) }
          confirm = null
        }) { Text("Delete", color = MaterialTheme.colorScheme.error) }
      },
      dismissButton = { TextButton(onClick = { confirm = null }) { Text("Keep") } },
    )
  }
}

@Composable
private fun Heading(text: String) {
  Text(text, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold, modifier = Modifier.padding(start = 20.dp, end = 20.dp, top = 18.dp, bottom = 8.dp))
}

/** Velmora's share next to the free space, in one bar. */
@Composable
private fun StorageBar(used: Long, free: Long) {
  val total = (used + free).coerceAtLeast(1)
  val share = (used.toFloat() / total).coerceIn(if (used > 0) 0.02f else 0f, 1f)
  Column(Modifier.padding(top = 14.dp)) {
    Box(Modifier.fillMaxWidth().height(10.dp).clip(CircleShape).background(MaterialTheme.colorScheme.surfaceContainerHighest)) {
      Box(Modifier.fillMaxWidth(share).fillMaxHeight().clip(CircleShape).background(Brush.horizontalGradient(listOf(Brand.Blue, Brand.Violet))))
    }
    Row(Modifier.fillMaxWidth().padding(top = 6.dp)) {
      Text("${VelmoraDownloads.human(used)} in Velmora", style = MaterialTheme.typography.labelMedium, modifier = Modifier.weight(1f))
      Text("${VelmoraDownloads.human(free)} free", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
  }
}

@Composable
private fun WifiNote() {
  Surface(shape = RoundedCornerShape(18.dp), color = MaterialTheme.colorScheme.tertiaryContainer, modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp).fillMaxWidth()) {
    Row(Modifier.padding(horizontal = 14.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
      Icon(Icons.Rounded.Wifi, contentDescription = null, tint = MaterialTheme.colorScheme.onTertiaryContainer)
      Spacer(Modifier.width(10.dp))
      Text("You're on mobile data. Downloads carry on when you're on Wi-Fi.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onTertiaryContainer, modifier = Modifier.weight(1f))
      TextButton(onClick = { VelmoraDownloads.setWifiOnly(false) }) { Text("Use data") }
    }
  }
}

private fun episodeLine(d: DownloadItem): String? {
  val r = d.record
  if (r.season == null || r.episode == null) return null
  return "S${r.season} · E${r.episode}" + (r.episodeName?.takeIf { it.isNotBlank() }?.let { " · $it" } ?: "")
}

private fun eta(d: DownloadItem): String? {
  val total = d.total ?: return null
  if (d.speed <= 0 || total <= d.downloaded) return null
  val s = (total - d.downloaded) / d.speed
  return when {
    s < 60 -> "under a minute left"
    s < 3600 -> "${s / 60} min left"
    else -> "${s / 3600} h ${(s % 3600) / 60} min left"
  }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun ActiveCard(d: DownloadItem, modifier: Modifier) {
  val r = d.record
  Surface(shape = RoundedCornerShape(22.dp), color = MaterialTheme.colorScheme.surfaceContainer, modifier = modifier.padding(horizontal = 16.dp, vertical = 5.dp).fillMaxWidth()) {
    Row(Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
      AsyncImage(
        model = d.titleValue.poster,
        contentDescription = null,
        contentScale = ContentScale.Crop,
        modifier = Modifier.width(54.dp).aspectRatio(2f / 3f).clip(RoundedCornerShape(12.dp)).background(MaterialTheme.colorScheme.surfaceContainerHighest),
      )
      Spacer(Modifier.width(14.dp))
      Column(Modifier.weight(1f)) {
        Text(d.titleValue.name, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
        episodeLine(d)?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis) }
        Spacer(Modifier.height(8.dp))
        when (r.phase) {
          DownloadPhase.Running ->
            if (d.total != null && d.total > 0) {
              LinearWavyProgressIndicator(progress = { d.fraction }, modifier = Modifier.fillMaxWidth(), color = Brand.Blue)
            } else {
              LinearWavyProgressIndicator(modifier = Modifier.fillMaxWidth(), color = Brand.Blue)
            }
          DownloadPhase.Paused, DownloadPhase.WaitingWifi -> LinearProgressIndicator(progress = { d.fraction }, modifier = Modifier.fillMaxWidth(), drawStopIndicator = {})
          DownloadPhase.Finding, DownloadPhase.Queued -> LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
          else -> {}
        }
        Spacer(Modifier.height(6.dp))
        val status =
          when (r.phase) {
            DownloadPhase.Finding -> "Finding a ${r.quality.label} source…"
            DownloadPhase.WaitingWifi -> "Waiting for Wi-Fi" + if (d.downloaded > 0) " · ${VelmoraDownloads.human(d.downloaded)} so far" else ""
            DownloadPhase.Queued -> "Queued"
            DownloadPhase.Paused -> "Paused" + if (d.downloaded > 0) " · ${VelmoraDownloads.human(d.downloaded)}" + (d.total?.let { " of ${VelmoraDownloads.human(it)}" } ?: "") else ""
            DownloadPhase.Running ->
              listOfNotNull(
                VelmoraDownloads.human(d.downloaded) + (d.total?.let { " of ${VelmoraDownloads.human(it)}" } ?: ""),
                d.speed.takeIf { it > 0 }?.let { "${VelmoraDownloads.human(it)}/s" },
                eta(d),
              ).joinToString(" · ")
            DownloadPhase.Failed -> r.error ?: "Didn't finish"
            DownloadPhase.Done -> "Done"
          }
        Text(
          status,
          style = MaterialTheme.typography.labelMedium,
          color = if (r.phase == DownloadPhase.Failed) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant,
          maxLines = 2,
          overflow = TextOverflow.Ellipsis,
        )
      }
      Spacer(Modifier.width(6.dp))
      when (r.phase) {
        DownloadPhase.Failed -> IconButton(onClick = { VelmoraDownloads.retry(r.id) }) { Icon(Icons.Rounded.Refresh, contentDescription = "Try again") }
        DownloadPhase.Paused -> IconButton(onClick = { VelmoraDownloads.resume(r.id) }) { Icon(Icons.Rounded.PlayArrow, contentDescription = "Resume") }
        else -> IconButton(onClick = { VelmoraDownloads.pause(r.id) }) { Icon(Icons.Rounded.Pause, contentDescription = "Pause") }
      }
      IconButton(onClick = { VelmoraDownloads.remove(r.id) }) { Icon(Icons.Rounded.Close, contentDescription = "Cancel") }
    }
  }
}

@Composable
private fun MovieRow(d: DownloadItem, onDelete: () -> Unit, onOpen: () -> Unit, modifier: Modifier) {
  val context = LocalContext.current
  Surface(shape = RoundedCornerShape(22.dp), color = MaterialTheme.colorScheme.surfaceContainer, modifier = modifier.padding(horizontal = 16.dp, vertical = 5.dp).fillMaxWidth()) {
    Row(Modifier.padding(10.dp), verticalAlignment = Alignment.CenterVertically) {
      Box(Modifier.width(132.dp).aspectRatio(16f / 9f).clip(RoundedCornerShape(14.dp)).background(MaterialTheme.colorScheme.surfaceContainerHighest).pressable { VideoSession.play(context, d.title, Target.Movie) }) {
        AsyncImage(model = d.titleValue.backdrop ?: d.titleValue.poster, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
        Box(Modifier.align(Alignment.Center).size(34.dp).clip(CircleShape).background(Color.Black.copy(alpha = 0.5f)), contentAlignment = Alignment.Center) {
          Icon(Icons.Rounded.PlayArrow, contentDescription = "Play", tint = Color.White, modifier = Modifier.size(22.dp))
        }
      }
      Spacer(Modifier.width(14.dp))
      Column(Modifier.weight(1f).clickable(onClick = onOpen)) {
        Text(d.titleValue.name, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
        Text(listOfNotNull(d.titleValue.year?.toString(), VelmoraDownloads.human(d.record.size)).joinToString(" · "), style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Progressed(d)
      }
      IconButton(onClick = onDelete) { Icon(Icons.Rounded.Delete, contentDescription = "Delete") }
    }
  }
}

/** "Watched 40%", from the saved spot. */
@Composable
private fun Progressed(d: DownloadItem) {
  val progress by VelmoraLibrary.progress.collectAsState()
  val p = progress.firstOrNull { VelmoraLibrary.key(it.title) == VelmoraLibrary.key(d.title) && it.season == d.record.season && it.episode == d.record.episode } ?: return
  if (p.fraction <= 0.01f) return
  Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(top = 6.dp)) {
    Box(Modifier.width(64.dp).height(3.dp).clip(CircleShape).background(MaterialTheme.colorScheme.surfaceContainerHighest)) {
      Box(Modifier.fillMaxWidth(p.fraction).fillMaxHeight().clip(CircleShape).background(Brand.Blue))
    }
    Spacer(Modifier.width(6.dp))
    Text(if (p.fraction >= 0.95f) "Watched" else "${(p.fraction * 100).toInt()}% watched", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
  }
}

@Composable
private fun ShowGroup(list: List<DownloadItem>, onDelete: (List<DownloadItem>) -> Unit, onOpen: () -> Unit, modifier: Modifier) {
  val context = LocalContext.current
  val first = list.first()
  var open by rememberSaveable(VelmoraLibrary.key(first.title)) { mutableStateOf(list.size <= 3) }
  val episodes = list.sortedWith(compareBy({ it.record.season ?: 0 }, { it.record.episode ?: 0 }))
  Surface(shape = RoundedCornerShape(22.dp), color = MaterialTheme.colorScheme.surfaceContainer, modifier = modifier.padding(horizontal = 16.dp, vertical = 5.dp).fillMaxWidth()) {
    Column(Modifier.animateContentSize(spring(stiffness = Spring.StiffnessMediumLow))) {
      Row(Modifier.fillMaxWidth().clickable { open = !open }.padding(10.dp), verticalAlignment = Alignment.CenterVertically) {
        AsyncImage(
          model = first.titleValue.poster,
          contentDescription = null,
          contentScale = ContentScale.Crop,
          modifier = Modifier.width(54.dp).aspectRatio(2f / 3f).clip(RoundedCornerShape(12.dp)).background(MaterialTheme.colorScheme.surfaceContainerHighest).clickable(onClick = onOpen),
        )
        Spacer(Modifier.width(14.dp))
        Column(Modifier.weight(1f)) {
          Text(first.titleValue.name, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
          Text(
            "${if (list.size == 1) "1 episode" else "${list.size} episodes"} · ${VelmoraDownloads.human(list.sumOf { it.record.size })}",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
          )
        }
        Icon(if (open) Icons.Rounded.ExpandLess else Icons.Rounded.ExpandMore, contentDescription = if (open) "Hide episodes" else "Show episodes")
      }
      AnimatedVisibility(visible = open, enter = expandVertically() + fadeIn(), exit = shrinkVertically() + fadeOut()) {
        Column(Modifier.padding(bottom = 6.dp)) {
          episodes.forEach { e ->
            Row(
              Modifier.fillMaxWidth().clickable { VideoSession.play(context, e.title, e.record.target) }.padding(start = 16.dp, end = 4.dp, top = 6.dp, bottom = 6.dp),
              verticalAlignment = Alignment.CenterVertically,
            ) {
              FilledIconButton(
                onClick = { VideoSession.play(context, e.title, e.record.target) },
                colors = IconButtonDefaults.filledIconButtonColors(containerColor = MaterialTheme.colorScheme.secondaryContainer, contentColor = MaterialTheme.colorScheme.onSecondaryContainer),
                modifier = Modifier.size(36.dp),
              ) {
                Icon(Icons.Rounded.PlayArrow, contentDescription = "Play", modifier = Modifier.size(20.dp))
              }
              Spacer(Modifier.width(12.dp))
              Column(Modifier.weight(1f)) {
                Text(episodeLine(e) ?: e.titleValue.name, style = MaterialTheme.typography.bodyMedium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(VelmoraDownloads.human(e.record.size), style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Progressed(e)
              }
              IconButton(onClick = { onDelete(listOf(e)) }) { Icon(Icons.Rounded.Delete, contentDescription = "Delete", modifier = Modifier.size(20.dp)) }
            }
          }
          if (list.size > 1) {
            TextButton(onClick = { onDelete(list) }, modifier = Modifier.padding(start = 8.dp)) { Text("Delete all ${list.size}", color = MaterialTheme.colorScheme.error) }
          }
        }
      }
    }
  }
}

@Composable
private fun SettingsCard() {
  var wifi by remember { mutableStateOf(VelmoraDownloads.wifiOnly()) }
  var quality by remember { mutableStateOf(VelmoraDownloads.defaultQuality()) }
  Surface(shape = RoundedCornerShape(22.dp), color = MaterialTheme.colorScheme.surfaceContainerLow, modifier = Modifier.padding(16.dp).fillMaxWidth()) {
    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
      Text("Download settings", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
      Row(verticalAlignment = Alignment.CenterVertically) {
        Column(Modifier.weight(1f)) {
          Text("Wi-Fi only", style = MaterialTheme.typography.bodyLarge)
          Text("On mobile data, downloads wait", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Switch(checked = wifi, onCheckedChange = {
          wifi = it
          VelmoraDownloads.setWifiOnly(it)
        })
      }
      Text("Quality", style = MaterialTheme.typography.bodyLarge)
      Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        DownloadQuality.entries.forEach { q ->
          FilterChip(selected = quality == q, onClick = {
            quality = q
            VelmoraDownloads.setDefaultQuality(q)
          }, label = { Text(q.label) })
        }
      }
      Text("Smaller is quicker and leaves room for more. 720p is about 1 GB an hour.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
  }
}
