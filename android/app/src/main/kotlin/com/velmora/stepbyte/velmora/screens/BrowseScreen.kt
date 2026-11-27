package com.velmora.stepbyte.velmora.screens

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
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.CloudOff
import androidx.compose.material.icons.rounded.LiveTv
import androidx.compose.material.icons.rounded.Movie
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.SportsSoccer
import androidx.compose.material.icons.rounded.Tv
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.PrimaryScrollableTabRow
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Tab
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import coil3.compose.AsyncImage
import com.velmora.stepbyte.LocalPlayerAwareWindowInsets
import com.velmora.stepbyte.engine.Facet
import com.velmora.stepbyte.engine.Match
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.engine.TvChannel
import com.velmora.stepbyte.engine.TvQuery
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.velmora.shell.Brand
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray

enum class BrowseTab(val label: String, val icon: ImageVector) {
  Movies("Movies", Icons.Rounded.Movie),
  Series("Series", Icons.Rounded.Tv),
  Anime("Anime", Icons.Rounded.AutoAwesome),
  LiveTv("Live TV", Icons.Rounded.LiveTv),
  Sports("Sports", Icons.Rounded.SportsSoccer),
}

/** Lets Home open Browse on a given tab ("Live now" opens Sports). */
object BrowseTabs {
  val requested = MutableStateFlow<BrowseTab?>(null)

  fun request(tab: BrowseTab) {
    requested.value = tab
  }
}

// ── The catalogs, kept for the whole run so a title page and back is instant ──

/** One TMDB catalog (movies or series) with a genre and an order, a page at a time. */
class TmdbCatalog(private val tv: Boolean) {
  enum class Order(val label: String) { Popular("Popular"), Trending("Trending"), Top("Top rated"), New("New") }

  val genres: List<Pair<Int, String>> =
    if (tv) {
      listOf(10759 to "Action", 35 to "Comedy", 80 to "Crime", 18 to "Drama", 99 to "Documentary", 10751 to "Family", 9648 to "Mystery", 10764 to "Reality", 10765 to "Sci-Fi & Fantasy", 10768 to "War & Politics", 37 to "Western", 10762 to "Kids")
    } else {
      listOf(28 to "Action", 12 to "Adventure", 35 to "Comedy", 80 to "Crime", 18 to "Drama", 27 to "Horror", 53 to "Thriller", 878 to "Sci-Fi", 10749 to "Romance", 14 to "Fantasy", 16 to "Animation", 9648 to "Mystery", 10751 to "Family", 99 to "Documentary", 36 to "History", 10752 to "War", 37 to "Western", 10402 to "Music")
    }

  var genre by mutableStateOf<Int?>(null)
    private set

  var order by mutableStateOf(Order.Popular)
    private set

  val items = mutableStateListOf<Title>()
  var loading by mutableStateOf(false)
    private set

  var error by mutableStateOf<String?>(null)
    private set

  private var page = 0
  private var end = false
  private val lock = Mutex()

  fun choose(context: Context, scope: kotlinx.coroutines.CoroutineScope, genre: Int? = this.genre, order: Order = this.order) {
    if (genre == this.genre && order == this.order && items.isNotEmpty()) return
    this.genre = genre
    this.order = order
    items.clear()
    page = 0
    end = false
    error = null
    scope.launch { more(context) }
  }

  suspend fun more(context: Context) {
    lock.withLock {
      if (end) return
      loading = true
      error = null
      try {
        val next = page + 1
        val want = genre to order
        val got = fetch(context, next)
        // The choice changed while this page was out: drop it.
        if (want != (genre to order)) return
        val seen = items.map(::titleKey).toHashSet()
        items += got.filter { seen.add(titleKey(it)) }
        page = next
        if (got.isEmpty() || next >= 30) end = true
      } catch (e: CancellationException) {
        throw e
      } catch (e: Throwable) {
        error = e.plain()
      } finally {
        loading = false
      }
    }
  }

