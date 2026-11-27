package com.velmora.stepbyte.velmora.account

import java.io.IOException
import java.net.HttpURLConnection
import java.net.URL
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/**
 * The few HTTP calls Velmora accounts need: Supabase's auth and REST APIs, and
 * the auth site's handoff API. Same project and publishable key as the desktop
 * app and the website (app/src/lib/account.svelte.ts, site/src/auth/config.ts).
 * The publishable key is public by design; row-level security guards the data.
 */
internal object Supa {
  const val BASE = "https://htvirqckkhfekgfwxops.supabase.co"
  const val KEY = "sb_publishable_YSf01mWc6H2QFTxcBQcCMA_lElqBvgQ"
  const val AUTH_SITE = "https://auth.getvelmora.dpdns.org"
  const val SITE = "https://getvelmora.dpdns.org"
  const val SOCIAL = "https://social.getvelmora.dpdns.org"

  class Reply(val code: Int, val body: String) {
    val ok: Boolean
      get() = code in 200..299
  }

  /** Couldn't reach the server at all (offline, DNS, timeout). */
  class Offline(cause: Throwable) : IOException(cause.message, cause)

  /**
   * One request. [token] is the signed-in user's access token, for anything
   * behind row-level security; [supabase] adds the project key.
   */
  suspend fun request(
    method: String,
    url: String,
    body: String? = null,
    token: String? = null,
    supabase: Boolean = true,
    headers: Map<String, String> = emptyMap(),
  ): Reply =
    withContext(Dispatchers.IO) {
      val conn =
        try {
          URL(url).openConnection() as HttpURLConnection
        } catch (e: Exception) {
          throw Offline(e)
        }
      try {
        conn.requestMethod = method
        conn.connectTimeout = 15_000
        conn.readTimeout = 20_000
        conn.setRequestProperty("Accept", "application/json")
        if (supabase) conn.setRequestProperty("apikey", KEY)
        if (token != null) conn.setRequestProperty("Authorization", "Bearer $token")
        headers.forEach { (k, v) -> conn.setRequestProperty(k, v) }
        if (body != null) {
          conn.doOutput = true
          conn.setRequestProperty("Content-Type", "application/json")
          conn.outputStream.use { it.write(body.toByteArray()) }
        }
        val code = conn.responseCode
        val stream = if (code >= 400) conn.errorStream else conn.inputStream
        Reply(code, stream?.bufferedReader()?.use { it.readText() } ?: "")
      } catch (e: IOException) {
        throw Offline(e)
      } finally {
        conn.disconnect()
      }
    }
}
