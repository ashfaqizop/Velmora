package com.velmora.stepbyte.velmora.player

import androidx.annotation.OptIn
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.ChevronRight
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.GraphicEq
import androidx.compose.material.icons.rounded.HighQuality
import androidx.compose.material.icons.rounded.Hub
import androidx.compose.material.icons.rounded.NetworkCell
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.SignalCellularAlt
import androidx.compose.material.icons.rounded.SkipNext
import androidx.compose.material.icons.rounded.Speed
import androidx.compose.material.icons.rounded.Subtitles
import androidx.compose.material.icons.rounded.Translate
import androidx.compose.material.icons.rounded.Verified
import androidx.compose.material.icons.rounded.VideoLibrary
import androidx.compose.material.icons.rounded.Headphones
import androidx.compose.material.icons.rounded.DataSaverOn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.media3.common.C
import androidx.media3.common.Format
import androidx.media3.common.TrackSelectionOverride
import androidx.media3.common.Tracks
import androidx.media3.common.util.UnstableApi
import coil3.compose.AsyncImage
import com.velmora.stepbyte.engine.Source
import com.velmora.stepbyte.engine.Target
import com.velmora.stepbyte.velmora.player.SourceLabels.key
import com.velmora.stepbyte.velmora.shell.Brand

private val SPEEDS = listOf(0.5f, 0.75f, 1f, 1.25f, 1.5f, 1.75f, 2f)
private val SUB_SIZES = listOf(0.8f to "Small", 1f to "Normal", 1.25f to "Large", 1.5f to "Huge")

@kotlin.OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PlayerSheets(sheet: Sheet, ui: PlaybackUi, onSheet: (Sheet) -> Unit, onFlash: (String) -> Unit) {
  if (sheet == Sheet.None) return
  val state = rememberModalBottomSheetState(skipPartiallyExpanded = true)
  ModalBottomSheet(onDismissRequest = { onSheet(Sheet.None) }, sheetState = state, containerColor = Color(0xFF12151D), contentColor = Color.White) {
    Column(Modifier.fillMaxWidth().heightIn(max = 520.dp).padding(bottom = 16.dp)) {
      when (sheet) {
        Sheet.Settings -> SettingsSheet(ui, onSheet)
        Sheet.Sources -> SourcesSheet(ui, onSheet)
        Sheet.Subtitles -> SubtitlesSheet(onSheet, onFlash)
        Sheet.Audio -> AudioSheet(ui, onSheet)
        Sheet.Quality -> QualitySheet(onSheet)
        Sheet.Speed -> SpeedSheet(onSheet, onFlash)
        Sheet.Episodes -> EpisodesSheet(ui, onSheet)
        Sheet.None -> {}
      }
    }
  }
}

@Composable
private fun SheetTitle(text: String, onBack: (() -> Unit)? = null) {
  Row(Modifier.fillMaxWidth().padding(start = if (onBack == null) 20.dp else 4.dp, end = 20.dp, bottom = 8.dp), verticalAlignment = Alignment.CenterVertically) {
    if (onBack != null) IconButton(onClick = onBack) { Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = "Back", tint = Color.White) }
    Text(text, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
  }
}

@Composable
private fun MenuRow(icon: ImageVector, title: String, value: String? = null, onClick: () -> Unit, trailing: @Composable (() -> Unit)? = null) {
  Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 20.dp, vertical = 14.dp), verticalAlignment = Alignment.CenterVertically) {
    Icon(icon, contentDescription = null, tint = Color.White.copy(alpha = 0.85f))
    Spacer(Modifier.width(16.dp))
    Column(Modifier.weight(1f)) {
      Text(title, style = MaterialTheme.typography.bodyLarge)
      value?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = Color.White.copy(alpha = 0.6f), maxLines = 1, overflow = TextOverflow.Ellipsis) }
    }
    if (trailing != null) trailing() else Icon(Icons.AutoMirrored.Rounded.ChevronRight, contentDescription = null, tint = Color.White.copy(alpha = 0.5f))
  }
}

