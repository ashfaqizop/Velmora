package com.velmora.stepbyte.velmora.screens

import android.widget.Toast
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.OpenInNew
import androidx.compose.material.icons.rounded.LiveTv
import androidx.compose.material.icons.rounded.Place
import androidx.compose.material.icons.rounded.Public
import androidx.compose.material.icons.rounded.Tv
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import coil3.compose.AsyncImage
import com.velmora.stepbyte.engine.Match
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.engine.WatchOption
import com.velmora.stepbyte.velmora.player.LiveTv
import com.velmora.stepbyte.velmora.shell.Brand
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

/**
 * A match: both teams, the score (refreshed while it's on), when and where,
 * and where to watch it in your country. A free channel Velmora has plays
 * right here; the rest open the broadcaster.
 */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun MatchScreen(initial: Match, navController: NavController) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  val uri = LocalUriHandler.current
  var m by remember { mutableStateOf(initial) }
  var region by remember { mutableStateOf(LiveTv.region(context)) }
  var options by remember { mutableStateOf<List<WatchOption>?>(null) }
  var picking by remember { mutableStateOf(false) }

  // Fresh score while it's on or about to be.
  LaunchedEffect(initial.id) {
    while (isActive) {
      runCatching { Universe.get(context).sportsLookup(initial.id) }.getOrNull()?.let { m = it }
      if (m.status == "finished" || m.status == "cancelled") break
      delay(if (m.live) 30_000 else 120_000)
    }
  }
  LaunchedEffect(region, m.id) {
    options = null
    options = runCatching { Universe.get(context).sportsWatch(m, region) }.getOrDefault(emptyList())
  }

  LazyColumn(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface), contentPadding = bottomInsets().asPaddingValues()) {
    item(key = "hero") { MatchHero(m, onBack = { navController.navigateUp() }) }
    item(key = "facts") {
      Column(Modifier.padding(horizontal = 20.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        m.kickoff?.let { k ->
          Fact(Icons.Rounded.Tv, Instant.ofEpochSecond(k).atZone(ZoneId.systemDefault()).format(DateTimeFormatter.ofPattern("EEEE d MMMM · h:mm a")))
        }
        m.venue?.let { Fact(Icons.Rounded.Place, it) }
        listOfNotNull(m.league.name, m.round, m.league.country).joinToString(" · ").takeIf { it.isNotBlank() }?.let { Fact(Icons.Rounded.Public, it) }
      }
    }
    item(key = "watch-h") {
      Row(Modifier.fillMaxWidth().padding(start = 20.dp, end = 12.dp, top = 12.dp, bottom = 6.dp), verticalAlignment = Alignment.CenterVertically) {
        Text("Where to watch", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold, modifier = Modifier.weight(1f))
        AssistChip(onClick = { picking = true }, label = { Text(flag(region) + " " + region) })
      }
    }
    when (val list = options) {
      null -> item(key = "loading") { Box(Modifier.fillMaxWidth().padding(24.dp), contentAlignment = Alignment.Center) { LoadingIndicator() } }
      else ->
        if (list.isEmpty()) {
          item(key = "none") {
            Text(
              "No broadcaster is listed for this match in $region. Try another country, or look in Live TV.",
              style = MaterialTheme.typography.bodyMedium,
              color = MaterialTheme.colorScheme.onSurfaceVariant,
              modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp),
            )
          }
        } else {
          // What plays here first, then free, then paid.
          val sorted = list.sortedWith(compareBy<WatchOption> { if (it.playChannel != null) 0 else 1 }.thenBy { if (it.kind == "free") 0 else 1 })
          items(sorted, key = { it.name + it.country }) { o ->
            WatchRow(
              o,
              onPlay = {
                scope.launch { if (!LiveTv.play(context, o.playChannel!!)) Toast.makeText(context, "That channel isn't available right now", Toast.LENGTH_SHORT).show() }
              },
              onOpen = { o.url?.let { runCatching { uri.openUri(it) } } },
            )
          }
        }
    }
    item(key = "end") { Spacer(Modifier.height(24.dp)) }
  }

  if (picking) {
    RegionSheet(region, onDismiss = { picking = false }) {
      picking = false
      region = it
      LiveTv.setRegion(context, it)
    }
  }
}

@Composable
private fun MatchHero(m: Match, onBack: () -> Unit) {
  Box(
    Modifier.fillMaxWidth()
      .background(Brush.verticalGradient(listOf(Brand.BlueDeep, MaterialTheme.colorScheme.surface)))
      .windowInsetsPadding(WindowInsets.statusBars),
  ) {
    IconButton(onClick = onBack, modifier = Modifier.padding(4.dp)) { Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = "Back", tint = Color.White) }
    Column(Modifier.fillMaxWidth().padding(top = 52.dp, bottom = 20.dp, start = 16.dp, end = 16.dp), horizontalAlignment = Alignment.CenterHorizontally) {
      Row(verticalAlignment = Alignment.CenterVertically) {
        AsyncImage(model = m.league.badge, contentDescription = null, modifier = Modifier.size(22.dp))
        Spacer(Modifier.width(6.dp))
        Text(m.league.name, color = Color.White.copy(alpha = 0.85f), style = MaterialTheme.typography.labelLarge)
      }
      Spacer(Modifier.height(18.dp))
      if (m.home != null && m.away != null) {
        Row(verticalAlignment = Alignment.CenterVertically) {
          Team(m.home!!.name, m.home!!.badge, Modifier.weight(1f))
          Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.padding(horizontal = 8.dp)) {
            val scored = m.homeScore != null && m.awayScore != null && m.status != "upcoming"
            Text(
              if (scored) "${m.homeScore} – ${m.awayScore}" else m.kickoff?.let { Instant.ofEpochSecond(it).atZone(ZoneId.systemDefault()).format(DateTimeFormatter.ofPattern("h:mm a")) } ?: "vs",
              color = Color.White,
              fontSize = if (scored) 46.sp else 30.sp,
              fontWeight = FontWeight.Bold,
            )
            StatusPill(m)
          }
          Team(m.away!!.name, m.away!!.badge, Modifier.weight(1f))
        }
      } else {
        Text(m.title, color = Color.White, fontFamily = Brand.Serif, fontSize = 34.sp, lineHeight = 36.sp, textAlign = TextAlign.Center)
        Spacer(Modifier.height(8.dp))
        StatusPill(m)
      }
    }
  }
}

