package com.velmora.stepbyte.lyrics

import android.content.Context
import com.velmora.stepbyte.youlyplus.YouLyPlus
import com.velmora.stepbyte.constants.EnableYouLyPlusKey
import com.velmora.stepbyte.utils.dataStore
import com.velmora.stepbyte.utils.get

object YouLyPlusLyricsProvider : LyricsProvider {
  override val name = "YouLyPlus"

  override fun isEnabled(context: Context): Boolean = context.dataStore[EnableYouLyPlusKey] ?: true

  override suspend fun getLyrics(
    id: String,
    title: String,
    artist: String,
    duration: Int,
    album: String?,
  ): Result<String> = YouLyPlus.getLyrics(title, artist, duration, album, id)

  override suspend fun getAllLyrics(
    id: String,
    title: String,
    artist: String,
    duration: Int,
    album: String?,
    callback: (String) -> Unit,
  ) {
    YouLyPlus.getAllLyrics(title, artist, duration, album, id, null, callback)
  }
}
