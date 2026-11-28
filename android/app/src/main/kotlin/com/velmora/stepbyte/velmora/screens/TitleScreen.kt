package com.velmora.stepbyte.velmora.screens

import android.content.Intent
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.Bookmark
import androidx.compose.material.icons.rounded.BookmarkBorder
import androidx.compose.material.icons.rounded.Groups
import androidx.compose.material.icons.rounded.Image
import androidx.compose.material.icons.rounded.Movie
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Share
import androidx.compose.material.icons.rounded.SmartDisplay
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material.icons.rounded.Theaters
import androidx.compose.material.icons.rounded.Tv
import androidx.compose.material.icons.rounded.WifiOff
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.FilledTonalIconToggleButton
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialShapes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedIconButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.toShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import coil3.compose.AsyncImage
import com.velmora.stepbyte.engine.Episode
import com.velmora.stepbyte.engine.Person
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.engine.TitleDetail
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.ui.component.shimmer.ShimmerHost
import com.velmora.stepbyte.ui.component.shimmer.TextPlaceholder
import com.velmora.stepbyte.velmora.library.VelmoraLibrary
import com.velmora.stepbyte.velmora.player.FoundSite
import com.velmora.stepbyte.velmora.player.PlayerPrefs
import com.velmora.stepbyte.velmora.player.SourceFinder
import com.velmora.stepbyte.velmora.player.VideoSession
import com.velmora.stepbyte.velmora.shell.Brand
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch

private const val HERO_HEIGHT = 440

/** Title pages already opened this run, so going back and forth is instant. */
object DetailCache {
  private val map = object : LinkedHashMap<String, TitleDetail>(32, 0.75f, true) {
    override fun removeEldestEntry(eldest: MutableMap.MutableEntry<String, TitleDetail>?) = size > 40
  }

  @Synchronized fun get(key: String) = map[key]

  @Synchronized fun put(key: String, d: TitleDetail) {
    map[key] = d
  }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun TitleScreen(title: Title, navController: NavController) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  val key = titleKey(title)
  var detail by remember(key) { mutableStateOf(DetailCache.get(key)) }
  var error by remember(key) { mutableStateOf<String?>(null) }
  var attempt by remember { mutableIntStateOf(0) }

  LaunchedEffect(key, attempt) {
    if (detail != null) return@LaunchedEffect
    error = null
    try {
      val d = Universe.get(context).detail(title)
      DetailCache.put(key, d)
      detail = d
    } catch (e: CancellationException) {
      throw e
    } catch (e: Throwable) {
      error = e.plain()
    }
  }

