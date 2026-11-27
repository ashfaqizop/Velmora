package com.velmora.stepbyte.ui.screens.equalizer

import com.velmora.stepbyte.dsp.core.models.SavedEQProfile

data class EQState(
  val profiles: List<SavedEQProfile> = emptyList(),
  val activeProfileId: String? = null,
  val importStatus: String? = null,
  val error: String? = null,
)
