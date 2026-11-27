// Velmora's Rust engine (UniverseAPI) for Kotlin.
//
// CI builds rust/ with cargo-ndk into src/main/jniLibs/<abi>/libvelmora_engine.so
// and writes the UniFFI bindings into src/main/kotlin/.../engine/ffi/ before
// Gradle runs (see .github/workflows/android.yml). Both are generated and
// gitignored. Kotlin code uses the typed API in this package, not ffi/.
plugins {
  id("com.android.library")
  alias(libs.plugins.kotlin.serialization)
}

android {
  namespace = "com.velmora.stepbyte.engine"
  compileSdk = 37

  defaultConfig {
    minSdk = 26
    consumerProguardFiles("consumer-rules.pro")
  }

  compileOptions {
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
  }
}

kotlin { jvmToolchain(21) }

dependencies {
  // UniFFI's Kotlin bindings load the library through JNA.
  api("net.java.dev.jna:jna:5.17.0@aar")
  api("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.10.2")
  api("org.jetbrains.kotlinx:kotlinx-serialization-json:1.9.0")
}
