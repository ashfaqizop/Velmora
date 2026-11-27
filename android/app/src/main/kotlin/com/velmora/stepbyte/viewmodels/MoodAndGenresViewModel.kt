package com.velmora.stepbyte.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.velmora.stepbyte.innertube.YouTube
import com.velmora.stepbyte.innertube.pages.MoodAndGenres
import dagger.hilt.android.lifecycle.HiltViewModel
import com.velmora.stepbyte.utils.reportException
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch

@HiltViewModel
class MoodAndGenresViewModel @Inject constructor() : ViewModel() {
  val moodAndGenres = MutableStateFlow<List<MoodAndGenres>?>(null)

  init {
    viewModelScope.launch {
      YouTube.moodAndGenres()
        .onSuccess { moodAndGenres.value = it }
        .onFailure { reportException(it) }
    }
  }
}