// ── Settings ──

@OptIn(UnstableApi::class)
@Composable
private fun SettingsSheet(ui: PlaybackUi, onSheet: (Sheet) -> Unit) {
  val context = LocalContext.current
  val p = VideoSession.player
  var background by remember { mutableStateOf(PlayerPrefs.background(context)) }
  var autoNext by remember { mutableStateOf(PlayerPrefs.autoNext(context)) }
  var saveData by remember { mutableStateOf(PlayerPrefs.saveData(context)) }
  var mobile by remember { mutableStateOf(PlayerPrefs.mobileData(context)) }
  val videoHeight = p?.videoFormat?.height?.takeIf { it > 0 }
  LazyColumn {
    item { SheetTitle("Playback") }
    item { MenuRow(Icons.Rounded.HighQuality, "Quality", videoHeight?.let { "${it}p now" } ?: "Auto", onClick = { onSheet(Sheet.Quality) }) }
    item { MenuRow(Icons.Rounded.Subtitles, "Subtitles", currentText(p?.currentTracks) ?: "Off", onClick = { onSheet(Sheet.Subtitles) }) }
    item { MenuRow(Icons.Rounded.GraphicEq, "Audio", currentAudio(p?.currentTracks) ?: "Default", onClick = { onSheet(Sheet.Audio) }) }
    item { MenuRow(Icons.Rounded.Speed, "Speed", speedLabel(p?.playbackParameters?.speed ?: 1f), onClick = { onSheet(Sheet.Speed) }) }
    item { MenuRow(Icons.Rounded.SignalCellularAlt, "Source", ui.current?.let { SourceLabels.title(it) + " · " + SourceLabels.chips(it).take(2).joinToString(" · ") }, onClick = { onSheet(Sheet.Sources) }) }
    item { HorizontalDivider(Modifier.padding(vertical = 6.dp), color = Color.White.copy(alpha = 0.08f)) }
    item {
      MenuRow(Icons.Rounded.Headphones, "Keep playing in the background", "Sound goes on with the screen off or in another app", onClick = {
        background = !background
        PlayerPrefs.setBackground(context, background)
      }) { Switch(checked = background, onCheckedChange = { background = it; PlayerPrefs.setBackground(context, it) }) }
    }
    item {
      MenuRow(Icons.Rounded.SkipNext, "Play the next episode", "Starts by itself when an episode ends", onClick = {
        autoNext = !autoNext
        PlayerPrefs.setAutoNext(context, autoNext)
      }) { Switch(checked = autoNext, onCheckedChange = { autoNext = it; PlayerPrefs.setAutoNext(context, it) }) }
    }
    item {
      MenuRow(Icons.Rounded.NetworkCell, "On mobile data", mobile.label + " · " + mobile.detail, onClick = {
        mobile = PlayerPrefs.MobileData.entries[(mobile.ordinal + 1) % PlayerPrefs.MobileData.entries.size]
        PlayerPrefs.setMobileData(context, mobile)
      }) { Text(mobile.label, style = MaterialTheme.typography.labelLarge, color = Brand.Violet) }
    }
    item {
      MenuRow(Icons.Rounded.DataSaverOn, "Save data", "Prefer 720p sources on mobile data", onClick = {
        saveData = !saveData
        PlayerPrefs.setSaveData(context, saveData)
      }) { Switch(checked = saveData, onCheckedChange = { saveData = it; PlayerPrefs.setSaveData(context, it) }) }
    }
  }
}

private fun speedLabel(s: Float) = if (s == 1f) "Normal" else "${s.toString().removeSuffix(".0")}x"

// ── Sources ──

