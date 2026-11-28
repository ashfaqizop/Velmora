package com.velmora.stepbyte.velmora.player

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.Send
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Groups
import androidx.compose.material.icons.rounded.VolumeOff
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialShapes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.toShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.velmora.stepbyte.velmora.account.AccountAvatar
import com.velmora.stepbyte.velmora.account.Profile
import com.velmora.stepbyte.velmora.shell.Brand
import com.velmora.stepbyte.velmora.theatre.ChatMessage
import com.velmora.stepbyte.velmora.theatre.Theatre
import com.velmora.stepbyte.velmora.theatre.TheatreRoom
import com.velmora.stepbyte.velmora.theatre.Watcher
import com.velmora.stepbyte.velmora.theatre.countdown
import com.velmora.stepbyte.velmora.theatre.localTime
import com.velmora.stepbyte.velmora.theatre.span
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/** How the chat sits over a screening. */
enum class ChatLayout(val label: String) { Overlay("Chat over the film"), Sidebar("Chat beside the film"), Hidden("Chat hidden") }

@Composable
private fun clockNow(): Long {
  var now by remember { mutableLongStateOf(Theatre.serverNow()) }
  LaunchedEffect(Unit) {
    while (true) {
      now = Theatre.serverNow()
      delay(250)
    }
  }
  return now
}

@Composable
fun WatcherFace(w: Watcher, size: androidx.compose.ui.unit.Dp = 28.dp, modifier: Modifier = Modifier) {
  AccountAvatar(Profile(id = w.uid, username = w.username, displayName = w.displayName.ifBlank { w.username }, avatar = w.avatar), size = size, modifier = modifier)
}

// ── The lobby ──────────────────────────────────────────────────────────

/**
 * Before the film: the backdrop, the title, a big countdown, whether this
 * phone is ready (source found, how much is loaded), who's here, and the
 * chat. The last five seconds are big numbers on shapes that change each
 * second; then the lobby lifts away into the film.
 */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun TheatreLobby(ui: PlaybackUi, show: TheatreShow, clock: Clock, onClose: () -> Unit, onSources: () -> Unit) {
  val now = clockNow()
  val left = show.startsAt - now
  val watchers by TheatreRoom.watchers.collectAsState()
  val drift = rememberInfiniteTransition(label = "drift")
  val zoom by drift.animateFloat(1.04f, 1.14f, infiniteRepeatable(tween(18_000, easing = LinearEasing), RepeatMode.Reverse), label = "zoom")

  BoxWithConstraints(Modifier.fillMaxSize().background(Color.Black)) {
    val wide = maxWidth > maxHeight
    AsyncImage(
      model = show.backdrop ?: show.poster,
      contentDescription = null,
      contentScale = ContentScale.Crop,
      modifier = Modifier.fillMaxSize().graphicsLayer {
        scaleX = zoom
        scaleY = zoom
        alpha = 0.55f
      },
    )
    Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0f to Color.Black.copy(alpha = 0.5f), 0.5f to Color.Black.copy(alpha = 0.25f), 1f to Color.Black.copy(alpha = 0.95f))))

    // Short screens (a phone on its side) get a tighter type scale so the
    // whole lobby fits under the Leave button without scrolling.
    val short = maxHeight < 460.dp
    Row(Modifier.fillMaxSize().windowInsetsPadding(WindowInsets.displayCutout)) {
      // The Leave button has its own row, so nothing below can slide under it.
      Column(Modifier.weight(1f).fillMaxHeight().padding(bottom = if (wide) 0.dp else 260.dp)) {
        Row(Modifier.fillMaxWidth().padding(start = 12.dp, top = 8.dp, end = 12.dp), verticalAlignment = Alignment.CenterVertically) {
          Surface(onClick = onClose, shape = CircleShape, color = Color.Black.copy(alpha = 0.35f), modifier = Modifier.size(40.dp)) {
            Box(contentAlignment = Alignment.Center) { Icon(Icons.Rounded.Close, contentDescription = "Leave the lobby", tint = Color.White) }
          }
        }
        Box(Modifier.weight(1f).fillMaxWidth(), contentAlignment = if (wide) Alignment.CenterStart else Alignment.Center) {
        Column(Modifier.verticalScroll(rememberScrollState()).padding(start = 32.dp, end = 32.dp, bottom = 12.dp), horizontalAlignment = if (wide) Alignment.Start else Alignment.CenterHorizontally) {
          Surface(shape = RoundedCornerShape(50), color = if (show.started) Color(0xFFE53935) else Brand.Blue) {
            Text(
              if (show.started) "JOINING LIVE" else "LOBBY OPEN",
              color = Color.White,
              style = MaterialTheme.typography.labelMedium,
              fontWeight = FontWeight.Bold,
              modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
            )
          }
          Spacer(Modifier.height(if (short) 8.dp else 12.dp))
          Text(show.name, color = Color.White, fontFamily = Brand.Serif, fontSize = if (short) 38.sp else if (wide) 48.sp else 40.sp, lineHeight = if (short) 40.sp else if (wide) 50.sp else 42.sp, maxLines = 2, overflow = TextOverflow.Ellipsis, textAlign = if (wide) TextAlign.Start else TextAlign.Center)
          Text(
            "Starts at ${localTime(show.startsAt)}" + if (show.runtimeMin > 0) " · ${span(show.runtimeMin * 60_000L)}" else "",
            color = Color.White.copy(alpha = 0.75f),
            style = MaterialTheme.typography.bodyLarge,
          )
          Spacer(Modifier.height(if (short) 10.dp else 22.dp))
          if (!show.started) {
            Text(countdown(left), color = Color.White, fontSize = if (short) 64.sp else if (wide) 84.sp else 72.sp, lineHeight = if (short) 68.sp else 88.sp, fontWeight = FontWeight.Light, letterSpacing = 2.sp)
          }
          Spacer(Modifier.height(if (short) 8.dp else 10.dp))
          Readiness(ui, clock, onSources)
          Spacer(Modifier.height(if (short) 12.dp else 18.dp))
          WatchersRow(watchers)
        }
        }
      }
      if (wide) {
        ChatPanel(show, clock, inLobby = true, modifier = Modifier.width(340.dp).fillMaxHeight().background(Color.Black.copy(alpha = 0.55f)))
      }
    }
    if (!wide) {
      ChatPanel(show, clock, inLobby = true, modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(260.dp).background(Color.Black.copy(alpha = 0.55f)))
    }

    // 5, 4, 3, 2, 1.
    if (!show.started && left in 1..5_400) {
      FinalCountdown(((left + 999) / 1000).toInt().coerceIn(1, 5))
    }
  }
}

