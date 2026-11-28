package com.velmora.stepbyte.velmora.screens

import android.content.Context
import android.net.Uri
import android.widget.Toast
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowForward
import androidx.compose.material.icons.rounded.Bookmark
import androidx.compose.material.icons.rounded.CloudOff
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.toShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import coil3.compose.AsyncImage
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.ui.component.shimmer.ShimmerHost
import com.velmora.stepbyte.ui.component.shimmer.TextPlaceholder
import com.velmora.stepbyte.velmora.library.Progress
import com.velmora.stepbyte.velmora.library.VelmoraLibrary
import com.velmora.stepbyte.velmora.shell.Brand

// ── Routes ─────────────────────────────────────────────────────────────

/** Velmora mode's inner pages (the four tabs are in Screens). */
object VRoutes {
  /** A title page. The title travels in the route, so it survives the process being killed. */
  const val TITLE = "v/title?t={t}"
  const val COLLECTION = "v/collection/{id}"
  const val MY_LIST = "v/mylist"
  const val MATCH = "v/match?m={m}"

  fun match(m: com.velmora.stepbyte.engine.Match) =
    "v/match?m=" + Uri.encode(com.velmora.stepbyte.engine.Universe.json.encodeToString(com.velmora.stepbyte.engine.Match.serializer(), m))

  fun title(t: Title) = "v/title?t=" + Uri.encode(t.json.toString())

  fun collection(id: String) = "v/collection/" + Uri.encode(id)
}

fun NavController.openTitle(t: Title) = navigate(VRoutes.title(t))

fun titleKey(t: Title) = VelmoraLibrary.key(t)

fun kindLabel(kind: String): String? =
  when (kind) {
    "movie" -> "Movie"
    "series" -> "Series"
    "anime" -> "Anime"
    "asian_drama" -> "Drama"
    else -> null
  }

fun Throwable.plain(): String = message?.takeIf { it.isNotBlank() } ?: javaClass.simpleName

/** Until the player lands, say plainly where it is. */
fun playerSoon(context: Context) {
  Toast.makeText(context, "The player arrives in phase 6", Toast.LENGTH_SHORT).show()
}

/** Bottom insets only (mini player, bottom bar), for pages without the shell's top bar. */
@Composable
fun bottomInsets(): WindowInsets = com.velmora.stepbyte.LocalPlayerAwareWindowInsets.current.only(WindowInsetsSides.Bottom + WindowInsetsSides.Horizontal)

/** A press that gives a little, like Echo's cards. */
@Composable
fun Modifier.pressable(onClick: () -> Unit): Modifier {
  val source = remember { MutableInteractionSource() }
  val pressed by source.collectIsPressedAsState()
  val scale by animateFloatAsState(if (pressed) 0.94f else 1f, spring(dampingRatio = Spring.DampingRatioMediumBouncy, stiffness = Spring.StiffnessMediumLow), label = "press")
  return this.graphicsLayer {
      scaleX = scale
      scaleY = scale
    }
    .clickable(interactionSource = source, indication = androidx.compose.material3.ripple(), onClick = onClick)
}

// ── Posters ────────────────────────────────────────────────────────────

