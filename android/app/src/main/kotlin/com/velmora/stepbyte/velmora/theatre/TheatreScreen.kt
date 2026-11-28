package com.velmora.stepbyte.velmora.theatre

import android.content.Context
import android.widget.Toast
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.ConfirmationNumber
import androidx.compose.material.icons.rounded.EventSeat
import androidx.compose.material.icons.rounded.Groups
import androidx.compose.material.icons.rounded.HourglassTop
import androidx.compose.material.icons.rounded.Login
import androidx.compose.material.icons.rounded.Theaters
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import coil3.compose.AsyncImage
import com.velmora.stepbyte.LocalPlayerAwareWindowInsets
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.velmora.account.AccountAvatar
import com.velmora.stepbyte.velmora.account.AccountState
import com.velmora.stepbyte.velmora.account.Profile
import com.velmora.stepbyte.velmora.account.VelmoraAccount
import com.velmora.stepbyte.velmora.player.TheatreShow
import com.velmora.stepbyte.velmora.player.VideoSession
import com.velmora.stepbyte.velmora.screens.EmptyState
import com.velmora.stepbyte.velmora.screens.pressable
import com.velmora.stepbyte.velmora.shell.Brand
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/** Asked for by a notification: open Theatre on this screening (and enter it if it's on). */
object TheatreFocus {
  val requested = kotlinx.coroutines.flow.MutableStateFlow<String?>(null)
}

// ── Entering ───────────────────────────────────────────────────────────

/** Enter a screening: ask the server, join the room, open the player in its lobby or at live. */
fun enterScreening(context: Context, scope: CoroutineScope, s: Screening) {
  scope.launch {
    when (val e = Theatre.enter(s)) {
      is Theatre.Entry.Refused -> Toast.makeText(context, refusal(e.reason), Toast.LENGTH_LONG).show()
      is Theatre.Entry.Allowed -> {
        val sc = e.screening
        val title = runCatching { Universe.get(context).tmdbTitles(listOf(sc.k)).firstOrNull() }.getOrNull()
        if (title == null) {
          Toast.makeText(context, "Couldn't load ${sc.name}. Check your connection.", Toast.LENGTH_LONG).show()
          return@launch
        }
        TheatreRoom.join(sc.id, e.capacity, sc.chatMode)
        VideoSession.playTheatre(
          context,
          title,
          TheatreShow(
            id = sc.id,
            name = sc.name,
            startsAt = sc.startMs,
            endsAt = sc.endMs,
            capacity = e.capacity,
            runtimeMin = sc.runtime,
            backdrop = sc.backdrop,
            poster = sc.poster,
            offset = Theatre.offset,
          ),
        )
      }
    }
  }
}

private fun refusal(reason: String): String =
  when (reason) {
    "not_yet" -> "The lobby opens 30 minutes before the start."
    "over" -> "This screening has finished."
    "waitlist" -> "You're on the waitlist. Velmora tells you if a seat frees up."
    "no_ticket" -> "This screening needs a ticket. Book one first."
    "sign_in" -> "Sign in to go to Theatre."
    else -> reason
  }

// ── Time words ─────────────────────────────────────────────────────────

private val TIME = DateTimeFormatter.ofPattern("h:mm a")

fun localTime(ms: Long): String = Instant.ofEpochMilli(ms).atZone(ZoneId.systemDefault()).format(TIME)

private fun dayLabel(ms: Long): String {
  val d = Instant.ofEpochMilli(ms).atZone(ZoneId.systemDefault()).toLocalDate()
  val today = LocalDate.now()
  return when (d) {
    today -> "Today"
    today.plusDays(1) -> "Tomorrow"
    today.minusDays(1) -> "Yesterday"
    else -> d.format(DateTimeFormatter.ofPattern("EEEE d MMMM"))
  }
}

/** "2 h 14 min", "14 min", "45 s". */
fun span(ms: Long): String {
  val s = (ms / 1000).coerceAtLeast(0)
  return when {
    s >= 3600 -> "${s / 3600} h ${(s % 3600) / 60} min"
    s >= 60 -> "${s / 60} min"
    else -> "$s s"
  }
}

