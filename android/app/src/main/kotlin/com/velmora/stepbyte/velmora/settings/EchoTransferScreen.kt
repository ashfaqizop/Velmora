package com.velmora.stepbyte.velmora.settings

import android.content.Context
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowForward
import androidx.compose.material.icons.automirrored.rounded.OpenInNew
import androidx.compose.material.icons.automirrored.rounded.QueueMusic
import androidx.compose.material.icons.rounded.Album
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material.icons.rounded.Favorite
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.material.icons.rounded.History
import androidx.compose.material.icons.rounded.LibraryMusic
import androidx.compose.material.icons.rounded.Person
import androidx.compose.material.icons.rounded.Tune
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialShapes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.toShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.navigation.NavController
import com.velmora.stepbyte.LocalPlayerAwareWindowInsets
import com.velmora.stepbyte.MainActivity
import com.velmora.stepbyte.R
import com.velmora.stepbyte.playback.MusicService
import com.velmora.stepbyte.playback.MusicService.Companion.PERSISTENT_QUEUE_FILE
import com.velmora.stepbyte.velmora.shell.Brand
import com.velmora.stepbyte.viewmodels.BackupRestoreViewModel
import java.io.File
import java.text.NumberFormat
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private sealed interface Step {
  data object Start : Step

  data object Reading : Step

  data class Ready(val preview: EchoTransfer.Preview) : Step

  data class Problem(val text: String) : Step

  data object Moving : Step
}

/** Settings > Velmora > Transfer from Echo Music. */
@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalLayoutApi::class)
@Composable
fun EchoTransferScreen(navController: NavController, viewModel: BackupRestoreViewModel = hiltViewModel()) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  var step by remember { mutableStateOf<Step>(Step.Start) }
  var confirm by remember { mutableStateOf(false) }
  val echo = remember { EchoTransfer.installedEcho(context) }

  val pick =
    rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri: Uri? ->
      if (uri == null) return@rememberLauncherForActivityResult
      step = Step.Reading
      scope.launch {
        val version = withContext(Dispatchers.IO) { runCatching { viewModel.database.openHelper.readableDatabase.version }.getOrDefault(Int.MAX_VALUE) }
        step =
          EchoTransfer.inspect(context, uri, version).fold(
            onSuccess = { p ->
              when {
                p.tooNew -> Step.Problem("This backup comes from a newer Echo Music than Velmora reads yet. Update Velmora, then try again.")
                p.empty -> Step.Problem("There's nothing in that file Velmora can use.")
                else -> Step.Ready(p)
              }
            },
            onFailure = { Step.Problem(it.message ?: "Couldn't read that file.") },
          )
      }
    }

  Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface).verticalScroll(rememberScrollState())) {
    PageHead("Transfer from Echo Music", "Bring your music library over", navController)
    Column(Modifier.padding(horizontal = 16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
      Hero()
      AnimatedContent(step, transitionSpec = { fadeIn() togetherWith fadeOut() }, label = "transfer") { s ->
        when (s) {
          Step.Start -> Steps(echo, onOpenEcho = { openEcho(context, echo) }, onPick = { pick.launch(arrayOf("*/*")) })
          Step.Reading -> Busy("Reading your backup…")
          Step.Moving -> Busy("Moving your library… Velmora restarts in a moment.")
          is Step.Problem ->
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
              Surface(shape = RoundedCornerShape(24.dp), color = MaterialTheme.colorScheme.errorContainer, modifier = Modifier.fillMaxWidth()) {
                Row(Modifier.padding(20.dp), verticalAlignment = Alignment.Top) {
                  Icon(Icons.Rounded.ErrorOutline, contentDescription = null, tint = MaterialTheme.colorScheme.onErrorContainer)
                  Spacer(Modifier.width(12.dp))
                  Text(s.text, color = MaterialTheme.colorScheme.onErrorContainer, style = MaterialTheme.typography.bodyMedium)
                }
              }
              OutlinedButton(onClick = { pick.launch(arrayOf("*/*")) }, modifier = Modifier.fillMaxWidth()) { Text("Choose another file") }
            }
          is Step.Ready ->
            Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
              Text("In this backup", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.primary, modifier = Modifier.padding(start = 8.dp))
              FlowRow(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalArrangement = Arrangement.spacedBy(10.dp), maxItemsInEachRow = 3) {
                val p = s.preview
                Stat(Icons.Rounded.LibraryMusic, p.songs, "songs", Modifier.weight(1f))
                Stat(Icons.Rounded.Favorite, p.liked, "liked", Modifier.weight(1f))
                Stat(Icons.AutoMirrored.Rounded.QueueMusic, p.playlists, "playlists", Modifier.weight(1f))
                Stat(Icons.Rounded.Album, p.albums, "albums", Modifier.weight(1f))
                Stat(Icons.Rounded.Person, p.artists, "artists", Modifier.weight(1f))
                Stat(Icons.Rounded.History, p.plays, "plays", Modifier.weight(1f))
              }
              if (s.preview.hasSettings) {
                Surface(shape = RoundedCornerShape(20.dp), color = MaterialTheme.colorScheme.surfaceContainer, modifier = Modifier.fillMaxWidth()) {
                  Row(Modifier.padding(16.dp), verticalAlignment = Alignment.CenterVertically) {
                    Icon(Icons.Rounded.Tune, contentDescription = null, tint = MaterialTheme.colorScheme.onSurfaceVariant)
                    Spacer(Modifier.width(12.dp))
                    Text(
                      "Your Music settings come too. Velmora's own settings, your account and downloads stay as they are.",
                      style = MaterialTheme.typography.bodyMedium,
                      color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                  }
                }
              }
              Text(
                "This replaces the music library on this phone with the one in the backup.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(horizontal = 8.dp),
              )
              Button(onClick = { confirm = true }, modifier = Modifier.fillMaxWidth().height(56.dp), colors = androidx.compose.material3.ButtonDefaults.buttonColors(containerColor = Brand.Blue)) {
                Text("Transfer and restart", style = MaterialTheme.typography.titleMedium)
                Spacer(Modifier.width(8.dp))
                Icon(Icons.AutoMirrored.Rounded.ArrowForward, contentDescription = null)
              }
              TextButton(onClick = { pick.launch(arrayOf("*/*")) }, modifier = Modifier.align(Alignment.CenterHorizontally)) { Text("Choose a different file") }
            }
        }
      }
    }
    Spacer(Modifier.windowInsetsPadding(LocalPlayerAwareWindowInsets.current.only(WindowInsetsSides.Bottom)))
    Spacer(Modifier.height(32.dp))
  }

  if (confirm) {
    AlertDialog(
      onDismissRequest = { confirm = false },
      title = { Text("Replace your music library?") },
      text = { Text("Songs, playlists and history on this phone are swapped for the ones from Echo Music. Velmora then restarts.") },
      confirmButton = {
        TextButton(onClick = {
          confirm = false
          step = Step.Moving
          scope.launch { move(context, viewModel) }
        }) { Text("Transfer") }
      },
      dismissButton = { TextButton(onClick = { confirm = false }) { Text("Cancel") } },
    )
  }
}