@Composable
fun PosterCard(title: Title, onClick: () -> Unit, modifier: Modifier = Modifier.width(118.dp), showMeta: Boolean = true) {
  val v = title.value
  val list by VelmoraLibrary.myList.collectAsState()
  val saved = remember(list) { list.any { titleKey(it) == titleKey(title) } }
  Column(modifier = modifier.clip(RoundedCornerShape(18.dp)).pressable(onClick)) {
    Box(
      Modifier.fillMaxWidth()
        .aspectRatio(2f / 3f)
        .clip(RoundedCornerShape(18.dp))
        .background(MaterialTheme.colorScheme.surfaceContainerHigh),
    ) {
      if (v.poster == null) {
        Text(
          v.name,
          fontFamily = Brand.Serif,
          fontSize = 20.sp,
          lineHeight = 22.sp,
          textAlign = TextAlign.Center,
          color = MaterialTheme.colorScheme.onSurfaceVariant,
          modifier = Modifier.align(Alignment.Center).padding(10.dp),
        )
      }
      AsyncImage(model = v.poster, contentDescription = v.name, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
      v.rating?.takeIf { it > 0f }?.let { r -> RatingPill(r, Modifier.align(Alignment.TopEnd).padding(6.dp)) }
      if (saved) {
        Box(
          Modifier.align(Alignment.TopStart).padding(6.dp).size(24.dp).clip(CircleShape).background(Brand.Blue),
          contentAlignment = Alignment.Center,
        ) {
          Icon(Icons.Rounded.Bookmark, contentDescription = "In My List", tint = Color.White, modifier = Modifier.size(14.dp))
        }
      }
    }
    if (showMeta) {
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
}

@Composable
fun RatingPill(rating: Float, modifier: Modifier = Modifier) {
  Surface(shape = RoundedCornerShape(8.dp), color = Color.Black.copy(alpha = 0.62f), modifier = modifier) {
    Row(Modifier.padding(horizontal = 5.dp, vertical = 2.dp), verticalAlignment = Alignment.CenterVertically) {
      Icon(Icons.Rounded.Star, contentDescription = null, tint = Brand.Amber, modifier = Modifier.size(11.dp))
      Spacer(Modifier.width(2.dp))
      Text("%.1f".format(rating), color = Color.White, style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.Bold)
    }
  }
}

/** A wide card with a progress bar, for Continue watching. */
@Composable
fun ContinueCard(p: Progress, onClick: () -> Unit) {
  val v = p.title.value
  Column(Modifier.width(240.dp).clip(RoundedCornerShape(18.dp)).pressable(onClick)) {
    Box(
      Modifier.fillMaxWidth().aspectRatio(16f / 9f).clip(RoundedCornerShape(18.dp)).background(MaterialTheme.colorScheme.surfaceContainerHigh),
    ) {
      AsyncImage(model = v.backdrop ?: v.poster, contentDescription = v.name, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
      Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0.5f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.7f))))
      Text(
        remaining(p),
        color = Color.White,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.Bold,
        modifier = Modifier.align(Alignment.BottomStart).padding(start = 10.dp, bottom = 12.dp),
      )
      LinearProgressIndicator(
        progress = { p.fraction },
        color = Brand.Blue,
        trackColor = Color.White.copy(alpha = 0.3f),
        drawStopIndicator = {},
        modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(4.dp),
      )
    }
    Text(
      v.name,
      style = MaterialTheme.typography.bodyMedium,
      fontWeight = FontWeight.Medium,
      maxLines = 1,
      overflow = TextOverflow.Ellipsis,
      modifier = Modifier.padding(top = 6.dp, start = 2.dp),
    )
    if (p.season != null && p.episode != null) {
      Text(
        "S${p.season} · E${p.episode}",
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.padding(start = 2.dp, bottom = 4.dp),
      )
    }
  }
}

private fun remaining(p: Progress): String {
  val left = ((p.duration - p.position) / 60).toInt().coerceAtLeast(1)
  return if (left >= 60) "${left / 60} h ${left % 60} min left" else "$left min left"
}

// ── Rows ───────────────────────────────────────────────────────────────

