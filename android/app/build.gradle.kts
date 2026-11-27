import java.util.Properties
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

val localProperties = Properties()
val localPropertiesFile = rootProject.file("local.properties")

if (localPropertiesFile.exists()) {
  localProperties.load(localPropertiesFile.inputStream())
}

plugins {
  id("com.android.application")
  alias(libs.plugins.hilt)
  alias(libs.plugins.kotlin.ksp)
  alias(libs.plugins.compose.compiler)
  alias(libs.plugins.kotlin.serialization)
  alias(libs.plugins.protobufPlugin)
}

android {
  namespace = "com.velmora.stepbyte"
  compileSdk = 37
  ndkVersion = "28.2.13676358"

  defaultConfig {
    applicationId = "com.velmora.stepbyte"
    minSdk = 26
    targetSdk = 36
    // Same numbers as the desktop app. Code = major * 1,000,000 + minor * 1,000 + patch.
    versionCode = 6006
    versionName = "0.6.6"

    testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    vectorDrawables.useSupportLibrary = true

    // LastFM API keys from GitHub Secrets
    val lastFmKey = "266d77b5790e413ada7e41ef100d017a"
    val lastFmSecret = "41d3ae3b039ddac06c37fb30055bf93b"

    buildConfigField("String", "LASTFM_API_KEY", "\"$lastFmKey\"")
    buildConfigField("String", "LASTFM_SECRET", "\"$lastFmSecret\"")

    // GitHub OAuth keys
    val githubClientId =
      localProperties.getProperty("GH_CLIENT_ID") ?: System.getenv("GH_CLIENT_ID") ?: ""
    val githubClientSecret =
      localProperties.getProperty("GH_CLIENT_SECRET") ?: System.getenv("GH_CLIENT_SECRET") ?: ""
    buildConfigField("String", "GH_CLIENT_ID", "\"$githubClientId\"")
    buildConfigField("String", "GH_CLIENT_SECRET", "\"$githubClientSecret\"")

    // add nightly build label support
    val isNightly = project.hasProperty("nightly") && project.property("nightly") == "true"
    buildConfigField("Boolean", "IS_NIGHTLY", isNightly.toString())

    val discordApplicationId = "1518210534070292541"
    val discordApplicationIdLong = 1518210534070292541L
    val discordRedirectScheme = "discord-$discordApplicationId"

    buildConfigField("String", "DISCORD_APPLICATION_ID", "\"$discordApplicationId\"")
    buildConfigField("long", "DISCORD_APPLICATION_ID_LONG", "${discordApplicationIdLong}L")
    buildConfigField("String", "DISCORD_REDIRECT_SCHEME", "\"$discordRedirectScheme\"")
    manifestPlaceholders["discordRedirectScheme"] = discordRedirectScheme
  }

  flavorDimensions += listOf("abi", "variant")
  productFlavors {
    create("gms") {
      dimension = "variant"
      buildConfigField("Boolean", "CAST_AVAILABLE", "true")
    }
    create("universal") {
      dimension = "abi"
      buildConfigField("String", "ARCHITECTURE", "\"universal\"")
      ndk {
        abiFilters.add("arm64-v8a")
        abiFilters.add("armeabi-v7a")
      }
    }
  }

  val releaseKeystore = rootProject.file("keystore.jks")
  signingConfigs {
    create("release") {
      if (releaseKeystore.exists()) storeFile = releaseKeystore
      storePassword = System.getenv("STORE_PASSWORD")
      keyAlias = System.getenv("KEY_ALIAS")
      keyPassword = System.getenv("KEY_PASSWORD")
    }
  }

  buildTypes {
    release {
      isMinifyEnabled = true
      isShrinkResources = true
      isCrunchPngs = false
      isDebuggable = false
      signingConfig =
        if (releaseKeystore.exists()) signingConfigs.getByName("release")
        else signingConfigs.getByName("debug")
      proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
      buildConfigField("String", "ARCHITECTURE", "\"release\"")
    }
    debug {
      applicationIdSuffix = ".debug"
      isDebuggable = true
      if (releaseKeystore.exists()) signingConfig = signingConfigs.getByName("release")
      buildConfigField("String", "ARCHITECTURE", "\"debug\"")
    }
  }

  compileOptions {
    isCoreLibraryDesugaringEnabled = true
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
  }

  kotlin {
    jvmToolchain(21)
    compilerOptions {
      freeCompilerArgs.add("-Xannotation-default-target=param-property")
      jvmTarget.set(JvmTarget.JVM_21)
    }
  }

  buildFeatures {
    compose = true
    buildConfig = true
  }

  dependenciesInfo {
    includeInApk = false
    includeInBundle = false
  }

  lint {
    lintConfig = file("lint.xml")
    warningsAsErrors = false
    abortOnError = false
    checkDependencies = false
  }

  androidResources { generateLocaleConfig = true }

  packaging {
    jniLibs {
      useLegacyPackaging = true
      keepDebugSymbols +=
        listOf("**/libandroidx.graphics.path.so", "**/libdatastore_shared_counter.so")
    }
    resources {
      excludes += "/META-INF/{AL2.0,LGPL2.1}"
      excludes += "META-INF/NOTICE.md"
      excludes += "META-INF/CONTRIBUTORS.md"
      excludes += "META-INF/LICENSE.md"
      excludes += "META-INF/INDEX.LIST"
      excludes += "META-INF/io.netty.versions.properties"
      excludes += "META-INF/DEPENDENCIES"
    }
  }
}

