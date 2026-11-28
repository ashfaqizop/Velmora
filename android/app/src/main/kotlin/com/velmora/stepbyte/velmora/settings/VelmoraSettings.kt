package com.velmora.stepbyte.velmora.settings

import android.content.Context
import android.content.Intent
import android.os.Build
import android.provider.Settings
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.KeyboardArrowRight
import androidx.compose.material.icons.automirrored.rounded.Logout
import androidx.compose.material.icons.rounded.Bookmarks
import androidx.compose.material.icons.rounded.ClosedCaption
import androidx.compose.material.icons.rounded.DataSaverOn
import androidx.compose.material.icons.rounded.Download
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.Forum
import androidx.compose.material.icons.rounded.HighQuality
import androidx.compose.material.icons.rounded.History
import androidx.compose.material.icons.rounded.Insights
import androidx.compose.material.icons.automirrored.rounded.Login
import androidx.compose.material.icons.rounded.MusicNote
import androidx.compose.material.icons.rounded.Notifications
import androidx.compose.material.icons.rounded.NotificationsActive
import androidx.compose.material.icons.rounded.Public
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.SignalCellularAlt
import androidx.compose.material.icons.rounded.SkipNext
import androidx.compose.material.icons.rounded.Theaters
import androidx.compose.material.icons.rounded.Translate
import androidx.compose.material.icons.rounded.Verified
import androidx.compose.material.icons.rounded.Visibility
import androidx.compose.material.icons.rounded.Wifi
import androidx.compose.material.icons.rounded.Headphones
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Slider
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import com.velmora.stepbyte.LocalPlayerAwareWindowInsets
import com.velmora.stepbyte.velmora.account.AccountAvatar
import com.velmora.stepbyte.velmora.account.AccountState
import com.velmora.stepbyte.velmora.account.VelmoraAccount
import com.velmora.stepbyte.velmora.downloads.DownloadQuality
import com.velmora.stepbyte.velmora.downloads.VelmoraDownloads
import com.velmora.stepbyte.velmora.player.LiveTv
import com.velmora.stepbyte.velmora.player.PlayerPrefs
import com.velmora.stepbyte.velmora.privacy.UsageStats
import com.velmora.stepbyte.velmora.shell.Brand
import com.velmora.stepbyte.velmora.shell.VelmoraRoutes
import com.velmora.stepbyte.velmora.social.ActivityPoster
import com.velmora.stepbyte.velmora.social.SocialLinks
import com.velmora.stepbyte.velmora.theatre.Probes
import com.velmora.stepbyte.velmora.theatre.TheatreAlerts
import com.velmora.stepbyte.ui.component.Material3SettingsGroup
import com.velmora.stepbyte.ui.component.Material3SettingsItem
import java.util.Locale
import kotlinx.coroutines.launch

/** Velmora's own settings pages, under Settings > Velmora. */
object VelmoraSettingsRoutes {
  const val ROUTE = "settings/velmora/{section}"
  const val TRANSFER = "settings/velmora_transfer"

  fun section(name: String) = "settings/velmora/$name"
}

/** One row of the Settings hub's Velmora group. */
data class VelmoraSection(val id: String, val title: String, val description: String, val icon: ImageVector)

val VELMORA_SECTIONS =
  listOf(
    VelmoraSection("account", "Account and Social", "Your Velmora account, profile and sign-in", Icons.Rounded.Verified),
    VelmoraSection("watching", "Watching", "Mobile data, next episode, subtitles, Live TV", Icons.Rounded.Theaters),
    VelmoraSection("downloads", "Downloads", "Wi-Fi only, quality and space", Icons.Rounded.Download),
    VelmoraSection("theatre", "Theatre", "Reminders and source checks", Icons.Rounded.NotificationsActive),
    VelmoraSection("privacy", "Privacy", "Anonymous counts and what Social shows", Icons.Rounded.Insights),
  )

