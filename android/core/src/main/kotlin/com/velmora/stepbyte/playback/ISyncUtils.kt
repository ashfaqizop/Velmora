package com.velmora.stepbyte.playback

import com.velmora.stepbyte.db.entities.SongEntity

interface ISyncUtils {
  fun likeSong(song: SongEntity)
}
