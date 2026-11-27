package com.velmora.stepbyte.velmora.library

import android.content.Context
import android.util.Log
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.engine.TitleSummary
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.engine.Wire
import com.velmora.stepbyte.velmora.account.AccountState
import com.velmora.stepbyte.velmora.account.Supa
import com.velmora.stepbyte.velmora.account.VelmoraAccount
import java.io.File
import java.net.URLEncoder
import java.time.Instant
import java.time.OffsetDateTime
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

/** Where someone is in a title: the desktop `HistoryEntry`, seconds. */
class Progress(
  val title: Title,
  val season: Int?,
  val episode: Int?,
  val position: Double,
  val duration: Double,
  val updatedAt: Long,
) {
  val fraction: Float
    get() = if (duration > 0) (position / duration).toFloat().coerceIn(0f, 1f) else 0f

  /** Started and not finished: what Continue watching shows. */
  val resumable: Boolean
    get() = duration > 0 && position > 60 && fraction < 0.95f
}

/**
 * My List and watch progress on this phone. Everything lives here first, so
 * it opens instantly and works offline; changes go to the account in the
 * background with desktop's sync tables (`supabase/sync.sql`): the newest
 * edit wins, removals travel as small "deleted" rows, and each pull pages
 * through `synced_at` from where the last one stopped.
 */
object VelmoraLibrary {
  private const val TAG = "VelmoraLibrary"
  private const val PAGE = 500

