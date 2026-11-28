package com.velmora.stepbyte.velmora.downloads

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat
import com.velmora.stepbyte.MainActivity
import com.velmora.stepbyte.R

/**
 * Keeps downloads going with the app closed, showing one notification with
 * how far along they are. It stops itself when nothing is downloading or
 * waiting to.
 */
class DownloadService : Service() {
  override fun onBind(intent: Intent?): IBinder? = null

  override fun onCreate() {
    super.onCreate()
    channels(this)
  }

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    when (intent?.action) {
      ACTION_PAUSE_ALL -> VelmoraDownloads.items.value.filter { it.record.phase in ACTIVE }.forEach { VelmoraDownloads.pause(it.record.id) }
    }
    val n = progress(this)
    // A service started for the foreground must get there, even to stop at once.
    try {
      ServiceCompat.startForeground(this, ID, n ?: placeholder(this), if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC else 0)
    } catch (e: Exception) {
      // Android 12+ refuses a start from the background; the downloads still run while the app lives.
      Log.w("VelmoraDownloads", "couldn't go to the foreground", e)
    }
    if (n == null) {
      running = false
      stop()
      return START_NOT_STICKY
    }
    return START_STICKY
  }

  private fun stop() {
    ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_REMOVE)
    stopSelf()
  }

  companion object {
    private const val CHANNEL = "velmora_downloads"
    private const val CHANNEL_DONE = "velmora_downloads_done"
    private const val ID = 4107
    private const val ACTION_PAUSE_ALL = "com.velmora.stepbyte.DOWNLOADS_PAUSE_ALL"
    private val ACTIVE = listOf(DownloadPhase.Finding, DownloadPhase.Queued, DownloadPhase.Running, DownloadPhase.WaitingWifi)
    @Volatile private var running = false
    private var lastText = ""

    fun start(context: Context) {
      running = true
      runCatching { ContextCompat.startForegroundService(context, Intent(context, DownloadService::class.java)) }
        .onFailure {
          running = false
          Log.w("VelmoraDownloads", "download service didn't start", it)
        }
    }

    /** Update the notification, or stop when nothing's left. */
    fun refresh(context: Context) {
      val active = VelmoraDownloads.items.value.any { it.record.phase in ACTIVE }
      if (!active) {
        if (running) {
          running = false
          runCatching { context.startService(Intent(context, DownloadService::class.java)) }
        }
        return
      }
      if (!running) {
        start(context)
        return
      }
      val n = progress(context) ?: return
      val text = n.extras?.getCharSequence(NotificationCompat.EXTRA_TEXT)?.toString() + n.extras?.getInt(NotificationCompat.EXTRA_PROGRESS)
      if (text == lastText) return
      lastText = text
      if (allowed(context)) runCatching { NotificationManagerCompat.from(context).notify(ID, n) }
    }

    /** "Downloaded: Frieren S1 E3", tapping opens Downloads. */
    fun notifyDone(context: Context, name: String, season: Int?, episode: Int?) {
      channels(context)
      if (!allowed(context)) return
      val label = if (season != null && episode != null) "$name · S$season E$episode" else name
      val n =
        NotificationCompat.Builder(context, CHANNEL_DONE)
          .setSmallIcon(R.drawable.download)
          .setContentTitle("Downloaded")
          .setContentText(label)
          .setContentIntent(openApp(context))
          .setAutoCancel(true)
          .build()
      runCatching { NotificationManagerCompat.from(context).notify(label.hashCode(), n) }
    }

    private fun allowed(context: Context) =
      Build.VERSION.SDK_INT < 33 || ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED

    private fun channels(context: Context) {
      if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
      val nm = context.getSystemService(NotificationManager::class.java) ?: return
      nm.createNotificationChannel(NotificationChannel(CHANNEL, "Downloads in progress", NotificationManager.IMPORTANCE_LOW).apply { setShowBadge(false) })
      nm.createNotificationChannel(NotificationChannel(CHANNEL_DONE, "Finished downloads", NotificationManager.IMPORTANCE_DEFAULT))
    }

    private fun openApp(context: Context): PendingIntent =
      PendingIntent.getActivity(
        context,
        41,
        Intent(context, MainActivity::class.java).setAction(Intent.ACTION_VIEW).putExtra("velmora_open", "downloads").addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
      )

    private fun placeholder(context: Context): android.app.Notification {
      channels(context)
      return NotificationCompat.Builder(context, CHANNEL).setSmallIcon(R.drawable.download).setContentTitle("Downloads").setContentText("Getting ready").setSilent(true).build()
    }

    /** The progress notification, or null when nothing is active. */
    private fun progress(context: Context): android.app.Notification? {
      channels(context)
      val items = VelmoraDownloads.items.value.filter { it.record.phase in ACTIVE }
      if (items.isEmpty()) return null
      val running = items.filter { it.record.phase == DownloadPhase.Running }
      val waiting = items.all { it.record.phase == DownloadPhase.WaitingWifi }
      val lead = running.firstOrNull() ?: items.first()
      val label = lead.titleValue.name + (lead.record.season?.let { s -> lead.record.episode?.let { " · S$s E$it" } } ?: "")
      val title =
        when {
          waiting -> "Waiting for Wi-Fi"
          items.size == 1 -> "Downloading"
          else -> "Downloading ${items.size}"
        }
      val down = running.sumOf { it.downloaded }
      val total = running.sumOf { it.total ?: 0 }
      val speed = running.sumOf { it.speed }
      val text =
        when {
          waiting -> "$label and ${items.size - 1} more wait for Wi-Fi".replace(" and 0 more wait", " waits")
          running.isEmpty() -> "$label · finding a source"
          else -> "$label · ${VelmoraDownloads.human(down)} of ${if (total > 0) VelmoraDownloads.human(total) else "?"} · ${VelmoraDownloads.human(speed)}/s"
        }
      val pauseAll =
        PendingIntent.getService(context, 42, Intent(context, DownloadService::class.java).setAction(ACTION_PAUSE_ALL), PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
      return NotificationCompat.Builder(context, CHANNEL)
        .setSmallIcon(R.drawable.download)
        .setContentTitle(title)
        .setContentText(text)
        .setOngoing(true)
        .setOnlyAlertOnce(true)
        .setSilent(true)
        .setContentIntent(openApp(context))
        .setProgress(100, if (total > 0) (down * 100 / total).toInt() else 0, total <= 0 && !waiting)
        .addAction(0, "Pause all", pauseAll)
        .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
        .build()
    }
  }
}

/** A page the app should open, asked for by a notification ("downloads"). */
object OpenRequests {
  val flow = kotlinx.coroutines.flow.MutableStateFlow<String?>(null)
}
