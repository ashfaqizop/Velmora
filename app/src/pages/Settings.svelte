<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import PoweredBy from "../components/PoweredBy.svelte";
  import ProviderLogo from "../components/ProviderLogo.svelte";
  import DownloadFolder from "../components/DownloadFolder.svelte";
  import AniKey from "../components/AniKey.svelte";
  import MaxLicense from "../components/MaxLicense.svelte";
  import MusicSettings from "../components/music/MusicSettings.svelte";
  import MusicStorage from "../components/music/MusicStorage.svelte";
  import { musicPlayer } from "../lib/music/player.svelte";
  import { tick } from "svelte";
  import { SETTINGS_GROUPS, SETTINGS_PAGES, findSettings, isSettingsPage, type SettingsPage } from "../lib/settings-pages";
  import PersonalizeDialog from "../components/PersonalizeDialog.svelte";
  import { personalize } from "../lib/ani/personalize.svelte";
  import * as api from "../lib/api";
  import { LANGUAGES, PROVIDER_GROUPS, providerLabel } from "../lib/options";
  import { ani, ANI_CHAT_MODEL, ANI_TOOL_MODEL, nav, settings } from "../lib/state.svelte";
  import { slider } from "../lib/slider";
  import { sound, type SoundOption } from "../lib/audio.svelte";
  import type { AniModel, AppInfo, MediaKind, Preferences, ProviderId } from "../lib/types";

  // ── Pages and search ──
  const PAGE_KEY = "velmora.settings.page";
  function lastPage(): SettingsPage {
    try {
      const v = localStorage.getItem(PAGE_KEY);
      return isSettingsPage(v) ? v : "playback";
    } catch {
      return "playback";
    }
  }
  let page = $state<SettingsPage>(lastPage());
  let find = $state("");
  let findInput: HTMLInputElement;
  const searching = $derived(find.trim().length > 0);
  const results = $derived(findSettings(find));
  const pageLabel = (id: SettingsPage) => SETTINGS_PAGES.find((x) => x.id === id)?.label ?? "";

  // A link straight to a page (nav.go({ name: "settings", page: "music" })).
  $effect(() => {
    const r = nav.route;
    if (r.name === "settings" && r.page && isSettingsPage(r.page)) open(r.page, r.setting);
  });

  /** Show a page, and with `setting`, scroll to that row and flash it. */
  async function open(to: SettingsPage, setting?: string) {
    find = "";
    page = to;
    try {
      localStorage.setItem(PAGE_KEY, to);
    } catch {
      // Not remembered.
    }
    await tick();
    if (!setting) {
      document.querySelector(".settings-content")?.scrollIntoView({ block: "start" });
      return;
    }
    // The page's settings may still be loading; give them a moment.
    for (let i = 0; i < 20; i++) {
      const el = document.querySelector<HTMLElement>(`[data-setting="${setting}"]`);
      if (el) {
        el.scrollIntoView({ block: "center", behavior: "smooth" });
        el.classList.remove("flash");
        void el.offsetWidth;
        el.classList.add("flash");
        setTimeout(() => el.classList.remove("flash"), 1800);
        return;
      }
      await new Promise((r) => setTimeout(r, 100));
    }
  }

  let info = $state<AppInfo | null>(null);
  const p = $derived(settings.prefs);

  let sync = $state<api.SyncStatus | null>(null);
  const unlisten: (() => void)[] = [];

  const refreshSync = () => api.syncStatus().then((s) => (sync = s));

  onMount(() => {
    // Ctrl+F finds a setting while Settings is open.
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "f") {
        e.preventDefault();
        findInput?.focus();
      }
    };
    window.addEventListener("keydown", onKey);
    unlisten.push(() => window.removeEventListener("keydown", onKey));
    settings.load();
    api.appInfo().then((i) => (info = i));
    refreshSync();
    ani.loadKey().catch((e) => console.warn("ani key status", e));
    listen("sync-changed", refreshSync).then((u) => unlisten.push(u));
    listen<string>("sync-error", (e) => M3eSnackbar.open(e.payload, true)).then((u) => unlisten.push(u));
  });

  onDestroy(() => unlisten.forEach((u) => u()));

  // Models the saved key can use; fetched again whenever the key changes.
  let models = $state<AniModel[]>([]);
  let personalizeDialog: ReturnType<typeof PersonalizeDialog>;
  const learned = $derived.by(() => {
    const c = personalize.counts;
    const titles = c.view + c.stream + c.finish + c.download;
    const parts = [`${titles} title${titles === 1 ? "" : "s"}`, `${c.search} search${c.search === 1 ? "" : "es"}`];
    if (personalize.notes.length) parts.push(`${personalize.notes.length} note${personalize.notes.length === 1 ? "" : "s"}`);
    return parts.join(", ");
  });
  $effect(() => {
    if (!ani.key?.set) {
      models = [];
      return;
    }
    api
      .aniModels()
      .then((m) => (models = m))
      .catch((e) => console.warn("ani models", e));
  });

  /** The picker's choices, always including the current pick and the default. */
  function modelChoices(current: string, fallback: string): AniModel[] {
    const list = models.length ? [...models] : [];
    for (const id of [fallback, current]) {
      if (!list.some((m) => m.id === id)) list.unshift({ id, displayName: id });
    }
    return list;
  }

  const ACCENTS: [string, string][] = [
    ["#6015EA", "Velmora violet"],
    ["#FF5A5A", "Coral"],
    ["#0B7FD9", "Ocean"],
    ["#00A67E", "Jade"],
    ["#E8A317", "Amber"],
    ["#D6336C", "Rose"],
    ["#7B6CFF", "Iris"],
    ["#4E5D6C", "Slate"],
  ];
  const isAccent = (c: string) => settings.accent.toLowerCase() === c.toLowerCase();

  const set = (patch: Partial<Preferences>) => settings.update(patch);
  const checked = (e: Event) => (e.currentTarget as HTMLElement & { checked: boolean }).checked;
  const selectValue = (e: Event) => (e.currentTarget as HTMLElement & { value: string }).value;

  const SOUND: [SoundOption, string, string][] = [
    ["dialogue", "Dialogue boost", "Makes voices clearer over music and effects. Works best on 5.1 and 7.1 audio."],
    ["night", "Night mode", "Quiets explosions and lifts whispers, so you don't keep reaching for the volume."],
    ["normalize", "Even out volume", "Keeps the level steady between scenes and between sources."],
    ["boost", "Volume up to 200%", "For quiet videos and laptop speakers. Above 100% can distort."],
  ];

  /** Put the chosen language first and keep the rest as fallbacks. */
  function preferLanguage(list: string[], lang: string) {
    return [lang, ...list.filter((l) => l !== lang)];
  }

  function move(kind: MediaKind, id: ProviderId, delta: number) {
    if (!p) return;
    const order = [...(p.provider_order[kind] ?? [])];
    const i = order.indexOf(id);
    const j = i + delta;
    if (i < 0 || j < 0 || j >= order.length) return;
    [order[i], order[j]] = [order[j], order[i]];
    set({ provider_order: { ...p.provider_order, [kind]: order } });
  }

  function toggleProvider(id: ProviderId, on: boolean) {
    if (!p) return;
    const disabled = p.disabled_providers.filter((x) => x !== id);
    set({ disabled_providers: on ? disabled : [...disabled, id] });
  }