  private suspend fun fetch(context: Context, page: Int): List<Title> {
    val engine = Universe.get(context)
    val kind = if (tv) "tv" else "movie"
    if (order == Order.Trending && genre == null) {
      return engine.tmdbList("/trending/$kind/week", mapOf("page" to page.toString()), kind = kind)
    }
    val today = LocalDate.now().toString()
    val q = buildMap {
      put("page", page.toString())
      put("include_adult", "false")
      genre?.let { put("with_genres", it.toString()) }
      // Anime has its own tab.
      if (tv && genre != 16) put("without_genres", "16")
      when (order) {
        Order.Popular,
        Order.Trending -> put("sort_by", "popularity.desc")
        Order.Top -> {
          put("sort_by", "vote_average.desc")
          put("vote_count.gte", if (tv) "200" else "500")
        }
        Order.New -> {
          put("sort_by", if (tv) "first_air_date.desc" else "primary_release_date.desc")
          put(if (tv) "first_air_date.lte" else "primary_release_date.lte", today)
          put("vote_count.gte", "30")
        }
      }
    }
    return engine.tmdbList("/discover/$kind", q, kind = kind)
  }
}

/** Anime from AniList: genres and a few orders (AniList gives one page of 24). */
class AnimeCatalog {
  enum class Order(val label: String) { Popular("Popular"), Trending("Trending"), Season("This season"), Top("Top rated"), Airing("Airing") }

  val genres = listOf("Action", "Adventure", "Comedy", "Drama", "Fantasy", "Romance", "Sci-Fi", "Slice of Life", "Mystery", "Psychological", "Supernatural", "Sports", "Mecha", "Horror", "Music", "Thriller")

  var genre by mutableStateOf<String?>(null)
    private set

  var order by mutableStateOf(Order.Popular)
    private set

  val items = mutableStateListOf<Title>()
  var loading by mutableStateOf(false)
    private set

  var error by mutableStateOf<String?>(null)
    private set

  private val lock = Mutex()

  fun choose(context: Context, scope: kotlinx.coroutines.CoroutineScope, genre: String? = this.genre, order: Order = this.order) {
    if (genre == this.genre && order == this.order && items.isNotEmpty()) return
    this.genre = genre
    this.order = order
    items.clear()
    scope.launch { load(context) }
  }

  suspend fun load(context: Context) {
    lock.withLock {
      loading = true
      error = null
      val want = genre to order
      try {
        val filters = buildJsonObject {
          genre?.let { putJsonArray("genres") { add(kotlinx.serialization.json.JsonPrimitive(it)) } }
          when (order) {
            Order.Popular -> put("sort", "POPULARITY_DESC")
            Order.Trending -> put("sort", "TRENDING_DESC")
            Order.Top -> put("sort", "SCORE_DESC")
            Order.Airing -> {
              put("sort", "POPULARITY_DESC")
              put("status", "RELEASING")
            }
            Order.Season -> {
              val now = LocalDate.now()
              val season = when (now.monthValue) { in 1..3 -> "WINTER"; in 4..6 -> "SPRING"; in 7..9 -> "SUMMER"; else -> "FALL" }
              put("sort", "POPULARITY_DESC")
              put("season", season)
              put("year", now.year)
            }
          }
        }
        val got = Universe.get(context).anilistFind(filters)
        if (want == (genre to order)) {
          items.clear()
          items += got.distinctBy(::titleKey)
        }
      } catch (e: CancellationException) {
        throw e
      } catch (e: Throwable) {
        error = e.plain()
      } finally {
        loading = false
      }
    }
  }
}

/** Live channels from iptv-org, filtered by category and text, a page at a time. */
class TvCatalog {
  var categories by mutableStateOf<List<Facet>>(emptyList())
    private set

  var category by mutableStateOf<String?>(null)
    private set

  var text by mutableStateOf("")
    private set

  val items = mutableStateListOf<TvChannel>()
  var total by mutableStateOf(0)
    private set

  var loading by mutableStateOf(false)
    private set

  var error by mutableStateOf<String?>(null)
    private set

  private val lock = Mutex()

  fun choose(context: Context, scope: kotlinx.coroutines.CoroutineScope, category: String? = this.category, text: String = this.text) {
    if (category == this.category && text == this.text && items.isNotEmpty()) return
    this.category = category
    this.text = text
    items.clear()
    total = 0
    scope.launch { more(context) }
  }

