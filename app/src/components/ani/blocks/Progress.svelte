<script lang="ts">
  import Art from "./Art.svelte";
  import { openTitle, play } from "../../../lib/ani/actions";
  import type { RefBook } from "../../../lib/ani/refs";
  import { formatLeft, keyOf, library, targetLabel } from "../../../lib/state.svelte";
  import type { Target } from "../../../lib/types";

  // render_progress: Continue Watching cards with progress and Resume.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const entries = $derived.by(() => {
    const wanted = ((args.refs as string[]) ?? []).flatMap((r) => refs.title(r) ?? []).map(keyOf);
    const list = library.continueWatching;
    return (wanted.length ? list.filter((h) => wanted.includes(keyOf(h.title))) : list).slice(0, 8);
  });

  const targetOf = (h: (typeof entries)[number]): Target =>
    h.episode !== undefined ? { type: "episode", season: h.season ?? 1, episode: h.episode } : { type: "movie" };
</script>

<div class="grid">
  {#each entries as h (keyOf(h.title))}
    {@const tg = targetOf(h)}
    <m3e-card variant="filled" class="card">
      <div slot="content" class="inner">
        <Art title={h.title} wide onclick={() => openTitle(h.title)} />
        <div class="bar"><span style:width={`${Math.min(100, (h.position / h.duration) * 100)}%`}></span></div>
        <div class="row">
          <div class="text">
            <strong>{h.title.name}</strong>
            <span>{tg.type === "episode" ? `${targetLabel(tg)} · ` : ""}{formatLeft(h.duration - h.position)}</span>
          </div>
          <m3e-icon-button variant="filled" aria-label="Resume {h.title.name}" onclick={() => play(h.title, tg)}>
            <m3e-icon name="play_arrow" filled></m3e-icon>
          </m3e-icon-button>
        </div>
      </div>
    </m3e-card>
  {:else}
    <p class="none">Nothing in progress right now.</p>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(230px, 1fr));
    gap: 12px;
  }
  .card {
    --m3e-card-shape: 20px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 10px;
  }
  .bar {
    height: 4px;
    border-radius: 2px;
    background: var(--md-sys-color-surface-container-highest);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--md-sys-color-primary);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .text strong {
    font-size: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .text span {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .none {
    margin: 0;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
