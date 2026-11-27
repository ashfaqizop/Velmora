package com.velmora.stepbyte.velmora.screens

import android.widget.Toast
import androidx.annotation.DrawableRes
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
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
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import coil3.compose.AsyncImage
import com.velmora.stepbyte.LocalPlayerAwareWindowInsets
import com.velmora.stepbyte.R
import com.velmora.stepbyte.engine.SearchBatch
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.ui.component.NavigationTitle
import com.velmora.stepbyte.ui.component.shimmer.ShimmerHost
import com.velmora.stepbyte.ui.component.shimmer.TextPlaceholder
import com.velmora.stepbyte.ui.screens.Screens
import com.velmora.stepbyte.velmora.shell.AppMode
import com.velmora.stepbyte.velmora.shell.Brand
import com.velmora.stepbyte.velmora.shell.HomeFeed
import com.velmora.stepbyte.velmora.shell.LocalAppMode
import com.velmora.stepbyte.velmora.shell.LocalSwitchMode
import com.velmora.stepbyte.velmora.shell.VelmoraRoutes
import com.velmora.stepbyte.velmora.shell.modeFromQuery
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/** Velmora mode's destinations, inside the "velmora" graph. */
fun NavGraphBuilder.velmoraGraph(navController: NavController) {
  composable(Screens.VelmoraHome.route) { VelmoraHomeScreen() }
  composable(Screens.VelmoraBrowse.route) {
    ComingScreen(
      icon = R.drawable.explore_filled,
      title = "Browse",
      text = "Movies, Series, Anime, Live TV and Sports, each with its own tab, filters and genres.",
      phase = 5,
    )
  }
  composable(Screens.VelmoraTheatre.route) {
    ComingScreen(
      icon = R.drawable.theaters_filled,
      title = "Theatre",
      text = "Scheduled screenings you watch together: the lineup, booking a seat, the lobby and live chat.",
      phase = 8,
    )
  }
  composable(Screens.VelmoraSearch.route) { VelmoraSearchScreen() }
  composable(VelmoraRoutes.DOWNLOADS) {
    ComingScreen(
      icon = R.drawable.download,
      title = "Downloads",
      text = "Movies and episodes you save, locked to Velmora and playable offline.",
      phase = 7,
    )
  }
}

private fun Throwable.text(): String = message?.takeIf { it.isNotBlank() } ?: javaClass.simpleName

private fun titleKey(t: Title) = "${t.value.origin.provider}:${t.value.origin.id}"

// ── Home ───────────────────────────────────────────────────────────────