/** "1:02:15", for the big countdowns. */
fun countdown(ms: Long): String {
  val s = (ms / 1000).coerceAtLeast(0)
  val h = s / 3600
  val m = (s % 3600) / 60
  val sec = s % 60
  return if (h > 0) "%d:%02d:%02d".format(h, m, sec) else "%d:%02d".format(m, sec)
}

// ── The page ───────────────────────────────────────────────────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TheatreScreen(navController: NavController) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  LaunchedEffect(Unit) { Theatre.start(context) }
  val lineup by Theatre.lineup.collectAsState()
  val tickets by Theatre.tickets.collectAsState()
  val loaded by Theatre.loaded.collectAsState()
  val failed by Theatre.failed.collectAsState()
  val settings by Theatre.settings.collectAsState()
  val now by Theatre.now.collectAsState()
  val account by VelmoraAccount.state.collectAsState()
  val signedIn = account is AccountState.SignedIn
  var tab by rememberSaveable { mutableStateOf(0) }
  var refreshing by remember { mutableStateOf(false) }
  val focus by TheatreFocus.requested.collectAsState()

  // A notification asked for a screening: enter it if it's on.
  LaunchedEffect(focus, lineup) {
    val id = focus ?: return@LaunchedEffect
    val s = lineup.firstOrNull { it.id == id } ?: return@LaunchedEffect
    TheatreFocus.requested.value = null
    if (Theatre.phase(s) in listOf(ShowPhase.Lobby, ShowPhase.Live)) enterScreening(context, scope, s)
  }

  val current = remember(lineup, now) { Theatre.current(lineup) }
  val mine = remember(current, tickets) { current.filter { Theatre.holding(it) } }
  val featured =
    remember(current, tickets, now) {
      current.firstOrNull { Theatre.holding(it) && Theatre.phase(it) in listOf(ShowPhase.Lobby, ShowPhase.Live) }
        ?: current.firstOrNull { Theatre.phase(it) == ShowPhase.Live }
        ?: current.firstOrNull { Theatre.phase(it) == ShowPhase.Lobby }
        ?: current.firstOrNull { it.status == "scheduled" }
    }
  val shown = (if (tab == 0) current else mine).filter { it.id != featured?.id || tab == 1 }
  val days = remember(shown) { shown.groupBy { dayLabel(it.startMs) } }

  PullToRefreshBox(
    isRefreshing = refreshing,
    onRefresh = {
      scope.launch {
        refreshing = true
        Theatre.load()
        refreshing = false
      }
    },
    modifier = Modifier.fillMaxSize(),
  ) {
    LazyColumn(Modifier.fillMaxSize(), contentPadding = LocalPlayerAwareWindowInsets.current.asPaddingValues()) {
      item(key = "head") {
        Column(Modifier.padding(horizontal = 20.dp, vertical = 6.dp)) {
          Text("Theatre", fontFamily = Brand.Serif, fontSize = 44.sp, lineHeight = 46.sp)
          Text(
            "Films everyone watches at the same moment, with a live chat that's never saved.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
          )
        }
      }
      if (settings.banner.isNotBlank() || settings.paused) {
        item(key = "banner") {
          Surface(shape = RoundedCornerShape(18.dp), color = MaterialTheme.colorScheme.tertiaryContainer, modifier = Modifier.padding(horizontal = 16.dp, vertical = 6.dp).fillMaxWidth()) {
            Text(
              settings.banner.ifBlank { "Theatre is paused for a bit. Booking will be back soon." },
              style = MaterialTheme.typography.bodyMedium,
              color = MaterialTheme.colorScheme.onTertiaryContainer,
              modifier = Modifier.padding(14.dp),
            )
          }
        }
      }
      featured?.let { s ->
        item(key = "featured") { Featured(s, now, onEnter = { enterScreening(context, scope, s) }) }
      }
      item(key = "tabs") {
        SingleChoiceSegmentedButtonRow(Modifier.padding(horizontal = 16.dp, vertical = 10.dp).fillMaxWidth()) {
          listOf("Lineup", "My tickets" + if (mine.isNotEmpty()) " · ${mine.size}" else "").forEachIndexed { i, label ->
            SegmentedButton(selected = tab == i, onClick = { tab = i }, shape = SegmentedButtonDefaults.itemShape(i, 2)) { Text(label) }
          }
        }
      }
      if (!signedIn) {
        item(key = "signin") {
          Surface(shape = RoundedCornerShape(18.dp), color = MaterialTheme.colorScheme.surfaceContainer, modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp).fillMaxWidth()) {
            Row(Modifier.padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
              Icon(Icons.Rounded.Login, contentDescription = null)
              Spacer(Modifier.width(10.dp))
              Text("Sign in to book seats and chat in screenings.", style = MaterialTheme.typography.bodyMedium)
            }
          }
        }
      }
      if (!loaded) {
        item(key = "loading") { Box(Modifier.fillMaxWidth().padding(48.dp), contentAlignment = Alignment.Center) { androidx.compose.material3.CircularProgressIndicator() } }
      } else if (failed && lineup.isEmpty()) {
        item(key = "failed") {
          EmptyState(Icons.Rounded.Theaters, "Theatre didn't load", "Check your connection, then pull down to try again.", action = "Try again", onAction = { scope.launch { Theatre.load() } })
        }
      } else if (shown.isEmpty() && featured == null) {
        item(key = "empty") {
          EmptyState(
            if (tab == 1) Icons.Rounded.ConfirmationNumber else Icons.Rounded.Theaters,
            if (tab == 1) "No tickets yet" else "Nothing on yet",
            if (tab == 1) "Book a screening from the lineup and it shows here, with a reminder before the lobby opens." else "Today's lineup is drafted every night. Check back soon.",
          )
        }
      } else if (shown.isEmpty() && tab == 1) {
        item(key = "empty-mine") { EmptyState(Icons.Rounded.ConfirmationNumber, "No tickets yet", "Book a screening and it shows here, with a reminder before the lobby opens.") }
      }
      days.forEach { (day, list) ->
        item(key = "d-$day-$tab") {
          Text(day, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold, modifier = Modifier.padding(start = 20.dp, end = 20.dp, top = 16.dp, bottom = 6.dp))
        }
        items(list, key = { "s-${it.id}-$tab" }) { s -> ScreeningCard(s, now, onEnter = { enterScreening(context, scope, s) }, modifier = Modifier.animateItem()) }
      }
      item(key = "end") { Spacer(Modifier.height(24.dp)) }
    }
  }
}