/** "Finding your source", "Ready · 6 min loaded ahead", or still looking. */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun Readiness(ui: PlaybackUi, clock: Clock, onSources: () -> Unit) {
  val ready = ui.phase == Phase.Playing && clock.firstFrame
  Surface(shape = RoundedCornerShape(16.dp), color = Color.White.copy(alpha = 0.1f)) {
    Row(Modifier.padding(horizontal = 14.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
      when {
        ready -> Box(Modifier.size(10.dp).clip(CircleShape).background(Color(0xFF5AD17A)))
        ui.phase == Phase.Failed -> Box(Modifier.size(10.dp).clip(CircleShape).background(Brand.Amber))
        else -> LoadingIndicator(modifier = Modifier.size(22.dp), color = Color.White)
      }
      Spacer(Modifier.width(10.dp))
      val ahead = (clock.buffered - clock.position).coerceAtLeast(0)
      Text(
        when {
          ready && ahead > 60_000 -> "Ready · ${span(ahead)} loaded ahead"
          ready -> "Ready"
          ui.phase == Phase.Failed -> "Still looking for a source. You can stay; it keeps trying."
          ui.phase == Phase.Opening -> "Loading your copy…"
          else -> "Finding your source…"
        },
        color = Color.White,
        style = MaterialTheme.typography.bodyMedium,
      )
      if (ui.phase == Phase.Failed && ui.sources.isNotEmpty()) {
        Spacer(Modifier.width(8.dp))
        OutlinedButton(onClick = onSources) { Text("Sources", color = Color.White) }
      }
    }
  }
}

@Composable
private fun WatchersRow(watchers: List<Watcher>) {
  if (watchers.isEmpty()) return
  Row(verticalAlignment = Alignment.CenterVertically) {
    Box(Modifier.width((26 + 18 * (watchers.take(6).size - 1)).dp).height(28.dp)) {
      watchers.take(6).forEachIndexed { i, w -> WatcherFace(w, 28.dp, Modifier.offset(x = (18 * i).dp).background(Color.Black, CircleShape).padding(1.5.dp)) }
    }
    Spacer(Modifier.width(8.dp))
    Text(if (watchers.size == 1) "Just you so far" else "${watchers.size} here", color = Color.White, style = MaterialTheme.typography.labelLarge)
  }
}

/** Big numbers, each on its own shape, springing in and spinning a little. */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun FinalCountdown(n: Int) {
  val haptics = LocalHapticFeedback.current
  LaunchedEffect(n) { haptics.performHapticFeedback(HapticFeedbackType.LongPress) }
  val shapes = listOf(MaterialShapes.Cookie9Sided, MaterialShapes.Sunny, MaterialShapes.Clover8Leaf, MaterialShapes.Pentagon, MaterialShapes.Gem)
  Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.45f)), contentAlignment = Alignment.Center) {
    AnimatedContent(
      targetState = n,
      transitionSpec = {
        (scaleIn(spring(dampingRatio = Spring.DampingRatioMediumBouncy, stiffness = Spring.StiffnessMediumLow), initialScale = 0.3f) + fadeIn(tween(120))) togetherWith
          (scaleOut(tween(220), targetScale = 1.6f) + fadeOut(tween(220)))
      },
      label = "five",
    ) { k ->
      val spin = remember { Animatable(-25f) }
      LaunchedEffect(k) { spin.animateTo(0f, spring(dampingRatio = Spring.DampingRatioMediumBouncy)) }
      Box(
        Modifier.size(220.dp).graphicsLayer { rotationZ = spin.value }.clip(shapes[(k - 1) % shapes.size].toShape()).background(Brush.linearGradient(listOf(Brand.Blue, Brand.Violet))),
        contentAlignment = Alignment.Center,
      ) {
        Text("$k", color = Color.White, fontFamily = Brand.Serif, fontSize = 120.sp, modifier = Modifier.graphicsLayer { rotationZ = -spin.value })
      }
    }
  }
}

