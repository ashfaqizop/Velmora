# Velmora for Android

Velmora's Android app: movies, series, anime, live TV, sports, Theatre and
music, in one app. Version numbers match the desktop app.

## Built on Echo Music

This app is built on [Echo Music](https://github.com/EchoMusicApp/Echo-Music)
by iad1tya and its contributors, licensed under GPL-3.0. Its Kotlin/Compose
app, YouTube Music player, lyrics, recognition and much of its UI came from
there and were renamed and reworked for Velmora. Thank you.

Echo Music in turn credits the projects it builds on (Metrolist, ArchiveTune,
SimpMusic and others); those credits carry over. Kyant0's backdrop code keeps
its Apache-2.0 notice in the files it came in.

## License

GPL-3.0, like Echo Music and UniverseAPI. See [LICENSE](LICENSE).

## Building

Nothing is built on a developer PC. GitHub Actions builds every push to
`main` that touches `android/` (`.github/workflows/android.yml`) and attaches
a signed debug APK to the run. Release builds come from the `app-v*` tags
(phase 10 of [the plan](../docs/android_build_plan.md)).

Decisions: [docs/android.md](../docs/android.md).
