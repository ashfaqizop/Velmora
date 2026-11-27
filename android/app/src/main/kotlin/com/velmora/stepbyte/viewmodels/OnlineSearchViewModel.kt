package com.velmora.stepbyte.viewmodels

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.velmora.stepbyte.innertube.YouTube
import com.velmora.stepbyte.innertube.models.filterExplicit
import com.velmora.stepbyte.innertube.models.filterVideoSongs
import com.velmora.stepbyte.innertube.models.filterYoutubeShorts
import com.velmora.stepbyte.innertube.pages.SearchSummaryPage
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import com.velmora.stepbyte.constants.HideExplicitKey
import com.velmora.stepbyte.constants.HideVideoSongsKey
import com.velmora.stepbyte.constants.HideYoutubeShortsKey
import com.velmora.stepbyte.models.ItemsPage
import com.velmora.stepbyte.utils.dataStore
import com.velmora.stepbyte.utils.get
import com.velmora.stepbyte.utils.reportException
import java.net.URLDecoder
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch

@HiltViewModel
class OnlineSearchViewModel
@Inject
constructor(
  @ApplicationContext val context: Context,
  savedStateHandle: SavedStateHandle,
  val extensionManager: com.velmora.stepbyte.extensions.ExtensionManager,
) : ViewModel() {
  val query =
    try {
      URLDecoder.decode(savedStateHandle.get<String>("query")!!, "UTF-8")
    } catch (e: IllegalArgumentException) {
      savedStateHandle.get<String>("query")!!
    }
  val filter = MutableStateFlow<YouTube.SearchFilter?>(null)
  var summaryPage by mutableStateOf<SearchSummaryPage?>(null)
  val viewStateMap = mutableStateMapOf<String, ItemsPage?>()

  init {
    viewModelScope.launch {
      filter.collect { filter ->
        if (filter == null) {
          if (summaryPage == null) {
            val hideExplicit = context.dataStore.get(HideExplicitKey, false)
            val hideVideoSongs = context.dataStore.get(HideVideoSongsKey, false)
            val hideYoutubeShorts = context.dataStore.get(HideYoutubeShortsKey, false)

            // No separate Addons section: a YouTube song plays from the addon when it has it,
            // so a second list of the same songs only made it look like the addon had to be
            // picked by hand.
            YouTube.searchSummary(query)
              .onSuccess {
                summaryPage =
                  it
                    .filterExplicit(hideExplicit)
                    .filterVideoSongs(hideVideoSongs)
                    .filterYoutubeShorts(hideYoutubeShorts)
              }
              .onFailure { reportException(it) }
          }
        } else {
          if (viewStateMap[filter.value] == null) {
            YouTube.search(query, filter)
              .onSuccess { result ->
                val hideExplicit = context.dataStore.get(HideExplicitKey, false)
                val hideVideoSongs = context.dataStore.get(HideVideoSongsKey, false)
                val hideYoutubeShorts = context.dataStore.get(HideYoutubeShortsKey, false)
                viewStateMap[filter.value] =
                  ItemsPage(
                    result.items
                      .distinctBy { it.id }
                      .filterExplicit(
                        hideExplicit,
                      )
                      .let { items ->
                        if (filter.value == YouTube.SearchFilter.FILTER_VIDEO.value) items
                        else items.filterVideoSongs(hideVideoSongs)
                      }
                      .filterYoutubeShorts(hideYoutubeShorts),
                    result.continuation,
                  )
              }
              .onFailure { reportException(it) }
          }
        }
      }
    }
  }

  fun loadMore() {
    val filter = filter.value?.value
    viewModelScope.launch {
      if (filter == null) return@launch
      val viewState = viewStateMap[filter] ?: return@launch
      val continuation = viewState.continuation
      if (continuation != null) {
        val searchResult = YouTube.searchContinuation(continuation).getOrNull() ?: return@launch
        val hideExplicit = context.dataStore.get(HideExplicitKey, false)
        val hideVideoSongs = context.dataStore.get(HideVideoSongsKey, false)
        val hideYoutubeShorts = context.dataStore.get(HideYoutubeShortsKey, false)
        val newItems =
          searchResult.items
            .filterExplicit(hideExplicit)
            .let { items ->
              if (filter == YouTube.SearchFilter.FILTER_VIDEO.value) items
              else items.filterVideoSongs(hideVideoSongs)
            }
            .filterYoutubeShorts(hideYoutubeShorts)
        viewStateMap[filter] =
          ItemsPage((viewState.items + newItems).distinctBy { it.id }, searchResult.continuation)
      }
    }
  }
}