  suspend fun more(context: Context) {
    lock.withLock {
      if (items.isNotEmpty() && items.size >= total) return
      loading = true
      error = null
      val want = category to text
      try {
        val engine = Universe.get(context)
        if (categories.isEmpty()) categories = runCatching { engine.tvCategories().sortedByDescending { it.count }.take(16) }.getOrDefault(emptyList())
        val page = engine.tvChannels(TvQuery(category = category, text = text.trim().ifEmpty { null }, offset = items.size, limit = 60))
        if (want == (category to text)) {
          items += page.items.filter { c -> items.none { it.id == c.id } }
          total = page.total
        }
      } catch (e: CancellationException) {
        throw e
      } catch (e: Throwable) {
        error = e.plain()
      } finally {
        loading = false
      }
    }
  }
}

/** Matches: live now, coming up, finished, refreshed while the tab is open. */
class SportsBoard {
  var live by mutableStateOf<List<Match>>(emptyList())
    private set

  var upcoming by mutableStateOf<List<Match>>(emptyList())
    private set

  var results by mutableStateOf<List<Match>>(emptyList())
    private set

  var loading by mutableStateOf(false)
    private set

  var error by mutableStateOf<String?>(null)
    private set

  var loaded by mutableStateOf(false)
    private set

  suspend fun load(context: Context) {
    loading = true
    try {
      val engine = Universe.get(context)
      coroutineScope {
        val a = async { runCatching { engine.sportsLive() } }
        val b = async { runCatching { engine.sportsUpcoming(3) } }
        val c = async { runCatching { engine.sportsResults(2) } }
        val (l, u, r) = Triple(a.await(), b.await(), c.await())
        l.onSuccess { live = it }
        u.onSuccess { upcoming = it.filter { m -> m.status == "upcoming" }.sortedBy { m -> m.kickoff ?: Long.MAX_VALUE } }
        r.onSuccess { results = it.sortedByDescending { m -> m.kickoff ?: 0 } }
        error = if (l.isFailure && u.isFailure && r.isFailure) (l.exceptionOrNull() ?: u.exceptionOrNull())?.plain() else null
      }
      loaded = true
    } catch (e: CancellationException) {
      throw e
    } catch (e: Throwable) {
      error = e.plain()
    } finally {
      loading = false
    }
  }
}

private object BrowseStore {
  val movies = TmdbCatalog(tv = false)
  val series = TmdbCatalog(tv = true)
  val anime = AnimeCatalog()
  val tv = TvCatalog()
  val sports = SportsBoard()
}

// ── The screen ─────────────────────────────────────────────────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun BrowseScreen(navController: NavController) {
  val tabs = BrowseTab.entries
  val pager = rememberPagerState { tabs.size }
  val scope = rememberCoroutineScope()
  val requested by BrowseTabs.requested.collectAsState()
  LaunchedEffect(requested) {
    requested?.let {
      pager.scrollToPage(it.ordinal)
      BrowseTabs.requested.value = null
    }
  }
  val top = LocalPlayerAwareWindowInsets.current.asPaddingValues().calculateTopPadding()

  Column(Modifier.fillMaxSize().padding(top = top)) {
    PrimaryScrollableTabRow(selectedTabIndex = pager.currentPage, edgePadding = 12.dp, divider = {}, containerColor = Color.Transparent) {
      tabs.forEachIndexed { i, tab ->
        Tab(
          selected = pager.currentPage == i,
          onClick = { scope.launch { pager.animateScrollToPage(i) } },
          text = { Text(tab.label, fontWeight = if (pager.currentPage == i) FontWeight.Bold else FontWeight.Medium) },
          icon = { Icon(tab.icon, contentDescription = null, modifier = Modifier.size(20.dp)) },
        )
      }
    }
    HorizontalPager(state = pager, modifier = Modifier.fillMaxSize(), beyondViewportPageCount = 1, key = { tabs[it].name }) { page ->
      when (tabs[page]) {
        BrowseTab.Movies -> TmdbPage(BrowseStore.movies, navController)
        BrowseTab.Series -> TmdbPage(BrowseStore.series, navController)
        BrowseTab.Anime -> AnimePage(BrowseStore.anime, navController)
        BrowseTab.LiveTv -> TvPage(BrowseStore.tv)
        BrowseTab.Sports -> SportsPage(BrowseStore.sports)
      }
    }
  }
}

@Composable
private fun bottomPad(): androidx.compose.ui.unit.Dp = bottomInsets().asPaddingValues().calculateBottomPadding() + 16.dp

