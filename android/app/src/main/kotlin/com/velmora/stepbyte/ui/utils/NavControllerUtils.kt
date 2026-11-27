package com.velmora.stepbyte.ui.utils

import androidx.navigation.NavController
import com.velmora.stepbyte.ui.screens.Screens

fun NavController.backToMain() {
  val mainRoutes = (Screens.MainScreens + Screens.VelmoraScreens).map { it.route }

  while (
    previousBackStackEntry != null && currentBackStackEntry?.destination?.route !in mainRoutes
  ) {
    popBackStack()
  }
}
