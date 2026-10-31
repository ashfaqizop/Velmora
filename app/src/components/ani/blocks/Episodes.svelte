<script lang="ts">
  import { play } from "../../../lib/ani/actions";
  import { detailOf } from "../../../lib/ani/tools/shape";
  import type { RefBook } from "../../../lib/ani/refs";
  import type { TitleDetail } from "../../../lib/types";

  // render_episodes: one season's episodes with stills and Play buttons.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const t = $derived(refs.title(args.ref as string));
  const seasonNo = $derived((args.season as number) ?? 1);
  const highlight = $derived(new Set((args.highlight as number[]) ?? []));
  let detail = $state<TitleDetail | null>(null);
  let failed = $state(false);

  $effect(() => {
    if (!t) return;
    detailOf(t)
      .then((d) => (detail = d))
      .catch(() => (failed = true));
  });

  const episodes = $derived(
    [...(detail?.seasons.find((s) => s.number === seasonNo)?.episodes ?? [])].sort((a, b) => a.number - b.number),
  );
</script>

{#if t}
  <m3e-card variant="filled" class="eps">
    <div slot="content" class="inner">
      <header>
        <m3e-heading variant="title" size="medium" level={3}>{t.name} · Season {seasonNo}</m3e-heading>
        {#if episodes.length}<span>{episodes.length} episodes</span>{/if}
      </header>
      {#if !detail && !failed}
        {#each Array(3) as _, i (i)}<m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>{/each}
      {:else if !episodes.length}
        <p class="empty">No episode list for this season.</p>
      {:else}
        <ol>
          {#each episodes as e (e.number)}
            <li class:hl={highlight.has(e.number)}>
              <div class="still">
                {#if e.thumbnail}<img src={e.thumbnail} alt="" loading="lazy" />{:else}<span>{e.number}</span>{/if}
              </div>
              <div class="text">
                <strong>{e.number}. {e.title ?? `Episode ${e.number}`}</strong>
                {#if e.air_date}<span class="date">{e.air_date}</span>{/if}
                {#if e.overview}<p>{e.overview}</p>{/if}
              </div>
              <m3e-icon-button
                variant="tonal"
                aria-label="Play episode {e.number}"
                onclick={() => play(t, { type: "episode", season: seasonNo, episode: e.number })}
              >
                <m3e-icon name="play_arrow" filled></m3e-icon>
              </m3e-icon-button>
            </li>
          {/each}
        </ol>
      {/if}
    </div>
  </m3e-card>
{/if}

<style>
  .eps {
    --m3e-card-shape: 24px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 18px;
  }
  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }
  header span,
  .empty {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    margin: 0;
  }
  .ghost {
    height: 72px;
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 520px;
    overflow-y: auto;
  }
  li {
    display: grid;
    grid-template-columns: 128px minmax(0, 1fr) auto;
    gap: 14px;
    align-items: center;
    padding: 8px;
    border-radius: 16px;
  }
  li:hover {
    background: var(--md-sys-color-surface-container-high);
  }
  li.hl {
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .still {
    aspect-ratio: 16 / 9;
    border-radius: 12px;
    overflow: hidden;
    display: grid;
    place-items: center;
    background: var(--md-sys-color-surface-container-highest);
    font-weight: 700;
  }
  .still img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .text strong {
    font-size: 14px;
  }
  .date {
    font-size: 12px;
    opacity: 0.75;
  }
  .text p {
    margin: 2px 0 0;
    font-size: 13px;
    line-height: 1.45;
    opacity: 0.85;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