@Composable
private fun SourcesSheet(ui: PlaybackUi, onSheet: (Sheet) -> Unit) {
  val context = LocalContext.current
  SheetTitle("Sources", onBack = { onSheet(Sheet.Settings) })
  if (ui.title?.value?.kind == "anime") {
    Row(Modifier.padding(horizontal = 20.dp, vertical = 4.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
      FilterChip(selected = !ui.dub, onClick = { if (ui.dub) VideoSession.setDub(false).also { onSheet(Sheet.None) } }, label = { Text("Subbed") }, leadingIcon = { Icon(Icons.Rounded.Subtitles, null, Modifier.size(18.dp)) })
      FilterChip(selected = ui.dub, onClick = { if (!ui.dub) VideoSession.setDub(true).also { onSheet(Sheet.None) } }, label = { Text("Dubbed") }, leadingIcon = { Icon(Icons.Rounded.Translate, null, Modifier.size(18.dp)) })
    }
  }
  if (ui.sources.isEmpty()) {
    Text("No sources yet.", color = Color.White.copy(alpha = 0.6f), modifier = Modifier.padding(20.dp))
    return
  }
  Text("${ui.sources.size} found, best first", style = MaterialTheme.typography.labelMedium, color = Color.White.copy(alpha = 0.6f), modifier = Modifier.padding(horizontal = 20.dp, vertical = 4.dp))
  LazyColumn {
    items(ui.sources, key = { it.key }) { s ->
      SourceRow(s, current = ui.current?.key == s.key, failed = s.key in ui.tried && ui.current?.key != s.key) {
        onSheet(Sheet.None)
        if (ui.current?.key != s.key) VideoSession.switchTo(s)
      }
    }
  }
}

@Composable
private fun SourceRow(s: Source, current: Boolean, failed: Boolean, onClick: () -> Unit) {
  val v = s.value
  Row(
    Modifier.fillMaxWidth().clickable(onClick = onClick).background(if (current) Brand.Blue.copy(alpha = 0.18f) else Color.Transparent).padding(horizontal = 20.dp, vertical = 12.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    Box(Modifier.size(40.dp).clip(RoundedCornerShape(12.dp)).background(if (current) Brand.Blue else Color.White.copy(alpha = 0.08f)), contentAlignment = Alignment.Center) {
      Icon(
        when {
          current -> Icons.Rounded.PlayArrow
          v.isTorrent -> Icons.Rounded.Hub
          else -> Icons.Rounded.SignalCellularAlt
        },
        contentDescription = null,
        tint = Color.White,
        modifier = Modifier.size(20.dp),
      )
    }
    Spacer(Modifier.width(14.dp))
    Column(Modifier.weight(1f)) {
      Row(verticalAlignment = Alignment.CenterVertically) {
        Text(SourceLabels.title(s), style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f, fill = false))
        if (v.verified == true) {
          Spacer(Modifier.width(6.dp))
          Icon(Icons.Rounded.Verified, contentDescription = "Checked", tint = Brand.Violet, modifier = Modifier.size(16.dp))
        }
      }
      val chips = SourceLabels.chips(s)
      if (chips.isNotEmpty()) {
        FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalArrangement = Arrangement.spacedBy(4.dp), modifier = Modifier.padding(top = 4.dp)) {
          chips.forEach { c ->
            Surface(shape = RoundedCornerShape(6.dp), color = Color.White.copy(alpha = 0.08f)) {
              Text(c, style = MaterialTheme.typography.labelSmall, modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp))
            }
          }
        }
      }
      val line = listOfNotNull(SourceLabels.audio(s), if (failed) "Didn't play" else null, v.filename).joinToString(" · ")
      if (line.isNotEmpty()) Text(line, style = MaterialTheme.typography.bodySmall, color = if (failed) Color(0xFFFF8A80) else Color.White.copy(alpha = 0.55f), maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(top = 2.dp))
    }
    if (current) Icon(Icons.Rounded.Check, contentDescription = "Playing", tint = Brand.Violet)
  }
}

// ── Tracks ──

