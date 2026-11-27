package com.velmora.stepbyte.velmora.shell

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import com.velmora.stepbyte.R
import com.velmora.stepbyte.ui.component.Material3SettingsGroup
import com.velmora.stepbyte.ui.component.Material3SettingsItem

/**
 * The avatar's quick panel in Velmora mode, built like Echo's music panel:
 * the Velmora account, Social, Max, Settings and About. Account, Social and
 * Max become real in phases 4 and 9; until then they say so.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun VelmoraQuickPanel(
  onDismissRequest: () -> Unit,
  onNavigate: (String) -> Unit,
  onSwitchToMusic: () -> Unit,
) {
  val uriHandler = LocalUriHandler.current
  val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
  ModalBottomSheet(
    onDismissRequest = onDismissRequest,
    sheetState = sheetState,
    dragHandle = { BottomSheetDefaults.DragHandle() },
    containerColor = MaterialTheme.colorScheme.surfaceContainer,
  ) {
    Column(
      modifier =
        Modifier.fillMaxWidth()
          .verticalScroll(rememberScrollState())
          .padding(bottom = 32.dp, start = 16.dp, end = 16.dp),
      verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
      Material3SettingsGroup(
        title = "Velmora account",
        compact = true,
        items =
          listOf(
            Material3SettingsItem(
              title = { Text("Sign in to Velmora") },
              description = { Text("My List, watch progress and Max, synced with desktop. Arrives in phase 4.") },
              icon = painterResource(R.drawable.account),
              onClick = { uriHandler.openUri(Brand.PROGRESS_URL) },
            ),
            Material3SettingsItem(
              title = { Text("Velmora Social") },
              description = { Text("Your profile, friends and what they watch") },
              icon = painterResource(R.drawable.person),
              onClick = {
                onDismissRequest()
                uriHandler.openUri("https://social.getvelmora.dpdns.org")
              },
            ),
          ),
      )
      Material3SettingsGroup(
        title = "App",
        compact = true,
        items =
          listOf(
            Material3SettingsItem(
              title = { Text("Switch to Music") },
              description = { Text("Songs, albums, podcasts and your YouTube Music library") },
              icon = painterResource(R.drawable.music_note),
              onClick = {
                onDismissRequest()
                onSwitchToMusic()
              },
            ),
            Material3SettingsItem(
              title = { Text("Settings") },
              description = { Text("Appearance, player, storage and more") },
              icon = painterResource(R.drawable.settings),
              onClick = {
                onDismissRequest()
                onNavigate("settings")
              },
            ),
            Material3SettingsItem(
              title = { Text("About") },
              description = { Text("Version, source code and credits") },
              icon = painterResource(R.drawable.info),
              onClick = {
                onDismissRequest()
                onNavigate("settings/about")
              },
            ),
          ),
      )
    }
  }
}