private suspend fun move(context: Context, viewModel: BackupRestoreViewModel) {
  withContext(Dispatchers.IO) {
    EchoTransfer.applySettings(context)
    EchoTransfer.database(context)?.let { incoming ->
      val db = viewModel.database
      val path = db.openHelper.writableDatabase.path ?: return@let
      runCatching { db.checkpoint() }
      db.close()
      File("$path-wal").delete()
      File("$path-shm").delete()
      incoming.copyTo(File(path), overwrite = true)
    }
    EchoTransfer.cleanUp(context)
  }
  context.stopService(android.content.Intent(context, MusicService::class.java))
  context.filesDir.resolve(PERSISTENT_QUEUE_FILE).delete()
  EchoTransfer.restart(context, MainActivity::class.java)
}

private fun openEcho(context: Context, pkg: String?) {
  pkg ?: return
  context.packageManager.getLaunchIntentForPackage(pkg)?.let { runCatching { context.startActivity(it) } }
}

/** Echo's mark, an arrow, Velmora's. */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun Hero() {
  Surface(shape = RoundedCornerShape(32.dp), color = Brand.Night, modifier = Modifier.fillMaxWidth()) {
    Column(Modifier.padding(24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
      Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.Center) {
        Box(Modifier.size(64.dp).clip(MaterialShapes.Cookie7Sided.toShape()).background(Brand.Cream.copy(alpha = 0.16f)), contentAlignment = Alignment.Center) {
          Icon(Icons.Rounded.LibraryMusic, contentDescription = null, tint = Brand.Cream, modifier = Modifier.size(30.dp))
        }
        Spacer(Modifier.width(14.dp))
        Icon(Icons.AutoMirrored.Rounded.ArrowForward, contentDescription = null, tint = Color.White.copy(alpha = 0.6f))
        Spacer(Modifier.width(14.dp))
        Box(Modifier.size(64.dp).clip(MaterialShapes.Cookie9Sided.toShape()).background(Brand.Blue.copy(alpha = 0.25f)), contentAlignment = Alignment.Center) {
          Icon(painterResource(R.drawable.ic_launcher_nobg), contentDescription = null, tint = Color.Unspecified, modifier = Modifier.size(40.dp))
        }
      }
      Spacer(Modifier.height(16.dp))
      Text("Moving from Echo Music?", color = Color.White, fontFamily = Brand.Serif, fontSize = 30.sp, textAlign = TextAlign.Center)
      Spacer(Modifier.height(6.dp))
      Text(
        "Velmora's Music is built on Echo, so everything fits: liked songs, playlists, albums, artists, your listening history and Music settings.",
        color = Color.White.copy(alpha = 0.72f),
        style = MaterialTheme.typography.bodyMedium,
        textAlign = TextAlign.Center,
        modifier = Modifier.widthIn(max = 440.dp),
      )
    }
  }
}

