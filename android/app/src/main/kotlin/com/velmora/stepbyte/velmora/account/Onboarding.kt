package com.velmora.stepbyte.velmora.account

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
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
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.datastore.preferences.core.booleanPreferencesKey
import com.velmora.stepbyte.R
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.utils.rememberPreference
import com.velmora.stepbyte.velmora.shell.Brand
import com.velmora.stepbyte.velmora.shell.LookPicker
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

/** Set once the first-run steps are finished. */
val OnboardingDoneKey = booleanPreferencesKey("velmora_onboarding_done")

private enum class Step { Welcome, SignIn, Look, Languages, Done }

/**
 * The first launch, like desktop's: welcome, sign in (required), pick the
 * look, languages, done. It covers the app until it is finished, and the
 * sign-in step comes back whenever the account is signed out.
 */
@Composable
fun OnboardingGate() {
  val account by VelmoraAccount.state.collectAsState()
  val (done, setDone) = rememberPreference(OnboardingDoneKey, defaultValue = false)
  val signedIn = account is AccountState.SignedIn
  // Waiting on the saved sign-in: the app opens straight away (offline is fine).
  if (account == AccountState.Loading) return
  if (done && signedIn) return

  var step by rememberSaveable { mutableStateOf(if (done) Step.SignIn else Step.Welcome) }
  // Signing in moves on by itself.
  LaunchedEffect(signedIn) {
    if (signedIn && step == Step.SignIn) step = if (done) Step.Done else Step.Look
  }
  if (done && signedIn) return

  BackHandler(enabled = step != Step.Welcome && step != Step.SignIn) {
    step = Step.entries[(step.ordinal - 1).coerceAtLeast(0)]
  }

  Box(
    Modifier.fillMaxSize()
      .background(Brand.Night)
      .drawBehind {
        drawCircle(
          Brush.radialGradient(listOf(Brand.Blue.copy(alpha = 0.45f), Color.Transparent), center = Offset(size.width * 0.85f, size.height * 0.1f), radius = size.width),
          center = Offset(size.width * 0.85f, size.height * 0.1f),
          radius = size.width,
        )
        drawCircle(
          Brush.radialGradient(listOf(Brand.Amber.copy(alpha = 0.18f), Color.Transparent), center = Offset(0f, size.height * 0.95f), radius = size.width * 0.9f),
          center = Offset(0f, size.height * 0.95f),
          radius = size.width * 0.9f,
        )
      }
      // Nothing behind the gate can be touched.
      .pointerInput(Unit) { awaitPointerEventScope { while (true) awaitPointerEvent() } }
      .windowInsetsPadding(WindowInsets.systemBars)
      .imePadding(),
  ) {
    AnimatedContent(
      targetState = step,
      transitionSpec = {
        val forward = targetState.ordinal > initialState.ordinal
        (slideInHorizontally(tween(420, easing = FastOutSlowInEasing)) { if (forward) it / 3 else -it / 3 } + fadeIn(tween(320))) togetherWith
          (slideOutHorizontally(tween(420, easing = FastOutSlowInEasing)) { if (forward) -it / 3 else it / 3 } + fadeOut(tween(220)))
      },
      label = "onboarding",
      modifier = Modifier.fillMaxSize(),
    ) { s ->
      when (s) {
        Step.Welcome -> WelcomeStep(onNext = { step = Step.SignIn })
        Step.SignIn -> SignInStep(returning = done)
        Step.Look -> LookStep(onNext = { step = Step.Languages })
        Step.Languages -> LanguagesStep(onNext = { step = Step.Done })
        Step.Done -> DoneStep(onFinish = { setDone(true) })
      }
    }
    if (step != Step.Done) {
      StepDots(current = step.ordinal, total = Step.entries.size - 1, modifier = Modifier.align(Alignment.TopCenter).padding(top = 16.dp))
    }
  }
}

@Composable
private fun StepDots(current: Int, total: Int, modifier: Modifier = Modifier) {
  Row(modifier, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
    repeat(total) { i ->
      Box(
        Modifier.height(6.dp)
          .width(if (i == current) 22.dp else 6.dp)
          .clip(CircleShape)
          .background(if (i <= current) Brand.Cream else Color.White.copy(alpha = 0.25f)),
      )
    }
  }
}

@Composable
private fun StepPage(
  title: String,
  text: String?,
  button: String?,
  onButton: () -> Unit,
  buttonEnabled: Boolean = true,
  content: @Composable () -> Unit = {},
) {
  Column(
    Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal = 24.dp, vertical = 48.dp),
    verticalArrangement = Arrangement.spacedBy(16.dp),
  ) {
    Spacer(Modifier.height(24.dp))
    Text(title, fontFamily = Brand.Serif, fontSize = 44.sp, lineHeight = 46.sp, color = Color.White)
    if (text != null) Text(text, style = MaterialTheme.typography.bodyLarge, color = Color.White.copy(alpha = 0.75f))
    content()
    Spacer(Modifier.height(12.dp))
    if (button != null) {
      Button(onClick = onButton, enabled = buttonEnabled, modifier = Modifier.fillMaxWidth().height(56.dp)) {
        Text(button, style = MaterialTheme.typography.titleMedium)
      }
    }
  }
}

