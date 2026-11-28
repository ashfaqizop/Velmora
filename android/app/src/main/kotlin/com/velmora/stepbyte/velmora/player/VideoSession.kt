package com.velmora.stepbyte.velmora.player

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.util.Log
import androidx.annotation.OptIn
import androidx.media3.common.AudioAttributes
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultDataSource
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.exoplayer.DefaultLoadControl
import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import com.velmora.stepbyte.engine.Episode
import com.velmora.stepbyte.engine.OpenedStream
import com.velmora.stepbyte.engine.Season
import com.velmora.stepbyte.engine.Source
import com.velmora.stepbyte.engine.Target
import com.velmora.stepbyte.engine.Title
import com.velmora.stepbyte.engine.TitleDetail
import com.velmora.stepbyte.engine.TorrentProgress
import com.velmora.stepbyte.engine.Universe
import com.velmora.stepbyte.velmora.library.VelmoraLibrary
import com.velmora.stepbyte.velmora.player.SourceLabels.key
import com.velmora.stepbyte.velmora.screens.DetailCache
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

/** Where a playback is. */
enum class Phase {
  /** Asking the sites. */
  Finding,
  /** Mobile data, and the best source is a torrent: asking first. */
  AskData,
  /** Registering the source with the stream server (a torrent joins its swarm). */
  Opening,
  Playing,
  /** Every source failed, or none was found. */
  Failed,
  /** The last episode (or the movie) finished. */
  Ended,
}

data class PlaybackUi(
  val title: Title? = null,
  val target: Target = Target.Movie,
  val phase: Phase = Phase.Finding,
  val finding: Finding? = null,
  val sources: List<Source> = emptyList(),
  val current: Source? = null,
  val tried: Set<String> = emptySet(),
  val message: String? = null,
  val torrent: TorrentProgress? = null,
  val seasons: List<Season> = emptyList(),
  val dub: Boolean = false,
  /** The next episode, offered near the end. */
  val next: Episode? = null,
  /** Playing a downloaded file, not a stream. */
  val local: Boolean = false,
)

/**
 * The video being watched, one at a time, for the whole process. The player
 * activity draws it; the background service keeps it going with the screen
 * off. Picking and switching sources, falling over to the next one when a
 * source fails, saving the spot and moving to the next episode live here.
 */
@OptIn(UnstableApi::class)
object VideoSession {
  private const val TAG = "VelmoraPlayer"

  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
  private val _ui = MutableStateFlow(PlaybackUi())
  val ui: StateFlow<PlaybackUi> = _ui.asStateFlow()

  private lateinit var app: Context
  var player: ExoPlayer? = null
    private set

  private var opened: OpenedStream? = null
  private var job: Job? = null
  private var ticker: Job? = null
  private var startAtMs = 0L
  /** The current source started playing, so a later error is mid-playback. */
  private var reachedReady = false
  private var retriedSame = false
  /** The title as sources are looked up for it (ids and sites from its details). */
  private var lookupTitle: Title? = null

  // ── Starting ──