@Composable
fun VelmoraHomeScreen() {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  val rails by HomeFeed.rails.collectAsState()
  val loading by HomeFeed.loading.collectAsState()
  val error by HomeFeed.error.collectAsState()

  LaunchedEffect(Unit) { HomeFeed.load(context) }

  LazyColumn(
    modifier = Modifier.fillMaxSize(),
    contentPadding = LocalPlayerAwareWindowInsets.current.asPaddingValues(),
  ) {
    if (rails.isEmpty() && (loading || error == null)) {
      item(key = "shimmer") { HomeShimmer() }
    }
    if (rails.isEmpty() && !loading && error != null) {
      item(key = "error") {
        ErrorCard(
          message = error ?: "",
          onRetry = { scope.launch { HomeFeed.load(context, force = true) } },
        )
      }
    }
    rails.firstOrNull()?.items?.firstOrNull()?.let { hero ->
      item(key = "hero") { HeroCard(hero, label = rails.first().title) }
    }
    rails.forEachIndexed { index, rail ->
      item(key = "rail:$index:${rail.id}") {
        Column {
          NavigationTitle(title = rail.title)
          LazyRow(
            contentPadding = PaddingValues(horizontal = 12.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
          ) {
            items(rail.items.distinctBy(::titleKey), key = ::titleKey) { t -> PosterCard(t) }
          }
          Spacer(Modifier.height(8.dp))
        }
      }
    }
  }
}

@Composable
private fun HeroCard(title: Title, label: String) {
  val v = title.value
  val context = LocalContext.current
  Box(
    modifier =
      Modifier.padding(horizontal = 12.dp, vertical = 8.dp)
        .fillMaxWidth()
        .height(230.dp)
        .clip(RoundedCornerShape(28.dp))
        .background(MaterialTheme.colorScheme.surfaceContainerHigh)
        .clickable { comingSoon(context) },
  ) {
    AsyncImage(
      model = v.backdrop ?: v.poster,
      contentDescription = null,
      contentScale = ContentScale.Crop,
      modifier = Modifier.fillMaxSize(),
    )
    Box(
      Modifier.fillMaxSize()
        .background(
          Brush.verticalGradient(
            0f to Color.Transparent,
            0.45f to Color.Black.copy(alpha = 0.15f),
            1f to Color.Black.copy(alpha = 0.85f),
          ),
        ),
    )
    Column(modifier = Modifier.align(Alignment.BottomStart).padding(18.dp)) {
      Surface(shape = RoundedCornerShape(50), color = Brand.Blue.copy(alpha = 0.9f)) {
        Text(
          label,
          color = Color.White,
          style = MaterialTheme.typography.labelMedium,
          fontWeight = FontWeight.Bold,
          modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
        )
      }
      Spacer(Modifier.height(8.dp))
      Text(
        v.name,
        color = Color.White,
        fontFamily = Brand.Serif,
        fontSize = 34.sp,
        lineHeight = 36.sp,
        maxLines = 2,
        overflow = TextOverflow.Ellipsis,
      )
      v.overview?.let {
        Text(
          it,
          color = Color.White.copy(alpha = 0.8f),
          style = MaterialTheme.typography.bodySmall,
          maxLines = 2,
          overflow = TextOverflow.Ellipsis,
          modifier = Modifier.padding(top = 4.dp),
        )
      }
    }
  }
}

@Composable
private fun PosterCard(title: Title, modifier: Modifier = Modifier.width(118.dp)) {
  val v = title.value
  val context = LocalContext.current
  Column(modifier = modifier.clip(RoundedCornerShape(16.dp)).clickable { comingSoon(context) }) {
    Box(
      modifier =
        Modifier.fillMaxWidth()
          .aspectRatio(2f / 3f)
          .clip(RoundedCornerShape(16.dp))
          .background(MaterialTheme.colorScheme.surfaceContainerHigh),
    ) {
      AsyncImage(
        model = v.poster,
        contentDescription = v.name,
        contentScale = ContentScale.Crop,
        modifier = Modifier.fillMaxSize(),
      )
      v.rating?.takeIf { it > 0f }?.let { r ->
        Surface(
          shape = RoundedCornerShape(8.dp),
          color = Color.Black.copy(alpha = 0.6f),
          modifier = Modifier.align(Alignment.TopEnd).padding(6.dp),
        ) {
          Text(
            "%.1f".format(r),
            color = Color.White,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
          )
        }
      }
    }
    Text(
      v.name,
      style = MaterialTheme.typography.bodyMedium,
      fontWeight = FontWeight.Medium,
      maxLines = 1,
      overflow = TextOverflow.Ellipsis,
      modifier = Modifier.padding(top = 6.dp, start = 2.dp, end = 2.dp),
    )
    Text(
      listOfNotNull(v.year?.toString(), kindLabel(v.kind)).joinToString(" · "),
      style = MaterialTheme.typography.bodySmall,
      color = MaterialTheme.colorScheme.onSurfaceVariant,
      maxLines = 1,
      modifier = Modifier.padding(horizontal = 2.dp).padding(bottom = 4.dp),
    )
  }
}

private fun kindLabel(kind: String) =
  when (kind) {
    "movie" -> "Movie"
    "series" -> "Series"
    "anime" -> "Anime"
    "asian_drama" -> "Drama"
    else -> null
  }

private fun comingSoon(context: android.content.Context) {
  Toast.makeText(context, "Title pages arrive in phase 5", Toast.LENGTH_SHORT).show()
}

@Composable
private fun HomeShimmer() {
  ShimmerHost {
    Box(
      Modifier.padding(horizontal = 12.dp, vertical = 8.dp)
        .fillMaxWidth()
        .height(230.dp)
        .clip(RoundedCornerShape(28.dp))
        .background(MaterialTheme.colorScheme.onSurface),
    )
    repeat(2) {
      TextPlaceholder(
        modifier = Modifier.padding(horizontal = 12.dp, vertical = 14.dp).width(160.dp),
        height = 20.dp,
        shape = RoundedCornerShape(8.dp),
      )
      Row(Modifier.padding(horizontal = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        repeat(4) {
          Box(
            Modifier.width(118.dp)
              .aspectRatio(2f / 3f)
              .clip(RoundedCornerShape(16.dp))
              .background(MaterialTheme.colorScheme.onSurface),
          )
        }
      }
    }
  }
}

@Composable
private fun ErrorCard(message: String, onRetry: () -> Unit) {
  Surface(
    shape = RoundedCornerShape(24.dp),
    color = MaterialTheme.colorScheme.errorContainer,
    modifier = Modifier.padding(12.dp).fillMaxWidth(),
  ) {
    Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
      Text(
        "Couldn't load Home",
        style = MaterialTheme.typography.titleMedium,
        color = MaterialTheme.colorScheme.onErrorContainer,
      )
      Text(message, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onErrorContainer)
      FilledTonalButton(onClick = onRetry) { Text("Try again") }
    }
  }
}

// ── Search ─────────────────────────────────────────────────────────────

@Composable
fun VelmoraSearchScreen() {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  val switchMode = LocalSwitchMode.current
  val mode = LocalAppMode.current

  var query by rememberSaveable { mutableStateOf("") }
  val batches = remember { mutableStateListOf<SearchBatch>() }
  var searching by remember { mutableStateOf(false) }
  var error by remember { mutableStateOf<String?>(null) }
  var job by remember { mutableStateOf<Job?>(null) }
  var searched by rememberSaveable { mutableStateOf("") }

  fun startSearch(q: String) {
    val text = q.trim()
    if (text.length < 2 || text == searched) return
    job?.cancel()
    batches.clear()
    error = null
    searched = text
    job =
      scope.launch {
        searching = true
        try {
          Universe.get(context).searchStream(text).collect { batches.add(it) }
        } catch (e: CancellationException) {
          throw e
        } catch (e: Throwable) {
          error = e.text()
          searched = ""
        } finally {
          searching = false
        }
      }
  }

  // Search as you type, after a short pause.
  LaunchedEffect(query) {
    delay(450)
    startSearch(query)
  }

  val results =
    batches
      .flatMap { it.items }
      .distinctBy { "${it.value.name.lowercase()}|${it.value.year}" }
      .take(60)
  val jump = modeFromQuery(query)?.takeIf { it != mode }

  LazyVerticalGrid(
    columns = GridCells.Adaptive(minSize = 108.dp),
    modifier = Modifier.fillMaxSize(),
    contentPadding = LocalPlayerAwareWindowInsets.current.asPaddingValues(),
    horizontalArrangement = Arrangement.spacedBy(10.dp),
    verticalArrangement = Arrangement.spacedBy(4.dp),
  ) {
    item(key = "field", span = { GridItemSpan(maxLineSpan) }) {
      TextField(
        value = query,
        onValueChange = { query = it },
        modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 6.dp),
        singleLine = true,
        shape = RoundedCornerShape(28.dp),
        placeholder = { Text("Movies, series, anime…") },
        leadingIcon = { Icon(painterResource(R.drawable.search), contentDescription = null) },
        trailingIcon = {
          if (query.isNotEmpty()) {
            IconButton(onClick = { query = "" }) {
              Icon(painterResource(R.drawable.close), contentDescription = "Clear")
            }
          }
        },
        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
        keyboardActions = KeyboardActions(onSearch = { startSearch(query) }),
        colors =
          TextFieldDefaults.colors(
            focusedIndicatorColor = Color.Transparent,
            unfocusedIndicatorColor = Color.Transparent,
          ),
      )
    }
    if (jump != null) {
      item(key = "jump", span = { GridItemSpan(maxLineSpan) }) { SwitchSuggestion(jump) { switchMode(jump) } }
    }
    if (searching) {
      item(key = "progress", span = { GridItemSpan(maxLineSpan) }) {
        LinearProgressIndicator(modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp))
      }
    }
    error?.let { e ->
      item(key = "error", span = { GridItemSpan(maxLineSpan) }) {
        Text(
          e,
          color = MaterialTheme.colorScheme.error,
          style = MaterialTheme.typography.bodySmall,
          modifier = Modifier.padding(12.dp),
        )
      }
    }
    if (query.isBlank() && results.isEmpty()) {
      item(key = "hint", span = { GridItemSpan(maxLineSpan) }) {
        Text(
          "One search across every source. Type \"music\" to jump to Music.",
          style = MaterialTheme.typography.bodyMedium,
          color = MaterialTheme.colorScheme.onSurfaceVariant,
          textAlign = TextAlign.Center,
          modifier = Modifier.fillMaxWidth().padding(24.dp),
        )
      }
    }
    items(results, key = ::titleKey) { t ->
      Box(Modifier.padding(horizontal = 2.dp)) { PosterCard(t, modifier = Modifier.fillMaxWidth()) }
    }
  }
}

