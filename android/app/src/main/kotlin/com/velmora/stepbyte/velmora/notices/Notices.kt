package com.velmora.stepbyte.velmora.notices

import android.content.Context
import com.velmora.stepbyte.velmora.account.AccountState
import com.velmora.stepbyte.velmora.account.Supa
import com.velmora.stepbyte.velmora.account.VelmoraAccount
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.Json

/** An announcement or maintenance message from the admin site. */
@Serializable
data class Notice(
  val id: Long,
  /** info, warning or maintenance. */
  val kind: String = "info",
  val title: String = "",
  val body: String = "",
  @SerialName("link_url") val linkUrl: String? = null,
  @SerialName("link_label") val linkLabel: String? = null,
  /** everyone, public or max. */
  val audience: String = "everyone",
  @SerialName("updated_at") val updatedAt: String = "",
) {
  val stamp: String
    get() = "$id@$updatedAt"

  val maintenance: Boolean
    get() = kind == "maintenance"
}

/**
 * Announcements, the same rows desktop shows (app/src/lib/notices.svelte.ts):
 * read from Supabase with the public key (row-level security returns only live
 * ones), at launch and every 10 minutes. Dismissals are per version, so an
 * edited announcement shows again. Maintenance can't be dismissed. The last
 * rows are kept, so a maintenance notice still shows offline.
 */
object Notices {
  private const val PREFS = "velmora_notices"
  private val json = Json { ignoreUnknownKeys = true; coerceInputValues = true }
  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

  private lateinit var app: Context
  private val rows = MutableStateFlow<List<Notice>>(emptyList())
  private val gone = MutableStateFlow<Set<String>>(emptySet())

  private fun prefs() = app.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

  /** The one to show: maintenance first, then the newest. */
  val current: StateFlow<Notice?> =
    combine(rows, gone, VelmoraAccount.state) { list, dismissed, account ->
        val tier = if ((account as? AccountState.SignedIn)?.max == true) "max" else "public"
        val open =
          list.filter { n -> (n.audience == "everyone" || n.audience == tier) && n.title.isNotBlank() && (n.maintenance || n.stamp !in dismissed) }
        open.firstOrNull { it.maintenance } ?: open.firstOrNull()
      }
      .stateIn(scope, SharingStarted.Eagerly, null)

  fun init(context: Context) {
    if (::app.isInitialized) return
    app = context.applicationContext
    gone.value = prefs().getStringSet("dismissed", emptySet()).orEmpty()
    rows.value =
      runCatching { json.decodeFromString(ListSerializer(Notice.serializer()), prefs().getString("rows", null) ?: "[]") }.getOrDefault(emptyList())
    scope.launch {
      while (true) {
        refresh()
        delay(10 * 60_000L)
      }
    }
  }

  suspend fun refresh() {
    runCatching {
      val r =
        Supa.request(
          "GET",
          "${Supa.BASE}/rest/v1/announcements?select=id,kind,title,body,link_url,link_label,audience,updated_at&place=in.(app,both)&order=starts_at.desc&limit=5",
        )
      if (!r.ok) return
      val list = json.decodeFromString(ListSerializer(Notice.serializer()), r.body)
      rows.value = list
      prefs().edit().putString("rows", r.body).apply()
    }
    // Offline or Supabase down: keep what we had.
  }

  fun dismiss(n: Notice) {
    if (n.maintenance) return
    val next = (gone.value + n.stamp).toList().takeLast(40).toSet()
    gone.value = next
    prefs().edit().putStringSet("dismissed", next).apply()
  }
}