  private val json = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
  }
  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
  private val lock = Mutex()

  @Serializable private data class Meta(val at: Long, val del: Boolean = false, val dirty: Boolean = false)

  @Serializable
  private data class Store(
    /** The account these rows belong to; another account starts clean. */
    val owner: String? = null,
    /** Newest first. */
    val list: List<JsonObject> = emptyList(),
    val listMeta: Map<String, Meta> = emptyMap(),
    /** title key -> desktop HistoryEntry JSON. */
    val progress: Map<String, JsonObject> = emptyMap(),
    val progressMeta: Map<String, Meta> = emptyMap(),
    val listCursor: String = "",
    val progressCursor: String = "",
  )

  private lateinit var file: File
  @Volatile private var store = Store()

  private val _myList = MutableStateFlow<List<Title>>(emptyList())
  val myList: StateFlow<List<Title>> = _myList.asStateFlow()

  private val _progress = MutableStateFlow<List<Progress>>(emptyList())
  /** Every title with progress, most recent first. */
  val progress: StateFlow<List<Progress>> = _progress.asStateFlow()

  private val _syncing = MutableStateFlow(false)
  val syncing: StateFlow<Boolean> = _syncing.asStateFlow()

  private var pending: Job? = null

  fun key(t: Title): String = key(t.value)

  fun key(t: TitleSummary): String = "${t.origin.provider}:${t.origin.id}"

  /** Load what's saved and follow the account; call once at startup. */
  fun init(context: Context) {
    if (::file.isInitialized) return
    file = File(context.applicationContext.filesDir, "velmora/library.json")
    store = runCatching { json.decodeFromString(Store.serializer(), file.readText()) }.getOrDefault(Store())
    publish()
    scope.launch {
      VelmoraAccount.state.collect { s ->
        // Online and confirmed: catch up both ways.
        if (s is AccountState.SignedIn && !s.offline) sync()
      }
    }
  }

  // ── My List ──

  fun inList(t: Title): Boolean = _myList.value.any { key(it) == key(t) }

  /** Add or remove; returns whether it's in the list now. */
  fun toggle(t: Title): Boolean {
    val k = key(t)
    val now = System.currentTimeMillis()
    val adding = !inList(t)
    val list = store.list.filterNot { titleKey(it) == k }
    store =
      store.copy(
        list = if (adding) listOf(t.json) + list else list,
        listMeta = store.listMeta + (k to Meta(at = now, del = !adding, dirty = true)),
      )
    commit()
    return adding
  }

  // ── Progress ──

  /** The saved spot in [t], if any. */
  fun progressOf(t: Title): Progress? = _progress.value.firstOrNull { key(it.title) == key(t) }

  /** Remember where playback is (the player calls this in phase 6). */
  fun saveProgress(t: Title, season: Int?, episode: Int?, position: Double, duration: Double) {
    val k = key(t)
    val now = System.currentTimeMillis()
    val entry = buildJsonObject {
      put("title", t.json)
      season?.let { put("season", it) }
      episode?.let { put("episode", it) }
      put("position", position)
      put("duration", duration)
      put("updatedAt", now)
    }
    store = store.copy(progress = store.progress + (k to entry), progressMeta = store.progressMeta + (k to Meta(now, dirty = true)))
    commit()
  }

  /** Drop a title from Continue watching (and from the account). */
  fun forgetProgress(t: Title) {
    val k = key(t)
    store = store.copy(progress = store.progress - k, progressMeta = store.progressMeta + (k to Meta(System.currentTimeMillis(), del = true, dirty = true)))
    commit()
  }

  // ── Saving ──

  private fun commit() {
    publish()
    save()
    // Push soon, but not once per tap.
    pending?.cancel()
    pending =
      scope.launch {
        delay(1500)
        sync()
      }
  }

  private fun publish() {
    _myList.value = store.list.mapNotNull(::toTitle)
    _progress.value =
      store.progress.values
        .mapNotNull(::toProgress)
        .sortedByDescending { it.updatedAt }
  }

  private fun save() {
    val snapshot = store
    scope.launch {
      lock.withLock {
        runCatching {
          file.parentFile?.mkdirs()
          val tmp = File(file.path + ".tmp")
          tmp.writeText(json.encodeToString(Store.serializer(), snapshot))
          tmp.renameTo(file)
        }
          .onFailure { Log.w(TAG, "couldn't save the library", it) }
      }
    }
  }

  // ── Sync ──

  /** Catch up with the account. Quiet when signed out or offline. */
  suspend fun sync() {
    val account = VelmoraAccount.state.value as? AccountState.SignedIn ?: return
    if (_syncing.value) return
    _syncing.value = true
    try {
      val uid = account.profile.id.ifEmpty { VelmoraAccount.uid ?: return }
      val token = VelmoraAccount.accessToken() ?: return
      adoptOwner(uid)
      pullList(token)
      pullProgress(token)
      adoptLocal()
      push(token)
      publish()
      save()
    } catch (e: CancellationException) {
      throw e
    } catch (e: Supa.Offline) {
      Log.i(TAG, "sync waits for the network: ${e.message}")
    } catch (e: Exception) {
      Log.w(TAG, "sync failed", e)
    } finally {
      _syncing.value = false
    }
  }

  /** A different account on this phone must not inherit the last one's list. */
  private fun adoptOwner(uid: String) {
    if (store.owner != null && store.owner != uid) {
      store = Store(owner = uid)
      publish()
    } else if (store.owner == null) {
      store = store.copy(owner = uid)
    }
  }

  private suspend fun pages(table: String, columns: String, cursor: String, token: String, each: (List<JsonObject>) -> Unit): String {
    var at = cursor
    while (true) {
      val filter = if (at.isNotEmpty()) "&synced_at=gte.${URLEncoder.encode(at, "UTF-8")}" else ""
      val reply =
        Supa.request("GET", "${Supa.BASE}/rest/v1/$table?select=$columns&order=synced_at.asc&limit=$PAGE$filter", token = token)
      if (!reply.ok) throw IllegalStateException("$table: HTTP ${reply.code}")
      val rows = (json.parseToJsonElement(reply.body) as? JsonArray)?.mapNotNull { it as? JsonObject } ?: return at
      if (rows.isEmpty()) return at
      each(rows)
      val last = rows.last().str("synced_at") ?: return at
      val same = last == at
      at = last
      if (rows.size < PAGE || same) return at
    }
  }

  private suspend fun pullList(token: String) {
    val cursor =
      pages("list_items", "title_key,title,updated_at,deleted,synced_at", store.listCursor, token) { rows ->
        var list = store.list
        var meta = store.listMeta
        for (r in rows) {
          val k = r.str("title_key") ?: continue
          val at = time(r.str("updated_at")) ?: continue
          val localAt: Long = meta[k]?.at ?: if (list.any { titleKey(it) == k }) 0L else -1L
          if (at <= localAt) continue
          val title = r["title"] as? JsonObject
          val live = r["deleted"]?.jsonPrimitive?.contentOrNull != "true" && title != null && toTitle(title) != null
          meta = meta + (k to Meta(at, del = !live))
          list = list.filterNot { titleKey(it) == k }
          if (live) list = listOf(title!!) + list
        }
        store = store.copy(list = list, listMeta = meta)
      }
    store = store.copy(listCursor = cursor)
  }

  private suspend fun pullProgress(token: String) {
    val cursor =
      pages("progress", "title_key,entry,updated_at,deleted,synced_at", store.progressCursor, token) { rows ->
        var progress = store.progress
        var meta = store.progressMeta
        for (r in rows) {
          val k = r.str("title_key") ?: continue
          val at = time(r.str("updated_at")) ?: continue
          val localAt: Long = maxOf(meta[k]?.at ?: -1L, progress[k]?.get("updatedAt")?.jsonPrimitive?.contentOrNull?.toDoubleOrNull()?.toLong() ?: -1L)
          if (at <= localAt) continue
          val entry = r["entry"] as? JsonObject
          val live = r["deleted"]?.jsonPrimitive?.contentOrNull != "true" && entry != null && toProgress(entry) != null
          meta = meta + (k to Meta(at, del = !live))
          progress = if (live) progress + (k to JsonObject(entry!! + ("updatedAt" to kotlinx.serialization.json.JsonPrimitive(at)))) else progress - k
        }
        store = store.copy(progress = progress, progressMeta = meta)
      }
    store = store.copy(progressCursor = cursor)
  }

  /** What this phone holds that the account hasn't seen goes up, stamped now. */
  private fun adoptLocal() {
    val now = System.currentTimeMillis()
    var listMeta = store.listMeta
    for (t in store.list) {
      val k = titleKey(t) ?: continue
      if (k !in listMeta) listMeta = listMeta + (k to Meta(now, dirty = true))
    }
    var progressMeta = store.progressMeta
    for ((k, e) in store.progress) {
      if (k !in progressMeta) {
        val at = e["updatedAt"]?.jsonPrimitive?.contentOrNull?.toLongOrNull() ?: now
        progressMeta = progressMeta + (k to Meta(at, dirty = true))
      }
    }
    store = store.copy(listMeta = listMeta, progressMeta = progressMeta)
  }

  private suspend fun push(token: String) {
    val lists = store.listMeta.filter { it.value.dirty }
    for (part in lists.entries.chunked(200)) {
      val rows = buildJsonArray {
        for ((k, m) in part) {
          val title = if (m.del) null else store.list.firstOrNull { titleKey(it) == k }
          add(
            buildJsonObject {
              put("title_key", k)
              put("title", title ?: JsonNull)
              put("updated_at", Instant.ofEpochMilli(m.at).toString())
              put("deleted", m.del || title == null)
            },
          )
        }
      }
      rpc("sync_push_list", rows, token)
      store = store.copy(listMeta = settle(store.listMeta, part.map { it.key to it.value }))
    }
    val progress = store.progressMeta.filter { it.value.dirty }
    for (part in progress.entries.chunked(100)) {
      val rows = buildJsonArray {
        for ((k, m) in part) {
          val entry = if (m.del) null else store.progress[k]
          add(
            buildJsonObject {
              put("title_key", k)
              put("entry", entry ?: JsonNull)
              put("updated_at", Instant.ofEpochMilli(m.at).toString())
              put("deleted", m.del || entry == null)
            },
          )
        }
      }
      rpc("sync_push_progress", rows, token)
      store = store.copy(progressMeta = settle(store.progressMeta, part.map { it.key to it.value }))
    }
  }

  /** Mark rows sent, unless they were edited again while the push was out. */
  private fun settle(now: Map<String, Meta>, sent: List<Pair<String, Meta>>): Map<String, Meta> {
    var out = now
    for ((k, m) in sent) {
      if (out[k]?.at == m.at) out = out + (k to m.copy(dirty = false))
    }
    return out
  }

  private suspend fun rpc(name: String, rows: JsonArray, token: String) {
    val reply = Supa.request("POST", "${Supa.BASE}/rest/v1/rpc/$name", buildJsonObject { put("rows", rows) }.toString(), token = token)
    if (!reply.ok) throw IllegalStateException("$name: HTTP ${reply.code} ${reply.body.take(200)}")
  }

  // ── Helpers ──

  private fun JsonObject.str(k: String): String? = this[k]?.jsonPrimitive?.contentOrNull

  private fun time(iso: String?): Long? = iso?.let { runCatching { OffsetDateTime.parse(it).toInstant().toEpochMilli() }.getOrNull() }

  private fun titleKey(o: JsonObject): String? {
    val origin = o["origin"] as? JsonObject ?: return null
    val p = origin.str("provider") ?: return null
    val id = origin.str("id") ?: return null
    return "$p:$id"
  }

  private fun toTitle(o: JsonObject): Title? =
    runCatching { Wire(Universe.json.decodeFromJsonElement(TitleSummary.serializer(), o), o) }.getOrNull()

  private fun toProgress(e: JsonObject): Progress? {
    val title = (e["title"] as? JsonObject)?.let(::toTitle) ?: return null
    val position = e["position"]?.jsonPrimitive?.doubleOrNull ?: return null
    val duration = e["duration"]?.jsonPrimitive?.doubleOrNull ?: return null
    return Progress(
      title = title,
      season = e["season"]?.jsonPrimitive?.intOrNull,
      episode = e["episode"]?.jsonPrimitive?.intOrNull,
      position = position,
      duration = duration,
      updatedAt = e["updatedAt"]?.jsonPrimitive?.contentOrNull?.toDoubleOrNull()?.toLong() ?: 0L,
    )
  }
}
