package com.velmora.stepbyte.velmora.screens

import android.content.Context
import androidx.annotation.DrawableRes
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
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
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.BookmarkBorder
import androidx.compose.material.icons.rounded.Bookmarks
import androidx.compose.material.icons.rounded.CalendarMonth
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.CloudOff
import androidx.compose.material.icons.rounded.Favorite
import androidx.compose.material.icons.rounded.History
import androidx.compose.material.icons.rounded.LocalFireDepartment
import androidx.compose.material.icons.rounded.Movie
import androidx.compose.material.icons.rounded.NewReleases
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.SearchOff
import androidx.compose.material.icons.rounded.SportsSoccer
import androidx.compose.material.icons.rounded.Theaters
import androidx.compose.material.icons.rounded.Translate
import androidx.compose.material.icons.rounded.Tv
import androidx.compose.material3.AssistChip
import androidx.compose.material3.AssistChipDefaults
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.InputChip
import androidx.compose.material3.LinearWavyProgressIndicator
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
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.lerp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavType
import androidx.navigation.compose.composable
import androidx.navigation.navArgument
import coil3.compose.AsyncImage
import com.velmora.stepbyte.LocalPlayerAwareWindowInsets
import com.velmora.stepbyte.R
import com.velmora.stepbyte.engine.Match
import com.velmora.stepbyte.engine.SearchBatch
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.engine.TitleSummary
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.engine.Wire
import com.velmora.stepbyte.ui.screens.Screens
import com.velmora.stepbyte.velmora.library.VelmoraLibrary
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
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonObject

/** Velmora mode's destinations, inside the "velmora" graph. */
fun NavGraphBuilder.velmoraGraph(navController: NavController) {
  composable(Screens.VelmoraHome.route) { VelmoraHomeScreen(navController) }
  composable(Screens.VelmoraBrowse.route) { BrowseScreen(navController) }
  composable(Screens.VelmoraTheatre.route) {
    ComingScreen(
      icon = R.drawable.theaters_filled,
      title = "Theatre",
      text = "Scheduled screenings you watch together: the lineup, booking a seat, the lobby and live chat.",
      phase = 8,
    )
  }
  composable(Screens.VelmoraSearch.route) { VelmoraSearchScreen(navController) }
  composable(VelmoraRoutes.DOWNLOADS) {
    ComingScreen(
      icon = R.drawable.download,
      title = "Downloads",
      text = "Movies and episodes you save, locked to Velmora and playable offline.",
      phase = 7,
    )
  }
  composable(VRoutes.TITLE, arguments = listOf(navArgument("t") { type = NavType.StringType })) { entry ->
    val title = remember(entry) { entry.arguments?.getString("t")?.let(::decodeTitle) }
    if (title != null) TitleScreen(title, navController) else LaunchedEffect(Unit) { navController.navigateUp() }
  }
  composable(VRoutes.COLLECTION, arguments = listOf(navArgument("id") { type = NavType.StringType })) { entry ->
    CollectionScreen(entry.arguments?.getString("id") ?: "", navController)
  }
  composable(VRoutes.MY_LIST) { MyListScreen(navController) }
}

private fun decodeTitle(text: String): Title? =
  runCatching {
      val o = Universe.json.parseToJsonElement(text) as JsonObject
      Wire(Universe.json.decodeFromJsonElement(TitleSummary.serializer(), o), o)
    }
    .getOrNull()

private fun railIcon(id: String): ImageVector =
  when (id) {
    "trending_movies" -> Icons.Rounded.LocalFireDepartment
    "trending_series" -> Icons.Rounded.Tv
    "trending_anime" -> Icons.Rounded.AutoAwesome
    "airing_today" -> Icons.Rounded.CalendarMonth
    "hdhub_latest" -> Icons.Rounded.NewReleases
    "kdrama" -> Icons.Rounded.Favorite
    "hdhub_bollywood" -> Icons.Rounded.Theaters
    "hdhub_south" -> Icons.Rounded.Translate
    else -> Icons.Rounded.Movie
  }

/** One colour per collection chip, as on desktop. */
private val DOTS = listOf(0xFFFF6B6B, 0xFFFFB547, 0xFF5AD17A, 0xFF4FA3FF, 0xFFA77BFF, 0xFFFF7AD9, 0xFF3FD3C6, 0xFFC792EA).map { Color(it) }

// ── Home ───────────────────────────────────────────────────────────────