protobuf {
  protoc { artifact = "com.google.protobuf:protoc:${libs.versions.protobuf.get()}" }
  generateProtoTasks {
    all().forEach { task ->
      task.builtins {
        create("java") { option("lite") }
        create("kotlin") { option("lite") }
      }
    }
  }
}

ksp {}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
  compilerOptions {
    freeCompilerArgs.addAll("-opt-in=kotlin.RequiresOptIn")
    suppressWarnings.set(false)
  }
}

dependencies {
  implementation("nl.dionsegijn:konfetti-compose:2.0.4")

  implementation(project(":core"))
  implementation(project(":playback"))
  implementation(project(":audio-dsp"))
  implementation(project(":dsp-core"))
  implementation(project(":usbaudio"))


  // GMS Location Services for high-accuracy weather AI context
  "gmsImplementation"(libs.play.services.location)

  implementation(libs.haze)
  implementation(libs.guava)
  implementation(libs.coroutines.guava)
  implementation(libs.concurrent.futures)

  implementation(libs.activity)
  implementation(libs.hilt.navigation)
  implementation(libs.datastore)

  implementation(libs.compose.runtime)
  implementation(libs.compose.foundation)
  implementation(libs.compose.ui)
  implementation(libs.compose.ui.util)
  implementation(libs.compose.ui.tooling)
  implementation(libs.compose.animation)
  implementation(libs.compose.reorderable)

  implementation(libs.viewmodel)
  implementation(libs.viewmodel.compose)
  implementation(libs.lifecycle.process)

  implementation(libs.material3)
  implementation(libs.androidx.adaptive)
  implementation(libs.androidx.adaptive.layout)
  implementation(libs.androidx.adaptive.navigation)
  implementation(libs.palette)
  implementation(libs.materialKolor)

  implementation(libs.appcompat)

  implementation(libs.coil)
  implementation(libs.coil.network.okhttp)

  implementation(libs.ucrop)

  implementation(libs.shimmer)

  implementation(libs.media3)
  implementation(libs.media3.session)
  implementation(libs.media3.hls)
  implementation(libs.media3.ui)
  implementation(libs.media3.okhttp)
  implementation(libs.media3.cronet)
  implementation("com.google.android.gms:play-services-cronet:18.0.1")

  // Google Cast - only included in GMS flavor (not available in F-Droid/FOSS builds)
  "gmsImplementation"(libs.mediarouter)
  "gmsImplementation"(libs.cast.framework)

  implementation(libs.room.runtime)
  implementation(libs.kuromoji.ipadic)
  implementation(libs.tinypinyin)

  implementation(libs.room.ktx)

  implementation(libs.apache.lang3)

  implementation(libs.hilt)
  ksp(libs.hilt.compiler)
  ksp("org.jetbrains.kotlin:kotlin-metadata-jvm:2.4.21")
  annotationProcessor("org.jetbrains.kotlin:kotlin-metadata-jvm:2.4.21")

  implementation(project(":innertube"))
  implementation("com.github.MetrolistGroup.innertubex:innertubex-android:v0.7.0")

  implementation(project(":lyrics"))
  implementation(project(":kugou"))
  implementation(project(":lrclib"))
  implementation(project(":betterlyrics"))
  implementation(project(":simpmusic"))
  implementation(project(":youlyplus"))
  implementation(project(":canvas"))
  implementation(project(":shazamkit"))
  implementation(project(":artistvideo"))
  implementation(project(":applecanvas"))
  implementation(project(":paxsenixlyrics"))
  implementation(project(":unison"))

  implementation(libs.ktor.client.core)
  implementation(libs.retrofit)
  implementation(libs.retrofit.gson)
  implementation(libs.ktor.client.okhttp)
  implementation(libs.ktor.client.content.negotiation)
  implementation(libs.ktor.serialization.json)

  // Protobuf for message serialization (lite version for Android)
  implementation(libs.protobuf.javalite)
  implementation(libs.protobuf.kotlin.lite)

  coreLibraryDesugaring(libs.desugaring)
  implementation(libs.timber)
  implementation(libs.smoothCorner)
  implementation(libs.lottie.compose)
  implementation("androidx.compose.material:material-icons-extended:1.7.8")
  implementation(libs.work.runtime.ktx)
  implementation(libs.androidx.core.splashscreen)
  implementation(libs.ffmpeg.kit.audio)
  implementation("io.github.dokar3:quickjs-kt-android:1.0.14")
  implementation("com.microsoft.onnxruntime:onnxruntime-android:1.28.0")

  testImplementation(libs.junit)
  testImplementation(libs.kotlinx.coroutines.test)
  testImplementation(libs.robolectric)
}
