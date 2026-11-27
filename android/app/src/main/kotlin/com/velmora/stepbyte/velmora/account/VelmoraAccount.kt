package com.velmora.stepbyte.velmora.account

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.util.Base64
import android.util.Log
import androidx.compose.ui.graphics.Color
import com.velmora.stepbyte.BuildConfig
import java.net.URLEncoder
import java.security.MessageDigest
import java.security.SecureRandom
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import kotlinx.serialization.json.put

/** A Velmora profile, as the `profiles` table has it. */
@Serializable
data class Profile(
  val id: String,
  val username: String,
  @SerialName("display_name") val displayName: String,
  /** "animal:owl" (or an old "preset:N"), or a photo URL from Google or Discord. */
  val avatar: String = "animal:panda",
)

sealed interface AccountState {
  /** Checking a saved sign-in. */
  data object Loading : AccountState

  data object SignedOut : AccountState

  /** The browser is open on the sign-in page. */
  data object Waiting : AccountState

  data class SignedIn(
    val profile: Profile,
    val email: String?,
    /** Opened from the saved sign-in; the server hasn't confirmed it yet. */
    val offline: Boolean,
    val max: Boolean,
  ) : AccountState
}

/**
 * The Velmora account on this phone. Signing in happens in the browser on the
 * auth site (Google, Discord or an emailed code), the same as desktop; the
 * site hands back a one-time code through velmora://auth/callback (or typed in
 * by hand), which is traded for the session with the PKCE secret made here.
 *
 * Like desktop, only the first launch needs the internet: a saved sign-in
 * opens the app straight away and the server checks it in the background.
 */
object VelmoraAccount {
  private const val TAG = "VelmoraAccount"
  private const val PREFS = "velmora_account"

  private val json = Json { ignoreUnknownKeys = true }
  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
  private val lock = Mutex()

  private val _state = MutableStateFlow<AccountState>(AccountState.Loading)
  val state: StateFlow<AccountState> = _state.asStateFlow()

  /** The last problem, for the screen to show. */
  private val _error = MutableStateFlow<String?>(null)
  val error: StateFlow<String?> = _error.asStateFlow()

  private lateinit var app: Context
  private lateinit var vault: SecureStore
  private var access: String? = null
  private var accessUntil = 0L
  private var userId: String? = null

  /** The signed-in user's id, if any. */
  val uid: String?
    get() = userId ?: (state.value as? AccountState.SignedIn)?.profile?.id

  private fun prefs() = app.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

  /**
   * Load the saved sign-in; call once at startup. Anyone who has signed in on
   * this phone opens straight into the app, online or not: what's saved here
   * decides the first screen, never the network. The server checks the
   * session in the background and only a real rejection (revoked, expired)
   * signs out. Being offline or slow never does.
   */
  fun init(context: Context) {
    if (::app.isInitialized) return
    app = context.applicationContext
    vault = SecureStore(app, "account")
    val saved = vault.read()
    val known = prefs().getBoolean("signed_in", false) || prefs().contains("profile")
    if (saved == null && !known) {
      _state.value = AccountState.SignedOut
      return
    }
    userId = prefs().getString("uid", null)
    _state.value = savedSignIn()
    if (saved != null) {
      scope.launch { lock.withLock { refresh(saved) } }
    } else {
      // The Keystore lost its key (it happens after some restores). Stay in
      // until the server can be asked; then a fresh sign-in is needed.
      Log.w(TAG, "saved session unreadable; keeping the cached profile until online")
    }
    watchNetwork()
  }

  /** The signed-in state built from what's saved, for offline use. */
  private fun savedSignIn(): AccountState.SignedIn {
    val cached =
      prefs().getString("profile", null)?.let { runCatching { json.decodeFromString(Profile.serializer(), it) }.getOrNull() }
    val profile =
      cached
        ?: Profile(
          id = prefs().getString("uid", null) ?: "",
          username = prefs().getString("username", null) ?: "",
          displayName = prefs().getString("display_name", null) ?: "You",
        )
    if (cached != null) userId = cached.id
    return AccountState.SignedIn(profile, prefs().getString("email", null), offline = true, max = prefs().getBoolean("max", false))
  }

