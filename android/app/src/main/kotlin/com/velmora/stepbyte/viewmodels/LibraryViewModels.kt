@file:OptIn(ExperimentalCoroutinesApi::class)

package com.velmora.stepbyte.viewmodels

import android.content.Context
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.velmora.stepbyte.innertube.YouTube
import com.velmora.stepbyte.innertube.models.PlaylistItem
import com.velmora.stepbyte.innertube.models.SongItem
import com.velmora.stepbyte.innertube.pages.HomePage
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import com.velmora.stepbyte.constants.AlbumFilter
import com.velmora.stepbyte.constants.AlbumFilterKey
import com.velmora.stepbyte.constants.AlbumSortDescendingKey
import com.velmora.stepbyte.constants.AlbumSortType
import com.velmora.stepbyte.constants.AlbumSortTypeKey
import com.velmora.stepbyte.constants.ArtistFilter
import com.velmora.stepbyte.constants.ArtistFilterKey
import com.velmora.stepbyte.constants.ArtistSongSortDescendingKey
import com.velmora.stepbyte.constants.ArtistSongSortType
import com.velmora.stepbyte.constants.ArtistSongSortTypeKey
import com.velmora.stepbyte.constants.ArtistSortDescendingKey
import com.velmora.stepbyte.constants.ArtistSortType
import com.velmora.stepbyte.constants.ArtistSortTypeKey
import com.velmora.stepbyte.constants.ExportedSongIdsKey
import com.velmora.stepbyte.constants.HideExplicitKey
import com.velmora.stepbyte.constants.HideVideoSongsKey
import com.velmora.stepbyte.constants.HideYoutubeShortsKey
import com.velmora.stepbyte.constants.LibraryFilter
import com.velmora.stepbyte.constants.PlaylistSortDescendingKey
import com.velmora.stepbyte.constants.PlaylistSortType
import com.velmora.stepbyte.constants.PlaylistSortTypeKey
import com.velmora.stepbyte.constants.SongFilter
import com.velmora.stepbyte.constants.SongFilterKey
import com.velmora.stepbyte.constants.SongSortDescendingKey
import com.velmora.stepbyte.constants.SongSortType
import com.velmora.stepbyte.constants.SongSortTypeKey
import com.velmora.stepbyte.constants.TopSize
import com.velmora.stepbyte.db.MusicDatabase
import com.velmora.stepbyte.extensions.filterExplicit
import com.velmora.stepbyte.extensions.filterExplicitAlbums
import com.velmora.stepbyte.extensions.filterVideoSongs
import com.velmora.stepbyte.extensions.filterYoutubeShorts
import com.velmora.stepbyte.extensions.toEnum
import com.velmora.stepbyte.playback.DownloadUtil
import com.velmora.stepbyte.utils.EpisodeDates
import com.velmora.stepbyte.utils.PodcastShowStore
import com.velmora.stepbyte.utils.SyncUtils
import com.velmora.stepbyte.utils.dataStore
import com.velmora.stepbyte.utils.reportException
import java.time.Duration
import java.time.LocalDateTime
import javax.inject.Inject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