/** Calls [onEnd] when the grid scrolls near its last item. */
@Composable
private fun LoadMore(state: LazyGridState, count: Int, onEnd: suspend () -> Unit) {
  LaunchedEffect(state, count) {
    snapshotFlow { state.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0 }
      .distinctUntilChanged()
      .filter { count > 0 && it >= state.layoutInfo.totalItemsCount - 10 }
      .collect { onEnd() }
  }
}

@Composable
private fun <T> ChipRow(options: List<T>, selected: T?, label: (T) -> String, onSelect: (T?) -> Unit, allLabel: String? = "All") {
  LazyRow(contentPadding = PaddingValues(horizontal = 4.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
    if (allLabel != null) {
      item {
        FilterChip(selected = selected == null, onClick = { onSelect(null) }, label = { Text(allLabel) }, shape = RoundedCornerShape(12.dp))
      }
    }
    items(options) { o ->
      FilterChip(
        selected = o == selected,
        onClick = { onSelect(if (o == selected) null else o) },
        label = { Text(label(o)) },
        shape = RoundedCornerShape(12.dp),
        colors = FilterChipDefaults.filterChipColors(selectedContainerColor = MaterialTheme.colorScheme.primaryContainer),
      )
    }
  }
}

@Composable
private fun <T> OrderRow(options: List<T>, selected: T, label: (T) -> String, onSelect: (T) -> Unit) {
  SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
    options.forEachIndexed { i, o ->
      SegmentedButton(
        selected = o == selected,
        onClick = { onSelect(o) },
        shape = SegmentedButtonDefaults.itemShape(index = i, count = options.size),
        icon = {},
        label = { Text(label(o), maxLines = 1, overflow = TextOverflow.Ellipsis, style = MaterialTheme.typography.labelMedium) },
      )
    }
  }
}

// ── Movies and Series ──

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun TmdbPage(catalog: TmdbCatalog, navController: NavController) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  val grid = rememberLazyGridState()
  LaunchedEffect(catalog) { if (catalog.items.isEmpty() && !catalog.loading) catalog.more(context) }
  LoadMore(grid, catalog.items.size) { catalog.more(context) }

  LazyVerticalGrid(
    state = grid,
    columns = GridCells.Adaptive(minSize = 108.dp),
    contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 8.dp, bottom = bottomPad()),
    horizontalArrangement = Arrangement.spacedBy(10.dp),
    verticalArrangement = Arrangement.spacedBy(6.dp),
    modifier = Modifier.fillMaxSize(),
  ) {
    item(span = { GridItemSpan(maxLineSpan) }, key = "order") {
      OrderRow(TmdbCatalog.Order.entries, catalog.order, { it.label }) { catalog.choose(context, scope, order = it) }
    }
    item(span = { GridItemSpan(maxLineSpan) }, key = "genres") {
      Box(Modifier.padding(top = 6.dp, bottom = 6.dp)) {
        ChipRow(catalog.genres.map { it.first }, catalog.genre, { id -> catalog.genres.first { it.first == id }.second }, { catalog.choose(context, scope, genre = it) })
      }
    }
    gridBody(catalog.items, catalog.loading, catalog.error, navController, onRetry = { scope.launch { catalog.more(context) } })
  }
}

// ── Anime ──

@Composable
private fun AnimePage(catalog: AnimeCatalog, navController: NavController) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  LaunchedEffect(catalog) { if (catalog.items.isEmpty() && !catalog.loading) catalog.load(context) }
  LazyVerticalGrid(
    columns = GridCells.Adaptive(minSize = 108.dp),
    contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 8.dp, bottom = bottomPad()),
    horizontalArrangement = Arrangement.spacedBy(10.dp),
    verticalArrangement = Arrangement.spacedBy(6.dp),
    modifier = Modifier.fillMaxSize(),
  ) {
    item(span = { GridItemSpan(maxLineSpan) }, key = "order") {
      ChipRow(AnimeCatalog.Order.entries, catalog.order, { it.label }, { it?.let { o -> catalog.choose(context, scope, order = o) } }, allLabel = null)
    }
    item(span = { GridItemSpan(maxLineSpan) }, key = "genres") {
      Box(Modifier.padding(top = 2.dp, bottom = 6.dp)) {
        ChipRow(catalog.genres, catalog.genre, { it }, { catalog.choose(context, scope, genre = it) })
      }
    }
    gridBody(catalog.items, catalog.loading, catalog.error, navController, onRetry = { scope.launch { catalog.load(context) } })
  }
}

