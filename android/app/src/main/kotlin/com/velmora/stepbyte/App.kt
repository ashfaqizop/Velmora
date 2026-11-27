package com.velmora.stepbyte

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.content.Intent
import android.os.Build
import android.widget.Toast
import androidx.datastore.preferences.core.edit
import androidx.work.Configuration
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.SingletonImageLoader
import coil3.disk.DiskCache
import coil3.disk.directory
import coil3.memory.MemoryCache
import coil3.request.CachePolicy
import coil3.request.allowHardware
import coil3.request.crossfade
import com.velmora.stepbyte.innertube.YouTube
import com.velmora.stepbyte.innertube.models.IpVersion
import com.velmora.stepbyte.innertube.models.YouTubeLocale
import com.velmora.stepbyte.kugou.KuGou
import dagger.hilt.android.HiltAndroidApp
import com.velmora.stepbyte.constants.*
import com.velmora.stepbyte.di.ApplicationScope
import com.velmora.stepbyte.extensions.toEnum
import com.velmora.stepbyte.extensions.toInetSocketAddress
import com.velmora.stepbyte.utils.AppContextHolder
import com.velmora.stepbyte.utils.CrashHandler
import com.velmora.stepbyte.utils.LocalAlbumArtFetcher
import com.velmora.stepbyte.utils.NewEpisodesWorker
import com.velmora.stepbyte.utils.YouTubeThumbnailFallbackInterceptor
import com.velmora.stepbyte.utils.dataStore
import com.velmora.stepbyte.utils.reportException
import java.net.Authenticator
import java.net.PasswordAuthentication
import java.net.Proxy
import java.util.Locale
import javax.inject.Inject
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import okhttp3.Credentials
import timber.log.Timber

@HiltAndroidApp
class App : Application(), SingletonImageLoader.Factory, Configuration.Provider {

  @Inject @ApplicationScope lateinit var applicationScope: CoroutineScope

  override val workManagerConfiguration: Configuration
    get() =
      Configuration.Builder()
        .setMinimumLoggingLevel(
          if (BuildConfig.DEBUG) android.util.Log.DEBUG else android.util.Log.ERROR,
        )
        .build()

  override fun startForegroundService(service: Intent): android.content.ComponentName? {
    return try {
      super.startForegroundService(service)
    } catch (e: Exception) {
      if (
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.S &&
          e is android.app.ForegroundServiceStartNotAllowedException
      ) {
        Timber.e(e, "Suppressed ForegroundServiceStartNotAllowedException in App")
        null
      } else {
        throw e
      }
    }
  }