@Composable
fun VelmoraHomeScreen(navController: NavController) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  val rails by HomeFeed.rails.collectAsState()
  val loading by HomeFeed.loading.collectAsState()
  val error by HomeFeed.error.collectAsState()
  val stale by HomeFeed.stale.collectAsState()
  val myList by VelmoraLibrary.myList.collectAsState()
  val progress by VelmoraLibrary.progress.collectAsState()
  val continuing = remember(progress) { progress.filter { it.resumable }.take(12) }
  var live by remember { mutableStateOf<List<Match>>(emptyList()) }

  LaunchedEffect(Unit) { HomeFeed.load(context) }
  // Scores while Home is open, once a minute; a failure just hides the row.
  LaunchedEffect(Unit) {
    while (isActive) {
      live = runCatching { Universe.get(context).sportsLive() }.getOrDefault(live)
      delay(60_000)
    }
  }
  val spotlight = remember(rails) { HomeFeed.spotlight() }

  LazyColumn(modifier = Modifier.fillMaxSize(), contentPadding = LocalPlayerAwareWindowInsets.current.asPaddingValues()) {
    if (stale) {
      item(key = "offline") {
        OfflineBanner("You're offline. These are the rows from last time.", onRetry = { scope.launch { HomeFeed.load(context, force = true) } })
      }
    }
    if (spotlight.isNotEmpty()) {
      item(key = "hero") { HeroCarousel(spotlight, onOpen = { navController.openTitle(it) }) }
    } else if (loading || (rails.isEmpty() && error == null)) {
      item(key = "hero-ghost") { HeroGhost() }
    }
    if (rails.isNotEmpty()) {
      item(key = "chips") {
        LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.padding(top = 4.dp)) {
          item {
            AssistChip(
              onClick = { navController.navigate(VRoutes.MY_LIST) },
              label = { Text("My List") },
              leadingIcon = { Icon(Icons.Rounded.Bookmarks, contentDescription = null, modifier = Modifier.size(AssistChipDefaults.IconSize)) },
            )
          }
          items(rails.size) { i ->
            val r = rails[i]
            AssistChip(
              onClick = { navController.navigate(VRoutes.collection(r.id)) },
              label = { Text(r.title) },
              leadingIcon = { Box(Modifier.size(10.dp).clip(CircleShape).background(DOTS[i % DOTS.size])) },
            )
          }
        }
      }
    }
    if (continuing.isNotEmpty()) {
      item(key = "continue") {
        SectionHeader("Continue watching", icon = Icons.Rounded.History)
        LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
          items(continuing, key = { titleKey(it.title) }) { p -> ContinueCard(p, onClick = { com.velmora.stepbyte.velmora.player.VideoSession.play(context, p.title) }) }
        }
      }
    }
    if (myList.isNotEmpty()) {
      item(key = "mylist") {
        SectionHeader("My List", icon = Icons.Rounded.Bookmarks, onSeeAll = { navController.navigate(VRoutes.MY_LIST) })
        PosterRow(myList.take(20), onOpen = { navController.openTitle(it) })
      }
    }
    if (live.isNotEmpty()) {
      item(key = "live") {
        SectionHeader("Live now", icon = Icons.Rounded.SportsSoccer, onSeeAll = {
          BrowseTabs.request(BrowseTab.Sports)
          navController.navigate(Screens.VelmoraBrowse.route) { launchSingleTop = true }
        })
        LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
          items(live.take(12), key = { it.id }) { m -> MatchCard(m, Modifier.width(280.dp)) }
        }
      }
    }
    if (rails.isEmpty() && loading) {
      item(key = "ghost1") { PosterRowShimmer() }
      item(key = "ghost2") { PosterRowShimmer() }
    }
    if (rails.isEmpty() && !loading && error != null) {
      item(key = "error") {
        EmptyState(
          icon = Icons.Rounded.CloudOff,
          title = "Nothing to show yet",
          text = error ?: "",
          action = "Try again",
          onAction = { scope.launch { HomeFeed.load(context, force = true) } },
        )
      }
    }
    rails.forEach { rail ->
      item(key = "rail:${rail.id}") {
        SectionHeader(rail.title, icon = railIcon(rail.id), onSeeAll = { navController.navigate(VRoutes.collection(rail.id)) })
        PosterRow(rail.items, onOpen = { navController.openTitle(it) })
      }
    }
    item(key = "end") { Spacer(Modifier.height(16.dp)) }
  }
}