@HiltViewModel
class LibrarySongsViewModel
@Inject
constructor(
  @ApplicationContext context: Context,
  database: MusicDatabase,
  downloadUtil: DownloadUtil,
  private val syncUtils: SyncUtils,
) : ViewModel() {
  val allSongs =
    context.dataStore.data
      .map {
        Triple(
          Triple(
            (try {
                it[SongFilterKey]
              } catch (e: Exception) {
                null
              })
              .toEnum(SongFilter.LIKED),
            (try {
                it[SongSortTypeKey]
              } catch (e: Exception) {
                null
              })
              .toEnum(SongSortType.CREATE_DATE),
            ((try {
              it[SongSortDescendingKey]
            } catch (e: Exception) {
              null
            }) ?: true),
          ),
          (try {
            it[ExportedSongIdsKey]
          } catch (e: Exception) {
            null
          }) ?: "",
          Pair(
            (try {
              it[HideExplicitKey]
            } catch (e: Exception) {
              null
            }) ?: false,
            (try {
              it[HideVideoSongsKey]
            } catch (e: Exception) {
              null
            }) ?: false,
          ),
        )
      }
      .distinctUntilChanged()
      .flatMapLatest { (filterSort, exportedSongIds, hideConfig) ->
        val (filter, sortType, descending) = filterSort
        val (hideExplicit, hideVideoSongs) = hideConfig
        when (filter) {
          SongFilter.LIBRARY ->
            database.songs(sortType, descending).map {
              it.filterExplicit(hideExplicit).filterVideoSongs(hideVideoSongs)
            }
          SongFilter.LIKED ->
            database.likedSongs(sortType, descending).map {
              it.filterExplicit(hideExplicit).filterVideoSongs(hideVideoSongs)
            }
          SongFilter.DOWNLOADED ->
            database.downloadedSongs(sortType, descending).map {
              it.filterExplicit(hideExplicit).filterVideoSongs(hideVideoSongs)
            }
          SongFilter.UPLOADED ->
            database.uploadedSongs(sortType, descending).map {
              it.filterExplicit(hideExplicit).filterVideoSongs(hideVideoSongs)
            }
          SongFilter.EXPORTED -> {
            val ids = exportedSongIds.split(",").filter { it.isNotBlank() }
            database.getSongsByIdsFlow(ids).map {
              it.filterExplicit(hideExplicit).filterVideoSongs(hideVideoSongs)
            }
          }
        }
      }
      .stateIn(viewModelScope, SharingStarted.Lazily, emptyList())

  fun syncLikedSongs() {
    viewModelScope.launch(Dispatchers.IO) { syncUtils.syncLikedSongs() }
  }

  fun syncLibrarySongs() {
    viewModelScope.launch(Dispatchers.IO) { syncUtils.syncLibrarySongs() }
  }

  fun syncUploadedSongs() {
    viewModelScope.launch(Dispatchers.IO) { syncUtils.syncUploadedSongs() }
  }
}

@HiltViewModel
class LibraryArtistsViewModel
@Inject
constructor(
  @ApplicationContext context: Context,
  database: MusicDatabase,
  private val syncUtils: SyncUtils,
) : ViewModel() {
  val allArtists =
    context.dataStore.data
      .map {
        Triple(
          (try {
              it[ArtistFilterKey]
            } catch (e: Exception) {
              null
            })
            .toEnum(ArtistFilter.LIKED),
          (try {
              it[ArtistSortTypeKey]
            } catch (e: Exception) {
              null
            })
            .toEnum(ArtistSortType.CREATE_DATE),
          (try {
            it[ArtistSortDescendingKey]
          } catch (e: Exception) {
            null
          }) ?: true,
        )
      }
      .distinctUntilChanged()
      .flatMapLatest { (filter, sortType, descending) ->
        when (filter) {
          ArtistFilter.LIKED -> database.artistsBookmarked(sortType, descending)
          ArtistFilter.LIBRARY -> database.artists(sortType, descending)
        }
      }
      .stateIn(viewModelScope, SharingStarted.Lazily, emptyList())

  fun sync() {
    viewModelScope.launch(Dispatchers.IO) { syncUtils.syncArtistsSubscriptions() }
  }

  init {
    viewModelScope.launch(Dispatchers.IO) {
      allArtists.collect { artists ->
        artists
          .map { it.artist }
          .filter {
            !it.isLocal &&
              !it.id.startsWith("LOCAL_ARTIST_") &&
              (it.thumbnailUrl == null ||
                Duration.between(it.lastUpdateTime, LocalDateTime.now()) > Duration.ofDays(10))
          }
          .forEach { artist ->
            YouTube.artist(artist.id).onSuccess { artistPage ->
              database.query { update(artist, artistPage) }
            }
          }
      }
    }
  }
}

