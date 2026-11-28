package com.velmora.stepbyte.velmora.privacy

import com.velmora.stepbyte.engine.Target
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.velmora.library.VelmoraLibrary
import com.velmora.stepbyte.velmora.social.ActivityPoster

/**
 * The player's once-a-second tick, turned into the two moments that count:
 * a minute in (usage counts, "watching" on Social) and the end (95%,
 * "finished"). Each once per title and episode per play.
 */
object WatchReports {
  private var playing: String? = null
  private var seconds = 0
  private var started = false
  private var done = false

  /** Called every second while the video plays. Live channels don't count. */
  fun tick(title: Title, target: Target, provider: String?, resolution: Int?, positionMs: Long, durationMs: Long) {
    val ep = target as? Target.Episode
    val key = VelmoraLibrary.key(title) + "|" + (ep?.let { "${it.season}x${it.episode}" } ?: "")
    if (key != playing) {
      playing = key
      seconds = 0
      started = false
      done = false
    }
    seconds++
    if (!started && seconds >= 60) {
      started = true
      UsageStats.watched(title, provider, resolution)
      ActivityPoster.watching(title, target)
    }
    if (!done && started && durationMs > 0 && positionMs >= durationMs * 0.95) {
      done = true
      ActivityPoster.finished(title, target, durationMs / 1000.0)
    }
  }
}
