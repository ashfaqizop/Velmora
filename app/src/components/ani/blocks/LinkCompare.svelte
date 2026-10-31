<script lang="ts">
  import PoweredBy from "../../PoweredBy.svelte";
  import { downloadLink } from "../../../lib/ani/actions";
  import type { Entity, RefBook } from "../../../lib/ani/refs";
  import { formatBytes, providerName } from "../../../lib/sources";

  // render_link_compare: two to four links in columns, Ani's pick highlighted.
  type Link = Extract<Entity, { type: "link" }> & { ref: string };
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const links = $derived(
    ((args.refs as string[]) ?? []).flatMap((r) => {
      const e = refs.get(r);
      return e?.type === "link" ? [{ ref: r, ...e } as Link] : [];
    }),
  );
  const pick = $derived(args.pick as string);
  const reasons = $derived((args.reasons as string[]) ?? []);
  let started = $state(new Set<string>());

  const ROWS: [string, (l: Link) => string][] = [
    ["Quality", (l) => (l.source.tags.resolution ? (l.source.tags.resolution >= 2160 ? "4K" : `${l.source.tags.resolution}p`) : "–")],
    ["Size", (l) => ((l.source.size_bytes ?? l.source.tags.size_bytes) ? formatBytes((l.source.size_bytes ?? l.source.tags.size_bytes)!) : "–")],
    ["Codec", (l) => (l.source.tags.codec === "h265" ? "HEVC" : (l.source.tags.codec?.toUpperCase() ?? "–"))],
    ["HDR", (l) => (l.source.tags.hdr && l.source.tags.hdr !== "sdr" ? (l.source.tags.hdr === "dolby_vision" ? "Dolby Vision" : "HDR") : "No")],
    ["Audio", (l) => (l.source.audio?.map((a) => a.lang ?? a.label) ?? l.source.tags.audio ?? l.source.tags.languages ?? []).slice(0, 3).join(", ") || (l.source.tags.multi_audio ? "Multi" : "–")],
    ["Subtitles", (l) => (l.source.subtitles?.length ? String(l.source.subtitles.length) : l.source.tags.has_subs ? "Yes" : "–")],
    ["Seeders", (l) => (l.source.torrent?.seeders != null ? String(l.source.torrent.seeders) : "–")],
  ];
  const rows = $derived(ROWS.filter(([label, f]) => label !== "Seeders" || links.some((l) => f(l) !== "–")));

  async function download(l: Link) {
    await downloadLink(l.source, l.title, l.target);
    started = new Set([...started, l.ref]);
  }
</script>

<m3e-card variant="filled" class="cmp">
  <div slot="content" class="inner">
    <div class="grid" style:--cols={links.length}>
      <span></span>
      {#each links as l (l.ref)}
        <div class="head" class:pick={l.ref === pick}>
          {#if l.ref === pick}<span class="best"><m3e-icon name="wand_stars" filled></m3e-icon>Ani's pick</span>{/if}
          <strong>{providerName(l.source)}</strong>
          <span class="host">{l.source.host}</span>
        </div>
      {/each}
      {#each rows as [label, f] (label)}
        <span class="label">{label}</span>
        {#each links as l (l.ref)}<span class="cell" class:pick={l.ref === pick}>{f(l)}</span>{/each}
      {/each}
      <span></span>
      {#each links as l (l.ref)}
        <div class="cell act" class:pick={l.ref === pick}>
          <m3e-button variant={l.ref === pick ? "filled" : "tonal"} size="small" disabled={started.has(l.ref)} onclick={() => download(l)}>
            <m3e-icon slot="icon" name={started.has(l.ref) ? "check" : "download"}></m3e-icon>{started.has(l.ref) ? "Added" : "Download"}
          </m3e-button>
        </div>
      {/each}
    </div>
    {#if reasons.length}
      <ul class="why">
        {#each reasons as r, i (i)}<li><m3e-icon name="check_circle" filled></m3e-icon>{r}</li>{/each}
      </ul>
    {/if}
    <div class="credit"><PoweredBy height={11} /></div>
  </div>
</m3e-card>

<style>
  .cmp {
    --m3e-card-shape: 24px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px;
  }
  .grid {
    display: grid;
    grid-template-columns: 92px repeat(var(--cols), minmax(0, 1fr));
    font-size: 14px;
  }
  .grid > * {
    padding: 8px 10px;
  }
  .head {
    display: flex;
    flex-direction: column;
    gap: 2px;
    border-radius: 16px 16px 0 0;
  }
  .pick {
    background: color-mix(in srgb, var(--md-sys-color-primary) 12%, transparent);
  }
  .act {
    border-radius: 0 0 16px 16px;
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
  .host {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .label {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
  }
  .why {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 14px;
  }
  .why li {
    display: flex;
    align-items: center;
    gap: 8px;
    --m3e-icon-size: 18px;
  }
  .why m3e-icon {
    color: var(--md-sys-color-primary);
  }
  .credit {
    display: flex;
    justify-content: flex-end;
  }
</style>
