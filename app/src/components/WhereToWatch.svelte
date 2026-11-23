<script lang="ts">
  // The row under a title's Play button: where it can be played, filling in
  // as each site answers (see lib/finder.svelte.ts). One chip per site with
  // the best quality it has; torrents wait behind one "+ torrents" chip.
  // Clicking a chip plays that site's best link.
  import { fade, scale } from "svelte/transition";
  import { flip } from "svelte/animate";
  import ProviderLogo from "./ProviderLogo.svelte";
  import { providerLabel } from "../lib/options";
  import { isTorrent } from "../lib/sources";
  import { spring } from "../lib/motion.svelte";
  import type { Finding, FoundSite } from "../lib/finder.svelte";
  import type { Source } from "../lib/types";

  let { finding, onplay }: { finding: Finding; onplay: (s: Source) => void } = $props();

  let torrentsOpen = $state(false);

  const TORRENT_SITES = new Set(["yts", "eztv", "nyaa"]);
  const torrentSite = (s: FoundSite) => TORRENT_SITES.has(s.provider) || (s.sources.length > 0 && s.sources.every(isTorrent));

  /** A site's best link: the best ranked overall once every site answered, else its own first. */
  function bestOf(site: FoundSite): Source | undefined {
    const ranked = finding.report?.sources.find((x) => x.provider === site.provider);
    if (ranked) return ranked;
    return [...site.sources].sort((a, b) => (b.tags.resolution ?? 0) - (a.tags.resolution ?? 0))[0];
  }

  function quality(s: Source | undefined): string {
    const r = s?.tags.resolution;
    if (!r) return "";
    return r >= 2160 ? "4K" : `${r}p`;
  }

  function extras(s: Source | undefined): string | null {
    if (!s) return null;
    const t = s.tags;
    if (t.hdr && t.hdr !== "sdr") return t.hdr === "dolby_vision" ? "Dolby Vision" : "HDR";
    if (t.multi_audio) return "Dual audio";
    return t.french ?? null;
  }

  const direct = $derived(finding.sites.filter((s) => !torrentSite(s)));
  const looking = $derived(direct.filter((s) => s.state === "looking"));
  const found = $derived(direct.filter((s) => s.state === "found"));
  const torrents = $derived(finding.sites.filter((s) => torrentSite(s) && s.state === "found"));
  const torrentLinks = $derived(torrents.reduce((n, s) => n + s.sources.length, 0));
  const status = $derived.by(() => {
    if (!finding.sites.length) return "Looking for sources…";
    if (looking.length) return `Looking on ${looking.length} more site${looking.length === 1 ? "" : "s"}…`;
    if (found.length || torrents.length) return `On ${found.length + torrents.length} site${found.length + torrents.length === 1 ? "" : "s"}`;
    return "No site has it right now";
  });
</script>

<div class="wtw" aria-live="polite">
  <div class="head">
    <span class="label">Where to watch</span>
    <span class="status">
      {#if looking.length || !finding.sites.length}<span class="dot"></span>{/if}
      {status}
    </span>
  </div>
  <div class="chips">
    {#each found as site (site.provider)}
      {@const best = bestOf(site)}
      <button
        class="chip"
        title={`Play on ${providerLabel(site.provider)}`}
        onclick={() => best && onplay(best)}
        animate:flip={spring("default-spatial")}
        in:scale={{ start: 0.85, opacity: 0, ...spring("fast-spatial") }}
      >
        <ProviderLogo id={site.provider} size={18} />
        <span class="name">{providerLabel(site.provider)}</span>
        {#if quality(best)}<span class="q">{quality(best)}</span>{/if}
        {#if extras(best)}<span class="x">{extras(best)}</span>{/if}
        {#if site.sources.length > 1}<span class="more">+{site.sources.length - 1}</span>{/if}
      </button>
    {/each}
    {#each looking as site (site.provider)}
      <span class="chip ghost" title={`Asking ${providerLabel(site.provider)}`} out:fade={spring("fast-effects")}>
        <ProviderLogo id={site.provider} size={18} />
        <span class="name">{providerLabel(site.provider)}</span>
      </span>
    {/each}
    {#if torrents.length}
      <button class="chip torrents" class:open={torrentsOpen} onclick={() => (torrentsOpen = !torrentsOpen)} aria-expanded={torrentsOpen}>
        <m3e-icon name="hub"></m3e-icon>
        {torrentsOpen ? "Hide torrents" : `+ torrents (${torrentLinks})`}
      </button>
      {#if torrentsOpen}
        {#each torrents as site (site.provider)}
          {@const best = bestOf(site)}
          <button class="chip" onclick={() => best && onplay(best)} in:scale={{ start: 0.85, opacity: 0, ...spring("fast-spatial") }}>
            <ProviderLogo id={site.provider} size={18} />
            <span class="name">{providerLabel(site.provider)}</span>
            {#if quality(best)}<span class="q">{quality(best)}</span>{/if}
            {#if best?.torrent?.seeders !== undefined}<span class="x">{best.torrent.seeders} seeds</span>{/if}
          </button>
        {/each}
      {/if}
    {/if}
  </div>
</div>

<style>
  .wtw {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 14px;
  }
  .head {
    display: flex;
    align-items: baseline;
    gap: 10px;
    font-size: 13px;
  }
  .label {
    font-weight: 700;
    letter-spacing: 0.02em;
  }
  .status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--md-sys-color-primary);
    animation: wtw-pulse 1.1s ease-in-out infinite;
  }
  @keyframes wtw-pulse {
    50% {
      opacity: 0.25;
      transform: scale(0.7);
    }
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .chip {
    all: unset;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 34px;
    padding: 0 12px 0 8px;
    border-radius: 12px;
    font-size: 13px;
    font-weight: 600;
    color: var(--md-sys-color-on-surface);
    background: color-mix(in srgb, var(--md-sys-color-surface-container-highest) 80%, transparent);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 60%, transparent);
    cursor: pointer;
    transition:
      background var(--spring-fast-effects),
      transform var(--spring-fast-spatial);
    --m3e-icon-size: 18px;
  }
  button.chip:hover {
    background: color-mix(in srgb, var(--md-sys-color-primary) 18%, var(--md-sys-color-surface-container-highest));
  }
  button.chip:active {
    transform: scale(0.95);
  }
  button.chip:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
    outline-offset: 2px;
  }
  .q {
    padding: 2px 6px;
    border-radius: 6px;
    font-size: 11.5px;
    font-weight: 800;
    color: var(--md-sys-color-on-primary);
    background: var(--md-sys-color-primary);
  }
  .x,
  .more {
    font-size: 12px;
    font-weight: 600;
    color: var(--md-sys-color-on-surface-variant);
  }
  .ghost {
    cursor: default;
    opacity: 0.55;
    background: linear-gradient(
      100deg,
      color-mix(in srgb, var(--md-sys-color-surface-container-highest) 60%, transparent) 30%,
      color-mix(in srgb, var(--md-sys-color-surface-container-highest) 100%, white 6%) 50%,
      color-mix(in srgb, var(--md-sys-color-surface-container-highest) 60%, transparent) 70%
    );
    background-size: 250% 100%;
    animation: wtw-shimmer 1.4s linear infinite;
  }
  @keyframes wtw-shimmer {
    from {
      background-position: 100% 0;
    }
    to {
      background-position: -150% 0;
    }
  }
  .torrents {
    color: var(--md-sys-color-on-surface-variant);
  }
  .torrents.open {
    color: var(--md-sys-color-on-surface);
  }
</style>
