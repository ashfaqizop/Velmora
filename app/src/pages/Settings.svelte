<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import PoweredBy from "../components/PoweredBy.svelte";
  import DownloadFolder from "../components/DownloadFolder.svelte";
  import AniKey from "../components/AniKey.svelte";
  import PersonalizeDialog from "../components/PersonalizeDialog.svelte";
  import { personalize } from "../lib/ani/personalize.svelte";
  import * as api from "../lib/api";
  import { LANGUAGES, PROVIDER_GROUPS, providerLabel } from "../lib/options";
  import { ani, ANI_CHAT_MODEL, ANI_TOOL_MODEL, settings } from "../lib/state.svelte";
  import { slider } from "../lib/slider";
  import { sound, type SoundOption } from "../lib/audio.svelte";
  import type { AniModel, AppInfo, MediaKind, Preferences, ProviderId } from "../lib/types";

  let info = $state<AppInfo | null>(null);
  const p = $derived(settings.prefs);

  let sync = $state<api.SyncStatus | null>(null);
  const unlisten: (() => void)[] = [];

  const refreshSync = () => api.syncStatus().then((s) => (sync = s));

  onMount(() => {
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
  <m3e-heading variant="headline" size="large" level={1}>Settings</m3e-heading>

  {#if !p}
    <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
  {:else}
    <section>
      <m3e-heading variant="title" size="large" level={2}>Playback</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <div class="row">
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
          <label class="row">
            <div class="text"><strong>Pick 4K automatically</strong><span>Off keeps 4K for when you choose it yourself; it uses a lot of data.</span></div>
            <m3e-switch
              checked={p.max_auto_resolution >= 2160}
              onchange={(e: Event) => set({ max_auto_resolution: checked(e) ? 2160 : 1080 })}
            ></m3e-switch>
          </label>
          <m3e-divider></m3e-divider>
          <div class="row">
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
          <div class="row">
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
          <div class="row">
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
          <div class="row">
            <div class="text"><strong>Anime audio</strong><span>Subtitled Japanese or an English dub when there is one.</span></div>
            <m3e-segmented-button aria-label="Anime audio">
              <m3e-button-segment checked={settings.audioMode === "sub"} onclick={() => settings.setAudioMode("sub")}>Sub</m3e-button-segment>
              <m3e-button-segment checked={settings.audioMode === "dub"} onclick={() => settings.setAudioMode("dub")}>Dub</m3e-button-segment>
            </m3e-segmented-button>
          </div>
        </div>
      </m3e-card>
    </section>

    <section>
      <m3e-heading variant="title" size="large" level={2}>Sound</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          {#each SOUND as [option, name, hint], i (option)}
            {#if i}<m3e-divider></m3e-divider>{/if}
            <label class="row">
              <div class="text"><strong>{name}</strong><span>{hint}</span></div>
              <m3e-switch checked={sound[option]} onchange={(e: Event) => sound.set(option, checked(e))}></m3e-switch>
            </label>
          {/each}
        </div>
      </m3e-card>
    </section>

    <section>
      <m3e-heading variant="title" size="large" level={2}>Sources</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <label class="row">
            <div class="text"><strong>Torrents</strong><span>Adds YTS, EZTV and Nyaa. Your IP is visible to peers, so a VPN is a good idea.</span></div>
            <m3e-switch checked={p.include_torrents} onchange={(e: Event) => set({ include_torrents: checked(e) })}></m3e-switch>
          </label>
          <m3e-divider></m3e-divider>
          <label class="row">
            <div class="text"><strong>French sources</strong><span>VF and VOSTFR releases, labelled as such.</span></div>
            <m3e-switch checked={p.include_french} onchange={(e: Event) => set({ include_french: checked(e) })}></m3e-switch>
          </label>
          <m3e-divider></m3e-divider>
          <label class="row">
            <div class="text"><strong>Check links first</strong><span>Drops dead links and ad pages before you see them. Slightly slower.</span></div>
            <m3e-switch checked={p.verify_sources} onchange={(e: Event) => set({ verify_sources: checked(e) })}></m3e-switch>
          </label>
          <m3e-divider></m3e-divider>
          <label class="row">
            <div class="text"><strong>Show 18+ titles</strong></div>
            <m3e-switch checked={p.show_adult} onchange={(e: Event) => set({ show_adult: checked(e) })}></m3e-switch>
          </label>
        </div>
      </m3e-card>

      <!-- Provider order per content type; closed by default to keep the page short. -->
      <m3e-accordion multi class="providers">
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
                  {providerLabel(id)}
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

    <section>
      <m3e-heading variant="title" size="large" level={2}>Downloads</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <div class="row folder-row">
            <div class="text"><strong>Download folder</strong><span>Movies and episodes you download are saved here.</span></div>
            <DownloadFolder />
          </div>
          <m3e-divider></m3e-divider>
          <div class="row">
            <div class="text"><strong>Downloads at once</strong><span>{p.max_concurrent_downloads} at a time; the rest wait in line.</span></div>
            <m3e-slider class="slider" min="1" max="5" step="1" discrete labelled aria-label="Downloads at once" use:slider>
              <m3e-slider-thumb
                value={p.max_concurrent_downloads}
                onchange={(e: Event) => set({ max_concurrent_downloads: Number((e.currentTarget as HTMLElement & { value: number }).value) })}
              ></m3e-slider-thumb>
            </m3e-slider>
          </div>
          <m3e-divider></m3e-divider>
          <label class="row">
            <div class="text"><strong>Keep seeding finished torrents</strong><span>Off stops uploading as soon as a download completes.</span></div>
            <m3e-switch checked={p.seed_after_download} onchange={(e: Event) => set({ seed_after_download: checked(e) })}></m3e-switch>
          </label>
        </div>
      </m3e-card>
    </section>

    <section>
      <m3e-heading variant="title" size="large" level={2}>Ani</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <label class="row">
            <div class="text">
              <strong>Use Ani</strong>
              <span>Velmora's assistant, in the menu on the left. It runs on your own Gemini key.</span>
            </div>
            <m3e-switch checked={ani.enabled} onchange={(e: Event) => ani.setEnabled(checked(e))}></m3e-switch>
          </label>
          {#if ani.enabled}
            <m3e-divider></m3e-divider>
            <div class="row key-row">
              <AniKey />
            </div>
            <m3e-divider></m3e-divider>
            <label class="row">
              <div class="text">
                <strong>Learn from what I watch</strong>
                <span>Searches, titles you open, watch and download go into Personalize, so Ani's picks fit you.</span>
              </div>
              <m3e-switch checked={personalize.learning} onchange={(e: Event) => personalize.setLearning(checked(e))}></m3e-switch>
            </label>
            <m3e-divider></m3e-divider>
            <div class="row">
              <div class="text"><strong>Personalize</strong><span>{learned}. Stays on this PC.</span></div>
              <m3e-button variant="tonal" onclick={() => personalizeDialog.open()}>
                <m3e-icon slot="icon" name="person_heart"></m3e-icon>See what Ani knows
              </m3e-button>
            </div>
            {#if ani.key?.set}
              <m3e-divider></m3e-divider>
              <div class="row">
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

    <section>
      <m3e-heading variant="title" size="large" level={2}>Accounts</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <div class="row">
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
          <div class="row">
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

    <section>
      <m3e-heading variant="title" size="large" level={2}>Appearance</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <div class="row">
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

    <section>
      <m3e-heading variant="title" size="large" level={2}>About</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <div class="row">
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
</div>

<PersonalizeDialog bind:this={personalizeDialog} />

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 28px;
    padding-bottom: 40px;
    max-width: 860px;
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
