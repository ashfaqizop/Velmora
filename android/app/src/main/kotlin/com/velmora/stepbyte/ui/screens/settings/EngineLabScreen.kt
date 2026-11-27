@file:OptIn(ExperimentalMaterial3Api::class)

package com.velmora.stepbyte.ui.screens.settings

import android.os.SystemClock
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.MediaItem
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView
import androidx.navigation.NavController
import com.velmora.stepbyte.R
import com.velmora.stepbyte.engine.OpenedStream
import com.velmora.stepbyte.engine.SearchBatch
import com.velmora.stepbyte.engine.SelfTest
import com.velmora.stepbyte.engine.Source
import com.velmora.stepbyte.engine.SourcesReport
import com.velmora.stepbyte.engine.SourcesUpdate
import com.velmora.stepbyte.engine.Target
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.engine.TitleDetail
import com.velmora.stepbyte.engine.TorrentProgress
import com.velmora.stepbyte.engine.Universe
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

// A hidden screen (tap the logo on About seven times) that drives the Rust
// engine end to end: start, self-test, search, sources, and playback through
// the loopback stream server. It proves the bridge on real devices before
// the real screens are built on it, and stays as a support tool after.

private sealed interface Boot {
  data object Starting : Boot

  class Ready(val engine: Universe, val ms: Long) : Boot

  class Failed(val message: String) : Boot
}

/** One provider's line in the sources progress list. */
private data class ProviderLine(val provider: String, val found: Int? = null, val error: String? = null, val cached: Boolean = false)

private fun Throwable.text(): String = message?.takeIf { it.isNotBlank() } ?: javaClass.simpleName

private fun Long.ms(): String = if (this >= 1000) "%.1f s".format(this / 1000.0) else "$this ms"

private fun bytes(n: Long): String =
  when {
    n >= 1L shl 30 -> "%.2f GB".format(n / (1L shl 30).toDouble())
    n >= 1L shl 20 -> "%.1f MB".format(n / (1L shl 20).toDouble())
    else -> "${n / 1024} KB"
  }

