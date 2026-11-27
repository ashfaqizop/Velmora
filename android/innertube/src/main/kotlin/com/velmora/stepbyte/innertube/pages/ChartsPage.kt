package com.velmora.stepbyte.innertube.pages

import com.velmora.stepbyte.innertube.models.*

data class ChartsPage(val sections: List<ChartSection>, val continuation: String?) {
  data class ChartSection(val title: String, val items: List<YTItem>, val chartType: ChartType)

  enum class ChartType {
    TRENDING,
    TOP,
    GENRE,
    NEW_RELEASES,
  }
}
