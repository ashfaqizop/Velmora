package com.velmora.stepbyte.velmora.notices

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowForward
import androidx.compose.material.icons.rounded.Campaign
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Construction
import androidx.compose.material.icons.rounded.Shield
import androidx.compose.material.icons.rounded.WarningAmber
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialShapes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.toShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.core.net.toUri
import com.velmora.stepbyte.velmora.privacy.UsageStats
import com.velmora.stepbyte.velmora.shell.Brand
import com.velmora.stepbyte.velmora.social.VelmoraLinks

/**
 * The top of Home: the team's announcement when there is one, otherwise (once)
 * the note that anonymous usage counts are on and where to switch them off.
 */
@Composable
fun HomeNotices(onPrivacy: () -> Unit, modifier: Modifier = Modifier) {
  val notice by Notices.current.collectAsState()
  val statsOn by UsageStats.enabled.collectAsState()
  val seen by UsageStats.noticeSeen.collectAsState()
  // Keep the last one drawn while it animates away.
  var last by remember { mutableStateOf<Notice?>(null) }
  notice?.let { last = it }
  Column(modifier.animateContentSize()) {
    AnimatedVisibility(notice != null, enter = fadeIn() + expandVertically(), exit = fadeOut() + shrinkVertically()) {
      last?.let { NoticeCard(it) }
    }
    AnimatedVisibility(notice == null && statsOn && !seen, enter = fadeIn() + expandVertically(), exit = fadeOut() + shrinkVertically()) {
      Banner(
        accent = Brand.Violet,
        icon = Icons.Rounded.Shield,
        label = "Your privacy",
        title = "Velmora counts plays, never people",
        body = "Which titles and sites get played, sent as plain daily totals with no name or ID attached. See exactly what's sent, or switch it off, in Settings.",
        action = "Privacy settings",
        onAction = {
          UsageStats.noticeDone()
          onPrivacy()
        },
        onDismiss = { UsageStats.noticeDone() },
      )
    }
  }
}

@Composable
private fun NoticeCard(n: Notice) {
  val uri = LocalUriHandler.current
  val link = n.linkUrl?.takeIf { it.startsWith("https://") || it.startsWith("velmora://") }
  val (accent, icon, label) =
    when (n.kind) {
      "maintenance" -> Triple(MaterialTheme.colorScheme.error, Icons.Rounded.Construction, "Maintenance")
      "warning" -> Triple(Brand.Amber, Icons.Rounded.WarningAmber, "Heads up")
      else -> Triple(Brand.Blue, Icons.Rounded.Campaign, "From the Velmora team")
    }
  Banner(
    accent = accent,
    icon = icon,
    label = label,
    title = n.title,
    body = n.body,
    action = link?.let { n.linkLabel?.takeIf { l -> l.isNotBlank() } ?: "Learn more" },
    onAction = {
      val target = link?.let { VelmoraLinks.parse(it.toUri()) }
      if (target != null) VelmoraLinks.requests.value = target else link?.let { runCatching { uri.openUri(it) } }
    },
    onDismiss = if (n.maintenance) null else ({ Notices.dismiss(n) }),
  )
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun Banner(
  accent: Color,
  icon: ImageVector,
  label: String,
  title: String,
  body: String,
  action: String?,
  onAction: () -> Unit,
  onDismiss: (() -> Unit)?,
) {
  var open by remember { mutableStateOf(false) }
  val container = accent.copy(alpha = 0.13f).compositeOver(MaterialTheme.colorScheme.surfaceContainerLow)
  Surface(
    shape = RoundedCornerShape(28.dp),
    color = container,
    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
  ) {
    Row(Modifier.padding(start = 16.dp, top = 16.dp, bottom = 14.dp, end = 4.dp), verticalAlignment = Alignment.Top) {
      Box(
        Modifier.size(44.dp).clip(MaterialShapes.Cookie6Sided.toShape()).background(accent.copy(alpha = 0.22f)),
        contentAlignment = Alignment.Center,
      ) {
        Icon(icon, contentDescription = null, tint = accent, modifier = Modifier.size(24.dp))
      }
      Spacer(Modifier.width(14.dp))
      Column(Modifier.weight(1f).animateContentSize(), verticalArrangement = Arrangement.spacedBy(2.dp)) {
        Text(label.uppercase(), color = accent, style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.Bold)
        Text(title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
        if (body.isNotBlank()) {
          Text(
            body,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = if (open) Int.MAX_VALUE else 3,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.clickable { open = !open },
          )
        }
        if (action != null) {
          Spacer(Modifier.height(6.dp))
          FilledTonalButton(onClick = onAction) {
            Text(action)
            Spacer(Modifier.width(6.dp))
            Icon(Icons.AutoMirrored.Rounded.ArrowForward, contentDescription = null, modifier = Modifier.size(16.dp))
          }
        }
      }
      if (onDismiss != null) {
        IconButton(onClick = onDismiss) { Icon(Icons.Rounded.Close, contentDescription = "Dismiss", tint = MaterialTheme.colorScheme.onSurfaceVariant) }
      } else {
        Spacer(Modifier.width(12.dp))
      }
    }
  }
}
