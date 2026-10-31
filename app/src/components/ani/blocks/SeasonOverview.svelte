<script lang="ts">
  import { openTitle } from "../../../lib/ani/actions";
  import { detailOf } from "../../../lib/ani/tools/shape";
  import type { RefBook } from "../../../lib/ani/refs";
  import type { TitleDetail } from "../../../lib/types";

  // render_season_overview: every season with its size and Ani's note.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const t = $derived(refs.title(args.ref as string));
  const notes = $derived(new Map(((args.notes as { season: number; text: string }[]) ?? []).map((n) => [n.season, n.text])));
  let detail = $state<TitleDetail | null>(null);

  $effect(() => {
    if (t) detailOf(t).then((d) => (detail = d)).catch(() => {});
  });

  const seasons = $derived([...(detail?.seasons ?? [])].sort((a, b) => a.number - b.number));
  const most = $derived(Math.max(1, ...seasons.map((s) => s.episodes.length)));
  const years = (eps: { air_date?: string }[]) => {
    const ys = eps.map((e) => e.air_date?.slice(0, 4)).filter(Boolean) as string[];
    if (!ys.length) return "";
    const a = ys[0];
    const b = ys[ys.length - 1];
    return a === b ? a : `${a}–${b}`;
  };
</script>

{#if t}
  <m3e-card variant="filled" class="seasons">
    <div slot="content" class="inner">
      <header>
        <m3e-heading variant="title" size="medium" level={3}>{t.name}</m3e-heading>
        <m3e-button variant="text" size="small" onclick={() => openTitle(t)}>Open</m3e-button>
      </header>
      {#if !detail}
        <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
      {:else}
        {#each seasons as s (s.number)}
          <div class="season">
            <span class="no on-shape"><m3e-shape name="circle"></m3e-shape><span>{s.number}</span></span>
            <div class="body">
              <div class="top">
                <strong>{s.name ?? `Season ${s.number}`}</strong>
                <span>{s.episodes.length} eps{years(s.episodes) ? ` · ${years(s.episodes)}` : ""}</span>
              </div>
              <div class="bar"><span style:width={`${(s.episodes.length / most) * 100}%`}></span></div>
              {#if notes.get(s.number)}<p>{notes.get(s.number)}</p>{/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </m3e-card>
{/if}

<style>
  .seasons {
    --m3e-card-shape: 24px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .ghost {
    height: 140px;
  }
  .season {
    display: grid;
    grid-template-columns: 36px minmax(0, 1fr);
    gap: 14px;
    align-items: start;
  }
  .no {
    --m3e-shape-size: 36px;
    --m3e-shape-container-color: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    font-weight: 700;
  }
  .no span {
    z-index: 1;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .top {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 14px;
  }
  .top span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .bar {
    height: 6px;
    border-radius: 3px;
    background: var(--md-sys-color-surface-container-highest);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    border-radius: 3px;
    background: var(--md-sys-color-primary);
  }
  p {
    margin: 0;
    font-size: 13px;
    line-height: 1.45;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