/** The posters, the loading ghosts and the error, shared by the catalog pages. */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
private fun androidx.compose.foundation.lazy.grid.LazyGridScope.gridBody(
  items: List<Title>,
  loading: Boolean,
  error: String?,
  navController: NavController,
  onRetry: () -> Unit,
) {
  items(items, key = ::titleKey) { t ->
    PosterCard(t, onClick = { navController.openTitle(t) }, modifier = Modifier.fillMaxWidth().animateItem())
  }
  if (loading && items.isEmpty()) {
    items(9, key = { "ghost$it" }) { PosterGhost() }
  } else if (loading) {
    item(span = { GridItemSpan(maxLineSpan) }, key = "more") {
      Box(Modifier.fillMaxWidth().padding(16.dp), contentAlignment = Alignment.Center) { LoadingIndicator() }
    }
  }
  if (error != null) {
    item(span = { GridItemSpan(maxLineSpan) }, key = "error") {
      if (items.isEmpty()) {
        EmptyState(Icons.Rounded.CloudOff, "Couldn't load this", error, action = "Try again", onAction = onRetry)
      } else {
        OfflineBanner("Couldn't load more: $error", onRetry = onRetry)
      }
    }
  }
}

// ── Live TV ──

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun TvPage(catalog: TvCatalog) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  val grid = rememberLazyGridState()
  var typed by remember { mutableStateOf(catalog.text) }
  // Also the first load: same text and no channels yet.
  LaunchedEffect(typed) {
    if (typed != catalog.text) delay(400)
    catalog.choose(context, scope, text = typed)
  }
  LoadMore(grid, catalog.items.size) { catalog.more(context) }

  LazyVerticalGrid(
    state = grid,
    columns = GridCells.Adaptive(minSize = 150.dp),
    contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 8.dp, bottom = bottomPad()),
    horizontalArrangement = Arrangement.spacedBy(10.dp),
    verticalArrangement = Arrangement.spacedBy(10.dp),
    modifier = Modifier.fillMaxSize(),
  ) {
    item(span = { GridItemSpan(maxLineSpan) }, key = "search") {
      TextField(
        value = typed,
        onValueChange = { typed = it },
        singleLine = true,
        shape = RoundedCornerShape(28.dp),
        placeholder = { Text("Channel name") },
        leadingIcon = { Icon(Icons.Rounded.Search, contentDescription = null) },
        trailingIcon = {
          if (typed.isNotEmpty()) IconButton(onClick = { typed = "" }) { Icon(Icons.Rounded.Close, contentDescription = "Clear") }
        },
        colors = TextFieldDefaults.colors(focusedIndicatorColor = Color.Transparent, unfocusedIndicatorColor = Color.Transparent),
        modifier = Modifier.fillMaxWidth(),
      )
    }
    if (catalog.categories.isNotEmpty()) {
      item(span = { GridItemSpan(maxLineSpan) }, key = "categories") {
        ChipRow(catalog.categories.map { it.id }, catalog.category, { id -> catalog.categories.first { it.id == id }.name }, { catalog.choose(context, scope, category = it) })
      }
    }
    if (catalog.total > 0) {
      item(span = { GridItemSpan(maxLineSpan) }, key = "count") {
        Text("%,d channels".format(catalog.total), style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(horizontal = 4.dp))
      }
    }
    items(catalog.items, key = { it.id }) { c -> ChannelCard(c, Modifier.animateItem()) }
    if (catalog.loading) {
      item(span = { GridItemSpan(maxLineSpan) }, key = "loading") {
        Box(Modifier.fillMaxWidth().padding(24.dp), contentAlignment = Alignment.Center) { LoadingIndicator() }
      }
    }
    catalog.error?.let { e ->
      item(span = { GridItemSpan(maxLineSpan) }, key = "error") {
        if (catalog.items.isEmpty()) {
          EmptyState(Icons.Rounded.CloudOff, "No channels", e, action = "Try again", onAction = { scope.launch { catalog.more(context) } })
        } else {
          OfflineBanner(e, onRetry = { scope.launch { catalog.more(context) } })
        }
      }
    }
    if (!catalog.loading && catalog.error == null && catalog.items.isEmpty()) {
      item(span = { GridItemSpan(maxLineSpan) }, key = "none") {
        EmptyState(Icons.Rounded.LiveTv, "No channels match", "Try another name or category.")
      }
    }
  }
}

