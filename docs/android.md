# Velmora for Android

The Android app is built on [Echo Music](https://github.com/EchoMusicApp/Echo-Music)
(GPL-3.0): its Kotlin/Compose app, YouTube Music player and UI/UX, renamed to
Velmora and reworked around Velmora's features. Video sources come from the
Rust engine (UniverseAPI); music stays on Echo's `:innertube` module.

No Gradle, Android Studio or cargo on the developer PC. Every build runs on
GitHub Actions.

## Decisions

Asked in batches on 2026-11-27.

1. **Repo:** monorepo. Echo's files (without its git history) live in `android/`
   next to `app/` (desktop), `site/` and `universeapi/`, with our own CI workflow.
2. **Identity:** application id and namespace `com.velmora.stepbyte`, name
   "Velmora". Every Kotlin package is renamed too (`com.music.echo...` →
   `com.velmora.stepbyte...`); no "echo" left internally. Debug builds use the
   `.debug` suffix and install next to release.
3. **Rust engine:** UniverseAPI built for Android in CI and called from Kotlin.
   ABIs arm64-v8a and armeabi-v7a only. One universal APK.
4. **Echo extras dropped:** Firebase Analytics/Crashlytics, Google Drive backup,
   Listen Together. **Kept:** Google Cast, Discord presence, Spotify import, AI
   lyrics translation, podcasts, music recognition, canvas backgrounds, LAN
   sync, Wrapped/stats, and everything else.
5. **Design:** two looks the user can switch between: Echo's Liquid Glass, and
   Material You (Material 3 Expressive, dynamic color).
6. **Accounts:** a Velmora account sign-in is required, like the desktop app.
   The Music part also requires a YouTube Music sign-in, like desktop.
7. **Scope:** everything Velmora desktop has ships on Android too, built over
   10 phases (plan to come from the user). Echo's backup and restore stays,
   including "Transfer from Echo Music" (restore an Echo backup, old or new
   format).
8. **Music:** a mode switch like desktop: Velmora opens on video, and Music
   turns the app into the music UI. The layout needs real design work, not a
   bolt-on.
9. **Video player:** ExoPlayer (Media3). Torrent sources (YTS, EZTV, Nyaa) work
   on Android, through the Rust engine.
10. **Versions:** same numbers as desktop. OTA updates in the app: download in
    the background, then prompt to install (exact flow to design).
11. **Signing:** one release keystore, made on 2026-11-27 (RSA 4096, alias
    `velmora`, valid to 2056, subject `CN=Velmora, O=StepByte`, SHA-256
    `D7:55:CF:03:86:08:B0:5A:D2:4C:20:2E:8D:6B:6A:A6:D8:58:2B:E4:21:E7:58:92:06:0D:CD:EB:A2:AB:A6:E2`).
    It lives only in GitHub Actions secrets (`ANDROID_KEYSTORE_BASE64`,
    `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`, `ANDROID_STORE_PASSWORD`) and
    in `%USERPROFILE%\.velmora-android\` on the developer PC as a backup. Never
    in the repo. Losing it means nobody can update without reinstalling.
12. **Devices:** phones and tablets, Android 8.0 (API 26) and up. No TV.
13. **Phases:** 10 phases, proposed by Claude once the questions are done. Grown to 13 on 2026-11-27: three UI/UX phases (10 to 12) before the release.
14. **Engine boundary:** Kotlin calls UniverseAPI for search, sources, stream
    resolving, metadata (TMDB/AniList), torrents, sports and Live TV. Kotlin
    owns UI, playback, download storage and the local database. Music stays on
    `:innertube`.
15. **Ani:** desktop only. Not in the Android app.
16. **Ads:** none on Android.
17. **Distribution:** sideload only (GitHub Releases and the website). No Play
    Store, so torrents, in-app APK updates and anything else Play forbids are fine.
18. **Echo's services:** replaced with Velmora's (site, repo, links); removed
    where Velmora has no equivalent.
19. **Max:** the badge and Theatre perks (early booking) only, restored from the
    account like desktop. Nothing else to unlock, since there are no ads.
20. **Echo's Discord, Patreon, donate links:** removed.
21. **First launch:** mirrors desktop (splash, welcome, sign in, personalize,
    done), plus a step to pick the look (Liquid Glass or Material You).
    Android 8 to 11: the AndroidX SplashScreen backport gives the same splash;
    Material You uses a fixed Velmora palette (no wallpaper colors before 12);
    Liquid Glass falls back to frosted panels without blur, labelled
    "simplified on this phone".
22. **Sync:** same Supabase sync as desktop (My List, watch progress, and the
    settings both platforms share: language, subtitles, sources, adult filter).
23. **Downloads:** public storage (`Movies/Velmora`) but locked like desktop:
    the same `.vlm` vault format (ChaCha20, per-install key) from the Rust
    engine, so only Velmora plays them. ExoPlayer reads them through a
    decrypting data source. Music downloads stay as Echo stores them.
24. **Picture-in-picture** on by default; **background audio** for video as a
    setting.
25. **Vault key backup:** the per-install download key is backed up to the
    Velmora account (encrypted) and restored on sign-in, so downloads survive a
    reinstall. Desktop gets the same later.
26. **Icon:** built from the existing Velmora icons (the fox mark in
    `app/src-tauri/icons/`): adaptive icon layers, round icon, and a monochrome
    layer for Android 13 themed icons.
27. **Cast:** music only for now (Echo's Google Cast). No video casting yet.
28. **Languages:** English only for now. Echo's translations are not kept.
29. **Releases:** one tag `app-vX.Y.Z` builds both apps into one GitHub Release
    (Windows installer + Android APK); the website offers both. CI also builds a
    debug APK on every push to `main` that touches `android/`.
30. **Test devices:** a phone on Android 9, a phone on Android 14, a tablet on
    Android 16.
31. **Video-mode navigation:** bottom bar Home | Browse | Theatre | Search.
    Downloads is a button at the top right. The account lives in Settings.
32. **Social:** the web Social pages shown inside the app (WebView), not native
    screens.
33. **Switching to Music (or back):** three ways, all of them: a pill in the top
    bar, a long-press on the logo, and search (typing "music", "movie" or
    "velmora" offers "Switch to Music" / "Switch to Velmora").
34. **Mobile data:** downloads on Wi-Fi only by default, lower streaming
    quality on mobile data, a warning before 4K on cellular (all settings).
35. **Feature names:** Velmora Find (recognition), Velmora Connect (LAN sync
    and handoff), Velmora Wrapped. Fine for now.
36. **App Links:** `velmora://`, `getvelmora.dpdns.org/open/...` and Social
    profile links open the installed app (verified App Links; one file on the site).
37. **Top bar (both modes):** "Velmora" wordmark (text, not the fox) on the
    left. On the right, from left to right: avatar, Downloads, the mode switch
    icon, then more icons as needed.
    - Avatar opens a quick panel like Echo's (video mode: Velmora account card,
      Social profile, Max, Settings, About; music mode: Echo's panel plus Settings).
    - Downloads opens downloaded movies in video mode, downloaded music in music mode.
    - The switch icon opens the mode-switch panel: a frosted panel dropping from
      the top with one track split into two worlds (Velmora left: drifting
      posters, blue; Music right: album art and a live waveform, cream). Drag
      the glass thumb left to right for Music, right to left for Velmora.
      Posters turn into album covers as the thumb passes, the film strip bends
      into a sound wave, colors flow across, haptic ticks every 10%. It commits
      at about 60% with a circular reveal from the thumb; releasing early
      springs back. Each mode keeps its own place. Tapping the other half,
      TalkBack ("Switch to Music") and reduced motion (crossfade) all work. A
      one-time nudge teaches the gesture. Long-pressing the wordmark and the
      search shortcut switch directly.
38. **Live TV and Sports:** both. Tabs inside Browse and rows on Home.
39. **Fonts and colors:** Plus Jakarta Sans and Instrument Serif, Velmora blue
    and cream, in both looks (replacing Echo's).
40. **Playback across modes:** music keeps playing with a mini-player when you
    switch to Velmora and pauses when a video starts; switching to Music closes
    a video's picture-in-picture.