// ── During the show ────────────────────────────────────────────────────

/** "2:14 behind · Back to live", only after a pause. */
@Composable
fun BehindPill(ui: PlaybackUi, modifier: Modifier = Modifier) {
  AnimatedVisibility(visible = ui.behindMs > 0, enter = fadeIn() + scaleIn(initialScale = 0.9f), exit = fadeOut(), modifier = modifier) {
    Surface(onClick = { VideoSession.backToLive() }, shape = RoundedCornerShape(50), color = Color.Black.copy(alpha = 0.65f)) {
      Row(Modifier.padding(horizontal = 14.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
        Text("${time(ui.behindMs)} behind", color = Color.White, style = MaterialTheme.typography.labelLarge)
        Spacer(Modifier.width(10.dp))
        Box(Modifier.size(8.dp).clip(CircleShape).background(Color(0xFFE53935)))
        Spacer(Modifier.width(6.dp))
        Text("Back to live", color = Color.White, style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.Bold)
      }
    }
  }
}

/** Reactions float up from the right, bigger when many land at once. */
@Composable
fun FloatingReactions(modifier: Modifier = Modifier) {
  data class Fly(val emoji: String, val x: Float, val big: Boolean, val key: Long)
  val flying = remember { mutableStateListOf<Fly>() }
  val recent = remember { ArrayDeque<Long>() }
  LaunchedEffect(Unit) {
    TheatreRoom.reactions.collect { r ->
      val now = System.currentTimeMillis()
      recent.addLast(now)
      while (recent.isNotEmpty() && now - recent.first() > 2_000) recent.removeFirst()
      if (flying.size < 40) flying += Fly(r.emoji, (0.55f + Math.random().toFloat() * 0.4f), recent.size >= 6, r.nonce)
    }
  }
  BoxWithConstraints(modifier.fillMaxSize()) {
    val w = constraints.maxWidth.toFloat()
    val h = constraints.maxHeight.toFloat()
    flying.forEach { f ->
      androidx.compose.runtime.key(f.key) {
        val rise = remember { Animatable(0f) }
        LaunchedEffect(Unit) {
          rise.animateTo(1f, tween(2_400, easing = FastOutSlowInEasing))
          flying.remove(f)
        }
        Text(
          f.emoji,
          fontSize = if (f.big) 42.sp else 30.sp,
          modifier =
            Modifier.graphicsLayer {
              translationX = f.x * w + kotlin.math.sin(rise.value * 6f) * 24f
              translationY = h * (0.92f - rise.value * 0.7f)
              alpha = (1f - rise.value).coerceIn(0f, 1f) * 1.3f
              val s = 0.7f + rise.value * 0.5f
              scaleX = s
              scaleY = s
            },
        )
      }
    }
  }
}


/** The emoji bar. */
@Composable
fun ReactionBar(modifier: Modifier = Modifier) {
  val haptics = LocalHapticFeedback.current
  LazyRow(modifier, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
    items(TheatreRoom.REACTIONS) { e ->
      Box(
        Modifier.size(40.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.1f)).clickable {
          haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
          TheatreRoom.react(e)
        },
        contentAlignment = Alignment.Center,
      ) {
        Text(e, fontSize = 20.sp)
      }
    }
  }
}