  /** Check the session again whenever the phone comes back online. */
  private fun watchNetwork() {
    val cm = app.getSystemService(Context.CONNECTIVITY_SERVICE) as? android.net.ConnectivityManager ?: return
    runCatching {
      cm.registerDefaultNetworkCallback(
        object : android.net.ConnectivityManager.NetworkCallback() {
          override fun onAvailable(network: android.net.Network) {
            val now = _state.value
            if (now is AccountState.SignedIn && now.offline) {
              scope.launch {
                lock.withLock {
                  val saved = vault.read()
                  if (saved != null) {
                    refresh(saved)
                  } else if ((_state.value as? AccountState.SignedIn)?.offline == true) {
                    Log.i(TAG, "online, but no readable session: signing in again")
                    clearLocal()
                    _error.value = "Sign in again to keep your library in sync."
                  }
                }
              }
            }
          }
        },
      )
    }
  }

  // ── Signing in ──

  /** Open the browser on the sign-in page. */
  fun beginSignIn(context: Context) {
    val verifier = random(32)
    val challenge = b64(MessageDigest.getInstance("SHA-256").digest(verifier.toByteArray()))
    val state = random(24)
    prefs().edit().putString("pending_verifier", verifier).putString("pending_state", state).putLong("pending_at", System.currentTimeMillis()).apply()
    val url =
      "${Supa.AUTH_SITE}/auth/?android=${enc(BuildConfig.VELMORA_SCHEME)}&state=${enc(state)}&challenge=${enc(challenge)}"
    _error.value = null
    _state.value = AccountState.Waiting
    try {
      context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
    } catch (e: Exception) {
      _error.value = "Couldn't open your browser: ${e.message}"
      _state.value = AccountState.SignedOut
    }
  }

  /** Is this the sign-in coming back from the browser? Handles it if so. */
  fun handleLink(uri: Uri?): Boolean {
    if (uri == null || uri.scheme != BuildConfig.VELMORA_SCHEME || uri.host != "auth") return false
    val code = uri.getQueryParameter("code") ?: return true
    val state = uri.getQueryParameter("state")
    if (state == null || state != prefs().getString("pending_state", null)) {
      _error.value = "That sign-in link is from an older attempt. Try signing in again."
      return true
    }
    scope.launch { finish(code) }
    return true
  }

  /** Finish with the short code the sign-in page shows. */
  fun redeem(code: String) {
    val clean = code.filterNot { it.isWhitespace() || it == '-' }.uppercase()
    scope.launch { finish(clean) }
  }

  /** Stop waiting for the browser. */
  fun cancel() {
    if (_state.value == AccountState.Waiting) _state.value = AccountState.SignedOut
  }

  private suspend fun finish(code: String) {
    val verifier = prefs().getString("pending_verifier", null)
    if (verifier == null) {
      _error.value = "Start signing in first, then enter the code."
      return
    }
    try {
      val body = buildJsonObject {
        put("code", code)
        put("verifier", verifier)
      }
      val reply = Supa.request("POST", "${Supa.AUTH_SITE}/api/auth/exchange", body.toString(), supabase = false)
      val o = runCatching { json.parseToJsonElement(reply.body).jsonObject }.getOrNull()
      val accessToken = o?.str("access_token")
      val refreshToken = o?.str("refresh_token")
      if (o?.get("ok")?.jsonPrimitive?.contentOrNull != "true" || accessToken == null || refreshToken == null) {
        _error.value = o?.str("message") ?: "Sign-in couldn't finish. Try again."
        if (_state.value == AccountState.Waiting) _state.value = AccountState.SignedOut
        return
      }
      prefs().edit().remove("pending_verifier").remove("pending_state").apply()
      adopt(accessToken, refreshToken, expiresIn = null)
    } catch (e: CancellationException) {
      throw e
    } catch (e: Supa.Offline) {
      _error.value = "Couldn't reach Velmora's server. Check your connection and try again."
    } catch (e: Exception) {
      Log.w(TAG, "sign-in failed", e)
      _error.value = "Sign-in couldn't finish. Try again."
    }
  }

