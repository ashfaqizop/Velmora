<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import Art from "./Art.svelte";
  import TitleActions from "./TitleActions.svelte";
  import { openTitle } from "../../../lib/ani/actions";
  import type { RefBook } from "../../../lib/ani/refs";
  import { meta } from "../../../lib/state.svelte";

  // render_title_list: poster on the left, then name, badge, why it fits,
  // bullet facts and actions. Rows are split by dividers.
  type Item = { ref: string; badge?: string; why?: string; bullets?: string[] };
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const heading = $derived(args.heading as string | undefined);
  const rows = $derived(
    ((args.items as Item[]) ?? []).flatMap((i) => {
      const t = refs.title(i.ref);
      return t ? [{ ...i, title: t }] : [];
    }),
  );
</script>

<div class="block">
  {#if heading}<m3e-heading variant="title" size="medium" level={3}>{heading}</m3e-heading>{/if}
  <m3e-card variant="filled" class="card">
    <div slot="content" class="rows">
      {#each rows as r, i (r.ref)}
        {#if i > 0}<m3e-divider></m3e-divider>{/if}
        <article class="row" in:fly={{ y: 10, duration: 340, delay: 60 * i, easing: cubicOut }}>
          <div class="poster"><Art title={r.title} onclick={() => openTitle(r.title)} /></div>
          <div class="text">
            <header>
              <button class="name display" onclick={() => openTitle(r.title)}>{r.title.name}</button>
              <span class="meta">
                {meta(r.title)}
                {#if r.title.rating}<span class="star"><m3e-icon name="star" filled></m3e-icon>{r.title.rating.toFixed(1)}</span>{/if}
              </span>
            </header>
            {#if r.badge}<m3e-chip variant="elevated" class="badge">{r.badge}</m3e-chip>{/if}
            {#if r.why}<p>{r.why}</p>{/if}
            {#if r.bullets?.length}
              <ul>
                {#each r.bullets as b, j (j)}<li>{b}</li>{/each}
              </ul>
            {/if}
            <TitleActions title={r.title} />
          </div>
        </article>
      {/each}
    </div>
  </m3e-card>
</div>

<style>
  .block {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .card {
    --m3e-card-shape: 24px;
  }
  .rows {
    display: flex;
    flex-direction: column;
    padding: 6px 4px;
  }
  .row {
    display: grid;
    grid-template-columns: 112px minmax(0, 1fr);
    gap: 18px;
    padding: 14px 8px;
  }
  .poster {
    align-self: start;
  }
  .text {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    min-width: 0;
  }
  header {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .name {
    all: unset;
    cursor: pointer;
    font-size: 24px;
    line-height: 1.1;
  }
  .name:hover {
    color: var(--md-sys-color-primary);
  }
  .name:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
    border-radius: 4px;
  }
  .meta {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .star {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    --m3e-icon-size: 15px;
  }
  .star m3e-icon {
    color: var(--brand-coral, #ff5a5a);
  }
  .badge {
    --m3e-chip-container-height: 24px;
    --m3e-chip-label-text-font-size: 12px;
    --m3e-chip-label-text-font-weight: 700;
    --m3e-chip-padding-start: 10px;
    --m3e-chip-padding-end: 10px;
    --m3e-elevated-chip-elevation: 0;
    --m3e-elevated-chip-hover-elevation: 0;
    --m3e-elevated-chip-container-color: var(--md-sys-color-tertiary-container);
    --m3e-chip-label-text-color: var(--md-sys-color-on-tertiary-container);
  }
  p {
    margin: 0;
    font-size: 14.5px;
    line-height: 1.55;
  }
  ul {
    margin: 0;
    padding-left: 20px;
    font-size: 14px;
    line-height: 1.6;
    color: var(--md-sys-color-on-surface-variant);
  }
  li::marker {
    color: var(--md-sys-color-primary);
  }
</style>