</script>

<div class="page">
  <header class="head">
    <m3e-heading variant="headline" size="large" level={1}>Settings</m3e-heading>
    <label class="find">
      <m3e-icon name="search"></m3e-icon>
      <input
        bind:this={findInput}
        bind:value={find}
        placeholder="Find a setting"
        aria-label="Find a setting"
        onkeydown={(e) => {
          if (e.key === "Escape") find = "";
          else if (e.key === "Enter" && results[0]) open(results[0].page, results[0].id);
        }}
      />
      {#if find}
        <button class="find-x" aria-label="Clear search" onclick={() => (find = "")}><m3e-icon name="close"></m3e-icon></button>
      {/if}
    </label>
  </header>

  <div class="layout">
    <nav class="rail" aria-label="Settings pages">
      {#each SETTINGS_GROUPS as g (g.label)}
        <div class="rail-group">{g.label}</div>
        {#each g.pages as pg (pg.id)}
          <button class="rail-item" class:on={page === pg.id && !searching} aria-current={page === pg.id ? "page" : undefined} onclick={() => open(pg.id)}>
            <m3e-icon name={pg.icon} filled={page === pg.id && !searching}></m3e-icon>{pg.label}
          </button>
        {/each}
      {/each}
    </nav>

    <div class="settings-content">
  {#if searching}
    <section>
      <m3e-heading variant="title" size="large" level={2}>Results</m3e-heading>
      {#if results.length}
        <m3e-card variant="filled">
          <div slot="content" class="rows">
            {#each results as r, i (r.id)}
              {#if i}<m3e-divider></m3e-divider>{/if}
              <button class="row result" onclick={() => open(r.page, r.id)}>
                <div class="text">
                  <strong>{r.label}</strong>
                  <span>{pageLabel(r.page)} · {r.hint}</span>
                </div>
                <m3e-icon name="arrow_forward"></m3e-icon>
              </button>
            {/each}
          </div>
        </m3e-card>
      {:else}
        <p class="none">No setting matches "{find.trim()}".</p>
      {/if}
    </section>
  {:else if !p}
    <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
  {:else}
    {#if page === "playback"}
    <section>
      <m3e-heading variant="title" size="large" level={2}>Playback</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <div class="row" data-setting="quality">
            <div class="text"><strong>Preferred quality</strong><span>Velmora picks the closest match first.</span></div>
            <m3e-segmented-button aria-label="Preferred quality">
              {#each [720, 1080, 2160] as r (r)}
                <m3e-button-segment
                  checked={p.preferred_resolution === r}
                  onclick={() => set({ preferred_resolution: r, max_auto_resolution: r === 2160 ? 2160 : p.max_auto_resolution })}
                >
                  {r === 2160 ? "4K" : `${r}p`}
                </m3e-button-segment>
              {/each}
            </m3e-segmented-button>
          </div>
          <m3e-divider></m3e-divider>
          <label class="row" data-setting="auto4k">
            <div class="text"><strong>Pick 4K automatically</strong><span>Off keeps 4K for when you choose it yourself; it uses a lot of data.</span></div>
            <m3e-switch
              checked={p.max_auto_resolution >= 2160}
              onchange={(e: Event) => set({ max_auto_resolution: checked(e) ? 2160 : 1080 })}
            ></m3e-switch>
          </label>
          <m3e-divider></m3e-divider>
          <div class="row" data-setting="codec">
            <div class="text"><strong>Video codec</strong><span>HEVC and AV1 are smaller but need a newer GPU to decode smoothly.</span></div>
            <m3e-segmented-button aria-label="Video codec">
              {#each [["h264", "H.264"], ["h265", "HEVC"], ["av1", "AV1"]] as [value, name] (value)}
                <m3e-button-segment
                  checked={p.preferred_codec === value}
                  onclick={() => set({ preferred_codec: value as Preferences["preferred_codec"] })}
                >
                  {name}
                </m3e-button-segment>
              {/each}
            </m3e-segmented-button>
          </div>
          <m3e-divider></m3e-divider>
          <div class="row" data-setting="audio-lang">
            <div class="text"><strong>Audio language</strong></div>
            <m3e-form-field class="field">
              <m3e-select
                aria-label="Audio language"
                value={p.audio_languages[0] ?? "en"}
                onchange={(e: Event) => set({ audio_languages: preferLanguage(p.audio_languages, selectValue(e)) })}
              >
                {#each LANGUAGES as [code, name] (code)}
                  <m3e-option value={code} selected={p.audio_languages[0] === code}>{name}</m3e-option>
                {/each}
              </m3e-select>
            </m3e-form-field>
          </div>
          <m3e-divider></m3e-divider>
          <div class="row" data-setting="sub-lang">
            <div class="text"><strong>Subtitle language</strong></div>
            <m3e-form-field class="field">
              <m3e-select
                aria-label="Subtitle language"
                value={p.subtitle_languages[0] ?? "en"}
                onchange={(e: Event) => set({ subtitle_languages: preferLanguage(p.subtitle_languages, selectValue(e)) })}
              >
                {#each LANGUAGES as [code, name] (code)}
                  <m3e-option value={code} selected={p.subtitle_languages[0] === code}>{name}</m3e-option>
                {/each}
              </m3e-select>
            </m3e-form-field>
          </div>
          <m3e-divider></m3e-divider>
          <div class="row" data-setting="anime-audio">
            <div class="text"><strong>Anime audio</strong><span>Subtitled Japanese or an English dub when there is one.</span></div>
            <m3e-segmented-button aria-label="Anime audio">
              <m3e-button-segment checked={settings.audioMode === "sub"} onclick={() => settings.setAudioMode("sub")}>Sub</m3e-button-segment>
              <m3e-button-segment checked={settings.audioMode === "dub"} onclick={() => settings.setAudioMode("dub")}>Dub</m3e-button-segment>
            </m3e-segmented-button>
          </div>
        </div>
      </m3e-card>
    </section>
    {/if}

    {#if page === "sound"}
    <section>
      <m3e-heading variant="title" size="large" level={2}>Sound</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          {#each SOUND as [option, name, hint], i (option)}
            {#if i}<m3e-divider></m3e-divider>{/if}
            <label class="row" data-setting={option}>
              <div class="text"><strong>{name}</strong><span>{hint}</span></div>
              <m3e-switch checked={sound[option]} onchange={(e: Event) => sound.set(option, checked(e))}></m3e-switch>
            </label>
          {/each}
        </div>
      </m3e-card>
    </section>
    {/if}

    {#if page === "sources"}
    <section>
      <m3e-heading variant="title" size="large" level={2}>Sources</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <label class="row" data-setting="torrents">
            <div class="text"><strong>Torrents</strong><span>Adds YTS, EZTV and Nyaa. Your IP is visible to peers, so a VPN is a good idea.</span></div>
            <m3e-switch checked={p.include_torrents} onchange={(e: Event) => set({ include_torrents: checked(e) })}></m3e-switch>
          </label>
          <m3e-divider></m3e-divider>
          <label class="row" data-setting="french">
            <div class="text"><strong>French sources</strong><span>VF and VOSTFR releases, labelled as such.</span></div>
            <m3e-switch checked={p.include_french} onchange={(e: Event) => set({ include_french: checked(e) })}></m3e-switch>
          </label>
          <m3e-divider></m3e-divider>
          <label class="row" data-setting="verify">
            <div class="text"><strong>Check links first</strong><span>Drops dead links and ad pages before you see them. Slightly slower.</span></div>
            <m3e-switch checked={p.verify_sources} onchange={(e: Event) => set({ verify_sources: checked(e) })}></m3e-switch>
          </label>
          <m3e-divider></m3e-divider>
          <label class="row" data-setting="adult">
            <div class="text"><strong>Show 18+ titles</strong></div>
            <m3e-switch checked={p.show_adult} onchange={(e: Event) => set({ show_adult: checked(e) })}></m3e-switch>
          </label>
        </div>
      </m3e-card>

      <!-- Provider order per content type; closed by default to keep the page short. -->
      <m3e-accordion multi class="providers" data-setting="providers">
        {#each PROVIDER_GROUPS as group (group.kind)}
          {@const order = p.provider_order[group.kind] ?? []}
          {@const on = order.filter((id) => !p.disabled_providers.includes(id))}
          <m3e-expansion-panel>
            <div slot="header" class="group-head">
              <strong>{group.label} sources</strong>
              <span>
                {on.length ? `${providerLabel(on[0])} first` : "All off"} · {on.length} of {order.length} on
              </span>
            </div>
            <p class="group-hint">Tried from top to bottom. Turn one off to skip it.</p>
            <m3e-list>
              {#each order as id, i (id)}
                <m3e-list-item>
                  <span slot="leading" class="rank">{i + 1}</span>
                  <span class="pname"><ProviderLogo {id} size={26} />{providerLabel(id)}</span>
                  <div slot="trailing" class="provider-actions">
                    <m3e-icon-button size="small" aria-label="Move up" disabled={i === 0} onclick={() => move(group.kind, id, -1)}>
                      <m3e-icon name="arrow_upward"></m3e-icon>
                    </m3e-icon-button>
                    <m3e-icon-button
                      size="small"
                      aria-label="Move down"
                      disabled={i === order.length - 1}
                      onclick={() => move(group.kind, id, 1)}
                    >
                      <m3e-icon name="arrow_downward"></m3e-icon>
                    </m3e-icon-button>
                    <m3e-switch
                      aria-label="Use {providerLabel(id)}"
                      checked={!p.disabled_providers.includes(id)}
                      onchange={(e: Event) => toggleProvider(id, checked(e))}
                    ></m3e-switch>
                  </div>
                </m3e-list-item>
              {/each}
            </m3e-list>
          </m3e-expansion-panel>
        {/each}
      </m3e-accordion>
    </section>
    {/if}

    {#if page === "downloads"}
    <section>
      <m3e-heading variant="title" size="large" level={2}>Downloads</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <div class="row folder-row" data-setting="folder">
            <div class="text"><strong>Download folder</strong><span>Movies and episodes you download are saved here.</span></div>
            <DownloadFolder />
          </div>
          <m3e-divider></m3e-divider>
          <div class="row" data-setting="concurrent">
            <div class="text"><strong>Downloads at once</strong><span>{p.max_concurrent_downloads} at a time; the rest wait in line.</span></div>
            <m3e-slider class="slider" min="1" max="5" step="1" discrete labelled aria-label="Downloads at once" use:slider>
              <m3e-slider-thumb
                value={p.max_concurrent_downloads}
                onchange={(e: Event) => set({ max_concurrent_downloads: Number((e.currentTarget as HTMLElement & { value: number }).value) })}
              ></m3e-slider-thumb>
            </m3e-slider>
          </div>
          <m3e-divider></m3e-divider>
          <label class="row" data-setting="seeding">
            <div class="text"><strong>Keep seeding finished torrents</strong><span>Off stops uploading as soon as a download completes.</span></div>
            <m3e-switch checked={p.seed_after_download} onchange={(e: Event) => set({ seed_after_download: checked(e) })}></m3e-switch>
          </label>
        </div>
      </m3e-card>
    </section>
    {/if}

    {#if page === "music"}
    <section>
      <m3e-heading variant="title" size="large" level={2}>Music</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <label class="row" data-setting="autoplay">
            <div class="text">
              <strong>Autoplay</strong>
              <span>When an album, playlist or your queue ends, keep going with songs like the last few you played.</span>
            </div>
            <m3e-switch checked={musicPlayer.autoplay} onchange={(e: Event) => musicPlayer.setAutoplay(checked(e))}></m3e-switch>
          </label>
          <m3e-divider></m3e-divider>
          <div class="row" data-setting="crossfade">
            <div class="text">
              <strong>Crossfade</strong>
              <span>
                {musicPlayer.prefs.crossfade
                  ? `The next song fades in over the last ${musicPlayer.prefs.crossfade} s of this one.`
                  : "Off: songs follow each other with no gap. Albums play as they were made."}
              </span>
            </div>
            <m3e-slider class="slider" min="0" max="12" step="1" discrete labelled aria-label="Crossfade seconds" use:slider>
              <m3e-slider-thumb
                value={musicPlayer.prefs.crossfade}
                onchange={(e: Event) => musicPlayer.setPrefs({ crossfade: Number((e.currentTarget as HTMLElement & { value: number }).value) })}
              ></m3e-slider-thumb>
            </m3e-slider>
          </div>
          <m3e-divider></m3e-divider>
          <div class="row" data-setting="music-quality">
            <div class="text">
              <strong>Audio quality</strong>
              <span>Low saves data (about 64 kbps). High is the best YouTube has, usually Opus at about 160 kbps.</span>
            </div>
            <m3e-segmented-button aria-label="Audio quality">
              {#each [["low", "Low"], ["normal", "Normal"], ["high", "High"]] as [q, name] (q)}
                <m3e-button-segment
                  checked={musicPlayer.prefs.quality === q}
                  onclick={() => musicPlayer.setPrefs({ quality: q as "low" | "normal" | "high" })}>{name}</m3e-button-segment
                >
              {/each}
            </m3e-segmented-button>
          </div>
          <m3e-divider></m3e-divider>
          <label class="row" data-setting="precache">
            <div class="text">
              <strong>Save the next songs ahead</strong>
              <span>Keeps the next 3 songs on this PC so skipping starts at once. Old ones are cleared past the limit below.</span>
            </div>
            <m3e-switch checked={musicPlayer.prefs.precache} onchange={(e: Event) => musicPlayer.setPrefs({ precache: checked(e) })}></m3e-switch>
          </label>
          {#if musicPlayer.prefs.precache}
            <m3e-divider></m3e-divider>
            <div class="row">
              <div class="text"><strong>Space for saved songs</strong><span>About 5 MB a song at High.</span></div>
              <m3e-segmented-button aria-label="Space for saved songs">
                {#each [250, 500, 1000, 2000] as mb (mb)}
                  <m3e-button-segment checked={musicPlayer.prefs.cacheMb === mb} onclick={() => musicPlayer.setPrefs({ cacheMb: mb })}
                    >{mb >= 1000 ? `${mb / 1000} GB` : `${mb} MB`}</m3e-button-segment
                  >
                {/each}
              </m3e-segmented-button>
            </div>
          {/if}
          <m3e-divider></m3e-divider>
          <div class="row" data-setting="speed">
            <div class="text">
              <strong>Speed</strong>
              <span>{musicPlayer.prefs.speed === 1 ? "Normal speed." : `${musicPlayer.prefs.speed}× for every song.`} Also in the full player.</span>
            </div>
            <m3e-slider class="slider" min="0.5" max="2" step="0.05" labelled aria-label="Speed" use:slider>
              <m3e-slider-thumb
                value={musicPlayer.prefs.speed}
                onchange={(e: Event) => musicPlayer.setPrefs({ speed: Number((e.currentTarget as HTMLElement & { value: number }).value) })}
              ></m3e-slider-thumb>
            </m3e-slider>
          </div>
          <m3e-divider></m3e-divider>
          <label class="row" data-setting="pitch">
            <div class="text">
              <strong>Keep pitch</strong>
              <span>Voices sound natural at any speed. Off makes faster songs higher, like a sped-up track.</span>
            </div>
            <m3e-switch checked={musicPlayer.prefs.keepPitch} onchange={(e: Event) => musicPlayer.setPrefs({ keepPitch: checked(e) })}></m3e-switch>
          </label>
        </div>
      </m3e-card>
      <m3e-card variant="filled">
        <div slot="content"><MusicSettings /></div>
      </m3e-card>
    </section>
    {/if}
    {#if page === "music-storage"}
    <section>
      <m3e-heading variant="title" size="large" level={2}>Storage and history</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content"><MusicStorage /></div>
      </m3e-card>
    </section>
    {/if}

    {#if page === "ani"}
    <section>
      <m3e-heading variant="title" size="large" level={2}>Ani</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <label class="row" data-setting="ani-use">
            <div class="text">
              <strong>Use Ani</strong>
              <span>Velmora's assistant, in the menu on the left. It runs on your own Gemini key.</span>
            </div>
            <m3e-switch checked={ani.enabled} onchange={(e: Event) => ani.setEnabled(checked(e))}></m3e-switch>
          </label>
          {#if ani.enabled}
            <m3e-divider></m3e-divider>
            <div class="row key-row" data-setting="ani-key">
              <AniKey />
            </div>
            <m3e-divider></m3e-divider>
            <label class="row" data-setting="ani-learn">
              <div class="text">
                <strong>Learn from what I watch</strong>
                <span>Searches, titles you open, watch and download go into Personalize, so Ani's picks fit you.</span>
              </div>
              <m3e-switch checked={personalize.learning} onchange={(e: Event) => personalize.setLearning(checked(e))}></m3e-switch>
            </label>
            <m3e-divider></m3e-divider>
            <div class="row" data-setting="ani-personalize">
              <div class="text"><strong>Personalize</strong><span>{learned}. Stays on this PC.</span></div>
              <m3e-button variant="tonal" onclick={() => personalizeDialog.open()}>
                <m3e-icon slot="icon" name="person_heart"></m3e-icon>See what Ani knows
              </m3e-button>
            </div>
            {#if ani.key?.set}
              <m3e-divider></m3e-divider>
              <div class="row" data-setting="ani-models">
                <div class="text"><strong>Chat model</strong><span>Writes Ani's answers.</span></div>
                <m3e-form-field class="field">
                  <m3e-select aria-label="Chat model" value={ani.chatModel} onchange={(e: Event) => ani.setChatModel(selectValue(e))}>
                    {#each modelChoices(ani.chatModel, ANI_CHAT_MODEL) as m (m.id)}
                      <m3e-option value={m.id} selected={ani.chatModel === m.id}>{m.displayName}</m3e-option>
                    {/each}
                  </m3e-select>
                </m3e-form-field>
              </div>
              <m3e-divider></m3e-divider>
              <div class="row">
                <div class="text"><strong>Research model</strong><span>Looks things up before Ani answers. A small, fast model works best.</span></div>
                <m3e-form-field class="field">
                  <m3e-select aria-label="Research model" value={ani.toolModel} onchange={(e: Event) => ani.setToolModel(selectValue(e))}>
                    {#each modelChoices(ani.toolModel, ANI_TOOL_MODEL) as m (m.id)}
                      <m3e-option value={m.id} selected={ani.toolModel === m.id}>{m.displayName}</m3e-option>
                    {/each}
                  </m3e-select>
                </m3e-form-field>
              </div>
            {/if}
          {/if}
        </div>
      </m3e-card>
    </section>
    {/if}

    {#if page === "accounts"}
    <section>
      <m3e-heading variant="title" size="large" level={2}>Accounts</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <div class="row" data-setting="anilist">
            <div class="text">
              <strong>AniList</strong>
              <span>
                {#if !sync?.anilistAvailable}Not set up in this build.
                {:else if sync.anilistUser}Signed in as {sync.anilistUser}. Finished anime episodes update your list.
                {:else}Keep your anime progress in step with your AniList list.{/if}
              </span>
            </div>
            {#if sync?.anilistUser}
              <m3e-button variant="outlined" onclick={() => api.syncDisconnect("anilist")}>Disconnect</m3e-button>
            {:else}
              <m3e-button variant="tonal" disabled={!sync?.anilistAvailable} onclick={() => api.anilistLogin()}>
                <m3e-icon slot="icon" name="login"></m3e-icon>Connect
              </m3e-button>
            {/if}
          </div>
          <m3e-divider></m3e-divider>
          <div class="row" data-setting="trakt">
            <div class="text">
              <strong>Trakt</strong>
              <span>
                {#if !sync?.traktAvailable}Not set up in this build.
                {:else if sync.traktUser}Signed in as {sync.traktUser}. Finished movies and episodes go to your history.
                {:else}Add what you finish to your Trakt history.{/if}
              </span>
            </div>
            {#if sync?.traktUser}
              <m3e-button variant="outlined" onclick={() => api.syncDisconnect("trakt")}>Disconnect</m3e-button>
            {:else}
              <m3e-button variant="tonal" disabled={!sync?.traktAvailable} onclick={() => api.traktLogin()}>
                <m3e-icon slot="icon" name="login"></m3e-icon>Connect
              </m3e-button>
            {/if}
          </div>
        </div>
      </m3e-card>
    </section>
    {/if}

    {#if page === "appearance"}
    <section>
      <m3e-heading variant="title" size="large" level={2}>Appearance</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <div class="row" data-setting="accent">
            <div class="text"><strong>Accent colour</strong><span>The whole palette is generated from it.</span></div>
            <div class="swatches" role="radiogroup" aria-label="Accent colour">
              {#each ACCENTS as [color, name] (color)}
                <button
                  class="swatch"
                  role="radio"
                  aria-checked={isAccent(color)}
                  aria-label={name}
                  title={name}
                  onclick={() => settings.setAccent(color)}
                >
                  <!-- Circle when idle; morphs to a cookie shape when chosen. -->
                  <span class="on-shape" style:--m3e-shape-container-color={color}>
                    <m3e-shape name={isAccent(color) ? "12-sided-cookie" : "circle"}></m3e-shape>
                    {#if isAccent(color)}<m3e-icon name="check"></m3e-icon>{/if}
                  </span>
                </button>
              {/each}
            </div>
          </div>
        </div>
      </m3e-card>
    </section>
    {/if}

    {#if page === "license"}
    <section>
      <m3e-heading variant="title" size="large" level={2}>License</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows" data-setting="license"><MaxLicense /></div>
      </m3e-card>
    </section>
    {/if}

    {#if page === "about"}
    <section>
      <m3e-heading variant="title" size="large" level={2}>About</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <div class="row" data-setting="version">
            <div class="text">
              <strong>Velmora {info?.version ?? ""}</strong>
              <span>Updates install by themselves when Velmora starts.</span>
            </div>
            <m3e-button variant="tonal" onclick={() => location.reload()}>
              <m3e-icon slot="icon" name="update"></m3e-icon>Check now
            </m3e-button>
          </div>
          <m3e-divider></m3e-divider>
          <div class="row">
            <PoweredBy height={14} />
          </div>
        </div>
      </m3e-card>
    </section>
    {/if}
  {/if}
    </div>
  </div>
</div>

<PersonalizeDialog bind:this={personalizeDialog} />

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 40px;
    max-width: 1100px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
  }
  .find {
    display: flex;
    align-items: center;
    gap: 10px;
    width: min(360px, 50%);
    height: 48px;
    padding: 0 8px 0 16px;
    border-radius: 24px;
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 22px;
  }
  .find:focus-within {
    outline: 2px solid var(--md-sys-color-primary);
  }
  .find input {
    all: unset;
    flex: 1;
    min-width: 0;
    font-size: 15px;
    color: var(--md-sys-color-on-surface);
  }
  .find input::placeholder {
    color: var(--md-sys-color-on-surface-variant);
  }
  .find-x {
    all: unset;
    cursor: pointer;
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    --m3e-icon-size: 18px;
  }
  .find-x:hover {
    background: var(--md-sys-color-surface-container-highest);
  }
  .layout {
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    gap: 28px;
    align-items: start;
  }
  .rail {
    position: sticky;
    top: 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .rail-group {
    padding: 14px 16px 6px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
  }
  .rail-group:first-child {
    padding-top: 0;
  }
  .rail-item {
    all: unset;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 12px;
    height: 44px;
    padding: 0 16px;
    border-radius: 22px;
    font-size: 14.5px;
    font-weight: 600;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 22px;
    transition: background 150ms;
  }
  .rail-item:hover {
    background: var(--md-sys-color-surface-container-high);
  }
  .rail-item:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
  }
  .rail-item.on {
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .settings-content {
    display: flex;
    flex-direction: column;
    gap: 28px;
    min-width: 0;
    scroll-margin-top: 12px;
  }
  .result {
    all: unset;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 14px 8px;
    border-radius: 12px;
  }
  .result:hover {
    background: var(--md-sys-color-surface-container-high);
  }
  .none {
    margin: 0;
    color: var(--md-sys-color-on-surface-variant);
  }
  /* A row found by search flashes once. */
  :global([data-setting].flash) {
    animation: flash 1.8s ease-out;
    border-radius: 12px;
  }
  @keyframes flash {
    0%,
    30% {
      background: color-mix(in srgb, var(--md-sys-color-primary) 22%, transparent);
    }
    100% {
      background: transparent;
    }
  }
  /* Narrow windows: the pages become a row of tabs along the top. */
  @media (max-width: 1180px) {
    .layout {
      grid-template-columns: minmax(0, 1fr);
      gap: 16px;
    }
    .rail {
      position: static;
      flex-direction: row;
      overflow-x: auto;
      scrollbar-width: none;
      gap: 6px;
    }
    .rail-group {
      display: none;
    }
    .rail-item {
      flex: none;
      height: 40px;
      background: var(--md-sys-color-surface-container);
    }
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .rows {
    display: flex;
    flex-direction: column;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 14px 4px;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .folder-row .text {
    flex: none;
  }
  /* The path takes the rest of the row and ellipsises. */
  .folder-row > :global(.folder) {
    flex: 1;
    max-width: 560px;
  }
  .text strong {
    font-weight: 700;
  }
  .key-row {
    display: block;
  }
  .text span,
  .group-head span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .field {
    width: 220px;
  }
  .slider {
    width: 220px;
  }
  .providers {
    display: block;
    --m3e-expansion-panel-container-color: var(--md-sys-color-surface-container);
    --m3e-expansion-header-collapsed-height: 68px;
    --m3e-expansion-header-expanded-height: 68px;
  }
  .group-head {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .group-head strong {
    font-weight: 700;
  }
  .group-hint {
    margin: 0 0 4px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .rank {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    font-size: 13px;
    font-weight: 700;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .pname {
    display: inline-flex;
    align-items: center;
    gap: 12px;
  }
  .provider-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .provider-actions m3e-switch {
    margin-left: 8px;
  }
  .swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    justify-content: flex-end;
  }
  .swatch {
    all: unset;
    cursor: pointer;
    display: grid;
    border-radius: 50%;
    transition: transform var(--md-sys-motion-spring-fast-spatial, 350ms ease);
  }
  .swatch:hover {
    transform: scale(1.1);
  }
  .swatch:focus-visible {
    outline: 2px solid var(--md-sys-color-on-surface);
    outline-offset: 3px;
  }
  .swatch .on-shape {
    --m3e-shape-size: 40px;
    --m3e-shape-transition: var(--md-sys-motion-spring-default-spatial, 500ms ease);
    color: #fff;
    --m3e-icon-size: 22px;
  }
  .ghost {
    height: 320px;
  }
</style>