// ── Pieces ─────────────────────────────────────────────────────────────

@Composable
private fun Pulse(color: Color, label: String) {
  val t = rememberInfiniteTransition(label = "pulse")
  val a by t.animateFloat(0.35f, 1f, infiniteRepeatable(tween(850), RepeatMode.Reverse), label = "a")
  Surface(shape = RoundedCornerShape(50), color = color) {
    Row(Modifier.padding(horizontal = 9.dp, vertical = 3.dp), verticalAlignment = Alignment.CenterVertically) {
      Box(Modifier.size(7.dp).graphicsLayer { alpha = a }.clip(CircleShape).background(Color.White))
      Spacer(Modifier.width(5.dp))
      Text(label, color = Color.White, style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.Bold)
    }
  }
}

@Composable
fun Face(f: Friend, size: Dp = 26.dp, modifier: Modifier = Modifier) {
  AccountAvatar(Profile(id = f.id, username = f.username, displayName = f.displayName.ifBlank { f.username }, avatar = f.avatar), size = size, modifier = modifier)
}

/** Overlapping faces of people you follow who are going. */
@Composable
private fun FriendsGoing(list: List<Friend>) {
  if (list.isEmpty()) return
  Row(verticalAlignment = Alignment.CenterVertically) {
    Box(Modifier.width((22 + 16 * (list.take(4).size - 1)).dp).height(24.dp)) {
      list.take(4).forEachIndexed { i, f ->
        Face(f, size = 24.dp, modifier = Modifier.offset(x = (16 * i).dp).background(MaterialTheme.colorScheme.surface, CircleShape).padding(1.5.dp))
      }
    }
    Spacer(Modifier.width(6.dp))
    Text(
      if (list.size == 1) "${list.first().displayName.ifBlank { list.first().username }} is going" else "${list.size} people you follow are going",
      style = MaterialTheme.typography.labelMedium,
      color = MaterialTheme.colorScheme.onSurfaceVariant,
      maxLines = 1,
      overflow = TextOverflow.Ellipsis,
    )
  }
}

