package com.velmora.stepbyte.db.daos

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.velmora.stepbyte.db.entities.TasteProfileEntity

@Dao
interface TasteProfileDao {
  @Query("SELECT * FROM taste_profile WHERE id = 1 LIMIT 1") suspend fun get(): TasteProfileEntity?

  @Insert(onConflict = OnConflictStrategy.REPLACE) suspend fun upsert(profile: TasteProfileEntity)

  @Query("DELETE FROM taste_profile") suspend fun clear()
}