/** The big swipeable spotlight: art drifts as it slides, and it moves on by itself. */
@Composable
private fun HeroCarousel(items: List<Title>, onOpen: (Title) -> Unit) {
  val context = LocalContext.current
  val pager = rememberPagerState { items.size }
  LaunchedEffect(pager, items.size) {
    while (isActive && items.size > 1) {
      delay(6_500)
      if (!pager.isScrollInProgress) pager.animateScrollToPage((pager.currentPage + 1) % items.size, animationSpec = tween(700))
    }
  }
  Column {
    HorizontalPager(
      state = pager,
      contentPadding = PaddingValues(horizontal = 16.dp),
      pageSpacing = 10.dp,
      modifier = Modifier.fillMaxWidth().padding(top = 8.dp),
    ) { page ->
      val t = items[page]
      val v = t.value
      val offset = ((pager.currentPage - page) + pager.currentPageOffsetFraction)
      Box(
        Modifier.fillMaxWidth()
          .height(260.dp)
          .graphicsLayer {
            // The card leaving shrinks a touch, the one arriving grows in.
            val f = 1f - (kotlin.math.abs(offset) * 0.08f).coerceIn(0f, 0.08f)
            scaleX = f
            scaleY = f
          }
          .clip(RoundedCornerShape(30.dp))
          .background(MaterialTheme.colorScheme.surfaceContainerHigh)
          .pressable { onOpen(t) },
      ) {
        AsyncImage(
          model = v.backdrop ?: v.poster,
          contentDescription = null,
          contentScale = ContentScale.Crop,
          modifier =
            Modifier.fillMaxSize().graphicsLayer {
              // Parallax inside the card.
              translationX = offset * size.width * 0.25f
              scaleX = 1.15f
              scaleY = 1.15f
            },
        )
        Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0f to Color.Transparent, 0.4f to Color.Black.copy(alpha = 0.1f), 1f to Color.Black.copy(alpha = 0.88f))))
        Column(Modifier.align(Alignment.BottomStart).padding(20.dp)) {
          Surface(shape = RoundedCornerShape(50), color = Brand.Blue) {
            Text(
              listOfNotNull(kindLabel(v.kind), v.year?.toString()).joinToString(" · ").ifEmpty { "Trending" },
              color = Color.White,
              style = MaterialTheme.typography.labelMedium,
              fontWeight = FontWeight.Bold,
              modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
            )
          }
          Spacer(Modifier.height(8.dp))
          Text(v.name, color = Color.White, fontFamily = Brand.Serif, fontSize = 34.sp, lineHeight = 36.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
          v.overview?.let {
            Text(it, color = Color.White.copy(alpha = 0.8f), style = MaterialTheme.typography.bodySmall, maxLines = 2, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(top = 4.dp, end = 64.dp))
          }
        }
        FilledIconButton(
          onClick = { com.velmora.stepbyte.velmora.player.VideoSession.play(context, t) },
          colors = IconButtonDefaults.filledIconButtonColors(containerColor = Color.White, contentColor = Color.Black),
          modifier = Modifier.align(Alignment.BottomEnd).padding(18.dp).size(52.dp),
        ) {
          Icon(Icons.Rounded.PlayArrow, contentDescription = "Play", modifier = Modifier.size(30.dp))
        }
      }
    }
    // Page dots that stretch toward the next page as you swipe.
    Row(Modifier.fillMaxWidth().padding(top = 10.dp), horizontalArrangement = Arrangement.Center) {
      repeat(items.size) { i ->
        val distance = kotlin.math.abs((pager.currentPage + pager.currentPageOffsetFraction) - i).coerceIn(0f, 1f)
        Box(
          Modifier.padding(horizontal = 3.dp)
            .height(6.dp)
            .width(lerp(22.dp, 6.dp, distance))
            .clip(CircleShape)
            .background(MaterialTheme.colorScheme.primary.copy(alpha = 1f - distance * 0.7f)),
        )
      }
    }
  }
}

@Composable
private fun HeroGhost() {
  com.velmora.stepbyte.ui.component.shimmer.ShimmerHost {
    Box(Modifier.padding(horizontal = 16.dp, vertical = 8.dp).fillMaxWidth().height(260.dp).clip(RoundedCornerShape(30.dp)).background(MaterialTheme.colorScheme.onSurface))
  }
}

// ── Collection (a home row, all of it) ─────────────────────────────────

