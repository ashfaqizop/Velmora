package com.velmora.stepbyte.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.media3.datasource.cache.SimpleCache
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import com.velmora.stepbyte.constants.HideExplicitKey
import com.velmora.stepbyte.constants.HideVideoSongsKey
import com.velmora.stepbyte.db.MusicDatabase
import com.velmora.stepbyte.db.entities.Song
import com.velmora.stepbyte.di.DownloadCache
import com.velmora.stepbyte.di.PlayerCache
import com.velmora.stepbyte.extensions.filterExplicit
import com.velmora.stepbyte.extensions.filterVideoSongs
import com.velmora.stepbyte.utils.dataStore
import com.velmora.stepbyte.utils.get
import java.time.LocalDateTime
import javax.inject.Inject
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

@HiltViewModel
class CachePlaylistViewModel
@Inject
constructor(
  @ApplicationContext private val context: Context,
  private val database: MusicDatabase,
  @PlayerCache private val playerCache: SimpleCache,
  @DownloadCache private val downloadCache: SimpleCache,
) : ViewModel() {

  private val _cachedSongs = MutableStateFlow<List<Song>>(emptyList())
  val cachedSongs: StateFlow<List<Song>> = _cachedSongs

  init {
    viewModelScope.launch {
      while (true) {
        val hideExplicit = context.dataStore.get(HideExplicitKey, false)
        val hideVideoSongs = context.dataStore.get(HideVideoSongsKey, false)
        val cachedIds = playerCache.keys.toSet()
        val downloadedIds = downloadCache.keys.toSet()
        val pureCacheIds = cachedIds.subtract(downloadedIds)

        val songs =
          if (pureCacheIds.isNotEmpty()) {
            database.getSongsByIds(pureCacheIds.toList())
          } else {
            emptyList()
          }

        val completeSongs = songs.filter {
          val contentLength = it.format?.contentLength
          contentLength != null && playerCache.isCached(it.song.id, 0, contentLength)
        }

        if (completeSongs.isNotEmpty()) {
          database.query {
            completeSongs.forEach {
              if (it.song.dateDownload == null) {
                update(it.song.copy(dateDownload = LocalDateTime.now()))
              }
            }
          }
        }

        _cachedSongs.value =
          completeSongs
            .filter { it.song.dateDownload != null }
            .sortedByDescending { it.song.dateDownload }
            .filterExplicit(hideExplicit)
            .filterVideoSongs(hideVideoSongs)

        delay(1000)
      }
    }
  }

  fun removeSongFromCache(songId: String) {
    playerCache.removeResource(songId)
  }
}
