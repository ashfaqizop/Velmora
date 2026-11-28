package com.velmora.stepbyte.velmora.theatre

import android.util.Log
import com.velmora.stepbyte.velmora.account.Supa
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener

/**
 * A small Supabase Realtime client: the Phoenix channel protocol over one
 * WebSocket, with heartbeats, rejoin after a dropped connection, private
 * channels (the access token in the join), broadcast and presence. Enough
 * for Theatre's rooms; supabase-js does the same on desktop.
 */
class Realtime(private val token: suspend () -> String?) {
  private val json = Json { ignoreUnknownKeys = true }
  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
  private val client = OkHttpClient.Builder().pingInterval(25, TimeUnit.SECONDS).readTimeout(0, TimeUnit.MILLISECONDS).build()
  private val refs = AtomicInteger(0)
  private val channels = ConcurrentHashMap<String, Channel>()
  @Volatile private var ws: WebSocket? = null
  @Volatile private var open = false
  private var heartbeat: Job? = null
  private var retry: Job? = null
  private var backoff = 1_000L
  @Volatile private var closed = false

  private fun nextRef() = refs.incrementAndGet().toString()

  /** One topic: `theatre:<id>`. */
  inner class Channel(val name: String, private val config: JsonObject) {
    val topic = "realtime:$name"
    @Volatile var joinRef: String? = null
    @Volatile var joined = false

    var onBroadcast: (event: String, payload: JsonObject) -> Unit = { _, _ -> }
    var onPresence: (Map<String, List<JsonObject>>) -> Unit = {}
    /** "joined", "error: …", "closed". */
    var onStatus: (String) -> Unit = {}

    /** key -> metas */
    private val presence = mutableMapOf<String, MutableList<JsonObject>>()
    private var tracked: JsonObject? = null

    internal fun join() {
      scope.launch {
        val ref = nextRef()
        joinRef = ref
        val payload = buildJsonObject {
          put("config", config)
          token()?.let { put("access_token", it) }
        }
        send(topic, "phx_join", payload, ref, ref)
      }
    }

    internal fun handle(event: String, payload: JsonObject, ref: String?) {
      when (event) {
        "phx_reply" -> {
          if (ref != null && ref == joinRef) {
            val status = payload["status"]?.jsonPrimitive?.contentOrNull
            if (status == "ok") {
              joined = true
              onStatus("joined")
              tracked?.let { sendTrack(it) }
            } else {
              joined = false
              onStatus("error: " + (payload["response"]?.toString() ?: "refused"))
            }
          }
        }
        "phx_error", "phx_close" -> {
          joined = false
          onStatus(if (event == "phx_close") "closed" else "error: channel")
        }
        "system" -> {
          val status = payload["status"]?.jsonPrimitive?.contentOrNull
          if (status == "error") onStatus("error: " + (payload["message"]?.jsonPrimitive?.contentOrNull ?: "system"))
        }
        "broadcast" -> {
          val e = payload["event"]?.jsonPrimitive?.contentOrNull ?: return
          val inner = payload["payload"] as? JsonObject ?: JsonObject(emptyMap())
          onBroadcast(e, inner)
        }
        "presence_state" -> {
          synchronized(presence) {
            presence.clear()
            for ((key, v) in payload) presence[key] = metas(v)
          }
          firePresence()
        }
        "presence_diff" -> {
          synchronized(presence) {
            (payload["joins"] as? JsonObject)?.forEach { (key, v) -> presence.getOrPut(key) { mutableListOf() }.addAll(metas(v)) }
            (payload["leaves"] as? JsonObject)?.forEach { (key, v) ->
              val gone = metas(v).mapNotNull { it["phx_ref"]?.jsonPrimitive?.contentOrNull }.toSet()
              presence[key]?.removeAll { it["phx_ref"]?.jsonPrimitive?.contentOrNull in gone }
              if (presence[key].isNullOrEmpty()) presence.remove(key)
            }
          }
          firePresence()
        }
      }
    }

    private fun metas(v: JsonElement): MutableList<JsonObject> =
      runCatching { (v.jsonObject["metas"])!!.jsonArray.mapNotNull { it as? JsonObject }.toMutableList() }.getOrDefault(mutableListOf())

    private fun firePresence() {
      val snap = synchronized(presence) { presence.mapValues { it.value.toList() } }
      onPresence(snap)
    }

    /** Say who you are; sent again after a rejoin. */
    fun track(payload: JsonObject) {
      tracked = payload
      if (joined) sendTrack(payload)
    }

    private fun sendTrack(payload: JsonObject) {
      send(topic, "presence", buildJsonObject {
        put("type", "presence")
        put("event", "track")
        put("payload", payload)
      }, nextRef(), joinRef)
    }

    fun broadcast(event: String, payload: JsonObject) {
      if (!joined) return
      send(topic, "broadcast", buildJsonObject {
        put("type", "broadcast")
        put("event", event)
        put("payload", payload)
      }, nextRef(), joinRef)
    }

    fun leave() {
      channels.remove(topic)
      if (joined) send(topic, "phx_leave", JsonObject(emptyMap()), nextRef(), joinRef)
      joined = false
      if (channels.isEmpty()) close()
    }
  }

