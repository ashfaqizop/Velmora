package com.velmora.stepbyte.velmora.shell

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import com.velmora.stepbyte.R

/** Velmora's brand, the same as the desktop app and the website. */
object Brand {
  val Blue = Color(0xFF3271FE)
  val BlueDeep = Color(0xFF1B3FA8)
  val Violet = Color(0xFF6C9BFF)
  val Cream = Color(0xFFF8E8C8)
  val Amber = Color(0xFFFFB74D)
  val Night = Color(0xFF0A0D16)

  /** The display serif used for the wordmark and big titles. */
  val Serif =
    FontFamily(
      Font(R.font.instrument_serif, FontWeight.Normal, FontStyle.Normal),
      Font(R.font.instrument_serif_italic, FontWeight.Normal, FontStyle.Italic),
    )

  /** Where the Android build progress lives on the website. */
  const val PROGRESS_URL = "https://getvelmora.dpdns.org/android/"
}