  // ── The session ──

  private suspend fun adopt(accessToken: String, refreshToken: String, expiresIn: Long?) {
    vault.write(refreshToken)
    access = accessToken
    val claims = jwtClaims(accessToken)
    accessUntil = (claims?.get("exp")?.jsonPrimitive?.longOrNull?.times(1000)) ?: (System.currentTimeMillis() + (expiresIn ?: 3600) * 1000)
    userId = claims?.str("sub")
    val email = claims?.str("email")
    // Remembered right away, so an app closed mid-sign-in still opens offline.
    prefs().edit().putBoolean("signed_in", true).putString("uid", userId).putString("email", email).apply()
    loadProfile(email)
  }

  /** Trade the saved refresh token for a fresh session. */
  private suspend fun refresh(saved: String) {
    try {
      val reply =
        Supa.request(
          "POST",
          "${Supa.BASE}/auth/v1/token?grant_type=refresh_token",
          buildJsonObject { put("refresh_token", saved) }.toString(),
        )
      if (rejected(reply)) {
        // Revoked or expired: signed out everywhere else, so here too.
        Log.i(TAG, "saved sign-in no longer valid (${reply.code})")
        clearLocal()
        _error.value = "Your sign-in expired. Sign in again."
        return
      }
      if (!reply.ok) throw Supa.Offline(IllegalStateException("HTTP ${reply.code}"))
      val o = json.parseToJsonElement(reply.body).jsonObject
      adopt(o.str("access_token") ?: return, o.str("refresh_token") ?: return, o["expires_in"]?.jsonPrimitive?.longOrNull)
    } catch (e: CancellationException) {
      throw e
    } catch (e: Exception) {
      // No internet, a timeout, a captive portal, a server hiccup: keep the
      // saved sign-in (offline) and check again when the network comes back.
      Log.i(TAG, "couldn't refresh the session: ${e.message}")
      if (_state.value !is AccountState.SignedIn) _state.value = savedSignIn()
    }
  }

  /**
   * Did the server actually refuse the session? Only Supabase's own answer
   * for a bad refresh token counts; a proxy's 403 or a rate limit doesn't.
   */
  private fun rejected(reply: Supa.Reply): Boolean {
    if (reply.code != 400 && reply.code != 401) return false
    val body = reply.body.lowercase()
    return "refresh_token" in body || "refresh token" in body || "invalid_grant" in body || "session_not_found" in body
  }

  /** A valid access token for Supabase calls, refreshed when needed; null when signed out. */
  suspend fun accessToken(): String? {
    lock.withLock {
      val now = System.currentTimeMillis()
      val current = access
      if (current != null && now < accessUntil - 60_000) return current
      val saved = vault.read() ?: return null
      refresh(saved)
      return access
    }
  }

  private suspend fun loadProfile(email: String?) {
    val token = access ?: return
    val id = userId ?: return
    val reply =
      Supa.request("GET", "${Supa.BASE}/rest/v1/profiles?id=eq.$id&select=id,username,display_name,avatar", token = token)
    val profile =
      runCatching { (json.parseToJsonElement(reply.body) as JsonArray).firstOrNull()?.let { json.decodeFromJsonElement(Profile.serializer(), it) } }
        .getOrNull()
    if (profile == null) {
      if (reply.ok) {
        // Signed in, but the account was never finished on the auth site.
        _error.value = "Your account has no profile yet. Finish setting it up on the sign-in page."
        if (!prefs().contains("profile")) {
          prefs().edit().remove("signed_in").apply()
          _state.value = AccountState.SignedOut
        }
      } else if (_state.value !is AccountState.SignedIn) {
        // The server is having a moment; the session itself is fine.
        _state.value = savedSignIn()
      }
      return
    }
    val max = isMax(id)
    prefs()
      .edit()
      .putString("profile", json.encodeToString(Profile.serializer(), profile))
      .putString("uid", profile.id)
      .putString("username", profile.username)
      .putString("display_name", profile.displayName)
      .putString("email", email)
      .putBoolean("max", max)
      .putBoolean("signed_in", true)
      .apply()
    _error.value = null
    _state.value = AccountState.SignedIn(profile, email, offline = false, max = max)
    // Settings, the vault key backup and the rest of what follows a sign-in.
    scope.launch { AccountSync.afterSignIn(app, id, token) }
  }