@Composable
fun CollectionScreen(id: String, navController: NavController) {
  val rails by HomeFeed.rails.collectAsState()
  val rail = remember(rails, id) { rails.firstOrNull { it.id == id } }
  GridPage(
    title = rail?.title ?: "Collection",
    items = rail?.items.orEmpty(),
    navController = navController,
    empty = { EmptyState(Icons.Rounded.CloudOff, "Nothing here", "This row is empty right now. Home refreshes it when you're online.") },
  )
}

@Composable
fun MyListScreen(navController: NavController) {
  val list by VelmoraLibrary.myList.collectAsState()
  GridPage(
    title = "My List",
    subtitle = if (list.isEmpty()) null else if (list.size == 1) "1 title" else "${list.size} titles",
    items = list,
    navController = navController,
    empty = {
      EmptyState(
        Icons.Rounded.BookmarkBorder,
        "Nothing saved yet",
        "Tap the bookmark on any title to keep it here. My List syncs with desktop.",
        action = "Find something",
        onAction = { navController.navigate(Screens.VelmoraBrowse.route) },
      )
    },
  )
}

/** A page of posters with its own back bar. */
@Composable
private fun GridPage(title: String, items: List<Title>, navController: NavController, subtitle: String? = null, empty: @Composable () -> Unit) {
  Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
    Row(Modifier.fillMaxWidth().windowInsetsPadding(WindowInsets.statusBars).padding(horizontal = 4.dp, vertical = 4.dp), verticalAlignment = Alignment.CenterVertically) {
      IconButton(onClick = { navController.navigateUp() }) { Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = "Back") }
    }
    LazyVerticalGrid(
      columns = GridCells.Adaptive(minSize = 108.dp),
      contentPadding = PaddingValues(start = 16.dp, end = 16.dp, bottom = bottomInsets().asPaddingValues().calculateBottomPadding() + 16.dp),
      horizontalArrangement = Arrangement.spacedBy(12.dp),
      verticalArrangement = Arrangement.spacedBy(6.dp),
      modifier = Modifier.fillMaxSize(),
    ) {
      item(span = { GridItemSpan(maxLineSpan) }) {
        Column(Modifier.padding(bottom = 12.dp)) {
          Text(title, fontFamily = Brand.Serif, fontSize = 42.sp, lineHeight = 44.sp)
          subtitle?.let { Text(it, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant) }
        }
      }
      if (items.isEmpty()) {
        item(span = { GridItemSpan(maxLineSpan) }) { empty() }
      }
      items(items.distinctBy(::titleKey), key = ::titleKey) { t ->
        PosterCard(t, onClick = { navController.openTitle(t) }, modifier = Modifier.fillMaxWidth().animateItem())
      }
    }
  }
}

// ── Search ─────────────────────────────────────────────────────────────

private object RecentSearches {
  private const val PREFS = "velmora_search"
  private const val KEY = "recent"

  fun read(context: Context): List<String> =
    context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString(KEY, null)?.split('\n')?.filter { it.isNotBlank() } ?: emptyList()

  fun add(context: Context, q: String): List<String> {
    val next = (listOf(q) + read(context).filterNot { it.equals(q, ignoreCase = true) }).take(10)
    context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putString(KEY, next.joinToString("\n")).apply()
    return next
  }