/** "Switch to Music" (or back), offered when the query names the other mode. */
@Composable
fun SwitchSuggestion(to: AppMode, onClick: () -> Unit) {
  val music = to == AppMode.MUSIC
  Surface(
    onClick = onClick,
    shape = RoundedCornerShape(24.dp),
    color = Color.Transparent,
    modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp).fillMaxWidth(),
  ) {
    Row(
      modifier =
        Modifier.background(
            Brush.horizontalGradient(
              if (music) listOf(Brand.Blue, Brand.Amber) else listOf(Brand.Amber, Brand.Blue),
            ),
          )
          .padding(horizontal = 16.dp, vertical = 14.dp),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      Box(
        Modifier.size(40.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.25f)),
        contentAlignment = Alignment.Center,
      ) {
        Icon(
          painterResource(if (music) R.drawable.music_note else R.drawable.movie),
          contentDescription = null,
          tint = Color.White,
        )
      }
      Column(Modifier.weight(1f).padding(horizontal = 14.dp)) {
        Text(
          if (music) "Switch to Music" else "Switch to Velmora",
          color = Color.White,
          style = MaterialTheme.typography.titleMedium,
          fontWeight = FontWeight.Bold,
        )
        Text(
          if (music) "Songs, albums and your YouTube Music library" else "Movies, series, anime, live TV and Theatre",
          color = Color.White.copy(alpha = 0.85f),
          style = MaterialTheme.typography.bodySmall,
        )
      }
      Icon(painterResource(R.drawable.arrow_forward), contentDescription = null, tint = Color.White)
    }
  }
}

