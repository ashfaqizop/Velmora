package com.velmora.stepbyte.velmora.theatre

import android.content.Context
import android.util.Log
import androidx.work.ExistingWorkPolicy
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.workDataOf
import com.velmora.stepbyte.velmora.account.AccountState
import com.velmora.stepbyte.velmora.account.Supa
import com.velmora.stepbyte.velmora.account.VelmoraAccount
import java.net.URLEncoder
import java.time.Instant
import java.time.OffsetDateTime
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

/** One screening, as `theatre_screenings` has it (spec: docs/theatre.md). */
@Serializable
data class Screening(
  val id: String,
  val day: String = "",
  /** "movie/603": the TMDB title. */
  val k: String,
  val name: String,
  val year: Int? = null,
  val poster: String? = null,
  val backdrop: String? = null,
  val runtime: Int = 0,
  /** trending or pick (Tonight's pick). */
  val kind: String = "trending",
  @SerialName("starts_at") val startsAt: String,
  @SerialName("ends_at") val endsAt: String,
  /** scheduled or cancelled. */
  val status: String = "scheduled",
  /** open (walk in) or fixed (ticket needed). */
  val mode: String = "open",
  val capacity: Int = 0,
  val booked: Int = 0,
  val waitlisted: Int = 0,
  @SerialName("opens_at") val opensAt: String? = null,
  @SerialName("max_opens_at") val maxOpensAt: String? = null,
  val special: Boolean = false,
  val note: String = "",
  @SerialName("chat_mode") val chatMode: String = "normal",
) {
  val startMs: Long
    get() = parse(startsAt)

  val endMs: Long
    get() = parse(endsAt)

  val pick: Boolean
    get() = kind == "pick"

  val fixed: Boolean
    get() = mode == "fixed"

  val seatsLeft: Int
    get() = (capacity - booked).coerceAtLeast(0)
}

@Serializable data class Friend(val id: String, val username: String = "", @SerialName("display_name") val displayName: String = "", val avatar: String = "animal:panda")

@Serializable data class TheatreSettings(val paused: Boolean = false, val banner: String = "", val reminder: String = "")

/** Where a screening is in its life. */
enum class ShowPhase { Upcoming, Lobby, Live, After, Over }

fun parse(iso: String?): Long = iso?.let { runCatching { OffsetDateTime.parse(it).toInstant().toEpochMilli() }.getOrNull() } ?: 0L

/**
 * Theatre on this phone: the lineup, your tickets, who you follow is going,
 * the server's clock, booking and cancelling, entering, and reminders before
 * the lobby opens and the film starts. The same tables and functions as
 * desktop and the web (supabase/theatre.sql).
 */
object Theatre {
  private const val TAG = "VelmoraTheatre"
  const val LOBBY_MS = 30 * 60_000L
  const val AFTER_MS = 30 * 60_000L
  private const val COLUMNS =
    "id,day,k,name,year,poster,backdrop,runtime,kind,starts_at,ends_at,status,mode,capacity,booked,waitlisted,opens_at,max_opens_at,special,note,chat_mode"