// ── Welcome ──

@Composable
private fun WelcomeStep(onNext: () -> Unit) {
  val float = rememberInfiniteTransition(label = "fox")
  val lift by
    float.animateFloat(
      initialValue = -6f,
      targetValue = 6f,
      animationSpec = infiniteRepeatable(tween(2600, easing = LinearEasing), RepeatMode.Reverse),
      label = "foxLift",
    )
  Column(
    Modifier.fillMaxSize().padding(horizontal = 24.dp, vertical = 48.dp),
    horizontalAlignment = Alignment.CenterHorizontally,
    verticalArrangement = Arrangement.Center,
  ) {
    Image(
      painterResource(R.drawable.ic_launcher_nobg),
      contentDescription = null,
      colorFilter = ColorFilter.tint(Brand.Cream),
      modifier = Modifier.size(120.dp).padding(top = lift.coerceAtLeast(0f).dp, bottom = (-lift).coerceAtLeast(0f).dp),
    )
    Spacer(Modifier.height(16.dp))
    Text("Velmora", fontFamily = Brand.Serif, fontSize = 68.sp, color = Color.White)
    Text("Every source. One search.", fontFamily = Brand.Serif, fontStyle = FontStyle.Italic, fontSize = 26.sp, color = Brand.Cream)
    Spacer(Modifier.height(14.dp))
    Text(
      "Movies, series, anime, live TV, sports, Theatre and music. One account, on your phone and your PC.",
      style = MaterialTheme.typography.bodyLarge,
      color = Color.White.copy(alpha = 0.75f),
      textAlign = TextAlign.Center,
    )
    Spacer(Modifier.height(36.dp))
    Button(onClick = onNext, modifier = Modifier.fillMaxWidth().height(56.dp)) {
      Text("Get started", style = MaterialTheme.typography.titleMedium)
    }
  }
}

// ── Sign in ──

@Composable
private fun SignInStep(returning: Boolean) {
  val context = LocalContext.current
  val account by VelmoraAccount.state.collectAsState()
  val error by VelmoraAccount.error.collectAsState()
  val waiting = account == AccountState.Waiting
  var code by rememberSaveable { mutableStateOf("") }
  var showCode by rememberSaveable { mutableStateOf(false) }

  StepPage(
    title = if (returning) "Sign in again" else "Sign in to Velmora",
    text =
      "Velmora needs an account, the same one as on your PC. It keeps My List, what you've watched and your settings in step everywhere.",
    button = if (waiting) null else "Continue in the browser",
    onButton = { VelmoraAccount.beginSignIn(context) },
  ) {
    Text(
      "Google, Discord or a code by email. New here? You'll make your profile on the way.",
      style = MaterialTheme.typography.bodyMedium,
      color = Color.White.copy(alpha = 0.6f),
    )
    if (waiting) {
      Surface(shape = RoundedCornerShape(24.dp), color = Color.White.copy(alpha = 0.08f), modifier = Modifier.fillMaxWidth()) {
        Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
          Row(verticalAlignment = Alignment.CenterVertically) {
            CircularProgressIndicator(modifier = Modifier.size(20.dp), strokeWidth = 2.dp, color = Brand.Cream)
            Spacer(Modifier.width(12.dp))
            Text("Finish in your browser, then come back.", color = Color.White)
          }
          if (showCode) {
            OutlinedTextField(
              value = code,
              onValueChange = { code = it.take(12) },
              label = { Text("Code from the sign-in page") },
              singleLine = true,
              keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.Characters),
              modifier = Modifier.fillMaxWidth(),
            )
            Button(onClick = { VelmoraAccount.redeem(code) }, enabled = code.count { it.isLetterOrDigit() } >= 6) { Text("Use this code") }
          } else {
            TextButton(onClick = { showCode = true }) { Text("Enter a code instead", color = Brand.Cream) }
          }
          Row {
            TextButton(onClick = { VelmoraAccount.beginSignIn(context) }) { Text("Open the browser again", color = Color.White) }
            TextButton(onClick = { VelmoraAccount.cancel() }) { Text("Cancel", color = Color.White.copy(alpha = 0.7f)) }
          }
        }
      }
    }
    error?.let { Text(it, color = Color(0xFFFFB4AB), style = MaterialTheme.typography.bodyMedium) }
  }
}

// ── Look ──