@Composable
fun VelmoraSettingsScreen(section: String, navController: NavController) {
  val info = VELMORA_SECTIONS.firstOrNull { it.id == section }
  val snackbar = remember { SnackbarHostState() }
  Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
      PageHead(info?.title ?: "Velmora", info?.description, navController)
      Column(Modifier.padding(horizontal = 16.dp), verticalArrangement = Arrangement.spacedBy(20.dp)) {
        when (section) {
          "account" -> AccountPage(navController)
          "watching" -> WatchingPage()
          "downloads" -> DownloadsPage(navController)
          "theatre" -> TheatrePage()
          "privacy" -> PrivacyPage(navController, snackbar)
        }
      }
      Spacer(Modifier.windowInsetsPadding(LocalPlayerAwareWindowInsets.current.only(WindowInsetsSides.Bottom)))
      Spacer(Modifier.height(32.dp))
    }
    SnackbarHost(snackbar, Modifier.align(Alignment.BottomCenter).padding(16.dp))
  }
}

/** Back, then the page name in Velmora's serif. */
@Composable
internal fun PageHead(title: String, subtitle: String?, navController: NavController) {
  Column(Modifier.fillMaxWidth().windowInsetsPadding(WindowInsets.statusBars)) {
    IconButton(onClick = { navController.navigateUp() }, modifier = Modifier.padding(4.dp)) {
      Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = "Back")
    }
    Column(Modifier.padding(start = 24.dp, end = 24.dp, bottom = 20.dp)) {
      Text(title, fontFamily = Brand.Serif, fontSize = 42.sp, lineHeight = 46.sp)
      subtitle?.let { Text(it, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant) }
    }
  }
}

@Composable
private fun icon(v: ImageVector) = rememberVectorPainter(v)

@Composable
private fun Toggle(on: Boolean, enabled: Boolean = true, onChange: (Boolean) -> Unit) = Switch(checked = on, onCheckedChange = onChange, enabled = enabled)

// ── Account and Social ──

@Composable
private fun AccountPage(navController: NavController) {
  val context = LocalContext.current
  val account by VelmoraAccount.state.collectAsState()
  val signedIn = account as? AccountState.SignedIn
  var confirm by remember { mutableStateOf(false) }

  if (signedIn != null) {
    val p = signedIn.profile
    Surface(shape = RoundedCornerShape(28.dp), color = MaterialTheme.colorScheme.surfaceContainer, modifier = Modifier.fillMaxWidth()) {
      Row(Modifier.padding(20.dp), verticalAlignment = Alignment.CenterVertically) {
        AccountAvatar(p, size = 64.dp)
        Spacer(Modifier.width(16.dp))
        Column(Modifier.weight(1f)) {
          Text(p.displayName, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.SemiBold)
          Text("@" + p.username, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
          Spacer(Modifier.height(6.dp))
          Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            if (signedIn.max) Chip("Velmora Max", Brand.Amber)
            if (signedIn.offline) Chip("Offline", MaterialTheme.colorScheme.outline) else Chip("Synced", Brand.Blue)
          }
        }
      }
    }
    Material3SettingsGroup(
      title = "Velmora Social",
      items =
        listOf(
          Material3SettingsItem(
            icon = icon(Icons.Rounded.Forum),
            title = { Text("Your profile") },
            description = { Text("What you watch, your lists, reviews and Theatre stamps") },
            trailingContent = { Icon(Icons.AutoMirrored.Rounded.KeyboardArrowRight, contentDescription = null) },
            onClick = { navController.navigate(SocialLinks.route("/@" + p.username)) },
          ),
          Material3SettingsItem(
            icon = icon(Icons.Rounded.Edit),
            title = { Text("Edit profile") },
            description = { Text("Name, picture, bio and favourites") },
            trailingContent = { Icon(Icons.AutoMirrored.Rounded.KeyboardArrowRight, contentDescription = null) },
            onClick = { navController.navigate(SocialLinks.route("/settings")) },
          ),
          Material3SettingsItem(
            icon = icon(Icons.Rounded.Bookmarks),
            title = { Text("My List") },
            description = { Text("Synced with desktop") },
            trailingContent = { Icon(Icons.AutoMirrored.Rounded.KeyboardArrowRight, contentDescription = null) },
            onClick = { navController.navigate(com.velmora.stepbyte.velmora.screens.VRoutes.MY_LIST) },
          ),
        ),
    )
    Material3SettingsGroup(
      title = "This phone",
      items =
        listOf(
          Material3SettingsItem(
            icon = icon(Icons.AutoMirrored.Rounded.Logout),
            title = { Text("Sign out") },
            description = { Text("Downloads stay. My List and progress come back when you sign in.") },
            onClick = { confirm = true },
          ),
        ),
    )
  } else {
    Surface(shape = RoundedCornerShape(28.dp), color = Brand.Blue.copy(alpha = 0.12f), modifier = Modifier.fillMaxWidth()) {
      Column(Modifier.padding(24.dp)) {
        Text("You're not signed in", fontFamily = Brand.Serif, fontSize = 28.sp)
        Text(
          "Sign in to sync My List and watch progress with desktop, book Theatre seats and use Velmora Social.",
          style = MaterialTheme.typography.bodyMedium,
          color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(14.dp))
        androidx.compose.material3.Button(onClick = { VelmoraAccount.beginSignIn(context) }) {
          Icon(Icons.AutoMirrored.Rounded.Login, contentDescription = null, modifier = Modifier.size(18.dp))
          Spacer(Modifier.width(8.dp))
          Text("Sign in")
        }
      }
    }
  }

  if (confirm) {
    AlertDialog(
      onDismissRequest = { confirm = false },
      title = { Text("Sign out of Velmora?") },
      text = { Text("Velmora Social signs out on this phone too. Your downloads stay.") },
      confirmButton = {
        TextButton(onClick = {
          confirm = false
          VelmoraAccount.signOut()
        }) { Text("Sign out") }
      },
      dismissButton = { TextButton(onClick = { confirm = false }) { Text("Cancel") } },
    )
  }
}

