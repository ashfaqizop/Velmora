package com.velmora.stepbyte.velmora.theatre

import com.velmora.stepbyte.velmora.account.AccountState
import com.velmora.stepbyte.velmora.account.Supa
import com.velmora.stepbyte.velmora.account.VelmoraAccount
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import kotlinx.serialization.json.put

data class ChatMessage(
  val id: String,
  val uid: String,
  val username: String,
  val displayName: String,
  val avatar: String,
  val verified: Boolean,
  val body: String,
  /** Movie time when it was sent, ms; null in the lobby. */
  val at: Long?,
  /** It mentions you. */
  val mention: Boolean,
  val mine: Boolean,
)

data class Watcher(val uid: String, val username: String, val displayName: String, val avatar: String, val verified: Boolean, val at: Long)

/** A reaction landing: which emoji and who sent it. */
data class Reaction(val emoji: String, val from: String, val nonce: Long = System.nanoTime())

/**
 * A screening's live room: who's watching, the chat and the reactions. Two
 * private Realtime channels, as on desktop (rules in supabase/theatre.sql):
 * `theatre:<id>` for presence and reactions, `theatre-chat:<id>` to listen to
 * the chat that `theatre_say` sends after its checks. Nothing is stored; the
 * chat lives in memory while the room is open.
 */
object TheatreRoom {
  val REACTIONS = listOf("😂", "😮", "😭", "🔥", "❤️", "👏", "😱")
  private const val MAX_MESSAGES = 200
  private const val QUIET_RETRY_MS = 2 * 60_000L

  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
  private val realtime = Realtime { VelmoraAccount.accessToken() }

  private val _messages = MutableStateFlow<List<ChatMessage>>(emptyList())
  val messages: StateFlow<List<ChatMessage>> = _messages.asStateFlow()

  private val _watchers = MutableStateFlow<List<Watcher>>(emptyList())
  val watchers: StateFlow<List<Watcher>> = _watchers.asStateFlow()

  private val _connected = MutableStateFlow(false)
  val connected: StateFlow<Boolean> = _connected.asStateFlow()

  /** The room is full: watching in sync without chat for now. */
  private val _quiet = MutableStateFlow(false)
  val quiet: StateFlow<Boolean> = _quiet.asStateFlow()

  private val _notice = MutableStateFlow<String?>(null)
  val notice: StateFlow<String?> = _notice.asStateFlow()

  private val _mode = MutableStateFlow("normal")
  val mode: StateFlow<String> = _mode.asStateFlow()

  private val _sending = MutableStateFlow(false)
  val sending: StateFlow<Boolean> = _sending.asStateFlow()

  private val _reactions = MutableSharedFlow<Reaction>(extraBufferCapacity = 64)
  val reactions: SharedFlow<Reaction> = _reactions.asSharedFlow()

  private val _muted = MutableStateFlow<Set<String>>(emptySet())
  val muted: StateFlow<Set<String>> = _muted.asStateFlow()

  @Volatile var id: String? = null
    private set

  private var capacity = 1000
  private var joinedAt = 0L
  private var lastReact = 0L
  private var room: Realtime.Channel? = null
  private var chat: Realtime.Channel? = null
  private var quietJob: Job? = null

  private fun me() = (VelmoraAccount.state.value as? AccountState.SignedIn)?.profile

  fun join(screeningId: String, cap: Int, mode: String) {
    if (id == screeningId && (_connected.value || _quiet.value)) return
    leave()
    id = screeningId
    capacity = cap.coerceAtLeast(1)
    _mode.value = mode
    _messages.value = emptyList()
    _muted.value = emptySet()
    _notice.value = null
    joinedAt = System.currentTimeMillis()
    connect()
  }

  private fun connect() {
    val me = me() ?: return
    val sid = id ?: return
    val r = realtime.channel("theatre:$sid", isPrivate = true, presenceKey = me.id, self = true)
    r.onStatus = { status ->
      scope.launch {
        _connected.value = status == "joined"
        if (status.startsWith("error")) _notice.value = "Chat isn't connected. Watching still works."
        if (status == "joined") _notice.value = null
      }
    }
    r.onPresence = { state -> scope.launch { syncPresence(state) } }
    r.onBroadcast = { event, payload ->
      if (event == "react") {
        val e = payload["e"]?.jsonPrimitive?.contentOrNull
        val u = payload["u"]?.jsonPrimitive?.contentOrNull ?: ""
        if (e != null && e in REACTIONS && u !in _muted.value) _reactions.tryEmit(Reaction(e, u))
      }
    }
    r.track(
      buildJsonObject {
        put("uid", me.id)
        put("username", me.username)
        put("display_name", me.displayName)
        put("avatar", me.avatar)
        put("verified", false)
        put("at", joinedAt)
      },
    )
    val c = realtime.channel("theatre-chat:$sid", isPrivate = true, presenceKey = null, self = true)
    c.onBroadcast = { event, payload ->
      scope.launch {
        when (event) {
          "chat" -> receive(payload)
          "mode" -> {
            val m = payload["mode"]?.jsonPrimitive?.contentOrNull
            if (m == "normal" || m == "slow" || m == "closed") {
              _mode.value = m
              _notice.value =
                when (m) {
                  "closed" -> "Text chat was turned off for this screening. Reactions still work."
                  "slow" -> "Slow mode is on: one message every 10 seconds."
                  else -> "Chat is back to normal."
                }
            }
          }
        }
      }
    }
    room = r
    chat = c
  }