  // The summary from the poster shows at once; the detail fills in under it.
  val shown = detail?.title ?: title
  val v = shown.value
  val info = detail?.info
  val listState = rememberLazyListState()
  val density = LocalDensity.current
  val heroPx = with(density) { HERO_HEIGHT.dp.toPx() }
  // 0 at the top, 1 once the hero has scrolled away: drives the parallax and the bar.
  val collapse by remember {
    derivedStateOf {
      if (listState.firstVisibleItemIndex > 0) 1f else (listState.firstVisibleItemScrollOffset / (heroPx * 0.7f)).coerceIn(0f, 1f)
    }
  }

  Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
    LazyColumn(state = listState, contentPadding = bottomInsets().asPaddingValues(), modifier = Modifier.fillMaxSize()) {
      item(key = "hero") {
        Hero(
          title = shown,
          detail = detail,
          scrollOffset = { if (listState.firstVisibleItemIndex == 0) listState.firstVisibleItemScrollOffset.toFloat() else heroPx },
        )
      }
      item(key = "actions") { Actions(shown, detail, onTrailer = info?.trailer, onOpenDownloads = { navController.navigate(com.velmora.stepbyte.velmora.shell.VelmoraRoutes.DOWNLOADS) }) }
      item(key = "sources") { SourcesAhead(title, detail) }
      item(key = "theatre") { TheatreChip(shown, onOpen = { navController.navigate(com.velmora.stepbyte.ui.screens.Screens.VelmoraTheatre.route) { launchSingleTop = true } }) }

      info?.extra?.tagline?.takeIf { it.isNotBlank() }?.let { tagline ->
        item(key = "tagline") {
          Text(
            "“$tagline”",
            fontFamily = Brand.Serif,
            fontStyle = FontStyle.Italic,
            fontSize = 22.sp,
            lineHeight = 26.sp,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(horizontal = 20.dp, vertical = 6.dp),
          )
        }
      }
      v.overview?.let { overview -> item(key = "story") { Story(overview) } }

      if (detail == null && error == null) {
        item(key = "loading") { DetailShimmer() }
      }
      error?.let { e ->
        item(key = "error") {
          OfflineBanner(
            text = if (looksOffline(e)) "Episodes, cast and more need a connection." else "Couldn't load the rest of this page: $e",
            onRetry = { attempt++ },
          )
        }
      }

      if (info != null && info.seasons.isNotEmpty() && v.episodic) {
        item(key = "episodes") { Episodes(shown, info.seasons, onOpenDownloads = { navController.navigate(com.velmora.stepbyte.velmora.shell.VelmoraRoutes.DOWNLOADS) }) }
      }

      val cast = info?.extra?.cast?.takeIf { it.isNotEmpty() }
      if (cast != null) {
        item(key = "cast") {
          SectionHeader(if (v.kind == "anime") "Characters" else "Cast", icon = Icons.Rounded.Groups)
          People(cast)
        }
      } else if (info != null && info.cast.isNotEmpty()) {
        item(key = "cast-names") {
          SectionHeader("Cast", icon = Icons.Rounded.Groups)
          People(info.cast.map { Person(it) })
        }
      }

      if (info != null && info.screenshots.isNotEmpty()) {
        item(key = "gallery") {
          SectionHeader("Gallery", icon = Icons.Rounded.Image)
          LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            items(info.screenshots.take(12)) { shot ->
              AsyncImage(
                model = shot,
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.width(260.dp).aspectRatio(16f / 9f).clip(RoundedCornerShape(18.dp)).background(MaterialTheme.colorScheme.surfaceContainerHigh),
              )
            }
          }
        }
      }

      if (info != null) item(key = "details") { Details(shown, detail!!) }

      val more = detail?.recommendations.orEmpty()
      if (more.isNotEmpty()) {
        item(key = "more") {
          SectionHeader("More like this", icon = if (v.episodic) Icons.Rounded.Tv else Icons.Rounded.Movie)
          PosterRow(more, onOpen = { navController.openTitle(it) })
          Spacer(Modifier.height(16.dp))
        }
      }
    }

    // The bar over the hero: floating buttons at the top, a solid bar with the name once scrolled.
    TopBar(name = v.name, collapse = collapse, onBack = { navController.navigateUp() }, onShare = { share(context, shown) })
  }
}

private fun looksOffline(e: String): Boolean {
  val s = e.lowercase()
  return listOf("dns", "resolve", "connect", "network", "timed out", "timeout", "unreachable", "offline").any { it in s }
}

private fun share(context: android.content.Context, t: Title) {
  val v = t.value
  val text = buildString {
    append(v.name)
    v.year?.let { append(" ($it)") }
    append(" on Velmora")
    append("\nhttps://getvelmora.dpdns.org/")
  }
  val send = Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT, text)
  context.startActivity(Intent.createChooser(send, null).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
}

// ── Hero ───────────────────────────────────────────────────────────────