@HiltViewModel
class LibraryAlbumsViewModel
@Inject
constructor(
  @ApplicationContext context: Context,
  database: MusicDatabase,
  private val syncUtils: SyncUtils,
) : ViewModel() {
  val allAlbums =
    context.dataStore.data
      .map {
        Pair(
          Triple(
            (try {
                it[AlbumFilterKey]
              } catch (e: Exception) {
                null
              })
              .toEnum(AlbumFilter.LIKED),
            (try {
                it[AlbumSortTypeKey]
              } catch (e: Exception) {
                null
              })
              .toEnum(AlbumSortType.CREATE_DATE),
            (try {
              it[AlbumSortDescendingKey]
            } catch (e: Exception) {
              null
            }) ?: true,
          ),
          (try {
            it[HideExplicitKey]
          } catch (e: Exception) {
            null
          }) ?: false,
        )
      }
      .distinctUntilChanged()
      .flatMapLatest { (filterSort, hideExplicit) ->
        val (filter, sortType, descending) = filterSort
        when (filter) {
          AlbumFilter.LIKED ->
            database.albumsLiked(sortType, descending).map { it.filterExplicitAlbums(hideExplicit) }
          AlbumFilter.LIBRARY ->
            database.albums(sortType, descending).map { it.filterExplicitAlbums(hideExplicit) }
          AlbumFilter.UPLOADED ->
            database.albumsUploaded(sortType, descending).map {
              it.filterExplicitAlbums(hideExplicit)
            }
        }
      }
      .stateIn(viewModelScope, SharingStarted.Lazily, emptyList())

  fun sync() {
    viewModelScope.launch(Dispatchers.IO) { syncUtils.syncLikedAlbums() }
  }

  init {
    viewModelScope.launch(Dispatchers.IO) {
      allAlbums.collect { albums ->
        albums
          .filter {
            !it.album.isLocal && !it.id.startsWith("LOCAL_ALBUM_") && it.album.songCount == 0
          }
          .forEach { album ->
            YouTube.album(album.id)
              .onSuccess { albumPage ->
                database.query { update(album.album, albumPage, album.artists) }
              }
              .onFailure {
                reportException(it)
                if (it.message?.contains("NOT_FOUND") == true) {
                  database.query { delete(album.album) }
                }
              }
          }
      }
    }
  }
}

@HiltViewModel
class LibraryPlaylistsViewModel
@Inject
constructor(
  @ApplicationContext context: Context,
  database: MusicDatabase,
  private val syncUtils: SyncUtils,
) : ViewModel() {
  val allPlaylists =
    context.dataStore.data
      .map {
        Triple(
          (try {
              it[PlaylistSortTypeKey]
            } catch (e: Exception) {
              null
            })
            .toEnum(PlaylistSortType.CREATE_DATE),
          (try {
            it[PlaylistSortDescendingKey]
          } catch (e: Exception) {
            null
          }) ?: true,
          (try {
            it[HideYoutubeShortsKey]
          } catch (e: Exception) {
            null
          }) ?: false,
        )
      }
      .distinctUntilChanged()
      .flatMapLatest { (sortType, descending, hideYoutubeShorts) ->
        database.playlists(sortType, descending).map { it.filterYoutubeShorts(hideYoutubeShorts) }
      }
      .stateIn(viewModelScope, SharingStarted.Lazily, emptyList())

  fun sync() {
    viewModelScope.launch(Dispatchers.IO) { syncUtils.syncSavedPlaylists() }
  }

  val topValue =
    context.dataStore.data
      .map {
        (try {
          it[TopSize]
        } catch (e: Exception) {
          null
        }) ?: "50"
      }
      .distinctUntilChanged()
}

/**
 * Followed podcast shows (saved playlists known as shows, see [PodcastShowStore]) and their latest
 * episodes, for the Library's Podcast tab and the Home Podcast chip.
 */