@Composable
private fun StatusPill(m: Match) {
  val (text, color) =
    when (m.status) {
      "live" -> (listOfNotNull("LIVE", m.phase).joinToString(" · ")) to Color(0xFFE53935)
      "finished" -> "Full time" to Color.White.copy(alpha = 0.2f)
      "postponed" -> "Postponed" to Brand.Amber.copy(alpha = 0.6f)
      "cancelled" -> "Cancelled" to Color(0x66E53935)
      else -> "Upcoming" to Color.White.copy(alpha = 0.2f)
    }
  Surface(shape = RoundedCornerShape(50), color = color, modifier = Modifier.padding(top = 6.dp)) {
    Text(text, color = Color.White, style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.Bold, modifier = Modifier.padding(horizontal = 10.dp, vertical = 3.dp))
  }
}

@Composable
private fun Team(name: String, badge: String?, modifier: Modifier) {
  Column(modifier, horizontalAlignment = Alignment.CenterHorizontally) {
    Box(Modifier.size(72.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.12f)), contentAlignment = Alignment.Center) {
      AsyncImage(model = badge, contentDescription = name, modifier = Modifier.size(54.dp))
    }
    Spacer(Modifier.height(8.dp))
    Text(name, color = Color.White, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.SemiBold, textAlign = TextAlign.Center, maxLines = 2, overflow = TextOverflow.Ellipsis)
  }
}

@Composable
private fun Fact(icon: androidx.compose.ui.graphics.vector.ImageVector, text: String) {
  Row(verticalAlignment = Alignment.CenterVertically) {
    Icon(icon, contentDescription = null, tint = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.size(18.dp))
    Spacer(Modifier.width(10.dp))
    Text(text, style = MaterialTheme.typography.bodyMedium)
  }
}

@Composable
private fun WatchRow(o: WatchOption, onPlay: () -> Unit, onOpen: () -> Unit) {
  Surface(shape = RoundedCornerShape(20.dp), color = MaterialTheme.colorScheme.surfaceContainer, modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp).fillMaxWidth()) {
    Row(Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
      Box(Modifier.size(44.dp).clip(RoundedCornerShape(12.dp)).background(MaterialTheme.colorScheme.surfaceContainerHighest), contentAlignment = Alignment.Center) {
        Icon(Icons.Rounded.LiveTv, contentDescription = null, tint = MaterialTheme.colorScheme.onSurfaceVariant)
        AsyncImage(model = o.logo, contentDescription = o.name, modifier = Modifier.size(36.dp))
      }
      Spacer(Modifier.width(12.dp))
      Column(Modifier.weight(1f)) {
        Text(o.name, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Text(
          when (o.kind) {
            "free" -> "Free"
            "subscription" -> "Subscription"
            else -> "Broadcaster"
          },
          style = MaterialTheme.typography.labelMedium,
          color = if (o.kind == "free") Brand.Violet else MaterialTheme.colorScheme.onSurfaceVariant,
        )
      }
      when {
        o.playChannel != null ->
          Button(onClick = onPlay, colors = ButtonDefaults.buttonColors(containerColor = Brand.Blue)) { Text("Watch") }
        o.url != null ->
          OutlinedButton(onClick = onOpen) {
            Icon(Icons.AutoMirrored.Rounded.OpenInNew, contentDescription = null, modifier = Modifier.size(16.dp))
            Spacer(Modifier.width(4.dp))
            Text("Open")
          }
      }
    }
  }
}

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
private fun RegionSheet(current: String, onDismiss: () -> Unit, onPick: (String) -> Unit) {
  val codes = remember {
    java.util.Locale.getISOCountries().map { it to java.util.Locale("", it).getDisplayCountry(java.util.Locale.ENGLISH) }.sortedBy { it.second }
  }
  var q by remember { mutableStateOf("") }
  androidx.compose.material3.ModalBottomSheet(onDismissRequest = onDismiss) {
    Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp)) {
      Text("Your country", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
      Text("Broadcasters differ by country.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(bottom = 8.dp))
      androidx.compose.material3.TextField(value = q, onValueChange = { q = it }, singleLine = true, placeholder = { Text("Search") }, modifier = Modifier.fillMaxWidth())
      LazyColumn(Modifier.fillMaxWidth().height(420.dp)) {
        items(codes.filter { q.isBlank() || it.second.contains(q, ignoreCase = true) || it.first.equals(q, ignoreCase = true) }, key = { it.first }) { (code, name) ->
          androidx.compose.material3.ListItem(
            headlineContent = { Text(flag(code) + "  " + name) },
            trailingContent = { if (code == current) Text("✓") },
            modifier = Modifier.clickable { onPick(code) },
          )
        }
      }
    }
  }
}

