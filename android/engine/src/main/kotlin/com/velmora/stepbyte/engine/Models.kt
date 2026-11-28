package com.velmora.stepbyte.engine

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put

// Kotlin views of UniverseAPI's model (universeapi/crates/universeapi/src/model.rs).
// They hold the fields the app shows. Values that go back to the engine
// (titles, sources) keep the exact JSON the engine sent in `json`, so nothing
// is lost on the way back even when Kotlin doesn't model a field.

/** An engine value: typed fields for the app plus the JSON to hand back. */
class Wire<T>(val value: T, val json: JsonObject) {
  override fun toString(): String = value.toString()
}

typealias Title = Wire<TitleSummary>

typealias Source = Wire<SourceInfo>

@Serializable data class ProviderRef(val provider: String, val id: String)

@Serializable
data class TitleSummary(
  val origin: ProviderRef,
  val name: String,
  @SerialName("original_name") val originalName: String? = null,
  /** movie, series, anime or asian_drama. */
  val kind: String,
  val year: Int? = null,
  val poster: String? = null,
  val backdrop: String? = null,
  val overview: String? = null,
  val rating: Float? = null,
  val badges: List<String> = emptyList(),
  @SerialName("also_on") val alsoOn: List<ProviderRef> = emptyList(),
) {
  val episodic: Boolean
    get() = kind != "movie"
}

@Serializable
data class Episode(
  val season: Int,
  val number: Int,
  val title: String? = null,
  val overview: String? = null,
  @SerialName("air_date") val airDate: String? = null,
  val thumbnail: String? = null,
)

@Serializable
data class Season(val number: Int, val name: String? = null, val episodes: List<Episode> = emptyList())

@Serializable
data class TitleDetailInfo(
  val genres: List<String> = emptyList(),
  val cast: List<String> = emptyList(),
  val directors: List<String> = emptyList(),
  @SerialName("runtime_minutes") val runtimeMinutes: Int? = null,
  val trailer: String? = null,
  val screenshots: List<String> = emptyList(),
  val seasons: List<Season> = emptyList(),
  val extra: TitleExtra = TitleExtra(),
)

/** A person in the cast or crew. [role] is the character, or the job for crew. */
@Serializable data class Person(val name: String, val role: String? = null, val photo: String? = null)

/** What else the metadata source knows, for the title page. Every field is optional. */
@Serializable
data class TitleExtra(
  val tagline: String? = null,
  @SerialName("release_date") val releaseDate: String? = null,
  val status: String? = null,
  val certification: String? = null,
  @SerialName("original_language") val originalLanguage: String? = null,
  val languages: List<String> = emptyList(),
  val countries: List<String> = emptyList(),
  val studios: List<String> = emptyList(),
  @SerialName("vote_count") val voteCount: Int? = null,
  @SerialName("rating_source") val ratingSource: String? = null,
  @SerialName("season_count") val seasonCount: Int? = null,
  @SerialName("episode_count") val episodeCount: Int? = null,
  val format: String? = null,
  @SerialName("season_label") val seasonLabel: String? = null,
  val homepage: String? = null,
  val cast: List<Person> = emptyList(),
  val crew: List<Person> = emptyList(),
  val keywords: List<String> = emptyList(),
)

class TitleDetail(
  val title: Title,
  val info: TitleDetailInfo,
  /** "More like this", as titles that open their own pages. */
  val recommendations: List<Title> = emptyList(),
)

/** A row on the home screen (Trending movies, Popular anime...). */
class Rail(val id: String, val title: String, val items: List<Title>)

@Serializable
data class ReleaseTags(
  /** Vertical resolution: 360, 480, 720, 1080, 2160. */
  val resolution: Int? = null,
  /** h264, h265, av1 or vp9. */
  val codec: String? = null,
  @SerialName("bit_depth") val bitDepth: Int? = null,
  /** sdr, hdr10, hdr_plus or dolby_vision. */
  val hdr: String? = null,
  /** cam, web, bluray... */
  val source: String? = null,
  @SerialName("size_bytes") val sizeBytes: Long? = null,
  val audio: List<String> = emptyList(),
  val languages: List<String> = emptyList(),
  @SerialName("multi_audio") val multiAudio: Boolean = false,
  @SerialName("has_subs") val hasSubs: Boolean = false,
)

@Serializable
data class Track(
  val lang: String? = null,
  val label: String,
  val url: String? = null,
  val default: Boolean = false,
)

@Serializable
data class TorrentInfo(
  val magnet: String,
  @SerialName("info_hash") val infoHash: String? = null,
  val seeders: Int? = null,
  val leechers: Int? = null,
)

@Serializable
data class SourceInfo(
  val provider: String,
  /** The hoster or resolver that produced the URL ("HubCDN", "ZokoAnime"). */
  val host: String,
  val label: String,
  val url: String,
  /** hls, dash, mp4, mkv, webm, archive, torrent or unknown. */
  val format: String,
  val tags: ReleaseTags = ReleaseTags(),
  val filename: String? = null,
  @SerialName("size_bytes") val sizeBytes: Long? = null,
  val subtitles: List<Track> = emptyList(),
  val audio: List<Track> = emptyList(),
  val torrent: TorrentInfo? = null,
  @SerialName("is_pack") val isPack: Boolean = false,
  /** `true` when the verifier probed it and it served media. */
  val verified: Boolean? = null,
  val score: Int = 0,
) {
  val isTorrent: Boolean
    get() = format == "torrent" || torrent != null
}