@Composable
fun EngineLabScreen(navController: NavController) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()

  var boot by remember { mutableStateOf<Boot>(Boot.Starting) }
  val engine = (boot as? Boot.Ready)?.engine

  var selfTest by remember { mutableStateOf<SelfTest?>(null) }
  var testing by remember { mutableStateOf(false) }
  var testError by remember { mutableStateOf<String?>(null) }

  var query by rememberSaveable { mutableStateOf("") }
  val batches = remember { mutableStateListOf<SearchBatch>() }
  var searching by remember { mutableStateOf(false) }
  var searchMs by remember { mutableLongStateOf(0L) }
  var searchError by remember { mutableStateOf<String?>(null) }
  var searchJob by remember { mutableStateOf<Job?>(null) }

  var selected by remember { mutableStateOf<Title?>(null) }
  var detail by remember { mutableStateOf<TitleDetail?>(null) }
  var season by remember { mutableIntStateOf(1) }
  var episode by remember { mutableIntStateOf(1) }
  var dub by remember { mutableStateOf(false) }

  val lines = remember { mutableStateListOf<ProviderLine>() }
  var report by remember { mutableStateOf<SourcesReport?>(null) }
  var finding by remember { mutableStateOf(false) }
  var sourcesMs by remember { mutableLongStateOf(0L) }
  var sourcesError by remember { mutableStateOf<String?>(null) }
  var sourcesJob by remember { mutableStateOf<Job?>(null) }

  var opened by remember { mutableStateOf<OpenedStream?>(null) }
  var opening by remember { mutableStateOf<String?>(null) }
  var playError by remember { mutableStateOf<String?>(null) }
  var playState by remember { mutableStateOf("Idle") }
  var progress by remember { mutableStateOf<TorrentProgress?>(null) }

  val exo = remember {
    ExoPlayer.Builder(context).build().apply {
      addListener(
        object : Player.Listener {
          override fun onPlayerError(error: PlaybackException) {
            playError = "${error.errorCodeName}: ${error.message ?: ""}"
          }

          override fun onPlaybackStateChanged(state: Int) {
            playState =
              when (state) {
                Player.STATE_BUFFERING -> "Buffering"
                Player.STATE_READY -> "Playing"
                Player.STATE_ENDED -> "Ended"
                else -> "Idle"
              }
          }
        },
      )
    }
  }

  LaunchedEffect(Unit) {
    val start = SystemClock.elapsedRealtime()
    boot =
      try {
        Boot.Ready(Universe.get(context), SystemClock.elapsedRealtime() - start)
      } catch (e: CancellationException) {
        throw e
      } catch (e: Throwable) {
        // UnsatisfiedLinkError lands here too when the .so is missing.
        Boot.Failed("${e.javaClass.simpleName}: ${e.text()}")
      }
  }

  // Leaving the screen stops playback and frees the stream (and a torrent's pieces).
  DisposableEffect(Unit) {
    onDispose {
      exo.release()
      val e = (boot as? Boot.Ready)?.engine
      val s = opened
      if (e != null && s != null) CoroutineScope(Dispatchers.IO).launch { runCatching { e.closeStream(s) } }
    }
  }

  LaunchedEffect(opened) {
    val s = opened ?: return@LaunchedEffect
    if (!s.torrent || engine == null) return@LaunchedEffect
    while (true) {
      progress = runCatching { engine.torrentStatus(s) }.getOrNull()
      delay(1000)
    }
  }

  fun target(): Target = if (selected?.value?.episodic == true) Target.Episode(season, episode) else Target.Movie

  fun runSelfTest() {
    val e = engine ?: return
    scope.launch {
      testing = true
      testError = null
      try {
        selfTest = e.selfTest()
      } catch (x: CancellationException) {
        throw x
      } catch (x: Throwable) {
        testError = x.text()
      } finally {
        testing = false
      }
    }
  }

  fun search() {
    val e = engine ?: return
    val q = query.trim()
    if (q.isEmpty()) return
    searchJob?.cancel()
    batches.clear()
    searchError = null
    searchJob =
      scope.launch {
        searching = true
        val start = SystemClock.elapsedRealtime()
        try {
          e.searchStream(q).collect { batches.add(it) }
        } catch (x: CancellationException) {
          throw x
        } catch (x: Throwable) {
          searchError = x.text()
        } finally {
          searchMs = SystemClock.elapsedRealtime() - start
          searching = false
        }
      }
  }

  fun select(t: Title) {
    val e = engine ?: return
    selected = t
    detail = null
    season = 1
    episode = 1
    report = null
    lines.clear()
    sourcesError = null
    if (t.value.episodic) {
      scope.launch {
        val d = runCatching { e.detail(t) }.getOrNull() ?: return@launch
        if (selected !== t) return@launch
        detail = d
        d.info.seasons.firstOrNull { it.number > 0 && it.episodes.isNotEmpty() }?.let { s ->
          season = s.number
          episode = s.episodes.first().number
        }
      }
    }
  }

  fun findSources() {
    val e = engine ?: return
    val t = selected ?: return
    sourcesJob?.cancel()
    lines.clear()
    report = null
    sourcesError = null
    sourcesJob =
      scope.launch {
        finding = true
        val start = SystemClock.elapsedRealtime()
        try {
          e.sourcesWatch(t, target(), dub).collect { u ->
            when (u) {
              is SourcesUpdate.Plan -> {
                lines.clear()
                lines.addAll(u.providers.map { ProviderLine(it) })
              }
              is SourcesUpdate.Batch -> {
                val line = ProviderLine(u.provider, u.sources.size, u.error, u.cached)
                val i = lines.indexOfFirst { it.provider == u.provider }
                if (i >= 0) lines[i] = line else lines.add(line)
              }
              is SourcesUpdate.Done -> report = u.report
            }
          }
        } catch (x: CancellationException) {
          throw x
        } catch (x: Throwable) {
          sourcesError = x.text()
        } finally {
          sourcesMs = SystemClock.elapsedRealtime() - start
          finding = false
        }
      }
  }

  fun stop() {
    val e = engine
    val s = opened
    exo.stop()
    exo.clearMediaItems()
    opened = null
    progress = null
    if (e != null && s != null) scope.launch { runCatching { e.closeStream(s) } }
  }

  fun play(source: Source) {
    val e = engine ?: return
    scope.launch {
      opened?.let { old -> runCatching { e.closeStream(old) } }
      exo.stop()
      opened = null
      progress = null
      playError = null
      opening = source.value.label
      try {
        val s = e.open(source, target())
        opened = s
        val item =
          MediaItem.Builder()
            .setUri(s.url)
            .apply { if (source.value.format == "hls") setMimeType(MimeTypes.APPLICATION_M3U8) }
            .build()
        exo.setMediaItem(item)
        exo.prepare()
        exo.playWhenReady = true
      } catch (x: CancellationException) {
        throw x
      } catch (x: Throwable) {
        playError = x.text()
      } finally {
        opening = null
      }
    }
  }

  // The lab is one long list: jump to the sources card when a title is
  // picked, and to the player when a source starts opening.
  val listState = rememberLazyListState()
  val results = batches.flatMap { it.items }.distinctBy { it.value.origin }.take(40)
  val sourcesIndex = 2 + results.size
  val playerIndex = if (selected != null) sourcesIndex + 1 + (report?.sources?.size ?: 0) else sourcesIndex
  LaunchedEffect(selected) { if (selected != null) listState.animateScrollToItem(sourcesIndex) }
  LaunchedEffect(opening) { if (opening != null) listState.animateScrollToItem(playerIndex) }

  Scaffold(
    topBar = {
      TopAppBar(
        title = { Text("Engine lab", fontWeight = FontWeight.Bold) },
        navigationIcon = {
          IconButton(onClick = { navController.navigateUp() }) {
            Icon(painterResource(R.drawable.arrow_back), contentDescription = "Back")
          }
        },
      )
    },
  ) { inner ->
    LazyColumn(
      state = listState,
      modifier = Modifier.fillMaxSize(),
      contentPadding =
        PaddingValues(
          start = 12.dp,
          end = 12.dp,
          top = inner.calculateTopPadding() + 4.dp,
          bottom = WindowInsets.systemBars.asPaddingValues().calculateBottomPadding() + 24.dp,
        ),
      verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
      // ── Engine ──
      item {
        LabCard("Engine") {
          when (val b = boot) {
            Boot.Starting ->
              Row(verticalAlignment = Alignment.CenterVertically) {
                CircularProgressIndicator(modifier = Modifier.size(18.dp), strokeWidth = 2.dp)
                Spacer(Modifier.width(10.dp))
                Text("Starting the Rust engine…")
              }
            is Boot.Failed -> Status(false, "Didn't start", b.message)
            is Boot.Ready -> {
              Status(true, "Running", "Started in ${b.ms.ms()}")
              Mono("Bridge ${runCatching { Universe.version() }.getOrDefault("?")}")
              Mono("Stream server ${b.engine.streamBase}")
              Mono(if (b.engine.hasTmdb) "TMDB keys built in" else "No TMDB keys, using Cinemeta")
            }
          }
          if (engine != null) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
              FilledTonalButton(onClick = ::runSelfTest, enabled = !testing) {
                Text(if (testing) "Testing…" else "Run self-test")
              }
              if (testing) CircularProgressIndicator(modifier = Modifier.size(18.dp), strokeWidth = 2.dp)
            }
          }
          testError?.let { Status(false, "Self-test failed", it) }
          selfTest?.let { t ->
            HorizontalDivider()
            Text(
              if (t.ok) "All checks passed on ${t.arch}" else "Some checks failed on ${t.arch}",
              style = MaterialTheme.typography.titleSmall,
            )
            t.checks.forEach { c -> Status(c.ok, c.name + if (c.ms > 0) "  ·  ${c.ms.ms()}" else "", c.detail) }
          }
        }
      }

      // ── Search ──
      if (engine != null) {
        item {
          LabCard("Search") {
            OutlinedTextField(
              value = query,
              onValueChange = { query = it },
              modifier = Modifier.fillMaxWidth(),
              singleLine = true,
              placeholder = { Text("Interstellar, Frieren, The Bear…") },
              keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
              keyboardActions = KeyboardActions(onSearch = { search() }),
              trailingIcon = {
                IconButton(onClick = ::search) { Icon(painterResource(R.drawable.search), contentDescription = "Search") }
              },
            )
            if (searching) LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
            searchError?.let { Status(false, "Search failed", it) }
            if (batches.isNotEmpty()) {
              Text(
                "${batches.size} providers answered" + if (!searching) " in ${searchMs.ms()}" else "…",
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
              )
              batches.forEach { b ->
                Mono(
                  "${b.provider}: " + (b.error?.let { "error, $it" } ?: "${b.items.size} results"),
                  color = if (b.error != null) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant,
                )
              }
            }
          }
        }

        items(results, key = { "${it.value.origin.provider}:${it.value.origin.id}" }) { t ->
          TitleRow(t, chosen = selected === t, onClick = { select(t) })
        }
      }

      // ── Sources ──
      val chosen = selected
      if (engine != null && chosen != null) {
        item {
          LabCard("Sources for ${chosen.value.name}") {
            if (chosen.value.episodic) {
              val seasons = detail?.info?.seasons.orEmpty().filter { it.episodes.isNotEmpty() }
              Stepper("Season", season, onChange = { season = it.coerceAtLeast(0) })
              val count = seasons.firstOrNull { it.number == season }?.episodes?.size
              Stepper("Episode" + (count?.let { " of $it" } ?: ""), episode, onChange = { episode = it.coerceAtLeast(1) })
            }
            Row(verticalAlignment = Alignment.CenterVertically) {
              Text("Dub", modifier = Modifier.weight(1f))
              Switch(checked = dub, onCheckedChange = { dub = it })
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
              Button(onClick = ::findSources, enabled = !finding) { Text(if (finding) "Looking…" else "Find sources") }
              if (report != null && !finding) {
                OutlinedButton(
                  onClick = {
                    scope.launch {
                      runCatching { engine.forgetSources(chosen, target(), dub) }
                      findSources()
                    }
                  },
                ) {
                  Text("Look again")
                }
              }
            }
            if (finding) LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
            sourcesError?.let { Status(false, "Lookup failed", it) }
            lines.forEach { l ->
              val text =
                when {
                  l.error != null -> "error, ${l.error}"
                  l.found == null -> "asking…"
                  l.found == 0 -> "doesn't have it"
                  else -> "${l.found} sources" + if (l.cached) " (cached)" else ""
                }
              Mono(
                "${l.provider}: $text",
                color = if (l.error != null) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant,
              )
            }
            report?.let { r ->
              Text(
                "${r.sources.size} sources, ranked, in ${sourcesMs.ms()}",
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
              )
            }
          }
        }
        report?.let { r -> items(r.sources) { s -> SourceRow(s, busy = opening == s.value.label, onClick = { play(s) }) } }
      }

      // ── Player ──
      if (opened != null || opening != null || playError != null) {
        item {
          LabCard("Player") {
            opening?.let {
              Row(verticalAlignment = Alignment.CenterVertically) {
                CircularProgressIndicator(modifier = Modifier.size(18.dp), strokeWidth = 2.dp)
                Spacer(Modifier.width(10.dp))
                Text("Opening $it…", maxLines = 2, overflow = TextOverflow.Ellipsis)
              }
            }
            if (opened != null) {
              AndroidView(
                factory = { ctx -> PlayerView(ctx).apply { player = exo } },
                modifier = Modifier.fillMaxWidth().aspectRatio(16f / 9f).clip(RoundedCornerShape(12.dp)).padding(0.dp),
              )
              Mono("State: $playState")
              opened?.let { Mono(it.url, maxLines = 2) }
              progress?.let { p ->
                LinearProgressIndicator(progress = { p.fraction }, modifier = Modifier.fillMaxWidth())
                Mono("Torrent ${bytes(p.downloaded)} of ${bytes(p.total)}" + if (p.finished) ", done" else "")
              }
              OutlinedButton(onClick = ::stop) { Text("Stop") }
            }
            playError?.let { Status(false, "Playback error", it) }
          }
        }
      }
    }
  }
}