@HiltViewModel
class PodcastsViewModel
@Inject
constructor(
  database: MusicDatabase,
  podcastShows: PodcastShowStore,
) : ViewModel() {
  private val followed =
    combine(database.playlists(PlaylistSortType.NAME, false), podcastShows.ids) { playlists, ids ->
      playlists.filter { it.playlist.browseId in ids && it.playlist.bookmarkedAt != null }
    }

  val podcasts = followed.stateIn(viewModelScope, SharingStarted.Lazily, emptyList())

  val exploreSections = MutableStateFlow<List<HomePage.Section>>(emptyList())
  val isLoading = MutableStateFlow(false)
  val isRefreshing = MutableStateFlow(false)
  val isLoadingMore = MutableStateFlow(false)
  private var continuation: String? = null
  private var hasLoadedAll = false

  fun loadPodcastSections() {
    viewModelScope.launch(Dispatchers.IO) {
      if (exploreSections.value.isNotEmpty()) return@launch
      isLoading.value = true
      try {
        val homeRes = YouTube.home().getOrNull()
        val chip = homeRes?.chips?.firstOrNull { it.title.contains("podcast", ignoreCase = true) }
        val browseId = chip?.endpoint?.browseId ?: "FEmusic_home"
        val params = chip?.endpoint?.params
        val sectionsRes = YouTube.home(browseId = browseId, params = params).getOrNull()
        continuation = sectionsRes?.continuation
        val sections =
          sectionsRes?.sections?.mapNotNull { section ->
            val filtered =
              section.items.filter {
                (it is SongItem && it.isPodcastEpisode) ||
                  (it is PlaylistItem && it.isPodcast) ||
                  (it is PlaylistItem)
              }
            filtered.takeIf { it.isNotEmpty() }?.let { section.copy(items = it) }
          } ?: emptyList()
        exploreSections.value = sections
      } catch (e: Exception) {
        reportException(e)
      } finally {
        isLoading.value = false
      }
    }
  }

  fun loadMorePodcastSections() {
    val cont = continuation ?: return
    if (isLoadingMore.value || hasLoadedAll) return
    viewModelScope.launch(Dispatchers.IO) {
      isLoadingMore.value = true
      try {
        var currentContinuation: String? = cont
        while (currentContinuation != null) {
          val nextHome = YouTube.home(currentContinuation).getOrNull() ?: break
          currentContinuation = nextHome.continuation
          continuation = currentContinuation

          val newSections =
            nextHome.sections.mapNotNull { section ->
              val filtered =
                section.items.filter {
                  (it is SongItem && it.isPodcastEpisode) ||
                    (it is PlaylistItem && it.isPodcast) ||
                    (it is PlaylistItem)
                }
              filtered.takeIf { it.isNotEmpty() }?.let { section.copy(items = it) }
            }

          if (newSections.isNotEmpty()) {
            exploreSections.update { current ->
              val existingTitles = current.map { it.title }.toSet()
              current + newSections.filter { it.title !in existingTitles }
            }
          }
        }
        if (currentContinuation == null) {
          hasLoadedAll = true
        }
      } catch (e: Exception) {
        reportException(e)
      } finally {
        isLoadingMore.value = false
      }
    }
  }

  fun refresh() {
    viewModelScope.launch(Dispatchers.IO) {
      isRefreshing.value = true
      try {
        loadNewEpisodes()
        continuation = null
        hasLoadedAll = false
        val homeRes = YouTube.home().getOrNull()
        val chip = homeRes?.chips?.firstOrNull { it.title.contains("podcast", ignoreCase = true) }
        val browseId = chip?.endpoint?.browseId ?: "FEmusic_home"
        val params = chip?.endpoint?.params
        val sectionsRes = YouTube.home(browseId = browseId, params = params).getOrNull()
        continuation = sectionsRes?.continuation
        val sections =
          sectionsRes?.sections?.mapNotNull { section ->
            val filtered =
              section.items.filter {
                (it is SongItem && it.isPodcastEpisode) ||
                  (it is PlaylistItem && it.isPodcast) ||
                  (it is PlaylistItem)
              }
            filtered.takeIf { it.isNotEmpty() }?.let { section.copy(items = it) }
          } ?: emptyList()
        exploreSections.value = sections
      } catch (e: Exception) {
        reportException(e)
      } finally {
        isRefreshing.value = false
      }
    }
  }

  /** Latest episodes of the first [MAX_SHOWS] followed shows, newest first. */
  val newEpisodes = MutableStateFlow<List<SongItem>>(emptyList())
  private var newEpisodesFor: Set<String>? = null
  private var newEpisodesAtMs = 0L

  fun loadNewEpisodes() {
    viewModelScope.launch(Dispatchers.IO) {
      val showIds = followed.first().mapNotNull { it.playlist.browseId }.toSet()
      val now = System.currentTimeMillis()
      if (showIds == newEpisodesFor && now - newEpisodesAtMs < NEW_EPISODES_TTL_MS) return@launch
      val results =
        showIds
          .take(MAX_SHOWS)
          .map { id -> async { YouTube.podcast(id).getOrNull()?.songs?.take(PER_SHOW) } }
          .awaitAll()
      // Offline: keep the list for the same shows, and retry next time.
      if (showIds.isNotEmpty() && results.all { it == null } && showIds == newEpisodesFor) {
        return@launch
      }
      newEpisodes.value =
        results
          .filterNotNull()
          .flatten()
          .sortedByDescending { EpisodeDates.parse(it.publishedText, now) ?: 0L }
          .take(MAX_EPISODES)
      if (results.none { it == null }) {
        newEpisodesFor = showIds
        newEpisodesAtMs = now
      }
    }
  }

  private companion object {
    const val MAX_SHOWS = 20
    const val PER_SHOW = 5
    const val MAX_EPISODES = 20
    const val NEW_EPISODES_TTL_MS = 30 * 60 * 1000L
  }
}

