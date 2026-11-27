package com.velmora.stepbyte.extensions

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import com.velmora.stepbyte.innertube.utils.parseCookieString
import com.velmora.stepbyte.constants.InnerTubeCookieKey
import com.velmora.stepbyte.constants.YtmSyncKey
import com.velmora.stepbyte.utils.dataStore
import com.velmora.stepbyte.utils.get
import kotlinx.coroutines.runBlocking

fun Context.isSyncEnabled(): Boolean {
  return runBlocking { dataStore.get(YtmSyncKey, true) && isUserLoggedIn() }
}

fun Context.isUserLoggedIn(): Boolean {
  return runBlocking {
    val cookie = dataStore[InnerTubeCookieKey] ?: ""
    "SAPISID" in parseCookieString(cookie) && isInternetConnected()
  }
}

fun Context.isInternetConnected(): Boolean {
  val connectivityManager = getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
  val networkCapabilities =
    connectivityManager.getNetworkCapabilities(connectivityManager.activeNetwork)
  return networkCapabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) ?: false
}