@Composable
private fun LabCard(title: String, content: @Composable ColumnScope.() -> Unit) {
  Surface(
    shape = RoundedCornerShape(20.dp),
    color = MaterialTheme.colorScheme.surfaceContainer,
    modifier = Modifier.fillMaxWidth(),
  ) {
    Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
      Text(title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
      content()
    }
  }
}

@Composable
private fun Status(ok: Boolean, title: String, detail: String?) {
  Row(verticalAlignment = Alignment.Top) {
    Icon(
      painterResource(if (ok) R.drawable.check else R.drawable.error),
      contentDescription = null,
      tint = if (ok) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error,
      modifier = Modifier.size(20.dp),
    )
    Spacer(Modifier.width(10.dp))
    Column {
      Text(title, style = MaterialTheme.typography.bodyLarge)
      if (!detail.isNullOrBlank()) {
        Text(detail, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
      }
    }
  }
}

@Composable
private fun Mono(text: String, color: Color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines: Int = 3) {
  Text(
    text,
    style = MaterialTheme.typography.bodySmall,
    fontFamily = FontFamily.Monospace,
    color = color,
    maxLines = maxLines,
    overflow = TextOverflow.Ellipsis,
  )
}

@Composable
private fun Stepper(label: String, value: Int, onChange: (Int) -> Unit) {
  Row(verticalAlignment = Alignment.CenterVertically) {
    Text(label, modifier = Modifier.weight(1f))
    OutlinedButton(onClick = { onChange(value - 1) }, contentPadding = PaddingValues(0.dp), modifier = Modifier.size(40.dp)) {
      Text("−")
    }
    Text("$value", modifier = Modifier.width(44.dp), style = MaterialTheme.typography.titleMedium, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
    OutlinedButton(onClick = { onChange(value + 1) }, contentPadding = PaddingValues(0.dp), modifier = Modifier.size(40.dp)) {
      Text("+")
    }
  }
}

@Composable
private fun TitleRow(t: Title, chosen: Boolean, onClick: () -> Unit) {
  val v = t.value
  Surface(
    onClick = onClick,
    shape = RoundedCornerShape(16.dp),
    color = if (chosen) MaterialTheme.colorScheme.secondaryContainer else MaterialTheme.colorScheme.surfaceContainerLow,
    modifier = Modifier.fillMaxWidth(),
  ) {
    Row(modifier = Modifier.padding(10.dp), verticalAlignment = Alignment.CenterVertically) {
      Box(
        modifier =
          Modifier.width(48.dp).height(72.dp).clip(RoundedCornerShape(8.dp)).padding(0.dp),
      ) {
        if (v.poster != null) {
          coil3.compose.AsyncImage(
            model = v.poster,
            contentDescription = null,
            contentScale = ContentScale.Crop,
            modifier = Modifier.fillMaxSize(),
          )
        } else {
          Surface(color = MaterialTheme.colorScheme.surfaceVariant, modifier = Modifier.fillMaxSize()) {}
        }
      }
      Spacer(Modifier.width(12.dp))
      Column(modifier = Modifier.weight(1f)) {
        Text(v.name, style = MaterialTheme.typography.titleSmall, maxLines = 2, overflow = TextOverflow.Ellipsis)
        Text(
          listOfNotNull(v.year?.toString(), v.kind.replace('_', ' '), v.origin.provider).joinToString("  ·  "),
          style = MaterialTheme.typography.bodySmall,
          color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        if (v.alsoOn.isNotEmpty()) {
          Text(
            "also on " + v.alsoOn.joinToString { it.provider },
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
          )
        }
      }
    }
  }
}

@Composable
private fun SourceRow(s: Source, busy: Boolean, onClick: () -> Unit) {
  val v = s.value
  Surface(
    onClick = onClick,
    shape = RoundedCornerShape(16.dp),
    color = MaterialTheme.colorScheme.surfaceContainerLow,
    modifier = Modifier.fillMaxWidth(),
  ) {
    Row(modifier = Modifier.padding(horizontal = 14.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
      Column(modifier = Modifier.weight(1f)) {
        Text(v.label, style = MaterialTheme.typography.bodyMedium, maxLines = 2, overflow = TextOverflow.Ellipsis)
        val facts =
          listOfNotNull(
            v.tags.resolution?.let { "${it}p" },
            v.format.uppercase(),
            "${v.provider} / ${v.host}",
            (v.sizeBytes ?: 0L).takeIf { it > 0 }?.let { bytes(it) },
            v.torrent?.seeders?.let { "$it seeders" },
            if (v.verified == true) "verified" else null,
            "score ${v.score}",
          )
        Text(
          facts.joinToString("  ·  "),
          style = MaterialTheme.typography.bodySmall,
          color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
      }
      Spacer(Modifier.width(8.dp))
      if (busy) {
        CircularProgressIndicator(modifier = Modifier.size(20.dp), strokeWidth = 2.dp)
      } else {
        Icon(painterResource(R.drawable.play), contentDescription = "Play", tint = MaterialTheme.colorScheme.primary)
      }
    }
  }
}