  /** Play [title] ([target] for an episode), from the saved spot unless [fromStart]. */
  fun play(context: Context, title: Title, target: Target? = null, fromStart: Boolean = false) {
    app = context.applicationContext
    val saved = VelmoraLibrary.progressOf(title)
    val detail = DetailCache.get(VelmoraLibrary.key(title))
    val seasons = detail?.info?.seasons.orEmpty()
    val chosen = target ?: targetFor(title, seasons)
    val sameSpot =
      saved != null && saved.resumable && !fromStart &&
        when (chosen) {
          Target.Movie -> saved.season == null
          is Target.Episode -> saved.season == chosen.season && saved.episode == chosen.episode
        }
    startAtMs = if (sameSpot) (saved!!.position * 1000).toLong() else 0L
    begin(title, chosen, detail, seasons)
    context.startActivity(Intent(context, VideoPlayerActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
  }

  /** What Play plays: the saved episode, else the first one; a movie is the movie. */
  fun targetFor(title: Title, seasons: List<Season>): Target {
    val saved = VelmoraLibrary.progressOf(title)
    return when {
      !title.value.episodic -> Target.Movie
      saved?.season != null && saved.episode != null -> Target.Episode(saved.season, saved.episode)
      else -> firstEpisode(seasons) ?: Target.Episode(1, 1)
    }
  }

  private fun firstEpisode(seasons: List<Season>): Target.Episode? {
    val s = seasons.filter { it.number > 0 && it.episodes.isNotEmpty() }.minByOrNull { it.number } ?: return null
    val e = s.episodes.minByOrNull { it.number } ?: return null
    return Target.Episode(s.number, e.number)
  }

  private fun begin(title: Title, target: Target, detail: TitleDetail?, seasons: List<Season>) {
    job?.cancel()
    closeStream()
    ensurePlayer()
    player?.stop()
    player?.clearMediaItems()
    reachedReady = false
    retriedSame = false
    val dub = title.value.kind == "anime" && PlayerPrefs.dub(app)
    val lookup = SourceFinder.lookupTitle(title, detail)
    lookupTitle = lookup
    _ui.value = PlaybackUi(title = title, target = target, phase = Phase.Finding, seasons = seasons, dub = dub)
    // Episodes for Next and the picker, if the title page hadn't loaded them.
    // Not tied to the source lookup, which a quick start cancels.
    if (title.value.episodic && seasons.isEmpty()) {
      scope.launch {
        runCatching { Universe.get(app).detail(title) }.getOrNull()?.let { d ->
          DetailCache.put(VelmoraLibrary.key(title), d)
          if (_ui.value.title === title) _ui.update { it.copy(seasons = d.info.seasons) }
        }
      }
    }
    // Downloaded: play the file, no lookup, no network.
    com.velmora.stepbyte.velmora.downloads.VelmoraDownloads.localFile(title, target)?.let { path ->
      playLocal(path)
      return
    }
    job =
      scope.launch {
        val finding = SourceFinder.find(app, lookup, target, dub)
        _ui.update { it.copy(finding = finding) }
        launch { finding.sources.collect { s -> _ui.update { it.copy(sources = if (it.current == null) s else it.sources.ifEmpty { s }) } } }
        val report =
          try {
            finding.result.await()
          } catch (e: CancellationException) {
            throw e
          } catch (e: Throwable) {
            fail("Couldn't look for sources: ${e.message ?: e.javaClass.simpleName}")
            return@launch
          }
        // "Play now" may have started one already.
        if (_ui.value.current != null) {
          _ui.update { it.copy(sources = report.sources) }
          return@launch
        }
        _ui.update { it.copy(sources = report.sources) }
        pickAndPlay()
      }
  }

  private fun playLocal(path: String) {
    _ui.update { it.copy(phase = Phase.Opening, local = true, message = null) }
    job =
      scope.launch {
        try {
          val stream = Universe.get(app).openLocal(path)
          opened = null
          val title = _ui.value.title
          val subs =
            stream.subtitles.mapNotNull { t ->
              val mime = subtitleMime(t.url) ?: return@mapNotNull null
              MediaItem.SubtitleConfiguration.Builder(Uri.parse(t.url)).setMimeType(mime).setLanguage(t.lang).setLabel(t.label).build()
            }
          val item =
            MediaItem.Builder()
              .setUri(stream.url)
              .setMediaId(stream.id)
              .setSubtitleConfigurations(subs)
              .setMediaMetadata(MediaMetadata.Builder().setTitle(title?.value?.name).setSubtitle(episodeLabel(_ui.value.target)).build())
              .build()
          val p = ensurePlayer()
          p.setMediaItem(item, startAtMs.coerceAtLeast(0))
          p.prepare()
          p.playWhenReady = true
          _ui.update { it.copy(phase = Phase.Playing) }
          VideoService.start(app)
          startTicker()
        } catch (e: CancellationException) {
          throw e
        } catch (e: Throwable) {
          fail("Your download didn't open: ${e.message ?: e.javaClass.simpleName}")
        }
      }
  }

  /** Don't wait for the slow sites: play the best found so far. */
  fun playNow() {
    if (_ui.value.phase != Phase.Finding) return
    val sources = _ui.value.finding?.sources?.value.orEmpty()
    if (sources.isEmpty()) return
    _ui.update { it.copy(sources = rank(sources)) }
    pickAndPlay()
  }

  private fun rank(sources: List<Source>): List<Source> =
    sources.sortedWith(compareByDescending<Source> { it.value.verified == true }.thenByDescending { it.value.score })

  /** The best source not tried yet, following the mobile data rules. */
  private fun pickAndPlay() {
    val s = _ui.value
    val metered = PlayerPrefs.metered(app)
    val rule = PlayerPrefs.mobileData(app)
    var pool = s.sources.filter { it.key !in s.tried }
    if (metered && rule == PlayerPrefs.MobileData.NoTorrents) pool = pool.filterNot { it.value.isTorrent }
    if (metered && PlayerPrefs.saveData(app)) {
      val light = pool.filter { (it.value.tags.resolution ?: 0) in 1..720 }
      if (light.isNotEmpty()) pool = light + (pool - light.toSet())
    }
    val best = pool.firstOrNull()
    if (best == null) {
      fail(
        when {
          s.sources.isEmpty() -> "No source has this one yet. Try again later, or switch sub and dub."
          metered && rule == PlayerPrefs.MobileData.NoTorrents -> "Only torrents have this one, and torrents are off on mobile data."
          else -> "None of the sources would play."
        },
      )
      return
    }
    if (best.value.isTorrent && metered && rule == PlayerPrefs.MobileData.Ask) {
      _ui.update { it.copy(phase = Phase.AskData, current = best) }
      return
    }
    open(best, startAtMs)
  }

  /** After the mobile data question. */
  fun answerData(allow: Boolean) {
    val s = _ui.value
    val pending = s.current ?: return
    if (allow) {
      open(pending, startAtMs)
    } else {
      _ui.update { it.copy(tried = it.tried + pending.key, current = null) }
      val others = s.sources.filter { !it.value.isTorrent && it.key !in s.tried }
      if (others.isEmpty()) fail("Only torrents have this one. It plays on Wi-Fi.") else open(others.first(), startAtMs)
    }
  }

  /** Switch to another source, keeping the spot. */
  fun switchTo(source: Source) {
    val at = player?.currentPosition?.takeIf { it > 0 } ?: startAtMs
    open(source, at)
  }

  /** Every source failed or the user wants another: the next one in line. */
  fun tryNext() {
    _ui.value.current?.let { cur -> _ui.update { it.copy(tried = it.tried + cur.key) } }
    pickAndPlay()
  }

  private fun open(source: Source, atMs: Long) {
    job?.cancel()
    val target = _ui.value.target
    _ui.update { it.copy(phase = Phase.Opening, current = source, message = null, torrent = null, next = null) }
    reachedReady = false
    job =
      scope.launch {
        closeStream()
        try {
          val engine = Universe.get(app)
          val stream = engine.open(source, target)
          opened = stream
          val v = source.value
          val title = _ui.value.title
          val subs =
            stream.subtitles.mapNotNull { t ->
              val mime = subtitleMime(t.url) ?: return@mapNotNull null
              MediaItem.SubtitleConfiguration.Builder(Uri.parse(t.url))
                .setMimeType(mime)
                .setLanguage(t.lang)
                .setLabel(t.label)
                .build()
            }
          val item =
            MediaItem.Builder()
              .setUri(stream.url)
              .setMediaId(stream.id)
              .setSubtitleConfigurations(subs)
              .setMediaMetadata(
                MediaMetadata.Builder()
                  .setTitle(title?.value?.name)
                  .setSubtitle(episodeLabel(target))
                  .setArtworkUri(title?.value?.backdrop?.let(Uri::parse) ?: title?.value?.poster?.let(Uri::parse))
                  .build(),
              )
              .apply {
                when (v.format) {
                  "hls" -> setMimeType(MimeTypes.APPLICATION_M3U8)
                  "dash" -> setMimeType(MimeTypes.APPLICATION_MPD)
                }
              }
              .build()
          val p = ensurePlayer()
          p.setMediaItem(item, atMs.coerceAtLeast(0))
          p.prepare()
          p.playWhenReady = true
          _ui.update { it.copy(phase = Phase.Playing) }
          VideoService.start(app)
          startTicker()
        } catch (e: CancellationException) {
          throw e
        } catch (e: Throwable) {
          Log.w(TAG, "couldn't open ${source.value.label}", e)
          _ui.update { it.copy(tried = it.tried + source.key, message = "That source didn't open. Trying the next one…") }
          pickAndPlay()
        }
      }
  }

  private fun subtitleMime(url: String): String? {
    // Downloaded subtitles carry their type in the fragment: /f/{token}#.vtt
    val path = url.substringAfter('#', url.substringBefore('?')).lowercase()
    return when {
      path.endsWith(".vtt") -> MimeTypes.TEXT_VTT
      path.endsWith(".srt") -> MimeTypes.APPLICATION_SUBRIP
      path.endsWith(".ass") || path.endsWith(".ssa") -> MimeTypes.TEXT_SSA
      path.endsWith(".ttml") || path.endsWith(".xml") -> MimeTypes.APPLICATION_TTML
      // Unknown: most sites serve WebVTT.
      else -> MimeTypes.TEXT_VTT
    }
  }

  fun episodeLabel(target: Target): String? =
    (target as? Target.Episode)?.let { t ->
      val name = _ui.value.seasons.firstOrNull { it.number == t.season }?.episodes?.firstOrNull { it.number == t.episode }?.title
      "S${t.season} · E${t.episode}" + (name?.takeIf { it.isNotBlank() }?.let { " · $it" } ?: "")
    }

  private fun fail(message: String) {
    _ui.update { it.copy(phase = Phase.Failed, message = message) }
  }

  // ── The player ──

  private fun ensurePlayer(): ExoPlayer {
    player?.let {
      return it
    }
    // Torrents can sit a while before their first pieces: generous timeouts.
    val http = DefaultHttpDataSource.Factory().setConnectTimeoutMs(20_000).setReadTimeoutMs(60_000).setAllowCrossProtocolRedirects(true)
    val data = DefaultDataSource.Factory(app, http)
    val renderers =
      DefaultRenderersFactory(app)
        // The phone's own decoders first; FFmpeg for AC3, E-AC3, DTS and TrueHD where it has none.
        .setExtensionRendererMode(DefaultRenderersFactory.EXTENSION_RENDERER_MODE_ON)
        .setEnableDecoderFallback(true)
    val load = DefaultLoadControl.Builder().setBufferDurationsMs(20_000, 90_000, 2_500, 5_000).build()
    val p =
      ExoPlayer.Builder(app, renderers)
        .setMediaSourceFactory(DefaultMediaSourceFactory(data))
        .setLoadControl(load)
        .setAudioAttributes(AudioAttributes.Builder().setUsage(C.USAGE_MEDIA).setContentType(C.AUDIO_CONTENT_TYPE_MOVIE).build(), true)
        .setHandleAudioBecomingNoisy(true)
        .setSeekBackIncrementMs(10_000)
        .setSeekForwardIncrementMs(10_000)
        .build()
    p.addListener(listener)
    applyLanguages(p)
    player = p
    return p
  }

  /** The account's audio and subtitle languages, as the starting choice. */
  private fun applyLanguages(p: ExoPlayer) {
    runCatching {
      val prefs = Universe.savedPreferences(app)?.let { Universe.json.parseToJsonElement(it) as? kotlinx.serialization.json.JsonObject }
      fun list(k: String) =
        (prefs?.get(k) as? kotlinx.serialization.json.JsonArray)?.mapNotNull { (it as? kotlinx.serialization.json.JsonPrimitive)?.content }.orEmpty()
      val audio = list("audio_languages")
      val subs = list("subtitle_languages")
      p.trackSelectionParameters =
        p.trackSelectionParameters
          .buildUpon()
          .apply {
            if (audio.isNotEmpty()) setPreferredAudioLanguages(*audio.toTypedArray())
            if (subs.isNotEmpty()) setPreferredTextLanguages(*subs.toTypedArray())
          }
          .build()
    }
  }

  private val listener =
    object : Player.Listener {
      override fun onPlaybackStateChanged(state: Int) {
        when (state) {
          Player.STATE_READY -> {
            reachedReady = true
            retriedSame = false
            _ui.update { it.copy(message = null) }
          }
          Player.STATE_ENDED -> ended()
          else -> {}
        }
      }

      override fun onIsPlayingChanged(isPlaying: Boolean) {
        if (!isPlaying) saveSpot()
      }

      override fun onPlayerError(error: PlaybackException) {
        if (_ui.value.local) {
          fail("Your download didn't play (${error.errorCodeName.removePrefix("ERROR_CODE_").lowercase().replace('_', ' ')}). Delete it and download again, or stream it.")
          return
        }
        val cur = _ui.value.current ?: return
        Log.w(TAG, "playback error on ${cur.value.label}: ${error.errorCodeName}", error)
        val at = player?.currentPosition ?: 0L
        if (reachedReady && !retriedSame) {
          // It was playing: the connection dropped. Reopen the same one where it stopped.
          retriedSame = true
          _ui.update { it.copy(message = "Reconnecting…") }
          open(cur, at)
          return
        }
        if (!reachedReady) {
          // A cached source that won't play: look afresh next time.
          lookupTitle?.let { t -> SourceFinder.forget(app, t, _ui.value.target, _ui.value.dub) }
        }
        startAtMs = at.takeIf { it > 0 } ?: startAtMs
        _ui.update { it.copy(tried = it.tried + cur.key, message = "That source stopped (${error.errorCodeName.removePrefix("ERROR_CODE_").lowercase().replace('_', ' ')}). Trying the next one…") }
        pickAndPlay()
      }
    }

  // ── Progress ──

  private fun startTicker() {
    ticker?.cancel()
    ticker =
      scope.launch {
        var n = 0
        while (isActive) {
          delay(1_000)
          n++
          val p = player ?: continue
          val s = _ui.value
          if (opened?.torrent == true) {
            opened?.let { o -> runCatching { Universe.get(app).torrentStatus(o) }.getOrNull()?.let { t -> _ui.update { it.copy(torrent = t) } } }
          }
          if (p.isPlaying && n % 15 == 0) saveSpot()
          // Offer the next episode in the last 40 seconds.
          val left = p.duration - p.currentPosition
          if (p.duration > 0 && left in 1..40_000 && s.next == null) {
            nextEpisode()?.let { e -> _ui.update { it.copy(next = e) } }
          }
        }
      }
  }

  private fun saveSpot() {
    val p = player ?: return
    val s = _ui.value
    val title = s.title ?: return
    if (s.phase != Phase.Playing || !reachedReady) return
    val dur = p.duration
    if (dur <= 0 || dur == C.TIME_UNSET) return
    val ep = s.target as? Target.Episode
    VelmoraLibrary.saveProgress(title, ep?.season, ep?.episode, p.currentPosition / 1000.0, dur / 1000.0)
  }

  fun nextEpisode(): Episode? {
    val s = _ui.value
    val t = s.target as? Target.Episode ?: return null
    val seasons = s.seasons.filter { it.number > 0 }.sortedBy { it.number }
    val season = seasons.firstOrNull { it.number == t.season }
    season?.episodes?.sortedBy { it.number }?.firstOrNull { it.number > t.episode }?.let {
      return it
    }
    return seasons.firstOrNull { it.number > t.season }?.episodes?.minByOrNull { it.number }
  }

  private fun ended() {
    saveSpot()
    val next = nextEpisode()
    if (next != null && PlayerPrefs.autoNext(app)) {
      _ui.update { it.copy(next = next) }
    } else {
      _ui.update { it.copy(phase = Phase.Ended, next = next) }
    }
  }

  /** Go to an episode of the same title. */
  fun playEpisode(season: Int, episode: Int) {
    val title = _ui.value.title ?: return
    saveSpot()
    startAtMs = 0L
    val saved = VelmoraLibrary.progressOf(title)
    if (saved != null && saved.resumable && saved.season == season && saved.episode == episode) startAtMs = (saved.position * 1000).toLong()
    begin(title, Target.Episode(season, episode), DetailCache.get(VelmoraLibrary.key(title)), _ui.value.seasons)
  }

  fun playNext() {
    val e = nextEpisode() ?: return
    playEpisode(e.season, e.number)
  }

  fun dismissNext() {
    _ui.update { it.copy(next = null) }
  }

  /** Anime: switch between subtitled and dubbed, looking sources up again. */
  fun setDub(dub: Boolean) {
    val s = _ui.value
    val title = s.title ?: return
    PlayerPrefs.setDub(app, dub)
    startAtMs = player?.currentPosition ?: startAtMs
    begin(title, s.target, DetailCache.get(VelmoraLibrary.key(title)), s.seasons)
  }

  // ── Stopping ──

  private fun closeStream() {
    val o = opened ?: return
    opened = null
    val ctx = if (::app.isInitialized) app else return
    // A streaming torrent goes with its pieces.
    CoroutineScope(Dispatchers.IO).launch { runCatching { Universe.get(ctx).closeStream(o) } }
  }

  /** Done watching: save, free the stream and the player. */
  fun close() {
    saveSpot()
    job?.cancel()
    ticker?.cancel()
    closeStream()
    player?.removeListener(listener)
    player?.release()
    player = null
    if (::app.isInitialized) VideoService.stop(app)
    _ui.value = PlaybackUi()
  }
}