@Composable
private fun Chip(text: String, color: androidx.compose.ui.graphics.Color) {
  Surface(shape = RoundedCornerShape(50), color = color.copy(alpha = 0.16f)) {
    Text(text, color = color, style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.SemiBold, modifier = Modifier.padding(horizontal = 10.dp, vertical = 3.dp))
  }
}

// ── Watching ──

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun WatchingPage() {
  val context = LocalContext.current
  var mobile by remember { mutableStateOf(PlayerPrefs.mobileData(context)) }
  var saveData by remember { mutableStateOf(PlayerPrefs.saveData(context)) }
  var autoNext by remember { mutableStateOf(PlayerPrefs.autoNext(context)) }
  var dub by remember { mutableStateOf(PlayerPrefs.dub(context)) }
  var background by remember { mutableStateOf(PlayerPrefs.background(context)) }
  var subScale by remember { mutableFloatStateOf(PlayerPrefs.subtitleScale(context)) }
  var region by remember { mutableStateOf(LiveTv.region(context)) }
  var pickRegion by remember { mutableStateOf(false) }

  Material3SettingsGroup(
    title = "On mobile data",
    items =
      PlayerPrefs.MobileData.entries.map { m ->
        Material3SettingsItem(
          icon = icon(Icons.Rounded.SignalCellularAlt),
          title = { Text(m.label) },
          description = { Text(m.detail) },
          trailingContent = { RadioButton(selected = mobile == m, onClick = null) },
          onClick = {
            mobile = m
            PlayerPrefs.setMobileData(context, m)
          },
        )
      } +
        Material3SettingsItem(
          icon = icon(Icons.Rounded.DataSaverOn),
          title = { Text("Save data") },
          description = { Text("Pick 720p or lower on mobile data") },
          trailingContent = {
            Toggle(saveData) {
              saveData = it
              PlayerPrefs.setSaveData(context, it)
            }
          },
          onClick = {
            saveData = !saveData
            PlayerPrefs.setSaveData(context, saveData)
          },
        ),
  )
  Material3SettingsGroup(
    title = "Playback",
    items =
      listOf(
        Material3SettingsItem(
          icon = icon(Icons.Rounded.SkipNext),
          title = { Text("Play the next episode") },
          description = { Text("Starts by itself when an episode ends") },
          trailingContent = {
            Toggle(autoNext) {
              autoNext = it
              PlayerPrefs.setAutoNext(context, it)
            }
          },
          onClick = {
            autoNext = !autoNext
            PlayerPrefs.setAutoNext(context, autoNext)
          },
        ),
        Material3SettingsItem(
          icon = icon(Icons.Rounded.Translate),
          title = { Text("Anime: dubbed first") },
          description = { Text("English audio when a dub exists, otherwise subtitles") },
          trailingContent = {
            Toggle(dub) {
              dub = it
              PlayerPrefs.setDub(context, it)
            }
          },
          onClick = {
            dub = !dub
            PlayerPrefs.setDub(context, dub)
          },
        ),
        Material3SettingsItem(
          icon = icon(Icons.Rounded.Headphones),
          title = { Text("Keep playing in the background") },
          description = { Text("Sound goes on when you leave the app and picture-in-picture is off") },
          trailingContent = {
            Toggle(background) {
              background = it
              PlayerPrefs.setBackground(context, it)
            }
          },
          onClick = {
            background = !background
            PlayerPrefs.setBackground(context, background)
          },
        ),
      ),
  )
  // Subtitle size, with a live sample.
  Column {
    Text("Subtitles", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.primary, modifier = Modifier.padding(start = 16.dp, bottom = 8.dp))
    Surface(shape = RoundedCornerShape(24.dp), color = MaterialTheme.colorScheme.surfaceContainer, modifier = Modifier.fillMaxWidth()) {
      Column(Modifier.padding(20.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
          Icon(Icons.Rounded.ClosedCaption, contentDescription = null, tint = MaterialTheme.colorScheme.onSurfaceVariant)
          Spacer(Modifier.width(12.dp))
          Text("Size", style = MaterialTheme.typography.titleMedium, modifier = Modifier.weight(1f))
          Text("${(subScale * 100).toInt()}%", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Box(
          Modifier.fillMaxWidth().padding(vertical = 12.dp).heightIn(min = 84.dp).background(Brand.Night, RoundedCornerShape(16.dp)),
          contentAlignment = Alignment.BottomCenter,
        ) {
          Text(
            "I'll be right here, waiting.",
            color = androidx.compose.ui.graphics.Color.White,
            fontSize = (16 * subScale).sp,
            modifier = Modifier.padding(14.dp).background(androidx.compose.ui.graphics.Color.Black.copy(alpha = 0.55f), RoundedCornerShape(6.dp)).padding(horizontal = 8.dp, vertical = 2.dp),
          )
        }
        Slider(
          value = subScale,
          onValueChange = { subScale = (it * 20).toInt() / 20f },
          onValueChangeFinished = { PlayerPrefs.setSubtitleScale(context, subScale) },
          valueRange = 0.7f..1.8f,
        )
      }
    }
  }
  Material3SettingsGroup(
    title = "Live TV and Sports",
    items =
      listOf(
        Material3SettingsItem(
          icon = icon(Icons.Rounded.Public),
          title = { Text("Country") },
          description = { Text(countryName(region) + " · channels and where to watch matches") },
          trailingContent = { Text(flag(region), fontSize = 22.sp) },
          onClick = { pickRegion = true },
        ),
      ),
  )

  if (pickRegion) {
    val sheet = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val countries = remember {
      Locale.getISOCountries().map { it to countryName(it) }.sortedBy { it.second }
    }
    ModalBottomSheet(onDismissRequest = { pickRegion = false }, sheetState = sheet) {
      Text("Your country", fontFamily = Brand.Serif, fontSize = 30.sp, modifier = Modifier.padding(start = 24.dp, bottom = 8.dp))
      LazyColumn(Modifier.fillMaxWidth().heightIn(max = 520.dp)) {
        items(countries, key = { it.first }) { (code, name) ->
          Row(
            Modifier.fillMaxWidth().clickable {
              region = code
              LiveTv.setRegion(context, code)
              pickRegion = false
            }.padding(horizontal = 24.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
          ) {
            Text(flag(code), fontSize = 22.sp)
            Spacer(Modifier.width(16.dp))
            Text(name, style = MaterialTheme.typography.bodyLarge, modifier = Modifier.weight(1f))
            if (code == region) RadioButton(selected = true, onClick = null)
          }
        }
      }
    }
  }
}

private fun countryName(code: String): String = Locale("", code).displayCountry.ifBlank { code }

private fun flag(code: String): String =
  if (code.length != 2) "🌐" else code.uppercase().map { Character.toChars(0x1F1E6 + (it - 'A')).concatToString() }.joinToString("")

// ── Downloads ──

@Composable
private fun DownloadsPage(navController: NavController) {
  var wifiOnly by remember { mutableStateOf(VelmoraDownloads.wifiOnly()) }
  var quality by remember { mutableStateOf(VelmoraDownloads.defaultQuality()) }
  Material3SettingsGroup(
    title = "When to download",
    items =
      listOf(
        Material3SettingsItem(
          icon = icon(Icons.Rounded.Wifi),
          title = { Text("Wi-Fi only") },
          description = { Text("Downloads wait for Wi-Fi and carry on by themselves when it's back") },
          trailingContent = {
            Toggle(wifiOnly) {
              wifiOnly = it
              VelmoraDownloads.setWifiOnly(it)
            }
          },
          onClick = {
            wifiOnly = !wifiOnly
            VelmoraDownloads.setWifiOnly(wifiOnly)
          },
        ),
      ),
  )
  Material3SettingsGroup(
    title = "Quality",
    items =
      DownloadQuality.entries.map { q ->
        Material3SettingsItem(
          icon = icon(Icons.Rounded.HighQuality),
          title = { Text(q.label) },
          description = {
            Text(
              when (q) {
                DownloadQuality.Best -> "The biggest file there is, 4K when it exists"
                DownloadQuality.Full -> "Sharp on any phone or tablet · about 2 GB a film"
                DownloadQuality.High -> "Good on a phone · about 1 GB a film"
                DownloadQuality.Saver -> "Smallest · about 500 MB a film"
              },
            )
          },
          trailingContent = { RadioButton(selected = quality == q, onClick = null) },
          onClick = {
            quality = q
            VelmoraDownloads.setDefaultQuality(q)
          },
        )
      },
  )
  Material3SettingsGroup(
    items =
      listOf(
        Material3SettingsItem(
          icon = icon(Icons.Rounded.Download),
          title = { Text("Open Downloads") },
          description = { Text("What's on this phone, and how much space is left") },
          trailingContent = { Icon(Icons.AutoMirrored.Rounded.KeyboardArrowRight, contentDescription = null) },
          onClick = { navController.navigate(VelmoraRoutes.DOWNLOADS) },
        ),
      ),
  )
}

// ── Theatre ──

@Composable
private fun TheatrePage() {
  val context = LocalContext.current
  var reminders by remember { mutableStateOf(TheatreAlerts.reminders(context)) }
  var probes by remember { mutableStateOf(Probes.enabled(context)) }
  Material3SettingsGroup(
    title = "Your screenings",
    items =
      listOf(
        Material3SettingsItem(
          icon = icon(Icons.Rounded.Notifications),
          title = { Text("Reminders") },
          description = { Text("When the lobby opens, and 5 minutes before the start") },
          trailingContent = {
            Toggle(reminders) {
              reminders = it
              TheatreAlerts.setReminders(context, it)
            }
          },
          onClick = {
            reminders = !reminders
            TheatreAlerts.setReminders(context, reminders)
          },
        ),
        Material3SettingsItem(
          icon = icon(Icons.Rounded.NotificationsActive),
          title = { Text("Notification settings") },
          description = { Text("Sound, pop-ups and the lock screen, in Android's settings") },
          trailingContent = { Icon(Icons.AutoMirrored.Rounded.KeyboardArrowRight, contentDescription = null) },
          onClick = { openNotificationSettings(context) },
        ),
      ),
  )
  Material3SettingsGroup(
    title = "Help the lineup",
    items =
      listOf(
        Material3SettingsItem(
          icon = icon(Icons.Rounded.Search),
          title = { Text("Check sources for coming films") },
          description = { Text("Now and then, on Wi-Fi and while nothing plays, Velmora checks a planned film can be found and says yes or no. Nothing about you is sent.") },
          trailingContent = {
            Toggle(probes) {
              probes = it
              Probes.setEnabled(context, it)
            }
          },
          onClick = {
            probes = !probes
            Probes.setEnabled(context, probes)
          },
        ),
      ),
  )
}

private fun openNotificationSettings(context: Context) {
  val intent =
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).putExtra(Settings.EXTRA_APP_PACKAGE, context.packageName)
    } else {
      Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, android.net.Uri.parse("package:" + context.packageName))
    }
  runCatching { context.startActivity(intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)) }
}