  private fun getCurrentProcessName(): String? {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
      Application.getProcessName()
    } else {
      val pid = android.os.Process.myPid()
      val manager = getSystemService(Context.ACTIVITY_SERVICE) as? android.app.ActivityManager
      manager?.runningAppProcesses?.firstOrNull { it.pid == pid }?.processName
    }
  }

  private fun isCrashProcess(): Boolean {
    return getCurrentProcessName() == "$packageName:crash"
  }

  private fun isMainProcess(): Boolean {
    val processName = getCurrentProcessName()
    return processName == null || processName == packageName
  }

  override fun onCreate() {
    super.onCreate()

    if (isCrashProcess()) {
      CrashHandler.install(this)
      return
    }

    // Removed destructive database deletion to preserve user data

    CrashHandler.install(this)

    if (!isMainProcess()) {
      return
    }

    AppContextHolder.initialize(this)
    NewEpisodesWorker.schedule(this)
    com.velmora.stepbyte.utils.cipher.CipherDeobfuscator.initialize(this)
    com.velmora.stepbyte.utils.YTPlayerUtils.initialize()

    if (BuildConfig.DEBUG) {
      Timber.plant(Timber.DebugTree())
    }

    applicationScope.launch(Dispatchers.IO) {
      cachedCoilCacheSize =
        dataStore.data
          .map {
            (try {
              it[MaxImageCacheSizeKey]
            } catch (e: Exception) {
              null
            }) ?: 512
          }
          .first()
    }

    applicationScope.launch(Dispatchers.IO) {
      try {
        java.net.InetAddress.getByName("music.youtube.com")
        java.net.InetAddress.getByName("youtubei.googleapis.com")
        java.net.InetAddress.getByName("googlevideo.com")
      } catch (e: Exception) {
        // Ignore
      }
    }

    applicationScope.launch {
      initializeSettings()

      com.velmora.stepbyte.utils.InnerTubeXResolver.init(this@App)
      observeSettingsChanges()
    }
  }

  private suspend fun initializeSettings() {
    val settings = dataStore.data.first()
    val locale = Locale.getDefault()
    val languageTag = locale.language

    val currentAudioQuality =
      settings[AudioQualityKey]?.toEnum(defaultValue = AudioQuality.OPUS) ?: AudioQuality.OPUS
    val currentDownloadQuality =
      settings[DownloadQualityKey]?.toEnum(defaultValue = DownloadQuality.YOUTUBE)
        ?: DownloadQuality.YOUTUBE
    YouTube.locale =
      YouTubeLocale(
        gl =
          settings[ContentCountryKey]?.takeIf { it != SYSTEM_DEFAULT }
            ?: locale.country.takeIf { it in CountryCodeToName }
            ?: "US",
        hl =
          settings[ContentLanguageKey]?.takeIf { it != SYSTEM_DEFAULT }
            ?: locale.language.takeIf { it in LanguageCodeToName }
            ?: languageTag.takeIf { it in LanguageCodeToName }
            ?: "en",
      )

    if (languageTag == "zh-TW") {
      KuGou.useTraditionalChinese = true
    }

    if (settings[ProxyEnabledKey] == true) {
      val username = settings[ProxyUsernameKey].orEmpty()
      val password = settings[ProxyPasswordKey].orEmpty()
      val type = settings[ProxyTypeKey].toEnum(defaultValue = Proxy.Type.HTTP)

      if (username.isNotEmpty() || password.isNotEmpty()) {
        if (type == Proxy.Type.HTTP) {
          YouTube.proxyAuth = Credentials.basic(username, password)
        } else {
          Authenticator.setDefault(
            object : Authenticator() {
              override fun getPasswordAuthentication(): PasswordAuthentication =
                PasswordAuthentication(username, password.toCharArray())
            },
          )
        }
      }
      try {
        settings[ProxyUrlKey]?.let { YouTube.proxy = Proxy(type, it.toInetSocketAddress()) }
      } catch (e: Exception) {
        withContext(Dispatchers.Main) {
          Toast.makeText(this@App, getString(R.string.failed_to_parse_proxy), Toast.LENGTH_SHORT)
            .show()
        }
        reportException(e)
      }
    }

    YouTube.useLoginForBrowse = settings[UseLoginForBrowse] ?: true
    YouTube.ipVersion =
      settings[IpVersionKey]?.toEnum(defaultValue = IpVersion.AUTO) ?: IpVersion.AUTO

    // Set playback engine preference
    val engineName = settings[com.velmora.stepbyte.constants.PlaybackEngineKey]
    com.velmora.stepbyte.utils.YTPlayerUtils.playbackEngine =
      try {
        if (engineName != null) com.velmora.stepbyte.constants.PlaybackEngine.valueOf(engineName)
        else com.velmora.stepbyte.constants.PlaybackEngine.AUTO
      } catch (_: Exception) {
        com.velmora.stepbyte.constants.PlaybackEngine.AUTO
      }

    val channel =
      NotificationChannel(
          "updates",
          getString(R.string.update_channel_name),
          NotificationManager.IMPORTANCE_DEFAULT,
        )
        .apply { description = getString(R.string.update_channel_desc) }
    val nm = getSystemService(NotificationManager::class.java)
    nm.createNotificationChannel(channel)
  }

  private fun observeSettingsChanges() {
    applicationScope.launch(Dispatchers.IO) {
      dataStore.data
        .map {
          (try {
            it[VisitorDataKey]
          } catch (e: Exception) {
            null
          })
        }
        .distinctUntilChanged()
        .collect { visitorData ->
          YouTube.visitorData =
            visitorData?.takeIf { it != "null" }
              ?: YouTube.visitorData().getOrNull()?.also { newVisitorData ->
                dataStore.edit { settings -> settings[VisitorDataKey] = newVisitorData }
              }
        }
    }

    com.velmora.stepbyte.utils.lastfm.LastFM.initialize(
      apiKey = BuildConfig.LASTFM_API_KEY.takeIf { it.isNotEmpty() } ?: "",
      secret = BuildConfig.LASTFM_SECRET.takeIf { it.isNotEmpty() } ?: "",
    )

    applicationScope.launch(Dispatchers.IO) {
      dataStore.data
        .map {
          (try {
            it[DataSyncIdKey]
          } catch (e: Exception) {
            null
          })
        }
        .distinctUntilChanged()
        .collect { dataSyncId ->
          YouTube.dataSyncId = dataSyncId?.let {
            it.takeIf { !it.contains("||") }
              ?: it.takeIf { it.endsWith("||") }?.substringBefore("||")
              ?: it.substringAfter("||")
          }
        }
    }

    applicationScope.launch(Dispatchers.IO) {
      dataStore.data
        .map {
          (try {
            it[InnerTubeCookieKey]
          } catch (e: Exception) {
            null
          })
        }
        .distinctUntilChanged()
        .collect { cookie ->
          try {
            YouTube.cookie = cookie
          } catch (e: Exception) {
            Timber.e(e, "Could not parse cookie. Clearing existing cookie.")
            forgetAccount(this@App)
          }
        }
    }

    applicationScope.launch(Dispatchers.IO) {
      dataStore.data
        .map {
          Triple(
            (try {
              it[ContentCountryKey]
            } catch (e: Exception) {
              null
            }),
            (try {
              it[ContentLanguageKey]
            } catch (e: Exception) {
              null
            }),
            (try {
              it[AppLanguageKey]
            } catch (e: Exception) {
              null
            }),
          )
        }
        .distinctUntilChanged()
        .collect { (contentCountry, contentLanguage, appLanguage) ->
          val systemLocale = Locale.getDefault()
          val effectiveAppLocale =
            appLanguage?.takeUnless { it == SYSTEM_DEFAULT }?.let { Locale.forLanguageTag(it) }
              ?: systemLocale

          YouTube.locale =
            YouTubeLocale(
              gl =
                contentCountry?.takeIf { it != SYSTEM_DEFAULT }
                  ?: effectiveAppLocale.country.takeIf { it in CountryCodeToName }
                  ?: systemLocale.country.takeIf { it in CountryCodeToName }
                  ?: "US",
              hl =
                contentLanguage?.takeIf { it != SYSTEM_DEFAULT }
                  ?: effectiveAppLocale.toLanguageTag().takeIf { it in LanguageCodeToName }
                  ?: effectiveAppLocale.language.takeIf { it in LanguageCodeToName }
                  ?: "en",
            )
        }
    }

    applicationScope.launch(Dispatchers.IO) {
      dataStore.data
        .map {
          (try {
            it[IpVersionKey]
          } catch (e: Exception) {
            null
          })
        }
        .distinctUntilChanged()
        .collect { ipVersion ->
          YouTube.ipVersion = ipVersion?.toEnum(defaultValue = IpVersion.AUTO) ?: IpVersion.AUTO
        }
    }
  }

  @Volatile private var cachedCoilCacheSize: Int? = null

  override fun newImageLoader(context: PlatformContext): ImageLoader {
    val cacheSize =
      cachedCoilCacheSize
        ?: runBlocking {
          dataStore.data
            .map {
              (try {
                it[MaxImageCacheSizeKey]
              } catch (e: Exception) {
                null
              }) ?: 512
            }
            .first()
        }
    return ImageLoader.Builder(this)
      .apply {
        crossfade(true)
        allowHardware(Build.VERSION.SDK_INT >= Build.VERSION_CODES.P)
        components {
          add(YouTubeThumbnailFallbackInterceptor())
          add(LocalAlbumArtFetcher.Factory())
        }

        memoryCache { MemoryCache.Builder().maxSizePercent(context, 0.25).build() }
        if (cacheSize == 0) {
          diskCachePolicy(CachePolicy.DISABLED)
        } else {
          diskCache(
            DiskCache.Builder()
              .directory(cacheDir.resolve("coil"))
              .maxSizeBytes(cacheSize * 1024 * 1024L)
              .build(),
          )
        }
      }
      .build()
  }

  companion object {
    suspend fun forgetAccount(context: Context) {
      Timber.d("forgetAccount: Starting logout process")

      Timber.d("forgetAccount: Clearing DataStore preferences")
      context.dataStore.edit { settings ->
        settings.remove(InnerTubeCookieKey)
        settings.remove(VisitorDataKey)
        settings.remove(DataSyncIdKey)
        settings.remove(AccountNameKey)
        settings.remove(AccountEmailKey)
        settings.remove(AccountChannelHandleKey)
      }
      Timber.d("forgetAccount: DataStore preferences cleared")

      Timber.d("forgetAccount: Clearing YouTube object auth state")
      Timber.d(
        "forgetAccount: Before - cookie=${YouTube.cookie?.take(50)}, visitorData=${YouTube.visitorData?.take(20)}, dataSyncId=${YouTube.dataSyncId?.take(20)}",
      )
      YouTube.cookie = null
      YouTube.visitorData = null
      YouTube.dataSyncId = null
      com.velmora.stepbyte.utils.InnerTubeXResolver.onSessionChanged()
      Timber.d(
        "forgetAccount: After - cookie=${YouTube.cookie}, visitorData=${YouTube.visitorData}, dataSyncId=${YouTube.dataSyncId}",
      )

      Timber.d("forgetAccount: Clearing WebView CookieManager")
      withContext(Dispatchers.Main) {
        android.webkit.CookieManager.getInstance().apply {
          removeAllCookies { removed ->
            Timber.d("forgetAccount: CookieManager.removeAllCookies callback: removed=$removed")
          }
          flush()
        }
      }
      Timber.d("forgetAccount: Logout process complete")
    }
  }
}