  fun remove(context: Context, q: String): List<String> {
    val next = read(context).filterNot { it == q }
    context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putString(KEY, next.joinToString("\n")).apply()
    return next
  }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun VelmoraSearchScreen(navController: NavController) {
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
  var recent by remember { mutableStateOf(RecentSearches.read(context)) }

  fun startSearch(q: String, keep: Boolean = false) {
    val text = q.trim()
    if (keep && text.length >= 2) recent = RecentSearches.add(context, text)
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
          error = e.plain()
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

  val results = batches.flatMap { it.items }.distinctBy { "${it.value.name.lowercase()}|${it.value.year}" }.take(60)
  val answered = batches.size
  val jump = modeFromQuery(query)?.takeIf { it != mode }
  val nothing = !searching && searched.isNotEmpty() && results.isEmpty() && error == null && query.trim() == searched

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
        leadingIcon = { Icon(Icons.Rounded.Search, contentDescription = null) },
        trailingIcon = {
          if (query.isNotEmpty()) {
            IconButton(onClick = { query = "" }) { Icon(Icons.Rounded.Close, contentDescription = "Clear") }
          }
        },
        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
        keyboardActions = KeyboardActions(onSearch = { startSearch(query, keep = true) }),
        colors = TextFieldDefaults.colors(focusedIndicatorColor = Color.Transparent, unfocusedIndicatorColor = Color.Transparent),
      )
    }
    if (jump != null) {
      item(key = "jump", span = { GridItemSpan(maxLineSpan) }) { SwitchSuggestion(jump) { switchMode(jump) } }
    }
    if (searching) {
      item(key = "progress", span = { GridItemSpan(maxLineSpan) }) {
        Column(Modifier.padding(horizontal = 16.dp, vertical = 4.dp)) {
          LinearWavyProgressIndicator(modifier = Modifier.fillMaxWidth())
          Text(
            if (answered == 0) "Asking every source…" else "$answered sources answered",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(top = 4.dp),
          )
        }
      }
    }
    error?.let { e ->
      item(key = "error", span = { GridItemSpan(maxLineSpan) }) { OfflineBanner("Search couldn't finish: $e", onRetry = { startSearch(query) }) }
    }
    if (query.isBlank() && results.isEmpty()) {
      if (recent.isNotEmpty()) {
        item(key = "recent", span = { GridItemSpan(maxLineSpan) }) {
          Column(Modifier.padding(horizontal = 12.dp)) {
            Text("Recent", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, modifier = Modifier.padding(4.dp, 10.dp))
            androidx.compose.foundation.layout.FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
              recent.forEach { r ->
                InputChip(
                  selected = false,
                  onClick = { query = r },
                  label = { Text(r) },
                  leadingIcon = { Icon(Icons.Rounded.History, contentDescription = null, modifier = Modifier.size(18.dp)) },
                  trailingIcon = {
                    Icon(
                      Icons.Rounded.Close,
                      contentDescription = "Forget",
                      modifier = Modifier.size(18.dp).clip(CircleShape).pressable { recent = RecentSearches.remove(context, r) },
                    )
                  },
                )
              }
            }
          }
        }
      }
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
    if (nothing) {
      item(key = "nothing", span = { GridItemSpan(maxLineSpan) }) {
        EmptyState(Icons.Rounded.SearchOff, "No matches", "Nothing for “$searched”. Try the original title, or fewer words.")
      }
    }
    items(results, key = ::titleKey) { t ->
      Box(Modifier.padding(horizontal = 2.dp).animateItem()) {
        PosterCard(
          t,
          onClick = {
            recent = RecentSearches.add(context, searched.ifEmpty { query.trim() })
            navController.openTitle(t)
          },
          modifier = Modifier.fillMaxWidth(),
        )
      }
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
        Modifier.background(Brush.horizontalGradient(if (music) listOf(Brand.Blue, Brand.Amber) else listOf(Brand.Amber, Brand.Blue)))
          .padding(horizontal = 16.dp, vertical = 14.dp),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      Box(Modifier.size(40.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.25f)), contentAlignment = Alignment.Center) {
        Icon(painterResource(if (music) R.drawable.music_note else R.drawable.movie), contentDescription = null, tint = Color.White)
      }
      Column(Modifier.weight(1f).padding(horizontal = 14.dp)) {
        Text(if (music) "Switch to Music" else "Switch to Velmora", color = Color.White, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
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
    modifier = Modifier.fillMaxSize().padding(LocalPlayerAwareWindowInsets.current.asPaddingValues()).padding(horizontal = 32.dp),
    horizontalAlignment = Alignment.CenterHorizontally,
    verticalArrangement = Arrangement.Center,
  ) {
    Box(
      Modifier.size(96.dp).clip(RoundedCornerShape(32.dp)).background(Brush.linearGradient(listOf(Brand.Blue.copy(alpha = 0.35f), Brand.Cream.copy(alpha = 0.25f)))),
      contentAlignment = Alignment.Center,
    ) {
      Icon(painterResource(icon), contentDescription = null, modifier = Modifier.size(44.dp))
    }
    Spacer(Modifier.height(20.dp))
    Text(title, fontFamily = Brand.Serif, fontSize = 40.sp, textAlign = TextAlign.Center)
    Spacer(Modifier.height(8.dp))
    Text(text, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center)
    Spacer(Modifier.height(18.dp))
    Surface(shape = RoundedCornerShape(50), color = MaterialTheme.colorScheme.secondaryContainer) {
      Text("Arrives in phase $phase of 13", style = MaterialTheme.typography.labelLarge, modifier = Modifier.padding(horizontal = 14.dp, vertical = 6.dp))
    }
    Spacer(Modifier.height(12.dp))
    OutlinedButton(onClick = { uriHandler.openUri(Brand.PROGRESS_URL) }) { Text("See the build progress") }
  }
}