  /** A channel, joined now (or as soon as the socket is up). */
  fun channel(name: String, isPrivate: Boolean, presenceKey: String? = null, self: Boolean = true): Channel {
    val config = buildJsonObject {
      put("broadcast", buildJsonObject {
        put("self", self)
        put("ack", false)
      })
      put("presence", buildJsonObject {
        put("key", presenceKey ?: "")
        put("enabled", presenceKey != null)
      })
      put("private", isPrivate)
    }
    val ch = Channel(name, config)
    channels[ch.topic] = ch
    closed = false
    if (open) ch.join() else connect()
    return ch
  }

  private fun connect() {
    if (ws != null || closed) return
    val url = Supa.BASE.replace("https://", "wss://") + "/realtime/v1/websocket?apikey=${Supa.KEY}&vsn=1.0.0"
    ws = client.newWebSocket(Request.Builder().url(url).build(), listener)
  }

  private val listener =
    object : WebSocketListener() {
      override fun onOpen(webSocket: WebSocket, response: Response) {
        open = true
        backoff = 1_000L
        heartbeat?.cancel()
        heartbeat =
          scope.launch {
            while (isActive) {
              delay(25_000)
              send("phoenix", "heartbeat", JsonObject(emptyMap()), nextRef(), null)
              // Keep private channels authorised as the access token renews.
              token()?.let { t -> channels.values.filter { it.joined }.forEach { send(it.topic, "access_token", buildJsonObject { put("access_token", t) }, nextRef(), it.joinRef) } }
            }
          }
        channels.values.forEach { it.join() }
      }

      override fun onMessage(webSocket: WebSocket, text: String) {
        val o = runCatching { json.parseToJsonElement(text).jsonObject }.getOrNull() ?: return
        val topic = o["topic"]?.jsonPrimitive?.contentOrNull ?: return
        val event = o["event"]?.jsonPrimitive?.contentOrNull ?: return
        val payload = o["payload"] as? JsonObject ?: JsonObject(emptyMap())
        val ref = (o["ref"] as? JsonPrimitive)?.contentOrNull
        channels[topic]?.handle(event, payload, ref)
      }

      override fun onClosed(webSocket: WebSocket, code: Int, reason: String) = dropped(null)

      override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) = dropped(t)
    }

  private fun dropped(t: Throwable?) {
    if (t != null) Log.i("VelmoraRealtime", "socket dropped: ${t.message}")
    open = false
    ws = null
    heartbeat?.cancel()
    channels.values.forEach {
      it.joined = false
      it.onStatus("closed")
    }
    if (closed || channels.isEmpty()) return
    retry?.cancel()
    retry =
      scope.launch {
        delay(backoff)
        backoff = (backoff * 2).coerceAtMost(30_000)
        connect()
      }
  }

  private fun send(topic: String, event: String, payload: JsonObject, ref: String?, joinRef: String?) {
    val msg = buildJsonObject {
      put("topic", topic)
      put("event", event)
      put("payload", payload)
      put("ref", ref?.let { JsonPrimitive(it) } ?: kotlinx.serialization.json.JsonNull)
      put("join_ref", joinRef?.let { JsonPrimitive(it) } ?: kotlinx.serialization.json.JsonNull)
    }
    ws?.send(msg.toString())
  }

  fun close() {
    closed = true
    retry?.cancel()
    heartbeat?.cancel()
    ws?.close(1000, "bye")
    ws = null
    open = false
  }
}