@Serializable data class ProviderFailure(val provider: String, val message: String)

class SourcesReport(val sources: List<Source>, val errors: List<ProviderFailure>)

/** Progress of a source lookup, then the final report. */
sealed interface SourcesUpdate {
  /** The providers being asked, best first. */
  data class Plan(val providers: List<String>) : SourcesUpdate

  /** One provider finished. No sources and no error means it doesn't have it. */
  data class Batch(
    val provider: String,
    val sources: List<Source>,
    val error: String?,
    val cached: Boolean,
  ) : SourcesUpdate

  class Done(val report: SourcesReport) : SourcesUpdate
}

/** One provider's search results, as soon as it answers. */
class SearchBatch(val provider: String, val items: List<Title>, val error: String?)

/** What to look up sources for. */
sealed interface Target {
  data object Movie : Target

  data class Episode(val season: Int, val episode: Int) : Target

  fun toJson(): JsonObject = buildJsonObject {
    when (val t = this@Target) {
      Movie -> put("type", "movie")
      is Episode -> {
        put("type", "episode")
        put("season", t.season)
        put("episode", t.episode)
      }
    }
  }
}

@Serializable
data class SubtitleTrack(val label: String, val lang: String? = null, val url: String)

/** A source registered with the stream server. */
@Serializable
data class OpenedStream(
  /** Handle for [Universe.closeStream] and [Universe.torrentStatus]. */
  val id: String,
  /** Loopback URL for ExoPlayer. */
  val url: String,
  val torrent: Boolean,
  val subtitles: List<SubtitleTrack> = emptyList(),
)

@Serializable
data class TorrentProgress(
  val downloaded: Long,
  val total: Long,
  val uploaded: Long,
  val finished: Boolean,
) {
  val fraction: Float
    get() = if (total > 0) (downloaded.toDouble() / total).toFloat() else 0f
}

@Serializable
data class SelfTestCheck(val name: String, val ok: Boolean, val detail: String, val ms: Long)

@Serializable
data class SelfTest(
  val version: String,
  val arch: String,
  val streamBase: String,
  val checks: List<SelfTestCheck>,
) {
  val ok: Boolean
    get() = checks.all { it.ok }
}

// ── Live TV (universeapi/src/tv) ──

@Serializable
data class TvStream(
  val url: String,
  val quality: String? = null,
  @SerialName("geo_blocked") val geoBlocked: Boolean = false,
  @SerialName("not_24_7") val not247: Boolean = false,
)

@Serializable
data class TvChannel(
  val id: String,
  val name: String,
  val country: String? = null,
  val network: String? = null,
  val categories: List<String> = emptyList(),
  val website: String? = null,
  val logo: String? = null,
  val streams: List<TvStream> = emptyList(),
)

@Serializable data class TvPage(val total: Int = 0, val items: List<TvChannel> = emptyList())

/** A filter value with how many channels it has (a country, a category). */
@Serializable data class Facet(val id: String, val name: String, val count: Int = 0)

/** What to list: any mix of country, category and text, a page at a time. */
@Serializable
data class TvQuery(
  val country: String? = null,
  val category: String? = null,
  val text: String? = null,
  val offset: Int = 0,
  val limit: Int = 60,
)

// ── Sports (universeapi/src/sports) ──

@Serializable
data class League(val id: String, val name: String, val badge: String? = null, val country: String? = null)

@Serializable data class Team(val name: String, val badge: String? = null)

@Serializable
data class Match(
  /** `tsdb:<id>` or `fd:<id>`. */
  val id: String,
  val sport: String,
  val league: League,
  val title: String,
  val home: Team? = null,
  val away: Team? = null,
  /** upcoming, live, finished, postponed or cancelled. */
  val status: String,
  /** Kickoff, unix seconds (UTC). */
  val kickoff: Long? = null,
  @SerialName("home_score") val homeScore: Int? = null,
  @SerialName("away_score") val awayScore: Int? = null,
  /** "1H", "HT", "Q3" while live, when the source says. */
  val phase: String? = null,
  val venue: String? = null,
  val round: String? = null,
) {
  val live: Boolean
    get() = status == "live"
}

// ── Downloads (universeapi/src/download.rs) ──

/** A download this run. [state] is queued, running, paused, extracting, completed, failed or cancelled. */
@Serializable
data class DownloadStatus(
  val id: Long,
  val title: String,
  val state: DownloadState,
  val downloaded: Long = 0,
  val total: Long? = null,
  @SerialName("bytes_per_sec") val bytesPerSec: Long = 0,
) {
  val fraction: Float
    get() = total?.takeIf { it > 0 }?.let { (downloaded.toDouble() / it).toFloat().coerceIn(0f, 1f) } ?: 0f
}

@Serializable
data class DownloadState(
  val state: String,
  /** The finished file, once completed. */
  val path: String? = null,
  val error: String? = null,
) {
  val active: Boolean
    get() = state == "queued" || state == "running" || state == "extracting"
}

/** A finished video in the download folder. */
@Serializable
data class LocalFile(
  val path: String,
  /** File name without the extension: "Dune (2021) 1080p". */
  val title: String,
  val size: Long,
  /** Unix seconds. */
  val modified: Long,
)