@Composable
private fun Steps(echo: String?, onOpenEcho: () -> Unit, onPick: () -> Unit) {
  Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
    Surface(shape = RoundedCornerShape(28.dp), color = MaterialTheme.colorScheme.surfaceContainer, modifier = Modifier.fillMaxWidth()) {
      Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        StepRow(1, "Make a backup in Echo Music", "Settings › Backup and restore › Local backup. Save the file anywhere, Downloads is fine.")
        if (echo != null) {
          OutlinedButton(onClick = onOpenEcho, modifier = Modifier.padding(start = 44.dp)) {
            Icon(Icons.AutoMirrored.Rounded.OpenInNew, contentDescription = null, modifier = Modifier.size(18.dp))
            Spacer(Modifier.width(8.dp))
            Text("Open Echo Music")
          }
        }
        StepRow(2, "Pick that file here", "Old and new Echo backups both work. Velmora shows what's inside before anything changes.")
        StepRow(3, "Transfer", "Velmora restarts once, with your library in Music.")
      }
    }
    Button(onClick = onPick, modifier = Modifier.fillMaxWidth().height(56.dp), colors = androidx.compose.material3.ButtonDefaults.buttonColors(containerColor = Brand.Blue)) {
      Icon(Icons.Rounded.FolderOpen, contentDescription = null)
      Spacer(Modifier.width(10.dp))
      Text("Choose the backup file", style = MaterialTheme.typography.titleMedium)
    }
  }
}

@Composable
private fun StepRow(n: Int, title: String, text: String) {
  Row(verticalAlignment = Alignment.Top) {
    Box(Modifier.size(30.dp).clip(CircleShape).background(Brand.Blue.copy(alpha = 0.16f)), contentAlignment = Alignment.Center) {
      Text("$n", color = Brand.Blue, fontWeight = FontWeight.Bold, style = MaterialTheme.typography.labelLarge)
    }
    Spacer(Modifier.width(14.dp))
    Column {
      Text(title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
      Text(text, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
  }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun Busy(text: String) {
  Column(Modifier.fillMaxWidth().padding(vertical = 40.dp), horizontalAlignment = Alignment.CenterHorizontally) {
    LoadingIndicator(modifier = Modifier.size(56.dp))
    Spacer(Modifier.height(14.dp))
    Text(text, style = MaterialTheme.typography.bodyLarge, textAlign = TextAlign.Center)
  }
}

@Composable
private fun Stat(icon: ImageVector, value: Int?, label: String, modifier: Modifier) {
  Surface(shape = RoundedCornerShape(22.dp), color = MaterialTheme.colorScheme.surfaceContainerHigh, modifier = modifier) {
    Column(Modifier.padding(horizontal = 14.dp, vertical = 14.dp)) {
      Icon(icon, contentDescription = null, tint = Brand.Blue, modifier = Modifier.size(20.dp))
      Spacer(Modifier.height(8.dp))
      Text(value?.let { NumberFormat.getIntegerInstance().format(it) } ?: "–", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.SemiBold)
      Text(label, style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
  }
}