private fun trackName(f: Format, fallback: String): String {
  val lang = f.language?.takeIf { it.isNotBlank() && it != C.LANGUAGE_UNDETERMINED }?.let(SourceLabels::language)
  val label = f.label?.takeIf { it.isNotBlank() }
  return when {
    label != null && lang != null && !label.contains(lang, ignoreCase = true) -> "$label · $lang"
    label != null -> label
    lang != null -> lang
    else -> fallback
  }
}

private fun audioExtra(f: Format): String? {
  val ch =
    when (f.channelCount) {
      1 -> "Mono"
      2 -> "Stereo"
      6 -> "5.1"
      8 -> "7.1"
      Format.NO_VALUE -> null
      else -> "${f.channelCount} ch"
    }
  val codec =
    when (f.sampleMimeType) {
      "audio/ac3" -> "Dolby Digital"
      "audio/eac3", "audio/eac3-joc" -> "Dolby Digital+"
      "audio/vnd.dts", "audio/vnd.dts.hd" -> "DTS"
      "audio/true-hd" -> "TrueHD"
      "audio/mp4a-latm" -> "AAC"
      "audio/opus" -> "Opus"
      else -> null
    }
  return listOfNotNull(codec, ch).joinToString(" ").ifEmpty { null }
}

private fun currentText(tracks: Tracks?): String? =
  tracks?.groups?.firstOrNull { it.type == C.TRACK_TYPE_TEXT && it.isSelected }?.let { g ->
    (0 until g.length).firstOrNull { g.isTrackSelected(it) }?.let { trackName(g.getTrackFormat(it), "On") }
  }

private fun currentAudio(tracks: Tracks?): String? =
  tracks?.groups?.firstOrNull { it.type == C.TRACK_TYPE_AUDIO && it.isSelected }?.let { g ->
    (0 until g.length).firstOrNull { g.isTrackSelected(it) }?.let { trackName(g.getTrackFormat(it), "Audio") }
  }

@Composable
private fun TrackRow(title: String, detail: String?, selected: Boolean, onClick: () -> Unit) {
  Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 20.dp, vertical = 14.dp), verticalAlignment = Alignment.CenterVertically) {
    Box(Modifier.size(22.dp), contentAlignment = Alignment.Center) {
      if (selected) Icon(Icons.Rounded.Check, contentDescription = null, tint = Brand.Violet)
    }
    Spacer(Modifier.width(16.dp))
    Column(Modifier.weight(1f)) {
      Text(title, style = MaterialTheme.typography.bodyLarge, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal)
      detail?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = Color.White.copy(alpha = 0.55f)) }
    }
  }
}

