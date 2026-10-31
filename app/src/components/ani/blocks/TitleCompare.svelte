<script lang="ts">
  import Art from "./Art.svelte";
  import TitleActions from "./TitleActions.svelte";
  import { openTitle } from "../../../lib/ani/actions";
  import { detailOf } from "../../../lib/ani/tools/shape";
  import type { RefBook } from "../../../lib/ani/refs";
  import { kindLabel } from "../../../lib/state.svelte";
  import type { TitleDetail } from "../../../lib/types";

  // render_title_compare: two or three titles in columns. Year, rating and
  // runtime come from the data; Ani adds its own rows and a verdict.
  type Row = { label: string; values: string[] };
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const titles = $derived(((args.refs as string[]) ?? []).flatMap((r) => refs.title(r) ?? []));
  const rows = $derived((args.rows as Row[]) ?? []);
  let details = $state<(TitleDetail | null)[]>([]);

  $effect(() => {
    const list = titles;
    Promise.all(list.map((t) => detailOf(t).catch(() => null))).then((d) => (details = d));
  });

  const runtime = (d: TitleDetail | null) => {
    if (!d) return "…";
    if (d.runtime_minutes) return `${Math.floor(d.runtime_minutes / 60) ? `${Math.floor(d.runtime_minutes / 60)}h ` : ""}${d.runtime_minutes % 60}m`;
    const eps = d.seasons.reduce((n, s) => n + s.episodes.length, 0);
    return eps ? `${d.seasons.length} season${d.seasons.length === 1 ? "" : "s"}, ${eps} eps` : "–";
  };
</script>

<m3e-card variant="filled" class="compare">
  <div slot="content" class="grid" style:--cols={titles.length}>
    <span></span>
    {#each titles as t (t.origin.id)}
      <div class="head">
        <div class="art"><Art title={t} onclick={() => openTitle(t)} /></div>
        <strong class="display">{t.name}</strong>
      </div>
    {/each}

    <span class="label">Year</span>
    {#each titles as t (t.origin.id)}<span>{t.year ?? "–"}</span>{/each}
    <span class="label">Type</span>
    {#each titles as t (t.origin.id)}<span>{kindLabel[t.kind]}</span>{/each}
    <span class="label">Rating</span>
    {#each titles as t (t.origin.id)}<span>{t.rating ? `★ ${t.rating.toFixed(1)}` : "–"}</span>{/each}
    <span class="label">Length</span>
    {#each titles as t, i (t.origin.id)}<span>{runtime(details[i] ?? null)}</span>{/each}
    <span class="label">Genres</span>
    {#each titles as t, i (t.origin.id)}<span>{details[i]?.genres.slice(0, 3).join(", ") || "–"}</span>{/each}
    {#each rows as row (row.label)}
      <span class="label">{row.label}</span>
      {#each titles as t, i (t.origin.id)}<span>{row.values[i] ?? "–"}</span>{/each}
    {/each}

    <span></span>
    {#each titles as t (t.origin.id)}<div class="act"><TitleActions title={t} compact /></div>{/each}
  </div>
  {#if args.verdict}
    <div slot="content" class="verdict"><m3e-icon name="thumb_up" filled></m3e-icon>{args.verdict}</div>
  {/if}
</m3e-card>

<style>
  .compare {
    --m3e-card-shape: 24px;
  }
  .grid {
    display: grid;
    grid-template-columns: 110px repeat(var(--cols), minmax(0, 1fr));
    column-gap: 18px;
    row-gap: 10px;
    padding: 20px;
    font-size: 14px;
    align-items: start;
  }
  .head {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .head .art {
    width: 96px;
  }
  .head strong {
    font-size: 20px;
    line-height: 1.1;
    font-weight: 400;
  }
  .label {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
    padding-top: 2px;
  }
  .act {
    padding-top: 6px;
  }
  .verdict {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0 20px 20px;
    padding: 12px 16px;
    border-radius: 16px;
    background: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
    font-size: 14px;
    --m3e-icon-size: 18px;
  }
</style>