@HiltViewModel
class ArtistSongsViewModel
@Inject
constructor(
  @ApplicationContext context: Context,
  database: MusicDatabase,
  savedStateHandle: SavedStateHandle,
) : ViewModel() {
  private val artistId = savedStateHandle.get<String>("artistId")!!
  val artist = database.artist(artistId).stateIn(viewModelScope, SharingStarted.Lazily, null)

  val songs =
    context.dataStore.data
      .map {
        Triple(
          (try {
              it[ArtistSongSortTypeKey]
            } catch (e: Exception) {
              null
            })
            .toEnum(ArtistSongSortType.CREATE_DATE) to
            ((try {
              it[ArtistSongSortDescendingKey]
            } catch (e: Exception) {
              null
            }) ?: true),
          (try {
            it[HideExplicitKey]
          } catch (e: Exception) {
            null
          }) ?: false,
          (try {
            it[HideVideoSongsKey]
          } catch (e: Exception) {
            null
          }) ?: false,
        )
      }
      .distinctUntilChanged()
      .flatMapLatest { (sortDesc, hideExplicit, hideVideoSongs) ->
        val (sortType, descending) = sortDesc
        database.artistSongs(artistId, sortType, descending).map {
          it.filterExplicit(hideExplicit).filterVideoSongs(hideVideoSongs)
        }
      }
      .stateIn(viewModelScope, SharingStarted.Lazily, emptyList())
}

@HiltViewModel
class LibraryMixViewModel
@Inject
constructor(
  @ApplicationContext context: Context,
  database: MusicDatabase,
  private val syncUtils: SyncUtils,
) : ViewModel() {
  private val _isRefreshing = MutableStateFlow(false)
  val isRefreshing = _isRefreshing.asStateFlow()

  val syncAllLibrary = { viewModelScope.launch(Dispatchers.IO) { syncUtils.tryAutoSync() } }

  fun refresh() {
    viewModelScope.launch(Dispatchers.IO) {
      _isRefreshing.value = true
      syncUtils.performFullSyncSuspend()
      _isRefreshing.value = false
    }
  }

  val topValue =
    context.dataStore.data
      .map {
        (try {
          it[TopSize]
        } catch (e: Exception) {
          null
        }) ?: "50"
      }
      .distinctUntilChanged()
  var artists =
    database
      .artistsBookmarked(
        ArtistSortType.CREATE_DATE,
        true,
      )
      .stateIn(viewModelScope, SharingStarted.Lazily, emptyList())
  var albums =
    context.dataStore.data
      .map {
        (try {
          it[HideExplicitKey]
        } catch (e: Exception) {
          null
        }) ?: false
      }
      .distinctUntilChanged()
      .flatMapLatest { hideExplicit ->
        database.albumsLiked(AlbumSortType.CREATE_DATE, true).map {
          it.filterExplicitAlbums(hideExplicit)
        }
      }
      .stateIn(viewModelScope, SharingStarted.Lazily, emptyList())
  var playlists =
    context.dataStore.data
      .map {
        (try {
          it[HideYoutubeShortsKey]
        } catch (e: Exception) {
          null
        }) ?: false
      }
      .distinctUntilChanged()
      .flatMapLatest { hideYoutubeShorts ->
        database.playlists(PlaylistSortType.CREATE_DATE, true).map {
          it.filterYoutubeShorts(hideYoutubeShorts)
        }
      }
      .stateIn(viewModelScope, SharingStarted.Lazily, emptyList())

  init {
    viewModelScope.launch(Dispatchers.IO) {
      albums.collect { albums ->
        albums
          .filter {
            !it.album.isLocal && !it.id.startsWith("LOCAL_ALBUM_") && it.album.songCount == 0
          }
          .forEach { album ->
            YouTube.album(album.id)
              .onSuccess { albumPage ->
                database.query { update(album.album, albumPage, album.artists) }
              }
              .onFailure {
                reportException(it)
                if (it.message?.contains("NOT_FOUND") == true) {
                  database.query { delete(album.album) }
                }
              }
          }
      }
    }
    viewModelScope.launch(Dispatchers.IO) {
      artists.collect { artists ->
        artists
          .map { it.artist }
          .filter {
            !it.isLocal &&
              !it.id.startsWith("LOCAL_ARTIST_") &&
              (it.thumbnailUrl == null ||
                Duration.between(
                  it.lastUpdateTime,
                  LocalDateTime.now(),
                ) > Duration.ofDays(10))
          }
          .forEach { artist ->
            YouTube.artist(artist.id).onSuccess { artistPage ->
              database.query { update(artist, artistPage) }
            }
          }
      }
    }
  }
}

@HiltViewModel
class LibraryViewModel @Inject constructor() : ViewModel() {
  private val curScreen = mutableStateOf(LibraryFilter.LIBRARY)
  val filter: MutableState<LibraryFilter> = curScreen
}