/** Seats, open, sold out, waitlist: one short chip. */
@Composable
private fun SeatChip(s: Screening) {
  val (text, strong) =
    when {
      !s.fixed -> "Open · drop in" to false
      s.seatsLeft == 0 -> ("Sold out" + (if (s.waitlisted > 0) " · ${s.waitlisted} waiting" else "")) to true
      s.seatsLeft <= 10 -> "${s.seatsLeft} seats left" to true
      else -> "${s.seatsLeft} seats" to false
    }
  Surface(shape = RoundedCornerShape(8.dp), color = if (strong) MaterialTheme.colorScheme.errorContainer else MaterialTheme.colorScheme.surfaceContainerHighest) {
    Row(Modifier.padding(horizontal = 7.dp, vertical = 3.dp), verticalAlignment = Alignment.CenterVertically) {
      Icon(Icons.Rounded.EventSeat, contentDescription = null, modifier = Modifier.size(13.dp), tint = if (strong) MaterialTheme.colorScheme.onErrorContainer else MaterialTheme.colorScheme.onSurfaceVariant)
      Spacer(Modifier.width(4.dp))
      Text(text, style = MaterialTheme.typography.labelSmall, color = if (strong) MaterialTheme.colorScheme.onErrorContainer else MaterialTheme.colorScheme.onSurfaceVariant)
    }
  }
}

@Composable
private fun PickChip() {
  Surface(shape = RoundedCornerShape(8.dp), color = Brand.Amber.copy(alpha = 0.22f)) {
    Row(Modifier.padding(horizontal = 7.dp, vertical = 3.dp), verticalAlignment = Alignment.CenterVertically) {
      Icon(Icons.Rounded.AutoAwesome, contentDescription = null, modifier = Modifier.size(13.dp), tint = Brand.Amber)
      Spacer(Modifier.width(4.dp))
      Text("Tonight's pick", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurface)
    }
  }
}