  /** Whether the account has Velmora Max (linked to a Max key on desktop). */
  private suspend fun isMax(id: String): Boolean =
    runCatching {
        val reply = Supa.request("GET", "${Supa.SITE}/api/social/max?ids=$id", supabase = false)
        val list = json.parseToJsonElement(reply.body).jsonObject["max"] as? JsonArray
        list?.any { it.jsonPrimitive.contentOrNull == id } == true
      }
      .getOrDefault(prefs().getBoolean("max", false))

  /** Sign out on this phone (and end this session on the server). */
  fun signOut() {
    val token = access
    scope.launch {
      if (token != null) runCatching { Supa.request("POST", "${Supa.BASE}/auth/v1/logout?scope=local", token = token) }
      clearLocal()
    }
  }

  private fun clearLocal() {
    vault.clear()
    access = null
    accessUntil = 0
    userId = null
    prefs()
      .edit()
      .remove("profile")
      .remove("email")
      .remove("max")
      .remove("uid")
      .remove("username")
      .remove("display_name")
      .remove("signed_in")
      .apply()
    _state.value = AccountState.SignedOut
  }

  // ── Helpers ──

  private fun JsonObject.str(key: String): String? = this[key]?.jsonPrimitive?.contentOrNull

  private fun jwtClaims(token: String): JsonObject? =
    runCatching {
        val part = token.split('.')[1]
        val bytes = Base64.decode(part, Base64.URL_SAFE or Base64.NO_PADDING or Base64.NO_WRAP)
        json.parseToJsonElement(String(bytes)).jsonObject
      }
      .getOrNull()

  private fun random(bytes: Int): String = b64(ByteArray(bytes).also { SecureRandom().nextBytes(it) })

  private fun b64(bytes: ByteArray): String = Base64.encodeToString(bytes, Base64.URL_SAFE or Base64.NO_PADDING or Base64.NO_WRAP)

  private fun enc(s: String): String = URLEncoder.encode(s, "UTF-8")
}

/** The avatar animals (same ids and colours as the website's). */
object Avatars {
  private val animals =
    listOf(
      "animal:panda" to Color(0xFF35CAA3),
      "animal:owl" to Color(0xFF7651F9),
      "animal:rabbit" to Color(0xFFDEB90E),
      "animal:chameleon" to Color(0xFF04ABA4),
      "animal:octopus" to Color(0xFFFC4185),
      "animal:dragon" to Color(0xFFDB4A2B),
      "animal:tiger" to Color(0xFFFB7E20),
      "animal:wolf" to Color(0xFF3F5C8D),
      "animal:cat" to Color(0xFF9BC933),
    )

  fun isPhoto(avatar: String): Boolean = avatar.startsWith("https://")

  /** The animal's colour (old "preset:N" values map onto the animals, as on the website). */
  fun color(avatar: String): Color {
    animals.firstOrNull { it.first == avatar }?.let {
      return it.second
    }
    val n = Regex("^preset:(\\d{1,2})$").find(avatar)?.groupValues?.get(1)?.toIntOrNull()
    return if (n != null) animals[(n - 1).mod(animals.size)].second else Color(0xFF3271FE)
  }

  /** "Owl", for the placeholder. */
  fun label(avatar: String): String = avatar.substringAfter("animal:", "").replaceFirstChar { it.uppercase() }
}
