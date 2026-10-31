<script lang="ts">
  import Logo from "../../Logo.svelte";
  import PoweredBy from "../../PoweredBy.svelte";
  import * as api from "../../../lib/api";
  import { personalize } from "../../../lib/ani/personalize.svelte";
  import { ani, nav, settings } from "../../../lib/state.svelte";
  import type { AppInfo } from "../../../lib/types";

  // render_about: what Velmora, UniverseAPI or Ani is, from real app data.
  let { args }: { args: Record<string, unknown> } = $props();

  const topic = $derived((args.topic as "velmora" | "universeapi" | "ani") ?? "velmora");
  const REPO = "https://github.com/ashfaqizop/Velmora";

  let info = $state<AppInfo | null>(null);
  api.appInfo().then((i) => (info = i)).catch(() => {});
  settings.load().catch(() => {});

  // Every source UniverseAPI can use, across kinds, and how many are on.
  const sources = $derived.by(() => {
    const p = settings.prefs;
    if (!p) return null;
    const all = new Set(Object.values(p.provider_order).flat());
    const on = [...all].filter((id) => !p.disabled_providers.includes(id)).length;
    return { all: all.size, on };
  });

  const FEATURES: Record<string, [string, string][]> = {
    velmora: [
      ["travel_explore", "Searches many sources at once and plays the best link"],
      ["play_circle", "Built-in mpv player: MKV, HEVC and 4K with no extra codecs"],
      ["history", "Continue Watching to the exact second, next episode offered"],
      ["download", "Downloads to a folder you pick, playable only in Velmora"],
      ["sync", "AniList and Trakt sync for what you finish"],
      ["wand_stars", "Ani, an assistant that knows what you watch"],
    ],
    universeapi: [
      ["hub", "TMDB, AniList and streaming, download and torrent sources"],
      ["merge", "Merges the same title found on different sites"],
      ["leaderboard", "Ranks links by quality, codec, language and size"],
      ["link", "Resolves hoster pages to files the player can open"],
      ["cloud_sync", "Mirror domains update remotely, without an app update"],
      ["code", "Open source Rust library, GPL-3.0"],
    ],
    ani: [
      ["psychology", "Looks things up with 37 Velmora tools before answering"],
      ["person_heart", "Learns your taste from what you search, watch and download"],
      ["view_carousel", "Answers with cards you can play, save or download from"],
      ["key", "Runs on Gemini with your own key, kept on this PC"],
      ["lock", "Personalize stays on this PC; see or clear it any time"],
      ["back_hand", "Never acts on its own: you press the buttons"],
    ],
  };
</script>

<m3e-card variant="filled" class="about">
  <div slot="content" class="inner">
    <header>
      {#if topic === "velmora"}
        <Logo size={44} word={false} />
        <div class="title">
          <strong class="display">Velmora</strong>
          <span>{info ? `Version ${info.version} · ${info.platform === "windows" ? "Windows" : info.platform}` : "Desktop app"} · GPL-3.0</span>
        </div>
      {:else if topic === "universeapi"}
        <div class="lockup"><PoweredBy variant="full" height={44} /></div>
      {:else}
        <span class="on-shape mark"><m3e-shape name="very-sunny"></m3e-shape><m3e-icon name="wand_stars" filled></m3e-icon></span>
        <div class="title">
          <strong class="display">Ani</strong>
          <span>New in Velmora 0.2.0 · {ani.chatModel}</span>
        </div>
      {/if}
    </header>

    {#if topic === "universeapi" && sources}
      <div class="stats">
        <div><strong>{sources.all}</strong><span>sources</span></div>
        <div><strong>{sources.on}</strong><span>turned on</span></div>
        <div><strong>4</strong><span>kinds: movies, series, anime, dramas</span></div>
      </div>
    {:else if topic === "ani"}
      <div class="stats">
        <div><strong>{personalize.counts.view + personalize.counts.stream + personalize.counts.finish}</strong><span>titles learned from</span></div>
        <div><strong>{personalize.notes.length}</strong><span>notes saved</span></div>
        <div><strong>{ani.key?.hint?.replace("…", "") ?? "–"}</strong><span>your key ends in</span></div>
      </div>
    {/if}

    <ul>
      {#each FEATURES[topic] as [icon, text] (text)}
        <li><m3e-icon name={icon}></m3e-icon>{text}</li>
      {/each}
    </ul>

    <div class="actions">
      {#if topic === "ani"}
        <m3e-button variant="tonal" size="small" onclick={() => nav.go({ name: "settings" })}>
          <m3e-icon slot="icon" name="person_heart"></m3e-icon>See what Ani knows
        </m3e-button>
      {:else}
        <m3e-button variant="tonal" size="small" onclick={() => api.openExternal(topic === "universeapi" ? `${REPO}/tree/main/universeapi` : REPO)}>
          <m3e-icon slot="icon" name="code"></m3e-icon>Source on GitHub
        </m3e-button>
        <m3e-button variant="text" size="small" onclick={() => api.openExternal(`${REPO}/releases`)}>What's new</m3e-button>
      {/if}
    </div>
  </div>
</m3e-card>

<style>
  .about {
    --m3e-card-shape: 24px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 22px;
  }
  header {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .title {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .title strong {
    font-size: 32px;
    font-weight: 400;
    line-height: 1;
  }
  .title span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .lockup {
    padding: 4px 0;
  }
  .mark {
    --m3e-shape-size: 48px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 24px;
  }
  .mark m3e-icon {
    z-index: 1;
  }
  .stats {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }
  .stats div {
    flex: 1;
    min-width: 120px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--md-sys-color-surface-container-high);
  }
  .stats strong {
    font-size: 26px;
    line-height: 1.1;
    color: var(--md-sys-color-primary);
  }
  .stats span {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px 20px;
    font-size: 14px;
  }
  li {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    line-height: 1.4;
    --m3e-icon-size: 20px;
  }
  li m3e-icon {
    flex: none;
    color: var(--md-sys-color-primary);
  }
  .actions {
    display: flex;
    gap: 6px;
  }
</style>