// ── The chat ───────────────────────────────────────────────────────────

/**
 * The chat: messages with the film time they were sent at, mentions lit up,
 * the box, refusals in plain words, quiet mode when the room is full, and the
 * reaction bar. Tap a name to mute them for yourself.
 */
@Composable
fun ChatPanel(show: TheatreShow, clock: Clock, inLobby: Boolean, modifier: Modifier = Modifier, overlay: Boolean = false) {
  val scope = rememberCoroutineScope()
  val messages by TheatreRoom.messages.collectAsState()
  val notice by TheatreRoom.notice.collectAsState()
  val quiet by TheatreRoom.quiet.collectAsState()
  val mode by TheatreRoom.mode.collectAsState()
  val sending by TheatreRoom.sending.collectAsState()
  val watchers by TheatreRoom.watchers.collectAsState()
  var text by remember { mutableStateOf("") }
  var muteAsk by remember { mutableStateOf<ChatMessage?>(null) }
  val list = rememberLazyListState()
  LaunchedEffect(messages.size) { if (messages.isNotEmpty()) list.animateScrollToItem(messages.size - 1) }

  Column(modifier.padding(horizontal = 12.dp, vertical = 8.dp).imePadding()) {
    if (!overlay) {
      Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(bottom = 6.dp)) {
        Text(if (inLobby) "Lobby chat" else "Chat", color = Color.White, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, modifier = Modifier.weight(1f))
        Icon(Icons.Rounded.Groups, contentDescription = null, tint = Color.White.copy(alpha = 0.7f), modifier = Modifier.size(16.dp))
        Spacer(Modifier.width(4.dp))
        Text("${watchers.size}", color = Color.White.copy(alpha = 0.7f), style = MaterialTheme.typography.labelMedium)
      }
    }
    LazyColumn(state = list, modifier = Modifier.weight(1f).fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(6.dp)) {
      if (messages.isEmpty() && !overlay) {
        item {
          Text(
            if (quiet) "Chat is full right now. You're watching in sync, and chat comes on as soon as a seat frees up." else "Say hello. Messages are never saved.",
            color = Color.White.copy(alpha = 0.6f),
            style = MaterialTheme.typography.bodySmall,
          )
        }
      }
      items(if (overlay) messages.takeLast(6) else messages, key = { it.id }) { m -> ChatLine(m, overlay, onMute = { muteAsk = m }) }
    }
    notice?.let { Text(it, color = Brand.Amber, style = MaterialTheme.typography.labelMedium, modifier = Modifier.padding(vertical = 4.dp)) }
    if (!overlay) {
      ReactionBar(Modifier.padding(vertical = 6.dp))
      if (mode == "closed") {
        Text("Text chat is off for this screening. Reactions still work.", color = Color.White.copy(alpha = 0.6f), style = MaterialTheme.typography.bodySmall)
      } else if (!quiet) {
        fun send() {
          val t = text
          scope.launch {
            val movieMs = if (show.started) VideoSession.player?.currentPosition else null
            if (TheatreRoom.say(t, movieMs)) text = ""
          }
        }
        Row(verticalAlignment = Alignment.CenterVertically) {
          TextField(
            value = text,
            onValueChange = { if (it.length <= 200) text = it },
            placeholder = { Text("Message", color = Color.White.copy(alpha = 0.5f)) },
            singleLine = true,
            shape = RoundedCornerShape(24.dp),
            colors =
              TextFieldDefaults.colors(
                focusedContainerColor = Color.White.copy(alpha = 0.12f),
                unfocusedContainerColor = Color.White.copy(alpha = 0.08f),
                focusedIndicatorColor = Color.Transparent,
                unfocusedIndicatorColor = Color.Transparent,
                focusedTextColor = Color.White,
                unfocusedTextColor = Color.White,
              ),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
            keyboardActions = KeyboardActions(onSend = { send() }),
            modifier = Modifier.weight(1f),
          )
          Spacer(Modifier.width(6.dp))
          FilledIconButton(
            onClick = { send() },
            enabled = text.isNotBlank() && !sending,
            colors = IconButtonDefaults.filledIconButtonColors(containerColor = Brand.Blue, contentColor = Color.White),
          ) {
            Icon(Icons.AutoMirrored.Rounded.Send, contentDescription = "Send")
          }
        }
      }
    }
  }

  muteAsk?.let { m ->
    androidx.compose.material3.AlertDialog(
      onDismissRequest = { muteAsk = null },
      icon = { Icon(Icons.Rounded.VolumeOff, contentDescription = null) },
      title = { Text("Mute ${m.displayName.ifBlank { m.username }}?") },
      text = { Text("You won't see their messages or reactions in this screening. Only you can tell.") },
      confirmButton = {
        androidx.compose.material3.TextButton(onClick = {
          TheatreRoom.mute(m.uid, true)
          muteAsk = null
        }) { Text("Mute") }
      },
      dismissButton = { androidx.compose.material3.TextButton(onClick = { muteAsk = null }) { Text("Cancel") } },
    )
  }
}