@OptIn(UnstableApi::class)
@Composable
private fun SubtitlesSheet(onSheet: (Sheet) -> Unit, onFlash: (String) -> Unit) {
  val context = LocalContext.current
  val p = VideoSession.player ?: return
  val groups = p.currentTracks.groups.filter { it.type == C.TRACK_TYPE_TEXT }
  val off = p.trackSelectionParameters.disabledTrackTypes.contains(C.TRACK_TYPE_TEXT) || groups.none { it.isSelected }
  var scale by remember { mutableStateOf(PlayerPrefs.subtitleScale(context)) }
  SheetTitle("Subtitles", onBack = { onSheet(Sheet.Settings) })
  Row(Modifier.padding(horizontal = 20.dp, vertical = 4.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
    SUB_SIZES.forEach { (v, label) ->
      FilterChip(selected = scale == v, onClick = {
        scale = v
        PlayerPrefs.setSubtitleScale(context, v)
      }, label = { Text(label) })
    }
  }
  LazyColumn {
    item {
      TrackRow("Off", null, off) {
        p.trackSelectionParameters = p.trackSelectionParameters.buildUpon().setTrackTypeDisabled(C.TRACK_TYPE_TEXT, true).build()
        onFlash("Subtitles off")
        onSheet(Sheet.None)
      }
    }
    if (groups.isEmpty()) {
      item { Text("This source has no subtitles. Another source may.", color = Color.White.copy(alpha = 0.6f), modifier = Modifier.padding(20.dp)) }
    }
    groups.forEach { g ->
      for (i in 0 until g.length) {
        if (!g.isTrackSupported(i)) continue
        val f = g.getTrackFormat(i)
        item {
          TrackRow(trackName(f, "Subtitles ${i + 1}"), if (f.selectionFlags and C.SELECTION_FLAG_FORCED != 0) "Forced" else null, !off && g.isTrackSelected(i)) {
            p.trackSelectionParameters =
              p.trackSelectionParameters.buildUpon().setTrackTypeDisabled(C.TRACK_TYPE_TEXT, false).setOverrideForType(TrackSelectionOverride(g.mediaTrackGroup, i)).build()
            onFlash(trackName(f, "Subtitles on"))
            onSheet(Sheet.None)
          }
        }
      }
    }
  }
}

@OptIn(UnstableApi::class)
@Composable
private fun AudioSheet(ui: PlaybackUi, onSheet: (Sheet) -> Unit) {
  val p = VideoSession.player ?: return
  val groups = p.currentTracks.groups.filter { it.type == C.TRACK_TYPE_AUDIO }
  SheetTitle("Audio", onBack = { onSheet(Sheet.Settings) })
  LazyColumn {
    if (groups.sumOf { it.length } <= 1) {
      item {
        Text(
          "This source has one audio track. Sources with other languages are in Sources" + if (ui.title?.value?.kind == "anime") ", with dubbed and subbed." else ".",
          color = Color.White.copy(alpha = 0.6f),
          modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp),
        )
      }
    }
    groups.forEach { g ->
      for (i in 0 until g.length) {
        val f = g.getTrackFormat(i)
        val supported = g.isTrackSupported(i)
        item {
          TrackRow(trackName(f, "Track ${i + 1}"), listOfNotNull(audioExtra(f), if (!supported) "Can't play on this phone" else null).joinToString(" · ").ifEmpty { null }, g.isTrackSelected(i)) {
            if (!supported) return@TrackRow
            p.trackSelectionParameters = p.trackSelectionParameters.buildUpon().setOverrideForType(TrackSelectionOverride(g.mediaTrackGroup, i)).build()
            onSheet(Sheet.None)
          }
        }
      }
    }
  }
}

@OptIn(UnstableApi::class)
@Composable
private fun QualitySheet(onSheet: (Sheet) -> Unit) {
  val p = VideoSession.player ?: return
  val groups = p.currentTracks.groups.filter { it.type == C.TRACK_TYPE_VIDEO }
  val options = groups.flatMap { g -> (0 until g.length).filter { g.isTrackSupported(it) }.map { g to it } }.sortedByDescending { (g, i) -> g.getTrackFormat(i).height }
  val auto = p.trackSelectionParameters.overrides.keys.none { it.type == C.TRACK_TYPE_VIDEO }
  SheetTitle("Quality", onBack = { onSheet(Sheet.Settings) })
  LazyColumn {
    if (options.size > 1) {
      item {
        TrackRow("Auto", "Follows your connection", auto) {
          p.trackSelectionParameters = p.trackSelectionParameters.buildUpon().clearOverridesOfType(C.TRACK_TYPE_VIDEO).build()
          onSheet(Sheet.None)
        }
      }
    }
    options.forEach { (g, i) ->
      val f = g.getTrackFormat(i)
      item {
        TrackRow(
          if (f.height > 0) "${f.height}p" else "Original",
          listOfNotNull(f.bitrate.takeIf { it > 0 }?.let { "%.1f Mbps".format(it / 1_000_000f) }, f.codecs).joinToString(" · ").ifEmpty { null },
          !auto && g.isTrackSelected(i) || options.size == 1,
        ) {
          if (options.size > 1) p.trackSelectionParameters = p.trackSelectionParameters.buildUpon().setOverrideForType(TrackSelectionOverride(g.mediaTrackGroup, i)).build()
          onSheet(Sheet.None)
        }
      }
    }
    if (options.size <= 1) {
      item { Text("This source comes in one quality. Sources lists the others, from SD to 4K.", color = Color.White.copy(alpha = 0.6f), modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp)) }
      item { MenuRow(Icons.Rounded.SignalCellularAlt, "Sources", null, onClick = { onSheet(Sheet.Sources) }) }
    }
  }
}

