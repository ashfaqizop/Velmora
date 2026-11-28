package com.velmora.stepbyte.velmora.social

import android.content.Context
import android.util.Log
import com.velmora.stepbyte.engine.Target
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.velmora.account.AccountState
import com.velmora.stepbyte.velmora.account.Supa
import com.velmora.stepbyte.velmora.account.VelmoraAccount
import com.velmora.stepbyte.velmora.privacy.UsageStats
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

/**
 * What you watch, shared on your Velmora Social profile, the same as desktop
 * (app/src/lib/activity.svelte.ts): started, finished, added to My List. Only
 * while signed in and with sharing on (Settings > Velmora > Privacy, or Edit
 * profile on the web; it's one setting on the account). 18+ titles and
 * anything without a TMDB number are never sent. Posts wait on the phone while
 * offline and go out later. Server: post_activity in supabase/activity.sql.
 */
object ActivityPoster {
  private const val TAG = "VelmoraActivity"
  private const val PREFS = "velmora_activity"
  private const val KEEP_MS = 3 * 86_400_000L
  private val json = Json { ignoreUnknownKeys = true }
  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
  private val lock = Mutex()

  @Serializable
  private data class Post(
    val what: String,
    val k: String,
    val season: Int? = null,
    val episode: Int? = null,
    val minutes: Int = 0,
    val at: Long,
    /** Whose it is, so a post never goes out under someone else's account. */
    val uid: String,
  )

  private lateinit var app: Context
  private val _sharing = MutableStateFlow(true)
  /** The account's "share what I watch" setting. */
  val sharing: StateFlow<Boolean> = _sharing.asStateFlow()

  private fun prefs() = app.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

  fun init(context: Context) {
    if (::app.isInitialized) return
    app = context.applicationContext
    _sharing.value = prefs().getBoolean("share", true)
    // Read the setting from the account whenever someone signs in.
    scope.launch {
      VelmoraAccount.state
        .map { (it as? AccountState.SignedIn)?.takeIf { s -> !s.offline }?.profile?.id }
        .distinctUntilChanged()
        .collect { uid ->
          if (uid == null) return@collect
          loadSetting(uid)
          flush()
        }
    }
    scope.launch {
      while (true) {
        delay(5 * 60_000L)
        flush()
      }
    }
  }

  private suspend fun loadSetting(uid: String) {
    val token = VelmoraAccount.accessToken() ?: return
    runCatching {
      val r = Supa.request("GET", "${Supa.BASE}/rest/v1/profiles?id=eq.$uid&select=share_activity", token = token)
      if (!r.ok) return
      val on = (json.parseToJsonElement(r.body) as? JsonArray)?.firstOrNull()?.jsonObject?.get("share_activity")?.jsonPrimitive?.booleanOrNull ?: true
      _sharing.value = on
      prefs().edit().putBoolean("share", on).apply()
    }
  }

  /** Turn sharing on or off on the account (the web's Edit profile shows the same switch). */
  suspend fun setSharing(on: Boolean): Boolean {
    val before = _sharing.value
    _sharing.value = on
    prefs().edit().putBoolean("share", on).apply()
    if (!on) clear()
    val uid = VelmoraAccount.uid ?: return true
    val token = VelmoraAccount.accessToken() ?: return true
    val ok =
      runCatching {
          val body = buildJsonObject { put("share_activity", on) }.toString()
          Supa.request("PATCH", "${Supa.BASE}/rest/v1/profiles?id=eq.$uid", body = body, token = token, headers = mapOf("Prefer" to "return=minimal")).ok
        }
        .getOrDefault(false)
    if (!ok) {
      _sharing.value = before
      prefs().edit().putBoolean("share", before).apply()
    }
    return ok
  }

  /** A minute into something. */
  fun watching(title: Title, target: Target) = add("watching", title, target)

  /** Watched to the end (95%). */
  fun finished(title: Title, target: Target, durationSeconds: Double) =
    add("finished", title, target, (durationSeconds / 60).coerceAtMost(600.0).toInt())

  /** Added to My List. */
  fun listed(title: Title) = add("listed", title, Target.Movie)

  private fun add(what: String, title: Title, target: Target, minutes: Int = 0) {
    if (!::app.isInitialized || !_sharing.value) return
    val signedIn = VelmoraAccount.state.value as? AccountState.SignedIn ?: return
    if (title.value.badges.any { it == "18+" }) return
    val k = UsageStats.titleKey(title) ?: return
    val ep = target as? Target.Episode
    scope.launch {
      lock.withLock {
        write((read() + Post(what, k, ep?.season, ep?.episode, minutes, System.currentTimeMillis(), signedIn.profile.id)).takeLast(100))
      }
      flush()
    }
  }

  suspend fun flush() {
    if (!::app.isInitialized) return
    val me = (VelmoraAccount.state.value as? AccountState.SignedIn)?.profile?.id ?: return
    lock.withLock {
      var queue = read().filter { it.uid == me && System.currentTimeMillis() - it.at < KEEP_MS }
      if (queue.isEmpty()) return
      val token = VelmoraAccount.accessToken() ?: return
      while (queue.isNotEmpty()) {
        val p = queue.first()
        val body =
          buildJsonObject {
              put("what", p.what)
              put("title", p.k)
              if (p.season != null) put("season_no", p.season) else put("season_no", JsonNull)
              if (p.episode != null) put("episode_no", p.episode) else put("episode_no", JsonNull)
              put("watched_minutes", p.minutes)
            }
            .toString()
        val sent = runCatching { Supa.request("POST", "${Supa.BASE}/rest/v1/rpc/post_activity", body = body, token = token) }
        // Offline keeps the rest for later; a refusal drops just this one.
        if (sent.isFailure) break
        if (!sent.getOrThrow().ok) Log.w(TAG, "post_activity refused: ${sent.getOrThrow().code}")
        queue = queue.drop(1)
        write(queue)
      }
    }
  }

  /** Signing out or turning sharing off forgets what hasn't gone out yet. */
  fun clear() {
    if (::app.isInitialized) prefs().edit().remove("queue").apply()
  }

  private fun read(): List<Post> =
    runCatching { json.decodeFromString(ListSerializer(Post.serializer()), prefs().getString("queue", null) ?: "[]") }.getOrDefault(emptyList())

  private fun write(q: List<Post>) {
    prefs().edit().putString("queue", json.encodeToString(ListSerializer(Post.serializer()), q)).apply()
  }
}
