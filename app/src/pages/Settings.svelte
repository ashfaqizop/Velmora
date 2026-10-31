<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import PoweredBy from "../components/PoweredBy.svelte";
  import * as api from "../lib/api";
  import { LANGUAGES, PROVIDER_GROUPS, providerLabel } from "../lib/options";
  import { settings } from "../lib/state.svelte";
  import type { AppInfo, MediaKind, Preferences, ProviderId } from "../lib/types";

  let info = $state<AppInfo | null>(null);
  const p = $derived(settings.prefs);

  let sync = $state<api.SyncStatus | null>(null);
  let traktDialog: HTMLElement & { show(): Promise<void>; hide(v?: string): Promise<void> };
  let device = $state<api.DeviceCode | null>(null);
  let traktError = $state<string | null>(null);
  let pollTimer: ReturnType<typeof setTimeout> | undefined;
  const unlisten: (() => void)[] = [];

  const refreshSync = () => api.syncStatus().then((s) => (sync = s));

  onMount(() => {
    settings.load();
    api.appInfo().then((i) => (info = i));
    refreshSync();
    listen("sync-changed", refreshSync).then((u) => unlisten.push(u));
    listen<string>("sync-error", (e) => M3eSnackbar.open(e.payload, true)).then((u) => unlisten.push(u));
  });

  onDestroy(() => {
    clearTimeout(pollTimer);
    unlisten.forEach((u) => u());
  });

  async function connectTrakt() {
    traktError = null;
    device = null;
    traktDialog.show();
    try {
      device = await api.traktStart();
      poll(device, Date.now() + device.expiresIn * 1000);
    } catch (e) {
      traktError = String(e);
    }
  }

  function poll(code: api.DeviceCode, deadline: number) {
    clearTimeout(pollTimer);
    pollTimer = setTimeout(async () => {
      if (device !== code) return; // dialog closed or restarted
      if (Date.now() > deadline) {
        traktError = "The code expired. Start again.";
        return;
      }
      try {
        if (await api.traktPoll(code.deviceCode)) {
          device = null;
          await traktDialog.hide();
          M3eSnackbar.open("Trakt connected");
          return;
        }
        poll(code, deadline);
      } catch (e) {
        traktError = String(e);
      }
    }, code.interval * 1000);
  }

  function traktClosed() {
    device = null;
    clearTimeout(pollTimer);
  }

  const ACCENTS = ["#6015EA", "#FF5A5A", "#0B7FD9", "#00A67E", "#E8A317", "#D6336C", "#7B6CFF", "#4E5D6C"];

  const set = (patch: Partial<Preferences>) => settings.update(patch);
  const checked = (e: Event) => (e.currentTarget as HTMLElement & { checked: boolean }).checked;
  const selectValue = (e: Event) => (e.currentTarget as HTMLElement & { value: string }).value;

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

      {#each PROVIDER_GROUPS as group (group.kind)}
        {@const order = p.provider_order[group.kind] ?? []}
        <m3e-card variant="outlined" class="providers">
          <div slot="content">
            <div class="group-head">
              <strong>{group.label}</strong>
              <span>Tried from top to bottom. Turn one off to skip it.</span>
            </div>
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
          </div>
        </m3e-card>
      {/each}
    </section>

    <section>
      <m3e-heading variant="title" size="large" level={2}>Downloads</m3e-heading>
      <m3e-card variant="filled">
        <div slot="content" class="rows">
          <div class="row">
            <div class="text"><strong>Downloads at once</strong><span>{p.max_concurrent_downloads} at a time; the rest wait in line.</span></div>
            <m3e-slider class="slider" min="1" max="5" step="1" discrete labelled aria-label="Downloads at once">
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
              <m3e-button variant="tonal" disabled={!sync?.traktAvailable} onclick={connectTrakt}>
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
              {#each ACCENTS as color (color)}
                <button
                  class="swatch"
                  class:active={settings.accent.toLowerCase() === color.toLowerCase()}
                  role="radio"
                  aria-checked={settings.accent.toLowerCase() === color.toLowerCase()}
                  aria-label={color}
                  onclick={() => settings.setAccent(color)}
                >
                  <m3e-theme-icon color={color} scheme="dark"></m3e-theme-icon>
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

<m3e-dialog bind:this={traktDialog} dismissible onclosed={traktClosed} class="trakt">
  <span slot="header">Connect Trakt</span>
  <div class="trakt-body">
    {#if traktError}
      <p class="error">{traktError}</p>
    {:else if !device}
      <m3e-loading-indicator></m3e-loading-indicator>
    {:else}
      <p>Open the Trakt page below and enter this code:</p>
      <span class="code">{device.userCode}</span>
      <m3e-button variant="filled" onclick={() => device && api.openSignInPage(device.verificationUrl)}>
        <m3e-icon slot="icon" name="open_in_new"></m3e-icon>Open trakt.tv/activate
      </m3e-button>
      <div class="waiting">
        <m3e-loading-indicator></m3e-loading-indicator>
        <span>Waiting for you to approve on Trakt</span>
      </div>
    {/if}
  </div>
</m3e-dialog>

<style>
  .trakt {
    --m3e-dialog-max-width: 420px;
  }
  .trakt-body {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    text-align: center;
    padding-block: 8px;
  }
  .trakt-body p {
    margin: 0;
  }
  .code {
    font-family: var(--app-display-font);
    font-size: 44px;
    letter-spacing: 0.12em;
    user-select: all;
  }
  .waiting {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-loading-indicator-container-size: 32px;
    --m3e-loading-indicator-active-indicator-size: 24px;
  }
  .error {
    color: var(--md-sys-color-error);
  }
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
  .text strong {
    font-weight: 700;
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
    --m3e-card-shape: var(--md-sys-shape-corner-large, 16px);
  }
  .group-head {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 4px 4px 8px;
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
    gap: 8px;
    justify-content: flex-end;
  }
  .swatch {
    all: unset;
    cursor: pointer;
    border-radius: 50%;
    padding: 3px;
    outline: 2px solid transparent;
    transition: outline-color 150ms ease;
  }
  .swatch.active {
    outline-color: var(--md-sys-color-primary);
  }
  .swatch:focus-visible {
    outline-color: var(--md-sys-color-on-surface);
  }
  m3e-theme-icon {
    --m3e-theme-icon-size: 36px;
  }
  .ghost {
    height: 320px;
  }
</style>
