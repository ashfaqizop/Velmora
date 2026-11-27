package com.velmora.stepbyte.domain.repositories

import com.velmora.stepbyte.domain.models.Song
import kotlinx.coroutines.flow.Flow

interface SongRepository {
  fun getSong(id: String): Flow<Song?>

  fun getLikedSongs(): Flow<List<Song>>

  fun getRecentlyPlayed(limit: Int): Flow<List<Song>>

  suspend fun updateLiked(id: String, isLiked: Boolean)

  suspend fun incrementPlayTime(id: String, durationMs: Long)
}