@Composable
private fun ChatLine(m: ChatMessage, overlay: Boolean, onMute: () -> Unit) {
  val bg =
    when {
      m.mention -> Brand.Blue.copy(alpha = 0.35f)
      overlay -> Color.Black.copy(alpha = 0.45f)
      else -> Color.Transparent
    }
  Row(Modifier.clip(RoundedCornerShape(12.dp)).background(bg).padding(horizontal = 6.dp, vertical = 4.dp), verticalAlignment = Alignment.Top) {
    AccountAvatar(Profile(id = m.uid, username = m.username, displayName = m.displayName.ifBlank { m.username }, avatar = m.avatar), size = 26.dp, modifier = Modifier.clickable(enabled = !m.mine, onClick = onMute))
    Spacer(Modifier.width(8.dp))
    Column(Modifier.widthIn(max = 420.dp)) {
      Row(verticalAlignment = Alignment.CenterVertically) {
        Text(
          if (m.mine) "You" else m.displayName.ifBlank { m.username },
          color = if (m.mine) Brand.Violet else Color.White,
          style = MaterialTheme.typography.labelMedium,
          fontWeight = FontWeight.Bold,
          maxLines = 1,
          modifier = Modifier.clickable(enabled = !m.mine, onClick = onMute),
        )
        m.at?.let {
          Spacer(Modifier.width(6.dp))
          Text(time(it), color = Color.White.copy(alpha = 0.45f), style = MaterialTheme.typography.labelSmall)
        }
      }
      Text(m.body, color = Color.White, style = MaterialTheme.typography.bodyMedium)
    }
  }
}

// ── After ──────────────────────────────────────────────────────────────

/** The end: "You were there", the chat still open until it closes, back to Theatre. */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun TheatreEnd(show: TheatreShow, clock: Clock, onClose: () -> Unit) {
  val now = clockNow()
  val closes = show.endsAt + Theatre.AFTER_MS - now
  BoxWithConstraints(Modifier.fillMaxSize().background(Color.Black)) {
    val wide = maxWidth > maxHeight
    AsyncImage(model = show.backdrop ?: show.poster, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize().graphicsLayer { alpha = 0.3f })
    Row(Modifier.fillMaxSize().windowInsetsPadding(WindowInsets.displayCutout)) {
      Column(Modifier.weight(1f).fillMaxHeight().padding(32.dp), verticalArrangement = Arrangement.Center, horizontalAlignment = Alignment.CenterHorizontally) {
        val pop = remember { Animatable(0.4f) }
        LaunchedEffect(Unit) { pop.animateTo(1f, spring(dampingRatio = Spring.DampingRatioMediumBouncy, stiffness = Spring.StiffnessLow)) }
        Box(
          Modifier.size(120.dp).graphicsLayer {
            scaleX = pop.value
            scaleY = pop.value
            rotationZ = (1f - pop.value) * -40f
          }.clip(MaterialShapes.Sunny.toShape()).background(Brush.linearGradient(listOf(Brand.Amber, Brand.Blue))),
          contentAlignment = Alignment.Center,
        ) {
          Text("🎬", fontSize = 48.sp)
        }
        Spacer(Modifier.height(18.dp))
        Text("You were there", color = Color.White, fontFamily = Brand.Serif, fontSize = 40.sp)
        Text(show.name, color = Color.White.copy(alpha = 0.8f), style = MaterialTheme.typography.titleMedium, textAlign = TextAlign.Center)
        Spacer(Modifier.height(8.dp))
        if (closes > 0) Text("Chat closes in ${countdown(closes)}", color = Color.White.copy(alpha = 0.6f), style = MaterialTheme.typography.bodyMedium)
        Spacer(Modifier.height(18.dp))
        OutlinedButton(onClick = onClose) { Text("Back to Theatre", color = Color.White) }
      }
      if (wide && closes > 0) ChatPanel(show, clock, inLobby = false, modifier = Modifier.width(340.dp).fillMaxHeight().background(Color.Black.copy(alpha = 0.55f)))
    }
    if (!wide && closes > 0) ChatPanel(show, clock, inLobby = false, modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(240.dp).background(Color.Black.copy(alpha = 0.55f)))
  }
}