@Composable
private fun Hero(title: Title, detail: TitleDetail?, scrollOffset: () -> Float) {
  val v = title.value
  val info = detail?.info
  val surface = MaterialTheme.colorScheme.surface
  Box(Modifier.fillMaxWidth().height(HERO_HEIGHT.dp)) {
    // Parallax: the art moves at half speed and dims as it leaves.
    AsyncImage(
      model = v.backdrop ?: v.poster,
      contentDescription = null,
      contentScale = ContentScale.Crop,
      modifier =
        Modifier.fillMaxSize().graphicsLayer {
          val y = scrollOffset()
          translationY = y * 0.5f
          alpha = (1f - y / size.height).coerceIn(0f, 1f)
          val zoom = 1.08f
          scaleX = zoom
          scaleY = zoom
        },
    )
    Box(
      Modifier.fillMaxSize()
        .background(
          Brush.verticalGradient(
            0f to Color.Black.copy(alpha = 0.45f),
            0.25f to Color.Transparent,
            0.55f to surface.copy(alpha = 0.35f),
            0.85f to surface.copy(alpha = 0.92f),
            1f to surface,
          ),
        ),
    )
    Row(Modifier.align(Alignment.BottomStart).padding(horizontal = 20.dp), verticalAlignment = Alignment.Bottom) {
      AsyncImage(
        model = v.poster,
        contentDescription = v.name,
        contentScale = ContentScale.Crop,
        modifier =
          Modifier.width(112.dp)
            .aspectRatio(2f / 3f)
            .shadow(18.dp, RoundedCornerShape(18.dp))
            .clip(RoundedCornerShape(18.dp))
            .background(MaterialTheme.colorScheme.surfaceContainerHigh),
      )
      Spacer(Modifier.width(16.dp))
      Column(Modifier.weight(1f).padding(bottom = 2.dp)) {
        kindLabel(v.kind)?.let { kind ->
          Text(
            kind.uppercase(),
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.primary,
            letterSpacing = 1.5.sp,
          )
        }
        Text(
          v.name,
          fontFamily = Brand.Serif,
          fontSize = if (v.name.length > 28) 30.sp else 38.sp,
          lineHeight = if (v.name.length > 28) 32.sp else 40.sp,
          maxLines = 3,
          overflow = TextOverflow.Ellipsis,
          color = MaterialTheme.colorScheme.onSurface,
        )
        Spacer(Modifier.height(6.dp))
        MetaLine(title, detail)
        if (info != null && info.genres.isNotEmpty()) {
          Spacer(Modifier.height(8.dp))
          FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            info.genres.take(3).forEach { g ->
              Surface(shape = RoundedCornerShape(50), color = MaterialTheme.colorScheme.secondaryContainer) {
                Text(
                  g,
                  style = MaterialTheme.typography.labelMedium,
                  color = MaterialTheme.colorScheme.onSecondaryContainer,
                  modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
                )
              }
            }
          }
        }
      }
    }
  }
}

@Composable
private fun MetaLine(title: Title, detail: TitleDetail?) {
  val v = title.value
  val extra = detail?.info?.extra
  val parts = buildList {
    v.year?.let { add(it.toString()) }
    detail?.info?.runtimeMinutes?.takeIf { it > 0 }?.let { m -> add(if (m >= 60) "${m / 60} h ${m % 60} min" else "$m min") }
    extra?.seasonCount?.takeIf { it > 0 && v.episodic }?.let { add(if (it == 1) "1 season" else "$it seasons") }
    extra?.certification?.takeIf { it.isNotBlank() }?.let { add(it) }
  }
  Row(verticalAlignment = Alignment.CenterVertically) {
    v.rating?.takeIf { it > 0f }?.let { r ->
      Icon(Icons.Rounded.Star, contentDescription = null, tint = Brand.Amber, modifier = Modifier.size(16.dp))
      Spacer(Modifier.width(3.dp))
      Text("%.1f".format(r), style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.Bold)
      if (parts.isNotEmpty()) Dot()
    }
    parts.forEachIndexed { i, p ->
      if (i > 0) Dot()
      Text(p, style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1)
    }
  }
}

// ── Actions ────────────────────────────────────────────────────────────