@Composable
private fun ChannelCard(c: TvChannel, modifier: Modifier = Modifier) {
  val context = LocalContext.current
  Surface(
    shape = RoundedCornerShape(20.dp),
    color = MaterialTheme.colorScheme.surfaceContainer,
    modifier = modifier.fillMaxWidth().clip(RoundedCornerShape(20.dp)).pressable { liveSoon(context) },
  ) {
    Column {
      Box(Modifier.fillMaxWidth().aspectRatio(16f / 10f).background(MaterialTheme.colorScheme.surfaceContainerHighest), contentAlignment = Alignment.Center) {
        Text(
          c.name.take(2).uppercase(),
          style = MaterialTheme.typography.headlineSmall,
          fontWeight = FontWeight.Bold,
          color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
        )
        AsyncImage(model = c.logo, contentDescription = c.name, contentScale = ContentScale.Fit, modifier = Modifier.fillMaxSize().padding(18.dp))
        Row(Modifier.align(Alignment.TopStart).padding(8.dp)) { LiveDot() }
      }
      Column(Modifier.padding(horizontal = 12.dp, vertical = 10.dp)) {
        Text(c.name, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Text(
          listOfNotNull(c.country?.let { flag(it) + " " + it }, c.categories.firstOrNull()?.replaceFirstChar { it.uppercase() }).joinToString(" · "),
          style = MaterialTheme.typography.bodySmall,
          color = MaterialTheme.colorScheme.onSurfaceVariant,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
        )
      }
    }
  }
}

private fun liveSoon(context: Context) {
  Toast.makeText(context, "Live channels play in phase 8", Toast.LENGTH_SHORT).show()
}

/** "US" to the flag emoji. */
private fun flag(code: String): String {
  if (code.length != 2 || !code.all { it.isLetter() }) return ""
  val base = 0x1F1E6 - 'A'.code
  return code.uppercase().map { String(Character.toChars(base + it.code)) }.joinToString("")
}

/** A small pulsing LIVE pill. */
@Composable
fun LiveDot() {
  val pulse = rememberInfiniteTransition(label = "live")
  val a by pulse.animateFloat(0.35f, 1f, infiniteRepeatable(tween(900), RepeatMode.Reverse), label = "a")
  Surface(shape = RoundedCornerShape(50), color = Color(0xFFE53935)) {
    Row(Modifier.padding(horizontal = 7.dp, vertical = 2.dp), verticalAlignment = Alignment.CenterVertically) {
      Box(Modifier.size(6.dp).graphicsLayer { alpha = a }.clip(CircleShape).background(Color.White))
      Spacer(Modifier.width(4.dp))
      Text("LIVE", color = Color.White, style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.Bold)
    }
  }
}

// ── Sports ──

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun SportsPage(board: SportsBoard) {
  val context = LocalContext.current
  LaunchedEffect(board) {
    while (isActive) {
      board.load(context)
      delay(60_000)
    }
  }
  val byDay = remember(board.upcoming) { board.upcoming.groupBy { day(it.kickoff) } }

  androidx.compose.foundation.lazy.LazyColumn(
    contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = bottomPad()),
    verticalArrangement = Arrangement.spacedBy(10.dp),
    modifier = Modifier.fillMaxSize(),
  ) {
    if (!board.loaded && board.loading) {
      item(key = "loading") { Box(Modifier.fillMaxWidth().padding(48.dp), contentAlignment = Alignment.Center) { LoadingIndicator() } }
    }
    if (board.loaded && board.live.isEmpty() && board.upcoming.isEmpty() && board.results.isEmpty()) {
      item(key = "none") {
        EmptyState(Icons.Rounded.SportsSoccer, "No matches right now", board.error ?: "Nothing live or coming up in the next few days.")
      }
    }
    if (board.live.isNotEmpty()) {
      item(key = "live-h") { SportsHeading("Live now") }
      items(board.live, key = { "l" + it.id }) { m -> MatchCard(m, Modifier.animateItem()) }
    }
    byDay.forEach { (label, matches) ->
      item(key = "d-$label") { SportsHeading(label) }
      items(matches.take(40), key = { "u" + it.id }) { m -> MatchCard(m, Modifier.animateItem()) }
    }
    if (board.results.isNotEmpty()) {
      item(key = "res-h") { SportsHeading("Results") }
      items(board.results.take(40), key = { "r" + it.id }) { m -> MatchCard(m, Modifier.animateItem()) }
    }
  }
}