/** The one button a screening needs right now. */
@Composable
private fun Action(s: Screening, now: Long, onEnter: () -> Unit, big: Boolean = false) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  val tickets by Theatre.tickets.collectAsState()
  val busy by Theatre.busy.collectAsState()
  var confirmCancel by remember { mutableStateOf(false) }
  val phase = Theatre.phase(s, now)
  val ticket = tickets[s.id]
  val opens = Theatre.opensAt(s)
  val h = if (big) 52.dp else 40.dp
  fun say(msg: String) = Toast.makeText(context, msg, Toast.LENGTH_LONG).show()
  val modifier = if (big) Modifier.fillMaxWidth().height(h) else Modifier.height(h)

  when {
    s.status == "cancelled" -> Text("Cancelled", color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.labelLarge)
    phase == ShowPhase.Lobby || phase == ShowPhase.Live ->
      if (!s.fixed || ticket == "booked" || ticket == "attended") {
        Button(onClick = onEnter, modifier = modifier, colors = ButtonDefaults.buttonColors(containerColor = Brand.Blue)) {
          Text(if (phase == ShowPhase.Live) "Join live" else "Enter the lobby", fontWeight = FontWeight.Bold)
        }
      } else if (ticket == "waitlist") {
        OutlinedButton(onClick = {}, enabled = false, modifier = modifier) { Text("On the waitlist") }
      } else {
        Text("Ticket holders only", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
      }
    phase == ShowPhase.After || phase == ShowPhase.Over -> Text("Finished", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
    ticket == "booked" ->
      FilledTonalButton(onClick = { confirmCancel = true }, modifier = modifier) {
        Icon(Icons.Rounded.Check, contentDescription = null, modifier = Modifier.size(18.dp))
        Spacer(Modifier.width(6.dp))
        Text("Booked")
      }
    ticket == "waitlist" -> FilledTonalButton(onClick = { confirmCancel = true }, modifier = modifier) {
      Icon(Icons.Rounded.HourglassTop, contentDescription = null, modifier = Modifier.size(18.dp))
      Spacer(Modifier.width(6.dp))
      Text("Waitlist")
    }
    opens != null -> OutlinedButton(onClick = {}, enabled = false, modifier = modifier) { Text("Booking opens ${localTime(opens)}") }
    !s.fixed ->
      // Open screenings need no ticket; booking just adds a reminder.
      OutlinedButton(onClick = { scope.launch { say(Theatre.book(s)) } }, enabled = busy != s.id, modifier = modifier) { Text("Remind me") }
    else ->
      Button(onClick = { scope.launch { say(Theatre.book(s)) } }, enabled = busy != s.id, modifier = modifier, colors = ButtonDefaults.buttonColors(containerColor = Brand.Blue)) {
        Text(if (s.seatsLeft == 0) "Join waitlist" else "Book a seat", fontWeight = FontWeight.Bold)
      }
  }
  if (confirmCancel) {
    AlertDialog(
      onDismissRequest = { confirmCancel = false },
      title = { Text(if (ticket == "waitlist") "Leave the waitlist?" else "Cancel your ticket?") },
      text = { Text("${s.name} at ${localTime(s.startMs)}. ${if (s.fixed) "Your seat goes to someone on the waitlist." else ""}") },
      confirmButton = {
        TextButton(onClick = {
          confirmCancel = false
          scope.launch { say(Theatre.cancel(s)) }
        }) { Text("Cancel ticket", color = MaterialTheme.colorScheme.error) }
      },
      dismissButton = { TextButton(onClick = { confirmCancel = false }) { Text("Keep it") } },
    )
  }
}

/** The big card at the top: live now, in the lobby, or next. */
@Composable
private fun Featured(s: Screening, now: Long, onEnter: () -> Unit) {
  val friends by Theatre.friends.collectAsState()
  val phase = Theatre.phase(s, now)
  Surface(shape = RoundedCornerShape(30.dp), color = MaterialTheme.colorScheme.surfaceContainerHigh, modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp).fillMaxWidth()) {
    Column {
      Box(Modifier.fillMaxWidth().aspectRatio(16f / 9f)) {
        AsyncImage(model = s.backdrop ?: s.poster, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
        Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0.35f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.85f))))
        Row(Modifier.padding(14.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
          when (phase) {
            ShowPhase.Live -> Pulse(Color(0xFFE53935), "LIVE")
            ShowPhase.Lobby -> Pulse(Brand.Blue, "LOBBY OPEN")
            else -> {}
          }
          if (s.pick) PickChip()
        }
        Column(Modifier.align(Alignment.BottomStart).padding(18.dp)) {
          Text(s.name, color = Color.White, fontFamily = Brand.Serif, fontSize = 34.sp, lineHeight = 36.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
          val line =
            when (phase) {
              ShowPhase.Live -> "${span(now - s.startMs)} in · ends ${localTime(s.endMs)}"
              ShowPhase.Lobby -> "Starts in ${countdown(s.startMs - now)}"
              else -> "${dayLabel(s.startMs)} at ${localTime(s.startMs)} · starts in ${span(s.startMs - now)}"
            }
          Text(line, color = Color.White.copy(alpha = 0.85f), style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.SemiBold)
        }
      }
      Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
          SeatChip(s)
          if (s.runtime > 0) {
            Surface(shape = RoundedCornerShape(8.dp), color = MaterialTheme.colorScheme.surfaceContainerHighest) {
              Text(span(s.runtime * 60_000L), style = MaterialTheme.typography.labelSmall, modifier = Modifier.padding(horizontal = 7.dp, vertical = 3.dp))
            }
          }
        }
        FriendsGoing(friends[s.id].orEmpty())
        if (s.note.isNotBlank()) Text(s.note, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Action(s, now, onEnter, big = true)
      }
    }
  }
}

