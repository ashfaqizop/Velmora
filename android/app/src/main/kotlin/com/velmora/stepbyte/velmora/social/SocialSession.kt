package com.velmora.stepbyte.velmora.social

import android.content.Context
import android.net.Uri
import android.os.Handler
import android.os.Looper
import android.util.Base64
import android.util.Log
import android.webkit.CookieManager
import android.webkit.WebStorage
import com.velmora.stepbyte.velmora.account.AccountState
import com.velmora.stepbyte.velmora.account.Supa
import com.velmora.stepbyte.velmora.account.VelmoraAccount
import java.security.MessageDigest
import java.security.SecureRandom
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

/** Velmora Social's addresses, and the app route that shows them. */
object SocialLinks {
  const val HOST = "social.getvelmora.dpdns.org"
  const val ORIGIN = "https://$HOST"

  /** The in-app Social page; [to] is a path on the social site ("/", "/@name", "/title/movie/603"). */
  const val ROUTE = "v/social?to={to}"

  fun route(to: String = "/"): String = "v/social?to=" + Uri.encode(to)

  /** The full address of a social path. */
  fun url(to: String): String = if (to == "/" || to.isBlank() || to.startsWith("/social")) "$ORIGIN/social/" else ORIGIN + to

  /** A path on the social site the app shows in its own Social page. */
  fun isSocialPath(path: String): Boolean =
    path == "/" ||
      path.startsWith("/social") ||
      Regex("^/@[A-Za-z0-9_]{3,20}(/[a-z]+)?$").matches(path) ||
      Regex("^/(title/(movie|tv)/\\d{1,9}|list/(new|[0-9a-f-]{36})|theatre|leaderboards|search|settings|trending(/[a-z]+)?)$").matches(path)
}

/**
 * Signing Velmora Social in inside the app. The page runs on the website, so
 * it keeps its own session; the app never hands over its refresh token
 * (Supabase rotates those on use, and two holders would sign each other out).
 * Instead the app asks the auth API for a second, separate session of the same
 * account (`mint`), protected the same way as a normal sign-in (PKCE): the
 * page gets a one-time code and the secret in the address fragment, which
 * never reaches any server, and trades them for the session.
 *
 * The page's session then stays in the WebView's storage, so this happens
 * once per account, and again only if the site signs out.
 */
object SocialSession {
  private const val TAG = "VelmoraSocial"
  private const val PREFS = "velmora_social"
  private val json = Json { ignoreUnknownKeys = true }

  private fun prefs(c: Context) = c.applicationContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

  /** Who the web page was last signed in as, by account id. */
  private fun signedInAs(c: Context): String? = prefs(c).getString("uid", null)

  /**
   * The address to load for [to]. Signed in to the app and not yet on the
   * page: a hand-over address that signs the page in first.
   */
  suspend fun startUrl(context: Context, to: String, force: Boolean = false): String {
    val plain = SocialLinks.url(to)
    val signedIn = VelmoraAccount.state.value as? AccountState.SignedIn ?: return plain
    val uid = signedIn.profile.id
    if (!force && signedInAs(context) == uid) return plain
    val token = VelmoraAccount.accessToken() ?: return plain
    val verifier = random(32)
    val challenge = b64(MessageDigest.getInstance("SHA-256").digest(verifier.toByteArray()))
    return runCatching {
        val body =
          buildJsonObject {
              put("access_token", token)
              put("challenge", challenge)
              put("mint", true)
            }
            .toString()
        val reply = Supa.request("POST", "${Supa.SITE}/api/auth/handoff", body = body, supabase = false)
        val code = json.parseToJsonElement(reply.body).jsonObject["code"]?.jsonPrimitive?.contentOrNull
        if (!reply.ok || code == null) {
          Log.w(TAG, "handoff refused: ${reply.code}")
          plain
        } else {
          prefs(context).edit().putString("uid", uid).apply()
          "${SocialLinks.ORIGIN}/social/#vapp=${Uri.encode(code)}&vv=${Uri.encode(verifier)}&to=${Uri.encode(if (to.isBlank()) "/" else to)}"
        }
      }
      .getOrElse {
        Log.w(TAG, "handoff failed: ${it.message}")
        plain
      }
  }

  /** The page asked to sign in again (its session ended): next load hands over afresh. */
  fun lost(context: Context) = prefs(context).edit().remove("uid").apply()

  /**
   * Forget the web session too: signing out of the app signs Social out on
   * this phone, and a different account never sees the last one's page.
   */
  fun forget(context: Context) {
    prefs(context).edit().clear().apply()
    Handler(Looper.getMainLooper()).post {
      runCatching {
        WebStorage.getInstance().deleteAllData()
        CookieManager.getInstance().removeAllCookies(null)
        CookieManager.getInstance().flush()
      }
    }
  }

  private fun random(bytes: Int): String = b64(ByteArray(bytes).also { SecureRandom().nextBytes(it) })

  private fun b64(bytes: ByteArray): String = Base64.encodeToString(bytes, Base64.URL_SAFE or Base64.NO_PADDING or Base64.NO_WRAP)
}
