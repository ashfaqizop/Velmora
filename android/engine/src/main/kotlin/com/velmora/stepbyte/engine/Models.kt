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
  val seasons: List<Season> = emptyList(),
)

class TitleDetail(val title: Title, val info: TitleDetailInfo)

@Serializable
data class ReleaseTags(
  /** Vertical resolution: 360, 480, 720, 1080, 2160. */
  val resolution: Int? = null,
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
