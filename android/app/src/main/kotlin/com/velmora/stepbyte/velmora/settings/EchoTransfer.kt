package com.velmora.stepbyte.velmora.settings

import android.content.Context
import android.content.Intent
import android.database.sqlite.SQLiteDatabase
import android.net.Uri
import android.util.Log
import java.io.ByteArrayOutputStream
import java.io.File
import java.util.zip.ZipInputStream
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/**
 * Transfer from Echo Music: reads an Echo backup (the zip Echo's "Local
 * backup" makes, old or new, or a bare song database) and brings the music
 * library and Music settings over.
 *
 * Velmora's music side is Echo's, so the database is the same file. Settings
 * are merged, not replaced: Echo's music settings come over, and Velmora's own
 * (the account-free bits: mode, look, onboarding) stay as they are, so the
 * app doesn't start over. Paths inside settings that name Echo's package are
 * moved to Velmora's.
 */
object EchoTransfer {
  private const val TAG = "VelmoraTransfer"
  const val SETTINGS = "settings.preferences_pb"
  const val DB = "song.db"

  /** Echo Music's package names over the years. */
  val ECHO_PACKAGES = listOf("iad1tya.echo.music", "com.music.echo", "com.echo.music")

  /** Settings Velmora keeps from this phone, whatever the backup says. */
  private fun keepOurs(key: String) = key.startsWith("velmora") || key in setOf("liquidGlassGlobalEnabled", "lastOpenedVersionCode")

  data class Preview(
    val songs: Int?,
    val liked: Int?,
    val playlists: Int?,
    val albums: Int?,
    val artists: Int?,
    val plays: Int?,
    val hasDatabase: Boolean,
    val hasSettings: Boolean,
    val version: Int,
    /** From a newer Echo than this app's database understands. */
    val tooNew: Boolean,
  ) {
    val empty: Boolean
      get() = !hasDatabase && !hasSettings
  }

  /** The installed Echo Music, if any, to open it from the steps. */
  fun installedEcho(context: Context): String? =
    ECHO_PACKAGES.firstOrNull { runCatching { context.packageManager.getLaunchIntentForPackage(it) != null }.getOrDefault(false) }

  private fun work(context: Context) = File(context.cacheDir, "echo_transfer").apply { mkdirs() }