  private fun syncPresence(state: Map<String, List<JsonObject>>) {
    val seen = linkedMapOf<String, Watcher>()
    for (metas in state.values) {
      for (m in metas) {
        val uid = m["uid"]?.jsonPrimitive?.contentOrNull ?: continue
        if (uid in seen) continue
        seen[uid] =
          Watcher(
            uid = uid,
            username = m["username"]?.jsonPrimitive?.contentOrNull ?: "",
            displayName = m["display_name"]?.jsonPrimitive?.contentOrNull ?: "",
            avatar = m["avatar"]?.jsonPrimitive?.contentOrNull ?: "animal:panda",
            verified = m["verified"]?.jsonPrimitive?.booleanOrNull ?: false,
            at = m["at"]?.jsonPrimitive?.longOrNull ?: Long.MAX_VALUE,
          )
      }
    }
    val friends = Theatre.following
    _watchers.value = seen.values.sortedWith(compareByDescending<Watcher> { it.uid in friends }.thenBy { it.at })
    // Over the room's size: the latest arrivals watch without chat for now,
    // which keeps Velmora inside its live-connection budget.
    val order = seen.values.sortedBy { it.at }
    val mine = order.indexOfFirst { it.uid == me()?.id }
    if (mine >= capacity) goQuiet()
  }

  private fun goQuiet() {
    _quiet.value = true
    val sid = id
    disconnect()
    quietJob?.cancel()
    quietJob =
      scope.launch {
        delay(QUIET_RETRY_MS)
        if (id != sid) return@launch
        _quiet.value = false
        connect()
      }
  }

  private fun receive(p: JsonObject) {
    val mid = p["id"]?.jsonPrimitive?.contentOrNull ?: return
    val body = p["body"]?.jsonPrimitive?.contentOrNull ?: return
    val uid = p["uid"]?.jsonPrimitive?.contentOrNull ?: ""
    if (uid in _muted.value) return
    if (_messages.value.any { it.id == mid }) return
    val me = me()
    val name = me?.username?.lowercase()
    val mention = name != null && uid != me?.id && Regex("(^|\\W)@${Regex.escape(name)}(\\W|$)", RegexOption.IGNORE_CASE).containsMatchIn(body)
    val msg =
      ChatMessage(
        id = mid,
        uid = uid,
        username = p["username"]?.jsonPrimitive?.contentOrNull ?: "",
        displayName = p["display_name"]?.jsonPrimitive?.contentOrNull ?: "",
        avatar = p["avatar"]?.jsonPrimitive?.contentOrNull ?: "animal:panda",
        verified = p["verified"]?.jsonPrimitive?.booleanOrNull ?: false,
        body = body,
        at = (p["at"] as? kotlinx.serialization.json.JsonPrimitive)?.longOrNull,
        mention = mention,
        mine = uid == me?.id,
      )
    _messages.value = (_messages.value + msg).takeLast(MAX_MESSAGES)
  }

  private fun friendly(body: String): String {
    val m = body.lowercase()
    val details = Regex("\"details\"\\s*:\\s*\"?(\\d+)").find(body)?.groupValues?.get(1)
    return when {
      "chat_closed" in m -> "Text chat is off for this screening. Reactions still work."
      "chat_muted" in m -> "You've been muted in this room."
      "chat_slow" in m -> "Slow mode: wait ${details ?: "a few"} s."
      "chat_link" in m -> "Links can't be posted in Theatre chat."
      "chat_blocked" in m -> "That message has a word that isn't allowed."
      "chat_repeat" in m -> "You just said that."
      "chat_length" in m -> "Messages can be up to 200 characters."
      "not_in_screening" in m -> "The chat for this screening is closed."
      else -> "Couldn't send. Try again."
    }
  }

  /** Send through `theatre_say` (which checks and broadcasts). True when it went. */
  suspend fun say(text: String, movieMs: Long?): Boolean {
    val body = text.trim()
    val sid = id ?: return false
    if (body.isEmpty() || _sending.value) return false
    _sending.value = true
    _notice.value = null
    return try {
      val token = VelmoraAccount.accessToken() ?: return false.also { _notice.value = "Sign in to chat." }
      val reply =
        Supa.request(
          "POST",
          "${Supa.BASE}/rest/v1/rpc/theatre_say",
          buildJsonObject {
            put("sid", sid)
            put("body", body)
            if (movieMs != null) put("movie_ms", movieMs) else put("movie_ms", JsonNull)
          }.toString(),
          token = token,
        )
      if (!reply.ok) {
        _notice.value = friendly(reply.body)
        false
      } else {
        true
      }
    } catch (e: Supa.Offline) {
      _notice.value = "You seem to be offline."
      false
    } finally {
      _sending.value = false
    }
  }

  fun react(emoji: String) {
    val now = System.currentTimeMillis()
    val r = room ?: return
    if (!_connected.value || now - lastReact < 600) return
    lastReact = now
    r.broadcast("react", buildJsonObject {
      put("e", emoji)
      put("u", me()?.id ?: "")
    })
  }

  fun mute(uid: String, on: Boolean) {
    _muted.value = if (on) _muted.value + uid else _muted.value - uid
    if (on) _messages.value = _messages.value.filterNot { it.uid == uid }
  }

  private fun disconnect() {
    room?.leave()
    chat?.leave()
    room = null
    chat = null
    _connected.value = false
  }

  fun leave() {
    quietJob?.cancel()
    disconnect()
    id = null
    _quiet.value = false
    _watchers.value = emptyList()
    _messages.value = emptyList()
  }
}