// ── Privacy ──

@Composable
private fun PrivacyPage(navController: NavController, snackbar: SnackbarHostState) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  val stats by UsageStats.enabled.collectAsState()
  val sharing by ActivityPoster.sharing.collectAsState()
  val account by VelmoraAccount.state.collectAsState()
  val signedIn = account is AccountState.SignedIn
  var preview by remember { mutableStateOf<String?>(null) }
  var busy by remember { mutableStateOf(false) }

  Material3SettingsGroup(
    title = "Anonymous usage counts",
    items =
      listOf(
        Material3SettingsItem(
          icon = icon(Icons.Rounded.Insights),
          title = { Text("Share anonymous counts") },
          description = { Text("Which titles, sites and songs get played, as daily totals. No name, no account, no ID. It shapes Popular on Velmora and Theatre's times.") },
          trailingContent = { Toggle(stats) { UsageStats.setEnabled(it) } },
          onClick = { UsageStats.setEnabled(!stats) },
        ),
        Material3SettingsItem(
          icon = icon(Icons.Rounded.Visibility),
          title = { Text("See what's sent") },
          description = { Text("The exact text waiting to go out") },
          trailingContent = { Icon(Icons.AutoMirrored.Rounded.KeyboardArrowRight, contentDescription = null) },
          onClick = { preview = UsageStats.preview() },
        ),
      ),
  )
  Material3SettingsGroup(
    title = "Velmora Social",
    items =
      listOf(
        Material3SettingsItem(
          icon = icon(Icons.Rounded.Forum),
          title = { Text("Show what I watch") },
          description = {
            Text(
              if (signedIn) "Started, finished and added to My List appear on your profile. 18+ titles never do."
              else "Sign in to choose what your profile shows",
            )
          },
          enabled = signedIn && !busy,
          trailingContent = {
            Toggle(sharing && signedIn, enabled = signedIn && !busy) { on ->
              busy = true
              scope.launch {
                val ok = ActivityPoster.setSharing(on)
                busy = false
                if (!ok) snackbar.showSnackbar("Couldn't save that. Check your connection.")
              }
            }
          },
          onClick = {
            if (signedIn && !busy) {
              busy = true
              scope.launch {
                val ok = ActivityPoster.setSharing(!sharing)
                busy = false
                if (!ok) snackbar.showSnackbar("Couldn't save that. Check your connection.")
              }
            }
          },
        ),
        Material3SettingsItem(
          icon = icon(Icons.Rounded.Edit),
          title = { Text("Profile privacy") },
          description = { Text("Who sees your lists, stats and visitors, on Velmora Social") },
          enabled = signedIn,
          trailingContent = { Icon(Icons.AutoMirrored.Rounded.KeyboardArrowRight, contentDescription = null) },
          onClick = { if (signedIn) navController.navigate(SocialLinks.route("/settings")) },
        ),
      ),
  )
  Material3SettingsGroup(
    title = "On this phone",
    items =
      listOf(
        Material3SettingsItem(
          icon = icon(Icons.Rounded.History),
          title = { Text("Clear recent searches") },
          description = { Text("The searches Velmora suggests when you tap the search box") },
          onClick = {
            context.getSharedPreferences("velmora_search", Context.MODE_PRIVATE).edit().remove("recent").apply()
            scope.launch { snackbar.showSnackbar("Recent searches cleared") }
          },
        ),
        Material3SettingsItem(
          icon = icon(Icons.Rounded.MusicNote),
          title = { Text("Music privacy") },
          description = { Text("Listening and search history in Music") },
          trailingContent = { Icon(Icons.AutoMirrored.Rounded.KeyboardArrowRight, contentDescription = null) },
          onClick = { navController.navigate("settings/privacy") },
        ),
      ),
  )

  preview?.let { text ->
    AlertDialog(
      onDismissRequest = { preview = null },
      title = { Text("What's sent") },
      text = {
        Column {
          Text(
            "Finished days go out once a day, then they're deleted here. Today's counts wait until tomorrow.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
          )
          Spacer(Modifier.height(10.dp))
          Surface(shape = RoundedCornerShape(12.dp), color = MaterialTheme.colorScheme.surfaceContainerHighest) {
            Text(
              text,
              fontFamily = FontFamily.Monospace,
              fontSize = 11.sp,
              lineHeight = 15.sp,
              modifier = Modifier.heightIn(max = 360.dp).verticalScroll(rememberScrollState()).horizontalScroll(rememberScrollState()).padding(12.dp),
            )
          }
        }
      },
      confirmButton = { TextButton(onClick = { preview = null }) { Text("Done") } },
    )
  }
}
