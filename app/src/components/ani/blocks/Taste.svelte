<script lang="ts">
  import Art from "./Art.svelte";
  import { openTitle } from "../../../lib/ani/actions";
  import { personalize } from "../../../lib/ani/personalize.svelte";
  import { kindLabel } from "../../../lib/state.svelte";
  import type { TitleSummary } from "../../../lib/types";

  // render_taste_profile: what Personalize has learned, drawn as bars.
  let { args }: { args: Record<string, unknown> } = $props();

  const stats = $derived(personalize.stats());
  const top = $derived(Math.max(1, ...stats.genres.map(([, n]) => n)));
  const recent = $derived(personalize.recentTitles(["finish", "stream", "download"], 6));
</script>

<m3e-card variant="filled" class="taste">
  <div slot="content" class="inner">
    <header>
      <span class="on-shape mark"><m3e-shape name="8-leaf-clover"></m3e-shape><m3e-icon name="person_heart" filled></m3e-icon></span>
      <div>
        <m3e-heading variant="title" size="medium" level={3}>Your taste</m3e-heading>
        {#if args.summary}<p>{args.summary}</p>{/if}
      </div>
    </header>

    {#if stats.genres.length}
      <div class="bars">
        {#each stats.genres as [g, n], i (g)}
          <span class="label">{g}</span>
          <div class="bar"><span style:width={`${(n / top) * 100}%`} style:animation-delay={`${i * 60}ms`}></span></div>
        {/each}
      </div>
    {:else}
      <p>Not enough yet. Watch a few things and this fills in.</p>
    {/if}

    {#if stats.kinds.length || stats.decades.length}
      <div class="chips">
        {#each stats.kinds as [k] (k)}<span class="chip">{kindLabel[k]}</span>{/each}
        {#each stats.decades as [d] (d)}<span class="chip soft">{d}</span>{/each}
      </div>
    {/if}

    {#if recent.length}
      <span class="small">Lately</span>
      <div class="recent">
        {#each recent as t (t.origin.provider + t.origin.id)}
          {@const full = { ...t, ids: t.ids ?? {} } as TitleSummary}
          <div class="art"><Art title={full} onclick={() => openTitle(full)} /></div>
        {/each}
      </div>
    {/if}
  </div>
</m3e-card>

<style>
  .taste {
    --m3e-card-shape: 24px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 20px;
  }
  header {
    display: flex;
    gap: 14px;
    align-items: center;
  }
  .mark {
    --m3e-shape-size: 48px;
    --m3e-shape-container-color: var(--md-sys-color-tertiary-container);
    color: var(--md-sys-color-on-tertiary-container);
    --m3e-icon-size: 22px;
    flex: none;
  }
  .mark m3e-icon {
    z-index: 1;
  }
  p {
    margin: 2px 0 0;
    font-size: 14px;
    line-height: 1.5;
    color: var(--md-sys-color-on-surface-variant);
  }
  .bars {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    gap: 8px 14px;
    align-items: center;
  }
  .label {
    font-size: 13px;
  }
  .bar {
    height: 10px;
    border-radius: 5px;
    background: var(--md-sys-color-surface-container-highest);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    border-radius: 5px;
    background: linear-gradient(90deg, var(--md-sys-color-primary), var(--brand-coral, #ff5a5a));
    transform-origin: left;
    animation: grow 700ms var(--md-sys-motion-easing-emphasized-decelerate, ease-out) both;
  }
  @keyframes grow {
    from {
      transform: scaleX(0);
    }
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    font-size: 12px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 999px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .chip.soft {
    background: var(--md-sys-color-surface-container-highest);
    color: var(--md-sys-color-on-surface-variant);
  }
  .small {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
  }
  .recent {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: 10px;
    max-width: 520px;
  }
</style>