@Composable
private fun LookStep(onNext: () -> Unit) {
  StepPage(
    title = "Pick your look",
    text = "You can change it any time in Settings, Appearance.",
    button = "Continue",
    onButton = onNext,
  ) {
    Surface(shape = RoundedCornerShape(28.dp), color = MaterialTheme.colorScheme.surface, modifier = Modifier.fillMaxWidth()) {
      Box(Modifier.padding(12.dp)) { LookPicker() }
    }
  }
}

// ── Languages ──

private val LANGUAGES =
  listOf(
    "en" to "English",
    "hi" to "Hindi",
    "bn" to "Bengali",
    "ta" to "Tamil",
    "te" to "Telugu",
    "ja" to "Japanese",
    "ko" to "Korean",
    "zh" to "Chinese",
    "es" to "Spanish",
    "fr" to "French",
    "de" to "German",
    "ar" to "Arabic",
  )

@Composable
private fun LanguagesStep(onNext: () -> Unit) {
  val context = LocalContext.current
  val scope = rememberCoroutineScope()
  val audio = remember { mutableStateListOf<String>() }
  val subs = remember { mutableStateListOf<String>() }
  var saving by remember { mutableStateOf(false) }

  // Start from what's saved (the account's settings may already have come down).
  LaunchedEffect(Unit) {
    val saved = Universe.savedPreferences(context)?.let { runCatching { Json.parseToJsonElement(it).jsonObject }.getOrNull() }
    fun list(key: String) = (saved?.get(key) as? JsonArray)?.mapNotNull { it.jsonPrimitive.contentOrNull }.orEmpty()
    audio.clear()
    audio.addAll(list("audio_languages").ifEmpty { listOf("en") })
    subs.clear()
    subs.addAll(list("subtitle_languages").ifEmpty { listOf("en") })
  }

  StepPage(
    title = "What do you watch in?",
    text = "Velmora picks sources and subtitles in these first. Choose as many as you like; the first one you pick comes first.",
    button = if (saving) "Saving…" else "Continue",
    buttonEnabled = !saving && audio.isNotEmpty(),
    onButton = {
      saving = true
      scope.launch {
        runCatching { AccountSync.saveLanguages(context, audio.toList(), subs.toList()) }
        saving = false
        onNext()
      }
    },
  ) {
    LanguageChips("Audio", audio)
    LanguageChips("Subtitles", subs)
  }
}

@Composable
private fun LanguageChips(label: String, picked: MutableList<String>) {
  Text(label, style = MaterialTheme.typography.titleMedium, color = Brand.Cream, fontWeight = FontWeight.Bold)
  FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
    LANGUAGES.forEach { (code, name) ->
      val on = code in picked
      FilterChip(
        selected = on,
        onClick = { if (on) picked.remove(code) else picked.add(code) },
        label = { Text(if (on && picked.size > 1) "${picked.indexOf(code) + 1}. $name" else name) },
      )
    }
  }
}

// ── Done ──

@Composable
private fun DoneStep(onFinish: () -> Unit) {
  val account by VelmoraAccount.state.collectAsState()
  val profile = (account as? AccountState.SignedIn)?.profile
  val max = (account as? AccountState.SignedIn)?.max == true
  Column(
    Modifier.fillMaxSize().padding(horizontal = 24.dp, vertical = 48.dp),
    horizontalAlignment = Alignment.CenterHorizontally,
    verticalArrangement = Arrangement.Center,
  ) {
    if (profile != null) AccountAvatar(profile, size = 96.dp)
    Spacer(Modifier.height(20.dp))
    Text(
      if (profile != null) "You're in, ${profile.displayName}" else "You're in",
      fontFamily = Brand.Serif,
      fontSize = 44.sp,
      lineHeight = 46.sp,
      color = Color.White,
      textAlign = TextAlign.Center,
    )
    if (profile != null) Text("@${profile.username}", color = Color.White.copy(alpha = 0.6f))
    if (max) {
      Spacer(Modifier.height(12.dp))
      Surface(shape = RoundedCornerShape(50), color = Brand.Amber) {
        Text(
          "Velmora Max",
          color = Color(0xFF3A2200),
          fontWeight = FontWeight.Bold,
          modifier = Modifier.padding(horizontal = 14.dp, vertical = 6.dp),
        )
      }
    }
    Spacer(Modifier.height(14.dp))
    Text(
      "Slide the pill at the top right any time to switch between Velmora and Music.",
      style = MaterialTheme.typography.bodyLarge,
      color = Color.White.copy(alpha = 0.75f),
      textAlign = TextAlign.Center,
    )
    Spacer(Modifier.height(36.dp))
    Button(onClick = onFinish, modifier = Modifier.fillMaxWidth().height(56.dp)) {
      Text("Start watching", style = MaterialTheme.typography.titleMedium)
    }
  }
}
