package com.velmora.stepbyte.di

import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import com.velmora.stepbyte.core.repository.PlaylistRepositoryImpl
import com.velmora.stepbyte.core.repository.SongRepositoryImpl
import com.velmora.stepbyte.domain.repositories.PlaylistRepository
import com.velmora.stepbyte.domain.repositories.SongRepository
import com.velmora.stepbyte.musicflow.MusicFlowFeedSource
import com.velmora.stepbyte.musicflow.MusicFlowRepository
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class DomainBindingsModule {

  @Binds
  @Singleton
  abstract fun bindSongRepository(
    impl: SongRepositoryImpl,
  ): SongRepository

  @Binds
  @Singleton
  abstract fun bindPlaylistRepository(
    impl: PlaylistRepositoryImpl,
  ): PlaylistRepository

  @Binds
  @Singleton
  abstract fun bindMusicFlowFeedSource(
    impl: MusicFlowRepository,
  ): MusicFlowFeedSource
}