  /** Unpack [uri] into a scratch folder and say what's in it. */
  suspend fun inspect(context: Context, uri: Uri, currentVersion: Int): Result<Preview> =
    withContext(Dispatchers.IO) {
      runCatching {
        val dir = work(context)
        dir.listFiles()?.forEach { it.delete() }
        val db = File(dir, DB)
        val settings = File(dir, SETTINGS)
        context.contentResolver.openInputStream(uri)?.use { raw ->
          val input = raw.buffered()
          input.mark(16)
          val head = ByteArray(16)
          val n = input.read(head)
          input.reset()
          if (n == 16 && String(head, Charsets.US_ASCII).startsWith("SQLite format 3")) {
            // A bare database file.
            db.outputStream().use { input.copyTo(it) }
          } else {
            ZipInputStream(input).use { zip -> unzip(zip, db, settings) }
          }
        } ?: error("Couldn't open that file.")
        val hasDb = db.length() > 0
        val hasSettings = settings.length() > 0
        if (!hasDb && !hasSettings) error("That isn't an Echo Music backup. In Echo, use Settings › Backup and restore › Local backup.")
        var version = 0
        var counts = List<Int?>(6) { null }
        if (hasDb) {
          SQLiteDatabase.openDatabase(db.path, null, SQLiteDatabase.OPEN_READONLY).use { d ->
            version = d.version
            counts =
              listOf(
                "SELECT COUNT(*) FROM song WHERE inLibrary IS NOT NULL",
                "SELECT COUNT(*) FROM song WHERE liked = 1",
                "SELECT COUNT(*) FROM playlist",
                "SELECT COUNT(*) FROM album WHERE bookmarkedAt IS NOT NULL",
                "SELECT COUNT(*) FROM artist WHERE bookmarkedAt IS NOT NULL",
                "SELECT COUNT(*) FROM event",
              ).map { q -> runCatching { d.rawQuery(q, null).use { c -> if (c.moveToFirst()) c.getInt(0) else 0 } }.getOrNull() }
          }
        }
        Preview(counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], hasDb, hasSettings, version, hasDb && version > currentVersion)
      }
    }

  private fun unzip(zip: ZipInputStream, db: File, settings: File) {
    var entry = runCatching { zip.nextEntry }.getOrNull()
    while (entry != null) {
      val name = entry.name.substringAfterLast('/')
      when {
        // Old and new backups name these the same; some tools add a folder.
        name == DB || (name.endsWith(".db") && !db.exists()) -> db.outputStream().use { zip.copyTo(it) }
        name == SETTINGS || (name.endsWith(".preferences_pb") && !settings.exists()) -> settings.outputStream().use { zip.copyTo(it) }
      }
      entry = runCatching { zip.nextEntry }.getOrNull()
    }
  }

  /**
   * Put the unpacked backup in place: the merged settings, then the database
   * (the caller closes it first). The app must restart afterwards.
   */
  fun applySettings(context: Context) {
    val incoming = File(work(context), SETTINGS)
    if (incoming.length() == 0L) return
    val target = File(File(context.filesDir, "datastore"), SETTINGS)
    val ours = if (target.exists()) entries(target.readBytes()) else emptyList()
    val theirs = entries(incoming.readBytes()).filterNot { keepOurs(it.key) }.map { it.moved(context.packageName) }
    val theirKeys = theirs.map { it.key }.toSet()
    val merged = theirs + ours.filter { keepOurs(it.key) || it.key !in theirKeys }
    val out = ByteArrayOutputStream()
    merged.forEach { e ->
      out.write(0x0A)
      writeVarint(out, e.raw.size)
      out.write(e.raw)
    }
    target.parentFile?.mkdirs()
    target.writeBytes(out.toByteArray())
    Log.i(TAG, "settings merged: ${theirs.size} from Echo, ${merged.size - theirs.size} kept")
  }

  /** The backup's database file, ready to copy over the app's. */
  fun database(context: Context): File? = File(work(context), DB).takeIf { it.length() > 0 }

  fun cleanUp(context: Context) {
    runCatching { work(context).deleteRecursively() }
  }

  /** Start the app again from the top, as a restore does. */
  fun restart(context: Context, activity: Class<*>) {
    context.startActivity(Intent(context, activity).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK))
    kotlin.system.exitProcess(0)
  }

  // ── The settings file ──
  //
  // A Preferences DataStore file is one protobuf message: a map of key to
  // value, written as repeated entries (field 1). Each entry holds the key
  // (field 1, a string) and the value (field 2, a message). Entries can be
  // copied as raw bytes, so merging needs no protobuf library. A value that's
  // a string is field 5 of the value message.

  private class Entry(val key: String, val raw: ByteArray) {
    /** The same entry with Echo's package name in a string value swapped for ours. */
    fun moved(ourPackage: String): Entry {
      val value = field(raw, 2) ?: return this
      val text = field(value, 5)?.toString(Charsets.UTF_8) ?: return this
      val echo = ECHO_PACKAGES.firstOrNull { text.contains(it) } ?: return this
      val fixed = text.replace(echo, ourPackage).toByteArray(Charsets.UTF_8)
      val newValue = ByteArrayOutputStream().apply { tag(this, 5, fixed) }.toByteArray()
      val newRaw = ByteArrayOutputStream().apply {
        tag(this, 1, key.toByteArray(Charsets.UTF_8))
        tag(this, 2, newValue)
      }.toByteArray()
      return Entry(key, newRaw)
    }
  }

  private fun entries(bytes: ByteArray): List<Entry> {
    val out = mutableListOf<Entry>()
    var i = 0
    while (i < bytes.size) {
      val (tag, a) = varint(bytes, i)
      i = a
      if (tag.toInt() != 0x0A) break
      val (len, b) = varint(bytes, i)
      i = b
      val end = i + len.toInt()
      if (end > bytes.size) break
      val raw = bytes.copyOfRange(i, end)
      i = end
      val key = field(raw, 1)?.toString(Charsets.UTF_8) ?: continue
      out += Entry(key, raw)
    }
    return out
  }

  /** The bytes of the first length-delimited [field] in a message. */
  private fun field(msg: ByteArray, field: Int): ByteArray? {
    var i = 0
    while (i < msg.size) {
      val (t, a) = varint(msg, i)
      i = a
      val f = (t shr 3).toInt()
      when ((t and 7).toInt()) {
        0 -> i = varint(msg, i).second
        1 -> i += 8
        5 -> i += 4
        2 -> {
          val (len, b) = varint(msg, i)
          val end = b + len.toInt()
          if (end > msg.size) return null
          if (f == field) return msg.copyOfRange(b, end)
          i = end
        }
        else -> return null
      }
    }
    return null
  }

  private fun varint(b: ByteArray, start: Int): Pair<Long, Int> {
    var result = 0L
    var shift = 0
    var i = start
    while (i < b.size && shift < 64) {
      val x = b[i].toInt() and 0xFF
      i++
      result = result or ((x and 0x7F).toLong() shl shift)
      if ((x and 0x80) == 0) break
      shift += 7
    }
    return result to i
  }

  private fun writeVarint(out: ByteArrayOutputStream, value: Int) {
    var v = value
    while ((v and 0x7F.inv()) != 0) {
      out.write((v and 0x7F) or 0x80)
      v = v ushr 7
    }
    out.write(v)
  }

  private fun tag(out: ByteArrayOutputStream, field: Int, bytes: ByteArray) {
    out.write((field shl 3) or 2)
    writeVarint(out, bytes.size)
    out.write(bytes)
  }
}