/** A section heading in Echo's style, with an optional "see all" arrow. */
@Composable
fun SectionHeader(title: String, icon: ImageVector? = null, label: String? = null, onSeeAll: (() -> Unit)? = null) {
  Row(
    Modifier.fillMaxWidth()
      .clickable(enabled = onSeeAll != null) { onSeeAll?.invoke() }
      .padding(start = 16.dp, end = 12.dp, top = 18.dp, bottom = 10.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    if (icon != null) {
      Box(
        Modifier.size(32.dp).clip(RoundedCornerShape(10.dp)).background(MaterialTheme.colorScheme.secondaryContainer),
        contentAlignment = Alignment.Center,
      ) {
        Icon(icon, contentDescription = null, tint = MaterialTheme.colorScheme.onSecondaryContainer, modifier = Modifier.size(18.dp))
      }
      Spacer(Modifier.width(12.dp))
    }
    Column(Modifier.weight(1f)) {
      label?.let { Text(it, style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant) }
      Text(title, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
    if (onSeeAll != null) {
      Icon(Icons.AutoMirrored.Rounded.ArrowForward, contentDescription = "See all", tint = MaterialTheme.colorScheme.onSurfaceVariant)
    }
  }
}

@Composable
fun PosterRow(items: List<Title>, onOpen: (Title) -> Unit, width: Dp = 118.dp) {
  LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
    items(items.distinctBy(::titleKey), key = ::titleKey) { t -> PosterCard(t, onClick = { onOpen(t) }, modifier = Modifier.width(width).animateItem()) }
  }
}

@Composable
fun PosterRowShimmer(count: Int = 4) {
  ShimmerHost {
    TextPlaceholder(modifier = Modifier.padding(horizontal = 16.dp, vertical = 14.dp).width(160.dp), height = 20.dp, shape = RoundedCornerShape(8.dp))
    Row(Modifier.padding(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
      repeat(count) {
        Box(Modifier.width(118.dp).aspectRatio(2f / 3f).clip(RoundedCornerShape(18.dp)).background(MaterialTheme.colorScheme.onSurface))
      }
    }
  }
}

/** A grid placeholder of poster shapes. */
@Composable
fun PosterGhost() {
  ShimmerHost {
    Box(Modifier.fillMaxWidth().aspectRatio(2f / 3f).clip(RoundedCornerShape(18.dp)).background(MaterialTheme.colorScheme.onSurface))
  }
}

// ── States ─────────────────────────────────────────────────────────────

@OptIn(androidx.compose.material3.ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun EmptyState(icon: ImageVector, title: String, text: String, action: String? = null, onAction: () -> Unit = {}) {
  Column(
    Modifier.fillMaxWidth().padding(horizontal = 32.dp, vertical = 48.dp),
    horizontalAlignment = Alignment.CenterHorizontally,
    verticalArrangement = Arrangement.spacedBy(10.dp),
  ) {
    Box(
      Modifier.size(88.dp)
        .clip(androidx.compose.material3.MaterialShapes.Cookie9Sided.toShape())
        .background(MaterialTheme.colorScheme.secondaryContainer),
      contentAlignment = Alignment.Center,
    ) {
      Icon(icon, contentDescription = null, tint = MaterialTheme.colorScheme.onSecondaryContainer, modifier = Modifier.size(40.dp))
    }
    Text(title, fontFamily = Brand.Serif, fontSize = 30.sp, textAlign = TextAlign.Center)
    Text(text, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center)
    if (action != null) {
      Spacer(Modifier.height(4.dp))
      FilledTonalButton(onClick = onAction) { Text(action) }
    }
  }
}

/** "You're offline", with the age of what's showing. */
@Composable
fun OfflineBanner(text: String, onRetry: (() -> Unit)? = null) {
  Surface(
    shape = RoundedCornerShape(20.dp),
    color = MaterialTheme.colorScheme.surfaceContainerHigh,
    modifier = Modifier.padding(horizontal = 16.dp, vertical = 6.dp).fillMaxWidth(),
  ) {
    Row(Modifier.padding(horizontal = 14.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
      Icon(Icons.Rounded.CloudOff, contentDescription = null, tint = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.size(20.dp))
      Spacer(Modifier.width(10.dp))
      Text(text, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.weight(1f))
      if (onRetry != null) {
        androidx.compose.material3.TextButton(onClick = onRetry) { Text("Retry") }
      }
    }
  }
}

/** A thin vertical divider for meta lines. */
@Composable
fun Dot() {
  Box(Modifier.padding(horizontal = 6.dp).size(3.dp).clip(CircleShape).background(MaterialTheme.colorScheme.onSurfaceVariant))
}