@Composable
private fun SportsHeading(text: String) {
  Text(text, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold, modifier = Modifier.padding(top = 12.dp, bottom = 2.dp))
}

private fun day(kickoff: Long?): String {
  if (kickoff == null) return "Later"
  val d = Instant.ofEpochSecond(kickoff).atZone(ZoneId.systemDefault()).toLocalDate()
  val today = LocalDate.now()
  return when (d) {
    today -> "Today"
    today.plusDays(1) -> "Tomorrow"
    else -> d.format(DateTimeFormatter.ofPattern("EEEE d MMM"))
  }
}

private fun clock(kickoff: Long?): String =
  kickoff?.let { Instant.ofEpochSecond(it).atZone(ZoneId.systemDefault()).format(DateTimeFormatter.ofPattern("HH:mm")) } ?: "TBC"

/** One fixture: league, both teams with badges, the score or the kickoff. */
@Composable
fun MatchCard(m: Match, modifier: Modifier = Modifier) {
  Surface(shape = RoundedCornerShape(22.dp), color = MaterialTheme.colorScheme.surfaceContainer, modifier = modifier.fillMaxWidth()) {
    Column(Modifier.padding(14.dp)) {
      Row(verticalAlignment = Alignment.CenterVertically) {
        AsyncImage(model = m.league.badge, contentDescription = null, modifier = Modifier.size(18.dp))
        Spacer(Modifier.width(6.dp))
        Text(
          listOfNotNull(m.league.name, m.round).joinToString(" · "),
          style = MaterialTheme.typography.labelMedium,
          color = MaterialTheme.colorScheme.onSurfaceVariant,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
          modifier = Modifier.weight(1f),
        )
        when {
          m.live -> LiveDot()
          m.status == "finished" -> Text("FT", style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.Bold)
          m.status == "postponed" || m.status == "cancelled" ->
            Text(m.status.replaceFirstChar { it.uppercase() }, style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.error)
        }
      }
      Spacer(Modifier.height(10.dp))
      if (m.home != null && m.away != null) {
        Row(verticalAlignment = Alignment.CenterVertically) {
          TeamSide(m.home!!.name, m.home!!.badge, Modifier.weight(1f), alignEnd = false)
          val score =
            if (m.homeScore != null && m.awayScore != null && m.status != "upcoming") "${m.homeScore}  –  ${m.awayScore}" else clock(m.kickoff)
          Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.padding(horizontal = 8.dp)) {
            Text(score, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold, color = if (m.live) Brand.Blue else MaterialTheme.colorScheme.onSurface)
            m.phase?.takeIf { m.live }?.let { Text(it, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant) }
          }
          TeamSide(m.away!!.name, m.away!!.badge, Modifier.weight(1f), alignEnd = true)
        }
      } else {
        Text(m.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
        Text(clock(m.kickoff), style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
      }
    }
  }
}

@Composable
private fun TeamSide(name: String, badge: String?, modifier: Modifier, alignEnd: Boolean) {
  Row(modifier, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = if (alignEnd) Arrangement.End else Arrangement.Start) {
    if (!alignEnd) {
      AsyncImage(model = badge, contentDescription = null, modifier = Modifier.size(28.dp))
      Spacer(Modifier.width(8.dp))
    }
    Text(
      name,
      style = MaterialTheme.typography.bodyMedium,
      fontWeight = FontWeight.SemiBold,
      maxLines = 2,
      overflow = TextOverflow.Ellipsis,
      textAlign = if (alignEnd) TextAlign.End else TextAlign.Start,
      modifier = Modifier.weight(1f, fill = false),
    )
    if (alignEnd) {
      Spacer(Modifier.width(8.dp))
      AsyncImage(model = badge, contentDescription = null, modifier = Modifier.size(28.dp))
    }
  }
}