  private val json = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
    coerceInputValues = true
  }
  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
  private val lock = Mutex()
  private lateinit var app: Context

  private val _lineup = MutableStateFlow<List<Screening>>(emptyList())
  val lineup: StateFlow<List<Screening>> = _lineup.asStateFlow()

  /** Your ticket per screening id: booked, waitlist, cancelled, released, attended, noshow. */
  private val _tickets = MutableStateFlow<Map<String, String>>(emptyMap())
  val tickets: StateFlow<Map<String, String>> = _tickets.asStateFlow()

  private val _friends = MutableStateFlow<Map<String, List<Friend>>>(emptyMap())
  val friends: StateFlow<Map<String, List<Friend>>> = _friends.asStateFlow()

  /** Everyone you follow, for "friends first" in a room. */
  @Volatile var following: Set<String> = emptySet()
    private set

  private val _settings = MutableStateFlow(TheatreSettings())
  val settings: StateFlow<TheatreSettings> = _settings.asStateFlow()

  private val _loaded = MutableStateFlow(false)
  val loaded: StateFlow<Boolean> = _loaded.asStateFlow()

  private val _failed = MutableStateFlow(false)
  val failed: StateFlow<Boolean> = _failed.asStateFlow()

  private val _busy = MutableStateFlow<String?>(null)
  val busy: StateFlow<String?> = _busy.asStateFlow()

  /** Server time minus this phone's, in ms. */
  @Volatile var offset = 0L
    private set

  /** Ticks every few seconds so phases and countdowns move. */
  private val _now = MutableStateFlow(System.currentTimeMillis())
  val now: StateFlow<Long> = _now.asStateFlow()

  private var started = false

  fun serverNow() = System.currentTimeMillis() + offset

  fun start(context: Context) {
    if (started) return
    started = true
    app = context.applicationContext
    scope.launch {
      while (isActive) {
        _now.value = serverNow()
        delay(1_000)
      }
    }
    scope.launch {
      while (isActive) {
        load()
        delay(5 * 60_000)
      }
    }
    // Reload when the account comes online (tickets are per account).
    scope.launch { VelmoraAccount.state.collect { if (it is AccountState.SignedIn && !it.offline) load() } }
    scope.launch {
      delay(3 * 60_000)
      while (isActive) {
        Probes.run(app)
        delay(4 * 3_600_000L)
      }
    }
  }

  fun phase(s: Screening, at: Long = serverNow()): ShowPhase =
    when {
      at < s.startMs - LOBBY_MS -> ShowPhase.Upcoming
      at < s.startMs -> ShowPhase.Lobby
      at < s.endMs -> ShowPhase.Live
      at < s.endMs + AFTER_MS -> ShowPhase.After
      else -> ShowPhase.Over
    }

  /** When booking opens for you (Max earlier), or null when it's open. */
  fun opensAt(s: Screening): Long? {
    val max = (VelmoraAccount.state.value as? AccountState.SignedIn)?.max == true
    val t = parse(if (max) s.maxOpensAt ?: s.opensAt else s.opensAt)
    return if (t > serverNow()) t else null
  }

  fun ticket(s: Screening): String? = _tickets.value[s.id]

  /** Booked or waitlisted for one that hasn't finished. */
  fun holding(s: Screening): Boolean {
    val t = ticket(s)
    return (t == "booked" || t == "waitlist") && s.status == "scheduled" && phase(s) != ShowPhase.Over
  }

  /** On now or next. */
  fun current(list: List<Screening> = _lineup.value): List<Screening> = list.filter { it.status == "scheduled" && phase(it) != ShowPhase.Over }

  /** The screening of this title that hasn't finished ("movie/603"), if any. */
  fun forTitle(k: String?): Screening? = k?.let { key -> current().firstOrNull { it.k == key } }

  // ── Loading ──

  suspend fun load() {
    if (!::app.isInitialized) return
    lock.withLock {
      try {
        val token = VelmoraAccount.accessToken()
        val since = Instant.ofEpochMilli(serverNow() - 4 * 3_600_000L - AFTER_MS).toString()
        coroutineScope {
          val clock = async { syncClock() }
          val lineup = async {
            Supa.request("GET", "${Supa.BASE}/rest/v1/theatre_screenings?select=$COLUMNS&ends_at=gte.${enc(since)}&order=starts_at.asc&limit=60", token = token)
          }
          val settings = async { runCatching { Supa.request("GET", "${Supa.BASE}/rest/v1/theatre_settings?select=paused,banner,reminder&limit=1", token = token) }.getOrNull() }
          clock.await()
          val reply = lineup.await()
          if (!reply.ok) throw IllegalStateException("lineup: HTTP ${reply.code}")
          val list = json.decodeFromString<List<Screening>>(reply.body)
          settings.await()?.takeIf { it.ok }?.let { s ->
            runCatching { json.decodeFromString<List<TheatreSettings>>(s.body).firstOrNull() }.getOrNull()?.let { _settings.value = it }
          }
          val before = _lineup.value
          _lineup.value = list
          _now.value = serverNow()
          loadTickets(list, token)
          loadFriends(list, token)
          announce(before, list)
          schedule(list)
        }
        _failed.value = false
      } catch (e: CancellationException) {
        throw e
      } catch (e: Exception) {
        Log.i(TAG, "lineup didn't load: ${e.message}")
        if (!_loaded.value) _failed.value = true
      } finally {
        _loaded.value = true
      }
    }
  }

  /** Ask the server for its time a few times; keep the quickest answer. */
  private suspend fun syncClock() {
    var best: Pair<Long, Long>? = null
    repeat(3) {
      val sent = System.currentTimeMillis()
      val reply = runCatching { Supa.request("POST", "${Supa.BASE}/rest/v1/rpc/theatre_now", "{}") }.getOrNull() ?: return@repeat
      val got = System.currentTimeMillis()
      if (!reply.ok) return@repeat
      val server = parse(reply.body.trim().trim('"'))
      if (server == 0L) return@repeat
      val rtt = got - sent
      val off = server + rtt / 2 - got
      if (best == null || rtt < best!!.first) best = rtt to off
    }
    best?.let { offset = it.second }
  }

  private fun me(): String? = VelmoraAccount.uid?.takeIf { it.isNotEmpty() }

  private suspend fun loadTickets(list: List<Screening>, token: String?) {
    val me = me()
    if (me == null || token == null || list.isEmpty()) {
      _tickets.value = emptyMap()
      return
    }
    val ids = list.joinToString(",") { it.id }
    val reply = Supa.request("GET", "${Supa.BASE}/rest/v1/theatre_tickets?select=screening_id,status&user_id=eq.$me&screening_id=in.(${enc(ids)})", token = token)
    if (!reply.ok) return
    val next =
      (json.parseToJsonElement(reply.body) as? JsonArray)?.mapNotNull { el ->
        val o = el as? JsonObject ?: return@mapNotNull null
        val sid = o["screening_id"]?.jsonPrimitive?.contentOrNull ?: return@mapNotNull null
        sid to (o["status"]?.jsonPrimitive?.contentOrNull ?: return@mapNotNull null)
      }?.toMap() ?: return
    // Off the waitlist since last time: say so.
    val before = _tickets.value
    for ((id, status) in next) {
      if (before[id] == "waitlist" && status == "booked") {
        list.firstOrNull { it.id == id }?.let { TheatreAlerts.notify(app, "You got a seat", "A seat freed up for ${it.name}. You're in.", "seat:$id", it.id) }
      }
    }
    _tickets.value = next
  }

  private suspend fun loadFriends(list: List<Screening>, token: String?) {
    val me = me()
    if (me == null || token == null || list.isEmpty()) {
      _friends.value = emptyMap()
      return
    }
    val follows = Supa.request("GET", "${Supa.BASE}/rest/v1/follows?select=followee&follower=eq.$me&limit=1000", token = token)
    val ids = (json.parseToJsonElement(follows.body.ifBlank { "[]" }) as? JsonArray)?.mapNotNull { (it as? JsonObject)?.get("followee")?.jsonPrimitive?.contentOrNull }.orEmpty()
    following = ids.toSet()
    if (ids.isEmpty()) {
      _friends.value = emptyMap()
      return
    }
    val sids = list.joinToString(",") { it.id }
    val who = ids.take(300).joinToString(",")
    val reply =
      Supa.request(
        "GET",
        "${Supa.BASE}/rest/v1/theatre_tickets?select=screening_id,p:profiles!theatre_tickets_user_id_fkey(id,username,display_name,avatar)&screening_id=in.(${enc(sids)})&user_id=in.(${enc(who)})&status=in.(booked,attended)",
        token = token,
      )
    if (!reply.ok) return
    val out = mutableMapOf<String, MutableList<Friend>>()
    (json.parseToJsonElement(reply.body) as? JsonArray)?.forEach { el ->
      val o = el as? JsonObject ?: return@forEach
      val sid = o["screening_id"]?.jsonPrimitive?.contentOrNull ?: return@forEach
      val p = (o["p"] as? JsonObject)?.let { runCatching { json.decodeFromJsonElement(Friend.serializer(), it) }.getOrNull() } ?: return@forEach
      out.getOrPut(sid) { mutableListOf() } += p
    }
    _friends.value = out
  }

  // ── Booking ──

  /** The server's refusals, in plain words. */
  fun friendly(body: String): String {
    val m = body.lowercase()
    return when {
      "paused" in m -> "Theatre is paused for a bit. Booking will be back soon."
      "not_open_yet" in m -> "Booking for this screening hasn't opened yet."
      "not_bookable" in m -> "This screening isn't taking bookings."
      "over" in m -> "This screening has finished."
      "sign_in" in m -> "Sign in to book."
      "not_allowed" in m -> "Your account can't book screenings right now."
      "not_found" in m -> "That screening isn't on any more."
      else -> "Something went wrong. Try again."
    }
  }

  /** Book a seat (or join the waitlist). Returns the message to show. */
  suspend fun book(s: Screening): String {
    val token = VelmoraAccount.accessToken() ?: return "Sign in to book."
    _busy.value = s.id
    return try {
      val reply = Supa.request("POST", "${Supa.BASE}/rest/v1/rpc/theatre_book", buildJsonObject { put("sid", s.id) }.toString(), token = token)
      if (!reply.ok) return friendly(reply.body)
      when (reply.body.trim().trim('"')) {
        "waitlist" -> {
          _tickets.value = _tickets.value + (s.id to "waitlist")
          "Sold out. You're on the waitlist for ${s.name}; Velmora tells you if a seat frees up."
        }
        else -> {
          _tickets.value = _tickets.value + (s.id to "booked")
          "Booked. Velmora reminds you 30 minutes before ${s.name}."
        }
      }.also { scope.launch { load() } }
    } catch (e: Supa.Offline) {
      "You seem to be offline."
    } finally {
      _busy.value = null
    }
  }

  suspend fun cancel(s: Screening): String {
    val token = VelmoraAccount.accessToken() ?: return "Sign in first."
    _busy.value = s.id
    return try {
      val reply = Supa.request("POST", "${Supa.BASE}/rest/v1/rpc/theatre_cancel", buildJsonObject { put("sid", s.id) }.toString(), token = token)
      if (!reply.ok) return friendly(reply.body)
      _tickets.value = _tickets.value - s.id
      TheatreAlerts.cancel(app, s.id)
      scope.launch { load() }
      "Ticket cancelled."
    } catch (e: Supa.Offline) {
      "You seem to be offline."
    } finally {
      _busy.value = null
    }
  }

  /** What entering a screening said. */
  sealed interface Entry {
    data class Allowed(val screening: Screening, val capacity: Int) : Entry

    /** not_yet, over, waitlist, no_ticket, or a message. */
    data class Refused(val reason: String) : Entry
  }

  suspend fun enter(s: Screening): Entry {
    val token = VelmoraAccount.accessToken() ?: return Entry.Refused("sign_in")
    return try {
      val reply = Supa.request("POST", "${Supa.BASE}/rest/v1/rpc/theatre_enter", buildJsonObject { put("sid", s.id) }.toString(), token = token)
      if (!reply.ok) return Entry.Refused(friendly(reply.body))
      val o = json.parseToJsonElement(reply.body).jsonObject
      // The server's clock comes with the answer: one more sample.
      parse(o["server_now"]?.jsonPrimitive?.contentOrNull).takeIf { it > 0 }?.let { server -> if (offset == 0L) offset = server - System.currentTimeMillis() }
      if (o["allowed"]?.jsonPrimitive?.booleanOrNull != true) return Entry.Refused(o["reason"]?.jsonPrimitive?.contentOrNull ?: "not_allowed")
      val fresh =
        s.copy(
          startsAt = o["starts_at"]?.jsonPrimitive?.contentOrNull ?: s.startsAt,
          endsAt = o["ends_at"]?.jsonPrimitive?.contentOrNull ?: s.endsAt,
          mode = o["mode"]?.jsonPrimitive?.contentOrNull ?: s.mode,
        )
      _tickets.value = _tickets.value + (s.id to "booked")
      Entry.Allowed(fresh, o["capacity"]?.jsonPrimitive?.intOrNull ?: s.capacity)
    } catch (e: Supa.Offline) {
      Entry.Refused("You seem to be offline.")
    }
  }

  // ── Reminders ──

  /** Ticket holders hear when a screening they hold was cancelled or moved. */
  private fun announce(before: List<Screening>, list: List<Screening>) {
    val was = before.associateBy { it.id }
    for (s in list) {
      val t = _tickets.value[s.id]
      if ((t != "booked" && t != "waitlist") || phase(s) == ShowPhase.Over) continue
      if (s.status == "cancelled") {
        TheatreAlerts.notify(app, "Theatre: screening cancelled", "${s.name} was cancelled.${if (s.note.isNotBlank()) " ${s.note}" else ""}", "cancel:${s.id}", null)
        continue
      }
      val old = was[s.id]
      if (old != null && old.startsAt != s.startsAt) {
        TheatreAlerts.notify(app, "Theatre: new start time", "${s.name} now starts at ${TheatreAlerts.clock(s.startMs)}.", "moved:${s.id}:${s.startsAt}", s.id)
      }
    }
  }

  /** Reminders for booked screenings: when the lobby opens, and 5 minutes before. */
  private fun schedule(list: List<Screening>) {
    val wm = runCatching { WorkManager.getInstance(app) }.getOrNull() ?: return
    for (s in list) {
      val booked = _tickets.value[s.id] == "booked" && s.status == "scheduled"
      if (!booked) {
        TheatreAlerts.cancel(app, s.id)
        continue
      }
      val now = serverNow()
      fun queue(tag: String, at: Long, title: String, body: String, key: String) {
        val wait = at - now
        if (wait < -60_000 || wait > 2 * 86_400_000L) return
        val req =
          OneTimeWorkRequestBuilder<TheatreReminder>()
            .setInitialDelay(wait.coerceAtLeast(0), TimeUnit.MILLISECONDS)
            .setInputData(workDataOf("title" to title, "body" to body, "key" to key, "sid" to s.id))
            .addTag("theatre:${s.id}")
            .build()
        wm.enqueueUniqueWork("theatre:${s.id}:$tag", ExistingWorkPolicy.REPLACE, req)
      }
      val lobby = (_settings.value.reminder.ifBlank { "{title} starts in 30 minutes. Chat is open." }).replace("{title}", s.name)
      queue("lobby", s.startMs - LOBBY_MS, "Theatre: the lobby is open", lobby, "lobby:${s.id}:${s.startsAt}")
      queue("soon", s.startMs - 5 * 60_000, "Theatre: starting soon", "${s.name} starts in 5 minutes.", "soon:${s.id}:${s.startsAt}")
    }
  }

  private fun enc(s: String) = URLEncoder.encode(s, "UTF-8")

  @Suppress("unused")
  private fun str(s: String) = JsonPrimitive(s)
}
