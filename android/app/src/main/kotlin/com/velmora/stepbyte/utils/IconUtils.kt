package com.velmora.stepbyte.utils

import android.content.ComponentName
import android.content.Context
import android.content.pm.PackageManager

enum class AppIconType(val value: Int) {
  DEFAULT(0),
  LEGACY(1),
  STATIC(2),
  CAT(3),
  CRAZY_BLUE(4),
  POOKIE(5),
  SKY(6),
  VELMORA_CAT(7),
  BILLIE_EILISH(13),
  SABRINA_CARPENTER(14),
  SABRINA_CARPENTER_2(15),
  CRAFT(16),
  SADIE(17),
  NEON_GLASS(18),
  MILLIE_BOBBY_BROWN(19),
}

object IconUtils {
  /** Velmora has one launcher icon; this just makes sure its launcher entry is on. */
  fun setIcon(context: Context, @Suppress("UNUSED_PARAMETER") iconType: AppIconType) {
    context.packageManager.setComponentEnabledSetting(
      ComponentName(context, "com.velmora.stepbyte.MainActivityAlias"),
      PackageManager.COMPONENT_ENABLED_STATE_ENABLED,
      PackageManager.DONT_KILL_APP,
    )
  }
}
