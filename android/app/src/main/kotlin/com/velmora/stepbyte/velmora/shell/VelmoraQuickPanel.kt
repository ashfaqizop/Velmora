package com.velmora.stepbyte.velmora.shell

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Bookmarks
import androidx.compose.ui.graphics.vector.rememberVectorPainter
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
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import com.velmora.stepbyte.velmora.account.AccountAvatar
import com.velmora.stepbyte.velmora.account.AccountState
import com.velmora.stepbyte.velmora.account.VelmoraAccount
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
  val context = LocalContext.current
  val account by VelmoraAccount.state.collectAsState()
  val signedIn = account as? AccountState.SignedIn
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
          buildList {
            if (signedIn != null) {
              val p = signedIn.profile
              add(
                Material3SettingsItem(
                  title = { Text(p.displayName) },
                  description = {
                    Text(
                      buildString {
                        append("@" + p.username)
                        if (signedIn.max) append("  ·  Velmora Max")
                        if (signedIn.offline) append("  ·  offline")
                      },
                    )
                  },
                  customIcon = { AccountAvatar(p, size = 36.dp) },
                  onClick = {
                    onDismissRequest()
                    onNavigate(com.velmora.stepbyte.velmora.settings.VelmoraSettingsRoutes.section("account"))
                  },
                ),
              )
            } else {
              add(
                Material3SettingsItem(
                  title = { Text("Sign in to Velmora") },
                  description = { Text("My List, watch progress and Max, synced with desktop") },
                  icon = painterResource(R.drawable.account),
                  onClick = {
                    onDismissRequest()
                    VelmoraAccount.beginSignIn(context)
                  },
                ),
              )
            }
            add(
              Material3SettingsItem(
                title = { Text("My List") },
                description = { Text("Everything you saved, on every device") },
                icon = rememberVectorPainter(Icons.Rounded.Bookmarks),
                onClick = {
                  onDismissRequest()
                  onNavigate(com.velmora.stepbyte.velmora.screens.VRoutes.MY_LIST)
                },
              ),
            )
            add(
              Material3SettingsItem(
                title = { Text("Velmora Social") },
                description = { Text("Your profile, friends and what they watch") },
                icon = painterResource(R.drawable.person),
                onClick = {
                  onDismissRequest()
                  onNavigate(com.velmora.stepbyte.velmora.social.SocialLinks.route(signedIn?.let { "/@" + it.profile.username } ?: "/"))
                },
              ),
            )
            if (signedIn != null) {
              add(
                Material3SettingsItem(
                  title = { Text("Sign out") },
                  description = { Text("Velmora asks you to sign in again next time") },
                  icon = painterResource(R.drawable.logout),
                  onClick = {
                    onDismissRequest()
                    VelmoraAccount.signOut()
                  },
                ),
              )
            }
          },
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
              description = { Text("Watching, downloads, privacy, Music and more") },
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
