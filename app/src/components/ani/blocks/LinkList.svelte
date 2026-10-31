<script lang="ts">
  import PoweredBy from "../../PoweredBy.svelte";
  import { downloadLink, play } from "../../../lib/ani/actions";
  import type { Entity, RefBook } from "../../../lib/ani/refs";
  import { audioLabel, hostLabel, providerName, releaseTags, sourceChips, subtitleLabel } from "../../../lib/sources";
  import { targetLabel } from "../../../lib/state.svelte";

  // render_download_links, render_torrents and render_stream_links: what
  // UniverseAPI found, one row per link. Buttons act only when clicked.
  type Link = Extract<Entity, { type: "link" }>;
  let { args, refs, mode }: { args: Record<string, unknown>; refs: RefBook; mode: "download" | "torrent" | "stream" } = $props();

  const links = $derived(
    ((args.refs as string[]) ?? []).flatMap((r) => {
      const e = refs.get(r);
      return e?.type === "link" ? [{ ref: r, ...(e as Link) }] : [];
    }),
  );
  const first = $derived(links[0]);
  const pick = $derived(args.pick as string | undefined);
  let started = $state(new Set<string>());

  const ICON = { download: "download", torrent: "cloud_download", stream: "play_circle" } as const;
  const TITLE = { download: "Downloads", torrent: "Torrents", stream: "Streams" } as const;

  async function download(l: (typeof links)[number]) {
    await downloadLink(l.source, l.title, l.target);
    started = new Set([...started, l.ref]);
  }

  const audio = (l: Link) => audioLabel(l.source);
  const subs = (l: Link) => subtitleLabel(l.source);
</script>

{#if first}
  <m3e-card variant="filled" class="links">
    <div slot="content" class="inner">
      <header>
        <m3e-icon name={ICON[mode]}></m3e-icon>
        <div class="what">
          <strong>{TITLE[mode]} for {first.title.name}{first.target.type !== "movie" ? ` · ${first.target.type === "season" ? `Season ${first.target.season}` : targetLabel(first.target)}` : ""}</strong>
          {#if args.note}<span>{args.note}</span>{/if}
        </div>
      </header>

      {#if mode === "torrent"}
        <m3e-callout variant="warning" class="vpn">
          <m3e-icon slot="icon" name="vpn_lock"></m3e-icon>
          Peers can see your IP address while a torrent runs. Use a VPN if that matters to you.
        </m3e-callout>
      {/if}

      <ul>
        {#each links as l (l.ref)}
          <li class:pick={pick === l.ref} title={l.source.filename ?? l.source.label}>
            <div class="text">
              <div class="top">
                <strong>{providerName(l.source)}</strong>
                {#if hostLabel(l.source)}<span class="host">{hostLabel(l.source)}</span>{/if}
                {#if pick === l.ref}<span class="best"><m3e-icon name="wand_stars" filled></m3e-icon>Ani's pick</span>{/if}
              </div>
              <div class="chips">
                {#each sourceChips(l.source) as c (c)}<span class="chip">{c}</span>{/each}
                {#if audio(l)}<span class="chip soft"><m3e-icon name="volume_up"></m3e-icon>{audio(l)}</span>{/if}
                {#if subs(l)}<span class="chip soft"><m3e-icon name="subtitles"></m3e-icon>{subs(l)}</span>{/if}
                {#each releaseTags(l.source) as tag (tag)}<span class="chip soft">{tag}</span>{/each}
              </div>
              {#if pick === l.ref && args.pick_reason}<span class="why">{args.pick_reason}</span>{/if}
            </div>
            {#if mode === "stream"}
              <m3e-button variant={pick === l.ref ? "filled" : "tonal"} size="small" onclick={() => play(l.title, l.target)}>
                <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>Play
              </m3e-button>
            {:else}
              <m3e-button
                variant={pick === l.ref ? "filled" : "tonal"}
                size="small"
                disabled={started.has(l.ref)}
                onclick={() => download(l)}
              >
                <m3e-icon slot="icon" name={started.has(l.ref) ? "check" : "download"}></m3e-icon>
                {started.has(l.ref) ? "Added" : "Download"}
              </m3e-button>
            {/if}
          </li>
        {/each}
      </ul>
      <div class="credit"><PoweredBy height={11} /></div>
    </div>
  </m3e-card>
{/if}

<style>
  .links {
    --m3e-card-shape: 24px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 18px;
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    --m3e-icon-size: 22px;
  }
  header > m3e-icon {
    color: var(--md-sys-color-primary);
  }
  .what {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .what span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .vpn {
    display: block;
    font-size: 13px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--md-sys-color-surface-container-high);
    border: 1px solid transparent;
  }
  li.pick {
    border-color: var(--md-sys-color-primary);
    background: color-mix(in srgb, var(--md-sys-color-primary) 10%, var(--md-sys-color-surface-container-high));
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .top {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    font-size: 14px;
  }
  .host {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .best {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-weight: 700;
    color: var(--md-sys-color-primary);
    --m3e-icon-size: 14px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 8px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    --m3e-icon-size: 14px;
  }
  .chip.soft {
    background: var(--md-sys-color-surface-container-highest);
    color: var(--md-sys-color-on-surface-variant);
  }
  .why {
    font-size: 13px;
    color: var(--md-sys-color-primary);
  }
  .credit {
    display: flex;
    justify-content: flex-end;
  }
</style>