@Composable
private fun SpeedSheet(onSheet: (Sheet) -> Unit, onFlash: (String) -> Unit) {
  val p = VideoSession.player ?: return
  val now = p.playbackParameters.speed
  SheetTitle("Speed", onBack = { onSheet(Sheet.Settings) })
  FlowRow(Modifier.padding(horizontal = 20.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
    SPEEDS.forEach { s ->
      FilterChip(selected = now == s, onClick = {
        p.setPlaybackSpeed(s)
        onFlash(speedLabel(s))
        onSheet(Sheet.None)
      }, label = { Text(speedLabel(s)) })
    }
  }
}

// ── Episodes ──

@Composable
private fun EpisodesSheet(ui: PlaybackUi, onSheet: (Sheet) -> Unit) {
  val seasons = ui.seasons.filter { it.episodes.isNotEmpty() }.sortedBy { if (it.number == 0) Int.MAX_VALUE else it.number }
  val now = ui.target as? Target.Episode
  var selected by remember { mutableIntStateOf(now?.season ?: seasons.firstOrNull()?.number ?: 1) }
  val season = seasons.firstOrNull { it.number == selected } ?: seasons.firstOrNull() ?: return
  SheetTitle("Episodes")
  if (seasons.size > 1) {
    LazyRow(contentPadding = PaddingValues(horizontal = 20.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
      items(seasons, key = { it.number }) { s ->
        FilterChip(selected = s.number == season.number, onClick = { selected = s.number }, label = { Text(s.name?.takeIf { it.isNotBlank() } ?: if (s.number == 0) "Specials" else "Season ${s.number}") })
      }
    }
    Spacer(Modifier.height(6.dp))
  }
  LazyColumn {
    items(season.episodes, key = { "${it.season}-${it.number}" }) { ep ->
      val playing = now?.season == ep.season && now.episode == ep.number
      Row(
        Modifier.fillMaxWidth().clickable {
          onSheet(Sheet.None)
          if (!playing) VideoSession.playEpisode(ep.season, ep.number)
        }.background(if (playing) Brand.Blue.copy(alpha = 0.18f) else Color.Transparent).padding(horizontal = 20.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
      ) {
        Box(Modifier.width(120.dp).aspectRatio(16f / 9f).clip(RoundedCornerShape(12.dp)).background(Color.White.copy(alpha = 0.08f))) {
          AsyncImage(model = ep.thumbnail ?: ui.title?.value?.backdrop, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
          if (playing) {
            Box(Modifier.align(Alignment.Center).size(30.dp).clip(CircleShape).background(Brand.Blue), contentAlignment = Alignment.Center) {
              Icon(Icons.Rounded.PlayArrow, contentDescription = null, tint = Color.White, modifier = Modifier.size(18.dp))
            }
          }
        }
        Spacer(Modifier.width(14.dp))
        Column(Modifier.weight(1f)) {
          Text("${ep.number}. ${ep.title?.takeIf { it.isNotBlank() } ?: "Episode ${ep.number}"}", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
          ep.overview?.takeIf { it.isNotBlank() }?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = Color.White.copy(alpha = 0.6f), maxLines = 2, overflow = TextOverflow.Ellipsis) }
        }
      }
    }
  }
}

@Suppress("unused")
private val unusedIcon = Icons.Rounded.VideoLibrary
