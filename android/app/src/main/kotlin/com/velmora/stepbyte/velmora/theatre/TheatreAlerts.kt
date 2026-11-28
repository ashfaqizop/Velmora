package com.velmora.stepbyte.velmora.theatre

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import androidx.work.CoroutineWorker
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import com.velmora.stepbyte.MainActivity
import com.velmora.stepbyte.R
import com.velmora.stepbyte.engine.Target
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.velmora.account.Supa
import com.velmora.stepbyte.velmora.player.PlayerPrefs
import com.velmora.stepbyte.velmora.player.SourceFinder
import com.velmora.stepbyte.velmora.player.VideoSession
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

/** Theatre's notifications: lobby open, starting soon, a seat freed, cancelled or moved. */
object TheatreAlerts {
  private const val CHANNEL = "velmora_theatre"

  fun clock(ms: Long): String = Instant.ofEpochMilli(ms).atZone(ZoneId.systemDefault()).format(DateTimeFormatter.ofPattern("h:mm a"))

  private fun prefs(c: Context) = c.getSharedPreferences("velmora_theatre", Context.MODE_PRIVATE)

  /** Once per [key]; tapping opens the screening (or Theatre). */
  fun notify(context: Context, title: String, body: String, key: String, screeningId: String?) {
    val done = prefs(context).getStringSet("reminded", emptySet()).orEmpty()
    if (key in done) return
    prefs(context).edit().putStringSet("reminded", (done + key).toList().takeLast(200).toSet()).apply()
    if (Build.VERSION.SDK_INT >= 33 && ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) return
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      context.getSystemService(NotificationManager::class.java)?.createNotificationChannel(NotificationChannel(CHANNEL, "Theatre", NotificationManager.IMPORTANCE_HIGH))
    }
    val open =
      PendingIntent.getActivity(
        context,
        key.hashCode(),
        Intent(context, MainActivity::class.java).setAction(Intent.ACTION_VIEW).putExtra("velmora_open", screeningId?.let { "theatre:$it" } ?: "theatre").addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
      )
    val n =
      NotificationCompat.Builder(context, CHANNEL)
        .setSmallIcon(R.drawable.theaters_filled)
        .setContentTitle(title)
        .setContentText(body)
        .setStyle(NotificationCompat.BigTextStyle().bigText(body))
        .setContentIntent(open)
        .setAutoCancel(true)
        .setCategory(NotificationCompat.CATEGORY_EVENT)
        .build()
    runCatching { NotificationManagerCompat.from(context).notify(key.hashCode(), n) }
  }

  /** Lobby-open and starting-soon reminders for booked screenings (Settings > Velmora > Theatre). */
  fun reminders(c: Context) = prefs(c).getBoolean("reminders", true)

  fun setReminders(c: Context, on: Boolean) = prefs(c).edit().putBoolean("reminders", on).apply()

  fun cancel(context: Context, screeningId: String) {
    runCatching { WorkManager.getInstance(context).cancelAllWorkByTag("theatre:$screeningId") }
  }
}

/** Fires a reminder WorkManager kept for a booked screening. */
class TheatreReminder(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
  override suspend fun doWork(): Result {
    val title = inputData.getString("title") ?: return Result.success()
    val body = inputData.getString("body") ?: return Result.success()
    val key = inputData.getString("key") ?: return Result.success()
    if (!TheatreAlerts.reminders(applicationContext)) return Result.success()
    TheatreAlerts.notify(applicationContext, title, body, key, inputData.getString("sid"))
    return Result.success()
  }
}

/**
 * The anonymous source checks of the spec: for a few drafted titles, look for
 * sources and report found / not found, so the lineup only has films people
 * can play. Only on Wi-Fi, when nothing is playing, each title once, and off
 * with the switch in Theatre's settings.
 */
object Probes {
  private fun prefs(c: Context) = c.getSharedPreferences("velmora_theatre", Context.MODE_PRIVATE)

  fun enabled(c: Context) = prefs(c).getBoolean("probe", true)

  fun setEnabled(c: Context, on: Boolean) = prefs(c).edit().putBoolean("probe", on).apply()

  suspend fun run(context: Context) {
    if (!enabled(context) || PlayerPrefs.metered(context) || VideoSession.player != null) return
    runCatching {
      val reply = Supa.request("GET", "${Supa.SITE}/api/theatre/probes", supabase = false)
      if (!reply.ok) return
      val o = Universe.json.parseToJsonElement(reply.body) as? JsonObject ?: return
      val done = prefs(context).getStringSet("probed", emptySet()).orEmpty()
      val todo =
        (o["titles"] as? JsonArray)
          ?.mapNotNull { (it as? JsonObject)?.get("k")?.jsonPrimitive?.contentOrNull }
          ?.filter { Regex("^movie/\\d+$").matches(it) && it !in done }
          ?.take(3)
          .orEmpty()
      val engine = Universe.get(context)
      for (k in todo) {
        if (VideoSession.player != null) break
        val title = engine.tmdbTitles(listOf(k)).firstOrNull()
        val found = title != null && runCatching { SourceFinder.find(context, title, Target.Movie, false).result.await().sources.isNotEmpty() }.getOrDefault(false)
        runCatching { Supa.request("POST", "${Supa.SITE}/api/theatre/probe", buildJsonObject { put("k", k); put("found", found) }.toString(), supabase = false) }
        prefs(context).edit().putStringSet("probed", (prefs(context).getStringSet("probed", emptySet()).orEmpty() + k).toList().takeLast(300).toSet()).apply()
      }
    }
  }
}
