package com.velmora.stepbyte.velmora.social

import android.annotation.SuppressLint
import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.content.Intent
import android.graphics.Bitmap
import android.net.Uri
import android.webkit.CookieManager
import android.webkit.ValueCallback
import android.webkit.WebChromeClient
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.OpenInNew
import androidx.compose.material.icons.rounded.CloudOff
import androidx.compose.material.icons.rounded.Forum
import androidx.compose.material.icons.rounded.Home
import androidx.compose.material.icons.rounded.MoreVert
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Share
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialShapes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.toShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.view.WindowCompat
import androidx.navigation.NavController
import com.velmora.stepbyte.BuildConfig
import com.velmora.stepbyte.velmora.account.AccountState
import com.velmora.stepbyte.velmora.account.VelmoraAccount
import com.velmora.stepbyte.velmora.shell.Brand
import kotlinx.coroutines.launch

/**
 * Velmora Social inside the app: the website's pages in a WebView, signed in
 * with the app's account (see [SocialSession]), framed by a native bar.
 * The site is dark, so the frame is too. Social and title links stay inside;
 * "Open in Velmora" links go to the app's own screens; anything else opens in
 * the browser.
 */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@SuppressLint("SetJavaScriptEnabled")
@Composable
fun SocialScreen(to: String, navController: NavController) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  var title by remember { mutableStateOf("") }
  var progress by remember { mutableFloatStateOf(0f) }
  var loading by remember { mutableStateOf(true) }
  var shown by remember { mutableStateOf(false) }
  var failed by remember { mutableStateOf(false) }
  var canGoBack by remember { mutableStateOf(false) }
  var menu by remember { mutableStateOf(false) }
  var pendingFiles by remember { mutableStateOf<ValueCallback<Array<Uri>>?>(null) }

  val pickFiles =
    rememberLauncherForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
      pendingFiles?.onReceiveValue(WebChromeClient.FileChooserParams.parseResult(result.resultCode, result.data))
      pendingFiles = null
    }

  // Light status bar icons on the dark frame, then back to how they were.
  val view = LocalView.current
  DisposableEffect(Unit) {
    val window = context.findActivity()?.window
    val controller = window?.let { WindowCompat.getInsetsController(it, view) }
    val before = controller?.isAppearanceLightStatusBars
    controller?.isAppearanceLightStatusBars = false
    onDispose { if (before != null) controller?.isAppearanceLightStatusBars = before }
  }

  val web = remember {
    WebView(context).apply {
      setBackgroundColor(Brand.Night.toArgb())
      settings.javaScriptEnabled = true
      settings.domStorageEnabled = true
      settings.mediaPlaybackRequiresUserGesture = true
      settings.allowFileAccess = false
      settings.allowContentAccess = false
      settings.setSupportMultipleWindows(false)
      settings.userAgentString = settings.userAgentString + " VelmoraAndroid/" + BuildConfig.VERSION_NAME
      CookieManager.getInstance().setAcceptCookie(true)
    }
  }

  fun currentPath(): String = web.url?.let { Uri.parse(it).path }?.takeIf { it.isNotBlank() } ?: to

  // The page asked to sign in: hand over the app's account instead of the
  // web sign-in (Google refuses to sign in inside apps).
  fun signInAgain() {
    SocialSession.lost(context)
    if (VelmoraAccount.state.value is AccountState.SignedIn) {
      scope.launch { web.loadUrl(SocialSession.startUrl(context, currentPath(), force = true)) }
    } else {
      VelmoraAccount.beginSignIn(context)
    }
  }

  fun open(target: VelmoraLinks.Target) {
    if (target is VelmoraLinks.Target.Social) web.loadUrl(SocialLinks.url(target.path)) else VelmoraLinks.requests.value = target
  }

  DisposableEffect(web) {
    web.webViewClient =
      object : WebViewClient() {
        override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean {
          if (!request.isForMainFrame) return false
          val url = request.url
          val host = url.host?.lowercase()
          if (host == SocialLinks.HOST) return false
          if (host == "auth.getvelmora.dpdns.org") {
            signInAgain()
            return true
          }
          VelmoraLinks.parse(url)?.let {
            open(it)
            return true
          }
          runCatching { context.startActivity(Intent(Intent.ACTION_VIEW, url).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)) }
          return true
        }

        override fun onPageStarted(view: WebView, url: String?, favicon: Bitmap?) {
          loading = true
          failed = false
        }

        override fun onPageFinished(view: WebView, url: String?) {
          loading = false
          shown = true
          canGoBack = view.canGoBack()
        }

        override fun doUpdateVisitedHistory(view: WebView, url: String?, isReload: Boolean) {
          canGoBack = view.canGoBack()
        }

        override fun onReceivedError(view: WebView, request: WebResourceRequest, error: WebResourceError) {
          if (request.isForMainFrame) failed = true
        }
      }
    web.webChromeClient =
      object : WebChromeClient() {
        override fun onProgressChanged(view: WebView, newProgress: Int) {
          progress = newProgress / 100f
          // The page paints well before every image is in.
          if (newProgress >= 70) shown = true
        }

        override fun onReceivedTitle(view: WebView, t: String?) {
          title = t.orEmpty()
        }

        override fun onShowFileChooser(view: WebView, callback: ValueCallback<Array<Uri>>, params: FileChooserParams): Boolean {
          pendingFiles?.onReceiveValue(null)
          pendingFiles = callback
          return runCatching { pickFiles.launch(params.createIntent()) }
            .map { true }
            .getOrElse {
              pendingFiles = null
              callback.onReceiveValue(null)
              false
            }
        }
      }
    onDispose {
      web.stopLoading()
      web.destroy()
    }
  }

  LaunchedEffect(to) { web.loadUrl(SocialSession.startUrl(context, to)) }

  BackHandler(enabled = canGoBack) { web.goBack() }

  Column(Modifier.fillMaxSize().background(Brand.Night)) {
    Row(
      Modifier.fillMaxWidth().windowInsetsPadding(WindowInsets.statusBars).padding(start = 4.dp, end = 4.dp, top = 4.dp, bottom = 2.dp),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      IconButton(onClick = { if (canGoBack) web.goBack() else navController.navigateUp() }) {
        Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = "Back", tint = Color.White)
      }
      Column(Modifier.weight(1f).padding(start = 4.dp)) {
        Text(
          pageName(title),
          color = Color.White,
          style = MaterialTheme.typography.titleMedium,
          fontWeight = FontWeight.SemiBold,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
        )
        Text("Velmora Social", color = Color.White.copy(alpha = 0.6f), style = MaterialTheme.typography.labelSmall)
      }
      IconButton(onClick = { web.reload() }) { Icon(Icons.Rounded.Refresh, contentDescription = "Reload", tint = Color.White) }
      Box {
        IconButton(onClick = { menu = true }) { Icon(Icons.Rounded.MoreVert, contentDescription = "More", tint = Color.White) }
        DropdownMenu(expanded = menu, onDismissRequest = { menu = false }) {
          DropdownMenuItem(
            text = { Text("Social home") },
            leadingIcon = { Icon(Icons.Rounded.Home, contentDescription = null) },
            onClick = {
              menu = false
              web.loadUrl(SocialLinks.url("/"))
            },
          )
          DropdownMenuItem(
            text = { Text("Share this page") },
            leadingIcon = { Icon(Icons.Rounded.Share, contentDescription = null) },
            onClick = {
              menu = false
              val link = shareable(web.url) ?: return@DropdownMenuItem
              val send = Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT, link)
              runCatching { context.startActivity(Intent.createChooser(send, "Share").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)) }
            },
          )
          DropdownMenuItem(
            text = { Text("Open in browser") },
            leadingIcon = { Icon(Icons.AutoMirrored.Rounded.OpenInNew, contentDescription = null) },
            onClick = {
              menu = false
              val link = shareable(web.url) ?: SocialLinks.url("/")
              runCatching { context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(link)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)) }
            },
          )
        }
      }
    }
    Box(Modifier.fillMaxWidth().height(3.dp)) {
      Fade(loading && !failed) {
        LinearProgressIndicator(
          progress = { progress.coerceAtLeast(0.05f) },
          modifier = Modifier.fillMaxWidth().height(3.dp),
          color = Brand.Blue,
          trackColor = Color.Transparent,
          drawStopIndicator = {},
        )
      }
    }
    Box(Modifier.weight(1f).fillMaxWidth().windowInsetsPadding(WindowInsets.navigationBars).imePadding()) {
      AndroidView(factory = { web }, modifier = Modifier.fillMaxSize())
      // Until the first paint: the mark on night, never a white flash.
      Fade(!shown && !failed) {
        Column(
          Modifier.fillMaxSize().background(Brand.Night),
          horizontalAlignment = Alignment.CenterHorizontally,
          verticalArrangement = Arrangement.Center,
        ) {
          Box(Modifier.size(84.dp).clip(MaterialShapes.Cookie9Sided.toShape()).background(Brand.Blue.copy(alpha = 0.18f)), contentAlignment = Alignment.Center) {
            Icon(Icons.Rounded.Forum, contentDescription = null, tint = Brand.Violet, modifier = Modifier.size(38.dp))
          }
          Spacer(Modifier.height(18.dp))
          LoadingIndicator(color = Brand.Violet, modifier = Modifier.size(36.dp))
          Spacer(Modifier.height(10.dp))
          Text("Opening Velmora Social…", color = Color.White.copy(alpha = 0.75f), style = MaterialTheme.typography.bodyMedium)
        }
      }
      Fade(failed) {
        Column(
          Modifier.fillMaxSize().background(Brand.Night).padding(32.dp),
          horizontalAlignment = Alignment.CenterHorizontally,
          verticalArrangement = Arrangement.Center,
        ) {
          Box(Modifier.size(88.dp).clip(MaterialShapes.Cookie9Sided.toShape()).background(Color.White.copy(alpha = 0.08f)), contentAlignment = Alignment.Center) {
            Icon(Icons.Rounded.CloudOff, contentDescription = null, tint = Color.White, modifier = Modifier.size(40.dp))
          }
          Spacer(Modifier.height(16.dp))
          Text("Social needs the internet", color = Color.White, fontFamily = Brand.Serif, fontSize = 30.sp, textAlign = TextAlign.Center)
          Spacer(Modifier.height(6.dp))
          Text(
            "Profiles, friends and reviews live online. Everything else in Velmora still works.",
            color = Color.White.copy(alpha = 0.7f),
            style = MaterialTheme.typography.bodyMedium,
            textAlign = TextAlign.Center,
          )
          Spacer(Modifier.height(16.dp))
          FilledTonalButton(onClick = {
            failed = false
            if (web.url.isNullOrBlank() || web.url == "about:blank") scope.launch { web.loadUrl(SocialSession.startUrl(context, to)) } else web.reload()
          }) {
            Icon(Icons.Rounded.Refresh, contentDescription = null, modifier = Modifier.size(18.dp))
            Spacer(Modifier.width(8.dp))
            Text("Try again")
          }
        }
      }
    }
  }
}

/** Fades in and out; outside any Row or Column scope, so it can sit in a Box. */
@Composable
private fun Fade(visible: Boolean, content: @Composable () -> Unit) {
  AnimatedVisibility(visible, enter = fadeIn(), exit = fadeOut()) { content() }
}

/** "Ana (@ana) · Velmora Social" → "Ana (@ana)". */
private fun pageName(title: String): String {
  val t = title.substringBefore(" · Velmora Social").substringBefore(" | Velmora Social").trim()
  return if (t.isBlank() || t.startsWith("http") || t.contains(SocialLinks.HOST)) "Velmora Social" else t
}

/** The page's public address, without the sign-in fragment. */
private fun shareable(url: String?): String? = url?.substringBefore('#')?.takeIf { it.startsWith("https://") }

private tailrec fun Context.findActivity(): Activity? =
  when (this) {
    is Activity -> this
    is ContextWrapper -> baseContext.findActivity()
    else -> null
  }
