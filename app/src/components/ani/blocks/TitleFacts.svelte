<script lang="ts">
  import { openExternal } from "../../../lib/api";
  import { openTitle } from "../../../lib/ani/actions";
  import { detailOf } from "../../../lib/ani/tools/shape";
  import type { RefBook } from "../../../lib/ani/refs";
  import type { TitleDetail } from "../../../lib/types";

  // render_ratings (score tiles) and render_facts (fact sheet), both from
  // the title's detail page data.
  let { args, refs, mode }: { args: Record<string, unknown>; refs: RefBook; mode: "ratings" | "facts" } = $props();

  const t = $derived(refs.title(args.ref as string));
  let detail = $state<TitleDetail | null>(null);
  $effect(() => {
    if (t) detailOf(t).then((d) => (detail = d)).catch(() => {});
  });

  const x = $derived(detail?.extra);
  const score = $derived(detail?.summary.rating ?? t?.rating);
  const imdb = $derived(detail?.summary.ids.imdb ?? t?.ids.imdb);

  const runtime = $derived.by(() => {
    const m = detail?.runtime_minutes;
    if (!m) return undefined;
    return m >= 60 ? `${Math.floor(m / 60)} h ${m % 60} min` : `${m} min`;
  });

  const facts = $derived.by(() => {
    if (!detail || !x) return [] as [string, string][];
    const eps = detail.seasons.reduce((n, s) => n + s.episodes.length, 0);
    const rows: [string, string | undefined][] = [
      ["Runtime", runtime],
      ["Released", x.release_date],
      ["Status", x.status],
      ["Seasons", detail.seasons.length ? `${detail.seasons.length} (${eps} episodes)` : undefined],
      ["Genres", detail.genres.slice(0, 4).join(", ") || undefined],
      ["Studio", x.studios.slice(0, 2).join(", ") || undefined],
      ["Language", x.languages.slice(0, 3).join(", ") || x.original_language],
      ["Country", x.countries.slice(0, 2).join(", ") || undefined],
      ["Rated", x.certification],
      ["Based on", x.source_material],
    ];
    const extra = ((args.extra as { label: string; value: string }[]) ?? []).map((e) => [e.label, e.value] as [string, string]);
    return [...rows.filter((r): r is [string, string] => !!r[1]), ...extra];
  });
</script>

{#if t}
  <m3e-card variant="filled" class="facts">
    <div slot="content" class="inner">
      <header>
        <button class="name display" onclick={() => openTitle(t)}>{t.name}</button>
        {#if t.year}<span>{t.year}</span>{/if}
      </header>
      {#if mode === "ratings"}
        <div class="tiles">
          <div class="tile main">
            <m3e-icon name="star" filled></m3e-icon>
            <strong>{score ? score.toFixed(1) : "–"}</strong>
            <span>{x?.rating_source ?? "Score"}{x?.vote_count ? ` · ${x.vote_count.toLocaleString()} votes` : ""}</span>
          </div>
          <div class="tile">
            <m3e-icon name="shield_person"></m3e-icon>
            <strong>{x?.certification ?? "–"}</strong>
            <span>Age rating</span>
          </div>
          {#if x?.awards}
            <div class="tile">
              <m3e-icon name="trophy"></m3e-icon>
              <strong class="small">{x.awards}</strong>
              <span>Awards</span>
            </div>
          {/if}
        </div>
        {#if imdb}
          <m3e-button variant="text" size="small" onclick={() => openExternal(`https://www.imdb.com/title/${imdb}/`)}>
            <m3e-icon slot="icon" name="open_in_new"></m3e-icon>See it on IMDb
          </m3e-button>
        {/if}
      {:else if !detail}
        <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
      {:else}
        <dl>
          {#each facts as [label, value] (label)}
            <dt>{label}</dt>
            <dd>{value}</dd>
          {/each}
        </dl>
      {/if}
    </div>
  </m3e-card>
{/if}

<style>
  .facts {
    --m3e-card-shape: 24px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 14px;
    padding: 18px 20px;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }
  header span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .name {
    all: unset;
    cursor: pointer;
    font-size: 24px;
  }
  .name:hover {
    color: var(--md-sys-color-primary);
  }
  .tiles {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    width: 100%;
  }
  .tile {
    flex: 1;
    min-width: 140px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 14px 16px;
    border-radius: 18px;
    background: var(--md-sys-color-surface-container-high);
    --m3e-icon-size: 20px;
  }
  .tile.main {
    background: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
  }
  .tile strong {
    font-size: 30px;
    line-height: 1.1;
  }
  .tile strong.small {
    font-size: 14px;
  }
  .tile span {
    font-size: 12px;
    opacity: 0.8;
  }
  .ghost {
    width: 100%;
    height: 160px;
  }
  dl {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    gap: 8px 24px;
    margin: 0;
    font-size: 14px;
    width: 100%;
  }
  dt {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
    padding-top: 2px;
  }
  dd {
    margin: 0;
  }
</style>
