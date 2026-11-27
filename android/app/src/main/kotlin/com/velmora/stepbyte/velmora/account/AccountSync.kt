package com.velmora.stepbyte.velmora.account

import android.content.Context
import android.util.Base64
import android.util.Log
import com.velmora.stepbyte.engine.Universe
import java.io.File
import java.time.Instant
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

/**
 * What follows a sign-in: the account's settings come down into the engine,
 * and the download key is backed up (or restored). My List and watch progress
 * join when those screens exist (phases 5 and 6); the server side is the same
 * as desktop's (supabase/sync.sql).
 */
object AccountSync {
  private const val TAG = "AccountSync"
  private val json = Json { ignoreUnknownKeys = true }

  /** The engine preferences that sync between devices (app/src/lib/syncmerge.ts PREF_KEYS). */
  private val PREF_KEYS =
    setOf(
      "preferred_resolution",
      "max_auto_resolution",
      "preferred_codec",
      "audio_languages",
      "subtitle_languages",
      "show_adult",
      "include_torrents",
      "include_french",
      "provider_order",
      "disabled_providers",
      "verify_sources",
      "seed_after_download",
      "region",
    )

  suspend fun afterSignIn(context: Context, uid: String, token: String) {
    step("settings") { pullSettings(context, uid, token) }
    step("vault key") { backupVaultKey(context, uid, token) }
  }

  private suspend fun step(name: String, block: suspend () -> Unit) {
    try {
      block()
    } catch (e: CancellationException) {
      throw e
    } catch (e: Exception) {
      Log.w(TAG, "$name: ${e.message}")
    }
  }

  // ── Settings ──

  private suspend fun settingsRow(uid: String, token: String): JsonObject? {
    val reply = Supa.request("GET", "${Supa.BASE}/rest/v1/user_settings?user_id=eq.$uid&select=data,updated_at", token = token)
    if (!reply.ok) return null
    return (json.parseToJsonElement(reply.body) as? JsonArray)?.firstOrNull()?.jsonObject
  }

  /** Bring the account's languages, sources and filters into the engine. */
  private suspend fun pullSettings(context: Context, uid: String, token: String) {
    val data = settingsRow(uid, token)?.get("data") as? JsonObject ?: return
    val prefs = data["prefs"] as? JsonObject ?: return
    val synced = JsonObject(prefs.filterKeys { it in PREF_KEYS })
    if (synced.isNotEmpty()) {
      Universe.updatePreferences(context, synced)
      Log.i(TAG, "settings from the account: ${synced.keys.joinToString()}")
    }
  }

  /**
   * Save audio and subtitle languages to the engine and the account, keeping
   * everything else in the account's settings as it is.
   */
  suspend fun saveLanguages(context: Context, audio: List<String>, subtitles: List<String>) {
    val patch = buildJsonObject {
      put("audio_languages", JsonArray(audio.map { JsonPrimitive(it) }))
      put("subtitle_languages", JsonArray(subtitles.map { JsonPrimitive(it) }))
    }
    Universe.updatePreferences(context, patch)
    val uid = VelmoraAccount.uid ?: return
    val token = VelmoraAccount.accessToken() ?: return
    step("push settings") {
      val current = settingsRow(uid, token)?.get("data") as? JsonObject
      val prefs = (current?.get("prefs") as? JsonObject).orEmpty()
      val blob = buildMap<String, JsonElement> {
        putAll(current.orEmpty())
        put("v", JsonPrimitive(1))
        put("prefs", JsonObject(prefs + patch))
      }
      val body = buildJsonObject {
        put("new_data", JsonObject(blob))
        put("edited_at", Instant.now().toString())
      }
      val reply = Supa.request("POST", "${Supa.BASE}/rest/v1/rpc/sync_push_settings", body.toString(), token = token)
      if (!reply.ok) Log.w(TAG, "push settings: ${reply.code} ${reply.body.take(200)}")
    }
  }

  // ── The download key ──

  private fun keyFile(context: Context) = File(context.noBackupFilesDir, "vault/key.bin")

  private fun downloadsExist(context: Context): Boolean {
    val dir = context.getExternalFilesDir(android.os.Environment.DIRECTORY_MOVIES) ?: return false
    return dir.walkTopDown().any { it.isFile && it.extension.equals("vlm", ignoreCase = true) }
  }

  /**
   * Keep the download key in the account. If the account has one and this
   * install has no downloads yet, take the account's (a reinstall: the old
   * downloads open again). If the account has none, back this one up.
   */
  private suspend fun backupVaultKey(context: Context, uid: String, token: String) {
    val reply = Supa.request("GET", "${Supa.BASE}/rest/v1/vault_keys?platform=eq.android&select=key", token = token)
    if (!reply.ok) {
      // Most likely supabase/vault.sql hasn't been run yet.
      Log.w(TAG, "vault key: ${reply.code}")
      return
    }
    val remote = (json.parseToJsonElement(reply.body) as? JsonArray)?.firstOrNull()?.jsonObject?.get("key")?.jsonPrimitive?.contentOrNull
    val file = keyFile(context)
    val local = file.takeIf { it.isFile && it.length() == 32L }?.readBytes()

    if (remote != null) {
      val bytes = runCatching { Base64.decode(remote, Base64.NO_WRAP) }.getOrNull()
      if (bytes == null || bytes.size != 32) return
      if (local != null && local.contentEquals(bytes)) return
      if (local != null && downloadsExist(context)) {
        // Both have downloads behind them; keep this phone's and say so in the log.
        Log.w(TAG, "vault key differs from the account's; keeping this install's (it has downloads)")
        return
      }
      file.parentFile?.mkdirs()
      file.writeBytes(bytes)
      Log.i(TAG, "download key restored from the account (used from the next start)")
      return
    }

    val key =
      local
        ?: ByteArray(32).also {
          java.security.SecureRandom().nextBytes(it)
          file.parentFile?.mkdirs()
          file.writeBytes(it)
        }
    val body = buildJsonObject {
      put("user_id", uid)
      put("platform", "android")
      put("key", Base64.encodeToString(key, Base64.NO_WRAP))
    }
    val up =
      Supa.request(
        "POST",
        "${Supa.BASE}/rest/v1/vault_keys?on_conflict=user_id,platform",
        body.toString(),
        token = token,
        headers = mapOf("Prefer" to "resolution=merge-duplicates,return=minimal"),
      )
    if (up.ok) Log.i(TAG, "download key backed up to the account") else Log.w(TAG, "vault key backup: ${up.code} ${up.body.take(200)}")
  }
}