@Composable
private fun Actions(title: Title, detail: TitleDetail?, onTrailer: String?, onOpenDownloads: () -> Unit) {
  val context = LocalContext.current
  val haptics = LocalHapticFeedback.current
  val uriHandler = LocalUriHandler.current
  val list by VelmoraLibrary.myList.collectAsState()
  val saved = remember(list) { list.any { titleKey(it) == titleKey(title) } }
  val progress by VelmoraLibrary.progress.collectAsState()
  val resume = remember(progress) { progress.firstOrNull { titleKey(it.title) == titleKey(title) }?.takeIf { it.resumable } }

  Row(
    Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 14.dp),
    horizontalArrangement = Arrangement.spacedBy(10.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    Button(
      onClick = { VideoSession.play(context, title) },
      modifier = Modifier.weight(1f).height(56.dp),
      shape = RoundedCornerShape(18.dp),
      colors = ButtonDefaults.buttonColors(containerColor = Brand.Blue, contentColor = Color.White),
    ) {
      Icon(Icons.Rounded.PlayArrow, contentDescription = null)
      Spacer(Modifier.width(6.dp))
      val label =
        when {
          resume != null && resume.season != null && resume.episode != null -> "Resume S${resume.season} E${resume.episode}"
          resume != null -> "Resume"
          title.value.episodic -> "Play S1 E1"
          else -> "Play"
        }
      Text(label, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
    FilledTonalIconToggleButton(
      checked = saved,
      onCheckedChange = {
        haptics.performHapticFeedback(HapticFeedbackType.LongPress)
        val added = VelmoraLibrary.toggle(title)
        android.widget.Toast.makeText(context, if (added) "Added to My List" else "Removed from My List", android.widget.Toast.LENGTH_SHORT).show()
      },
      modifier = Modifier.size(56.dp),
      shape = RoundedCornerShape(18.dp),
    ) {
      // The bookmark pops when it fills.
      AnimatedContent(
        targetState = saved,
        transitionSpec = {
          (scaleIn(spring(dampingRatio = Spring.DampingRatioHighBouncy, stiffness = Spring.StiffnessMedium), initialScale = 0.4f) + fadeIn()) togetherWith
            (scaleOut(tween(120), targetScale = 0.6f) + fadeOut(tween(120)))
        },
        label = "saved",
      ) { on ->
        Icon(if (on) Icons.Rounded.Bookmark else Icons.Rounded.BookmarkBorder, contentDescription = if (on) "Remove from My List" else "Add to My List")
      }
    }
    // Download: the movie, or the episode Play would play (the sheet offers the season).
    val seasons = detail?.info?.seasons.orEmpty()
    val target = remember(title, seasons, progress) { VideoSession.targetFor(title, seasons) }
    com.velmora.stepbyte.velmora.downloads.DownloadButton(title, target, seasons, onOpenDownloads = onOpenDownloads)
    if (onTrailer != null) {
      OutlinedIconButton(onClick = { uriHandler.openUri(onTrailer) }, modifier = Modifier.size(56.dp), shape = RoundedCornerShape(18.dp)) {
        Icon(Icons.Rounded.SmartDisplay, contentDescription = "Trailer")
      }
    }
  }
}

// ── Sources, found ahead ───────────────────────────────────────────────

/**
 * Sources for what Play would play are looked up as soon as the page has the
 * title's ids, like desktop, so Play starts at once. One line says how it's
 * going; tapping it plays.
 */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun SourcesAhead(title: Title, detail: TitleDetail?) {
  val context = LocalContext.current
  if (detail == null) return
  val target = remember(detail) { VideoSession.targetFor(title, detail.info.seasons) }
  val dub = title.value.kind == "anime" && PlayerPrefs.dub(context)
  val finding = remember(target, detail) { SourceFinder.find(context, SourceFinder.lookupTitle(title, detail), target, dub) }
  val sites by finding.sites.collectAsState()
  val sources by finding.sources.collectAsState()
  val report by finding.report.collectAsState()
  val best = sources.mapNotNull { it.value.tags.resolution }.maxOrNull()
  val what = (target as? com.velmora.stepbyte.engine.Target.Episode)?.let { "S${it.season} E${it.episode}" }
  Surface(
    onClick = { VideoSession.play(context, title) },
    shape = RoundedCornerShape(16.dp),
    color = MaterialTheme.colorScheme.surfaceContainer,
    modifier = Modifier.padding(horizontal = 20.dp).padding(bottom = 6.dp).fillMaxWidth(),
  ) {
    Row(Modifier.padding(horizontal = 14.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
      Box(Modifier.size(24.dp), contentAlignment = Alignment.Center) {
        if (report == null) {
          LoadingIndicator(modifier = Modifier.size(24.dp))
        } else {
          Icon(
            if (sources.isEmpty()) Icons.Rounded.WifiOff else Icons.Rounded.PlayArrow,
            contentDescription = null,
            tint = if (sources.isEmpty()) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.primary,
          )
        }
      }
      Spacer(Modifier.width(12.dp))
      val answered = sites.count { it.state != FoundSite.State.Looking }
      Text(
        when {
          report == null && sites.isEmpty() -> "Finding sources…"
          report == null -> "Finding sources · $answered of ${sites.size} sites · ${sources.size} found"
          sources.isEmpty() -> "No source has ${what ?: "this"} yet"
          else -> listOfNotNull("${sources.size} sources ready", best?.let { if (it >= 2160) "up to 4K" else "up to ${it}p" }, what).joinToString(" · ")
        },
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
      )
    }
  }
}

/** "In Theatre tonight at 9:30 PM", when this film is screening. */
@Composable
private fun TheatreChip(title: Title, onOpen: () -> Unit) {
  val lineup by com.velmora.stepbyte.velmora.theatre.Theatre.lineup.collectAsState()
  val k = title.value.origin.id.takeIf { title.value.origin.provider == "tmdb" && it.startsWith("movie/") } ?: return
  val s = remember(lineup, k) { com.velmora.stepbyte.velmora.theatre.Theatre.forTitle(k) } ?: return
  val phase = com.velmora.stepbyte.velmora.theatre.Theatre.phase(s)
  val text =
    when (phase) {
      com.velmora.stepbyte.velmora.theatre.ShowPhase.Live -> "Screening in Theatre now · join live"
      com.velmora.stepbyte.velmora.theatre.ShowPhase.Lobby -> "In Theatre at ${com.velmora.stepbyte.velmora.theatre.localTime(s.startMs)} · the lobby is open"
      else -> "In Theatre at ${com.velmora.stepbyte.velmora.theatre.localTime(s.startMs)}" + if (s.fixed) " · book a seat" else ""
    }
  Surface(onClick = onOpen, shape = RoundedCornerShape(16.dp), color = MaterialTheme.colorScheme.tertiaryContainer, modifier = Modifier.padding(horizontal = 20.dp).padding(bottom = 6.dp).fillMaxWidth()) {
    Row(Modifier.padding(horizontal = 14.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
      Icon(androidx.compose.material.icons.Icons.Rounded.Theaters, contentDescription = null, tint = MaterialTheme.colorScheme.onTertiaryContainer)
      Spacer(Modifier.width(12.dp))
      Text(text, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onTertiaryContainer)
    }
  }
}

// ── Story ──────────────────────────────────────────────────────────────

@Composable
private fun Story(overview: String) {
  var open by rememberSaveable { mutableStateOf(false) }
  Column(
    Modifier.fillMaxWidth()
      .padding(horizontal = 20.dp)
      .clip(RoundedCornerShape(12.dp))
      .clickable { open = !open }
      .animateContentSize(spring(stiffness = Spring.StiffnessMediumLow))
      .padding(vertical = 6.dp),
  ) {
    Text(
      overview,
      style = MaterialTheme.typography.bodyLarge,
      color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.88f),
      maxLines = if (open) Int.MAX_VALUE else 4,
      overflow = TextOverflow.Ellipsis,
    )
    if (!open && overview.length > 220) {
      Text("More", style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.primary, modifier = Modifier.padding(top = 4.dp))
    }
  }
}

// ── Episodes ───────────────────────────────────────────────────────────

@Composable
private fun Episodes(title: Title, seasons: List<com.velmora.stepbyte.engine.Season>, onOpenDownloads: () -> Unit) {
  val context = LocalContext.current
  val playable = seasons.filter { it.episodes.isNotEmpty() }.ifEmpty { seasons }
  // Specials (season 0) go last; the first real season opens.
  val ordered = playable.sortedBy { if (it.number == 0) Int.MAX_VALUE else it.number }
  val progress by VelmoraLibrary.progress.collectAsState()
  val last = remember(progress) { progress.firstOrNull { titleKey(it.title) == titleKey(title) } }
  var selected by rememberSaveable { mutableIntStateOf(last?.season ?: ordered.firstOrNull()?.number ?: 1) }
  val season = ordered.firstOrNull { it.number == selected } ?: ordered.firstOrNull() ?: return

  Column {
    SectionHeader("Episodes", icon = Icons.Rounded.Tv, label = if (ordered.size > 1) "${ordered.size} seasons" else null)
    if (ordered.size > 1) {
      LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(ordered, key = { it.number }) { s ->
          FilterChip(
            selected = s.number == season.number,
            onClick = { selected = s.number },
            label = { Text(s.name?.takeIf { it.isNotBlank() } ?: if (s.number == 0) "Specials" else "Season ${s.number}") },
            shape = RoundedCornerShape(12.dp),
          )
        }
      }
      Spacer(Modifier.height(8.dp))
    }
    AnimatedContent(
      targetState = season,
      transitionSpec = { (fadeIn(tween(220)) + slideInVertically(tween(260)) { it / 12 }) togetherWith fadeOut(tween(120)) },
      contentKey = { it.number },
      label = "season",
    ) { s ->
      Column(Modifier.fillMaxWidth()) {
        if (s.episodes.isEmpty()) {
          Text(
            "No episode list for this season yet.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(horizontal = 20.dp, vertical = 12.dp),
          )
        }
        s.episodes.forEach { ep ->
          val watching = last?.takeIf { it.season == ep.season && it.episode == ep.number }
          EpisodeRow(ep, title.value.backdrop, watching?.fraction, onClick = { VideoSession.play(context, title, com.velmora.stepbyte.engine.Target.Episode(ep.season, ep.number)) }) {
            com.velmora.stepbyte.velmora.downloads.EpisodeDownloadButton(title, ep.season, ep.number, ep.title, onOpenDownloads)
          }
        }
      }
    }
  }
}

@Composable
private fun EpisodeRow(ep: Episode, fallbackArt: String?, fraction: Float?, onClick: () -> Unit, trailing: @Composable () -> Unit = {}) {
  Row(
    Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 16.dp, vertical = 8.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    Box(Modifier.width(148.dp).aspectRatio(16f / 9f).clip(RoundedCornerShape(14.dp)).background(MaterialTheme.colorScheme.surfaceContainerHigh)) {
      AsyncImage(model = ep.thumbnail ?: fallbackArt, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize().graphicsLayer { alpha = if (ep.thumbnail == null) 0.45f else 1f })
      Box(
        Modifier.align(Alignment.Center).size(34.dp).clip(CircleShape).background(Color.Black.copy(alpha = 0.45f)),
        contentAlignment = Alignment.Center,
      ) {
        Icon(Icons.Rounded.PlayArrow, contentDescription = null, tint = Color.White, modifier = Modifier.size(22.dp))
      }
      if (fraction != null) {
        Box(Modifier.align(Alignment.BottomStart).fillMaxWidth(fraction).height(3.dp).background(Brand.Blue))
      }
    }
    Spacer(Modifier.width(14.dp))
    Column(Modifier.weight(1f)) {
      Text(
        "${ep.number}. ${ep.title?.takeIf { it.isNotBlank() } ?: "Episode ${ep.number}"}",
        style = MaterialTheme.typography.titleSmall,
        fontWeight = FontWeight.SemiBold,
        maxLines = 2,
        overflow = TextOverflow.Ellipsis,
      )
      ep.airDate?.let { Text(prettyDate(it), style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant) }
      ep.overview?.takeIf { it.isNotBlank() }?.let {
        Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(top = 2.dp))
      }
    }
    trailing()
  }
}

fun prettyDate(iso: String): String =
  runCatching {
      val d = java.time.LocalDate.parse(iso.take(10))
      d.format(java.time.format.DateTimeFormatter.ofPattern("d MMM yyyy"))
    }
    .getOrDefault(iso)

// ── People ─────────────────────────────────────────────────────────────

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun People(people: List<Person>) {
  val shape = MaterialShapes.Cookie9Sided.toShape()
  LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(14.dp)) {
    items(people.take(20)) { p ->
      Column(Modifier.width(84.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Box(Modifier.size(76.dp).clip(shape).background(MaterialTheme.colorScheme.secondaryContainer), contentAlignment = Alignment.Center) {
          Text(
            p.name.split(' ').mapNotNull { it.firstOrNull()?.uppercaseChar() }.take(2).joinToString(""),
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onSecondaryContainer,
          )
          AsyncImage(model = p.photo, contentDescription = p.name, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
        }
        Text(p.name, style = MaterialTheme.typography.labelLarge, maxLines = 2, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(top = 6.dp), textAlign = androidx.compose.ui.text.style.TextAlign.Center)
        p.role?.takeIf { it.isNotBlank() }?.let {
          Text(it, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
        }
      }
    }
  }
}

// ── Details ────────────────────────────────────────────────────────────

@Composable
private fun Details(title: Title, detail: TitleDetail) {
  val x = detail.info.extra
  val rows = buildList {
    x.status?.let { add("Status" to it) }
    x.releaseDate?.let { add((if (title.value.episodic) "First aired" else "Released") to prettyDate(it)) }
    x.episodeCount?.takeIf { it > 0 }?.let { add("Episodes" to it.toString()) }
    x.format?.let { add("Format" to listOfNotNull(it, x.seasonLabel).joinToString(" · ")) }
    detail.info.directors.takeIf { it.isNotEmpty() }?.let { add((if (it.size > 1) "Directors" else "Director") to it.joinToString(", ")) }
    x.studios.takeIf { it.isNotEmpty() }?.let { add("Studio" to it.take(3).joinToString(", ")) }
    x.countries.takeIf { it.isNotEmpty() }?.let { add("Country" to it.take(3).joinToString(", ")) }
    x.languages.takeIf { it.isNotEmpty() }?.let { add("Languages" to it.take(4).joinToString(", ")) }
    if (title.value.rating != null && x.voteCount != null) {
      add("Rating" to "%.1f from %,d votes%s".format(title.value.rating, x.voteCount, x.ratingSource?.let { " on $it" } ?: ""))
    }
  }
  if (rows.isEmpty() && x.keywords.isEmpty()) return
  Surface(
    shape = RoundedCornerShape(24.dp),
    color = MaterialTheme.colorScheme.surfaceContainer,
    modifier = Modifier.padding(horizontal = 16.dp, vertical = 18.dp).fillMaxWidth(),
  ) {
    Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
      Text("Details", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
      rows.forEach { (k, value) ->
        Row {
          Text(k, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.width(104.dp))
          Text(value, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
        }
      }
      if (x.keywords.isNotEmpty()) {
        FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp), modifier = Modifier.padding(top = 4.dp)) {
          x.keywords.take(12).forEach { k ->
            Surface(shape = RoundedCornerShape(8.dp), color = MaterialTheme.colorScheme.surfaceContainerHighest) {
              Text(k, style = MaterialTheme.typography.labelMedium, modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp))
            }
          }
        }
      }
    }
  }
}

// ── Loading ────────────────────────────────────────────────────────────

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun DetailShimmer() {
  Column {
    Box(Modifier.fillMaxWidth().padding(vertical = 18.dp), contentAlignment = Alignment.Center) { LoadingIndicator() }
    ShimmerHost {
      repeat(3) {
        Row(Modifier.padding(horizontal = 16.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
          Box(Modifier.width(148.dp).aspectRatio(16f / 9f).clip(RoundedCornerShape(14.dp)).background(MaterialTheme.colorScheme.onSurface))
          Spacer(Modifier.width(14.dp))
          Column {
            TextPlaceholder(modifier = Modifier.width(150.dp), height = 16.dp)
            Spacer(Modifier.height(6.dp))
            TextPlaceholder(modifier = Modifier.width(90.dp), height = 12.dp)
          }
        }
      }
    }
  }
}

// ── Top bar ────────────────────────────────────────────────────────────

@Composable
private fun TopBar(name: String, collapse: Float, onBack: () -> Unit, onShare: () -> Unit) {
  val bar = MaterialTheme.colorScheme.surfaceContainer
  Box(
    Modifier.fillMaxWidth()
      .background(bar.copy(alpha = collapse))
      .windowInsetsPadding(WindowInsets.statusBars)
      .height(64.dp)
      .padding(horizontal = 8.dp),
  ) {
    val chip = Color.Black.copy(alpha = 0.35f * (1f - collapse))
    val tint = androidx.compose.ui.graphics.lerp(Color.White, MaterialTheme.colorScheme.onSurface, collapse)
    FilledIconButton(
      onClick = onBack,
      colors = IconButtonDefaults.filledIconButtonColors(containerColor = chip, contentColor = tint),
      modifier = Modifier.align(Alignment.CenterStart),
    ) {
      Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = "Back")
    }
    Text(
      name,
      style = MaterialTheme.typography.titleLarge,
      fontFamily = Brand.Serif,
      maxLines = 1,
      overflow = TextOverflow.Ellipsis,
      modifier =
        Modifier.align(Alignment.Center).padding(horizontal = 56.dp).graphicsLayer {
          alpha = ((collapse - 0.6f) / 0.4f).coerceIn(0f, 1f)
          translationY = (1f - alpha) * 12.dp.toPx()
        },
    )
    FilledIconButton(
      onClick = onShare,
      colors = IconButtonDefaults.filledIconButtonColors(containerColor = chip, contentColor = tint),
      modifier = Modifier.align(Alignment.CenterEnd),
    ) {
      Icon(Icons.Rounded.Share, contentDescription = "Share")
    }
  }
}
