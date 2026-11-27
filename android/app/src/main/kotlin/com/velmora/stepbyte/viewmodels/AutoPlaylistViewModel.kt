package com.velmora.stepbyte.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import com.velmora.stepbyte.constants.DownloadedSongSortDescendingKey
import com.velmora.stepbyte.constants.DownloadedSongSortTypeKey
import com.velmora.stepbyte.constants.ExportedSongIdsKey
import com.velmora.stepbyte.constants.HideExplicitKey
import com.velmora.stepbyte.constants.HideVideoSongsKey
import com.velmora.stepbyte.constants.SongSortDescendingKey
import com.velmora.stepbyte.constants.SongSortType
import com.velmora.stepbyte.constants.SongSortTypeKey
import com.velmora.stepbyte.db.MusicDatabase
import com.velmora.stepbyte.extensions.filterExplicit
import com.velmora.stepbyte.extensions.filterVideoSongs
import com.velmora.stepbyte.extensions.toEnum
import com.velmora.stepbyte.utils.SyncUtils
import com.velmora.stepbyte.utils.dataStore
import javax.inject.Inject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AutoPlaylistViewModel
@Inject
constructor(
  @ApplicationContext context: Context,
  private val database: MusicDatabase,
  savedStateHandle: SavedStateHandle,
  private val syncUtils: SyncUtils,
) : ViewModel() {
  val playlist = savedStateHandle.get<String>("playlist")!!
  private val sortTypeKey =
    if (playlist == "downloaded") DownloadedSongSortTypeKey else SongSortTypeKey
  private val sortDescendingKey =
    if (playlist == "downloaded") DownloadedSongSortDescendingKey else SongSortDescendingKey

  private val _isRefreshing = MutableStateFlow(false)
  val isRefreshing = _isRefreshing.asStateFlow()

  @OptIn(ExperimentalCoroutinesApi::class)
  val likedSongs =
    context.dataStore.data
      .map {
        Triple(
          Triple(
            (try {
                it[sortTypeKey]
              } catch (e: Exception) {
                null
              })
              .toEnum(SongSortType.CREATE_DATE) to
              ((try {
                it[sortDescendingKey]
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
          ),
          (try {
            it[ExportedSongIdsKey]
          } catch (e: Exception) {
            null
          }) ?: "",
          Unit,
        )
      }
      .distinctUntilChanged()
      .flatMapLatest { (triple, exportedSongIds, _) ->
        val (sortDesc, hideExplicit, hideVideoSongs) = triple
        val (sortType, descending) = sortDesc
        when (playlist) {
          "liked" ->
            database.likedSongs(sortType, descending).map {
              it.filterExplicit(hideExplicit).filterVideoSongs(hideVideoSongs)
            }
          "downloaded" ->
            database.downloadedSongs(sortType, descending).map {
              it.filterExplicit(hideExplicit).filterVideoSongs(hideVideoSongs)
            }
          "uploaded" ->
            database.uploadedSongs(sortType, descending).map {
              it.filterExplicit(hideExplicit).filterVideoSongs(hideVideoSongs)
            }
          "exported" -> {
            val ids = exportedSongIds.split(",").filter { it.isNotBlank() }
            database.getSongsByIdsFlow(ids).map {
              it.filterExplicit(hideExplicit).filterVideoSongs(hideVideoSongs)
            }
          }
          else -> kotlinx.coroutines.flow.flowOf(emptyList())
        }
      }
      .stateIn(viewModelScope, kotlinx.coroutines.flow.SharingStarted.Lazily, emptyList())

  fun syncLikedSongs() {
    viewModelScope.launch(Dispatchers.IO) { syncUtils.syncLikedSongs() }
  }

  fun syncUploadedSongs() {
    viewModelScope.launch(Dispatchers.IO) { syncUtils.syncUploadedSongs() }
  }

  fun refresh() {
    viewModelScope.launch(Dispatchers.IO) {
      _isRefreshing.value = true
      when (playlist) {
        "liked" -> syncUtils.syncLikedSongsSuspend()
        "uploaded" -> syncUtils.syncUploadedSongsSuspend()
      }
      _isRefreshing.value = false
    }
  }
}