// ── Placeholders until their phase ─────────────────────────────────────

@Composable
fun ComingScreen(@DrawableRes icon: Int, title: String, text: String, phase: Int) {
  val uriHandler = LocalUriHandler.current
  Column(
    modifier =
      Modifier.fillMaxSize()
        .padding(LocalPlayerAwareWindowInsets.current.asPaddingValues())
        .padding(horizontal = 32.dp),
    horizontalAlignment = Alignment.CenterHorizontally,
    verticalArrangement = Arrangement.Center,
  ) {
    Box(
      Modifier.size(96.dp)
        .clip(RoundedCornerShape(32.dp))
        .background(
          Brush.linearGradient(listOf(Brand.Blue.copy(alpha = 0.35f), Brand.Cream.copy(alpha = 0.25f))),
        ),
      contentAlignment = Alignment.Center,
    ) {
      Icon(painterResource(icon), contentDescription = null, modifier = Modifier.size(44.dp))
    }
    Spacer(Modifier.height(20.dp))
    Text(title, fontFamily = Brand.Serif, fontSize = 40.sp, textAlign = TextAlign.Center)
    Spacer(Modifier.height(8.dp))
    Text(
      text,
      style = MaterialTheme.typography.bodyLarge,
      color = MaterialTheme.colorScheme.onSurfaceVariant,
      textAlign = TextAlign.Center,
    )
    Spacer(Modifier.height(18.dp))
    Surface(shape = RoundedCornerShape(50), color = MaterialTheme.colorScheme.secondaryContainer) {
      Text(
        "Arrives in phase $phase of 13",
        style = MaterialTheme.typography.labelLarge,
        fontStyle = FontStyle.Normal,
        modifier = Modifier.padding(horizontal = 14.dp, vertical = 6.dp),
      )
    }
    Spacer(Modifier.height(12.dp))
    OutlinedButton(onClick = { uriHandler.openUri(Brand.PROGRESS_URL) }) { Text("See the build progress") }
  }
}