@Composable
private fun ScreeningCard(s: Screening, now: Long, onEnter: () -> Unit, modifier: Modifier) {
  val friends by Theatre.friends.collectAsState()
  val phase = Theatre.phase(s, now)
  Surface(shape = RoundedCornerShape(24.dp), color = MaterialTheme.colorScheme.surfaceContainer, modifier = modifier.padding(horizontal = 16.dp, vertical = 5.dp).fillMaxWidth()) {
    Row(Modifier.padding(12.dp)) {
      Box {
        AsyncImage(
          model = s.poster,
          contentDescription = s.name,
          contentScale = ContentScale.Crop,
          modifier = Modifier.width(84.dp).aspectRatio(2f / 3f).clip(RoundedCornerShape(16.dp)).background(MaterialTheme.colorScheme.surfaceContainerHighest),
        )
        if (phase == ShowPhase.Live) Box(Modifier.padding(6.dp)) { Pulse(Color(0xFFE53935), "LIVE") }
      }
      Spacer(Modifier.width(14.dp))
      Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(5.dp)) {
        Row(verticalAlignment = Alignment.Bottom) {
          Text(localTime(s.startMs), style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
          if (s.runtime > 0) {
            Spacer(Modifier.width(8.dp))
            Text(span(s.runtime * 60_000L), style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(bottom = 3.dp))
          }
        }
        Text(s.name + (s.year?.let { " ($it)" } ?: ""), style = MaterialTheme.typography.titleSmall, maxLines = 2, overflow = TextOverflow.Ellipsis)
        FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
          if (s.pick) PickChip()
          SeatChip(s)
        }
        FriendsGoing(friends[s.id].orEmpty())
        if (phase == ShowPhase.Upcoming && s.startMs - now < 3 * 3_600_000L) {
          Text("Starts in ${span(s.startMs - now)}", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary)
        }
        if (s.note.isNotBlank()) Text(s.note, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(Modifier.height(2.dp))
        Action(s, now, onEnter)
      }
    }
  }
}

/** Home's "Tonight in Theatre" row card. */
@Composable
fun TheatreRowCard(s: Screening, now: Long, onClick: () -> Unit) {
  val phase = Theatre.phase(s, now)
  Column(Modifier.width(220.dp).clip(RoundedCornerShape(20.dp)).pressable(onClick)) {
    Box(Modifier.fillMaxWidth().aspectRatio(16f / 9f).clip(RoundedCornerShape(20.dp)).background(MaterialTheme.colorScheme.surfaceContainerHigh)) {
      AsyncImage(model = s.backdrop ?: s.poster, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
      Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0.4f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.8f))))
      Box(Modifier.padding(8.dp)) {
        when (phase) {
          ShowPhase.Live -> Pulse(Color(0xFFE53935), "LIVE")
          ShowPhase.Lobby -> Pulse(Brand.Blue, "LOBBY")
          else -> {}
        }
      }
      Text(localTime(s.startMs), color = Color.White, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, modifier = Modifier.align(Alignment.BottomStart).padding(10.dp))
    }
    Text(s.name, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(top = 6.dp, start = 2.dp))
    Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(start = 2.dp)) {
      Icon(Icons.Rounded.Groups, contentDescription = null, modifier = Modifier.size(14.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
      Spacer(Modifier.width(4.dp))
      Text(if (s.fixed) "${s.booked} booked" else "Open", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
  }
}

