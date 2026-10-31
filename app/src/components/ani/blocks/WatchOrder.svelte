<script lang="ts">
  import Art from "./Art.svelte";
  import { openTitle } from "../../../lib/ani/actions";
  import { personalize } from "../../../lib/ani/personalize.svelte";
  import type { RefBook } from "../../../lib/ani/refs";

  // render_watch_order: a numbered timeline through a franchise.
  type Item = { ref: string; label?: string; note?: string; optional?: boolean };
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const items = $derived(((args.items as Item[]) ?? []).flatMap((i) => (refs.title(i.ref) ? [{ ...i, title: refs.title(i.ref)! }] : [])));
</script>

<m3e-card variant="filled" class="order">
  <div slot="content" class="inner">
    <header>
      <m3e-heading variant="title" size="medium" level={3}>{(args.heading as string) ?? "Watch order"}</m3e-heading>
      <span class="mode">{args.order === "story" ? "Story order" : "Release order"}</span>
    </header>
    <ol>
      {#each items as it, i (it.ref + i)}
        {@const seen = personalize.finished(it.title)}
        <li class:optional={it.optional}>
          <span class="step on-shape" class:seen>
            <m3e-shape name={seen ? "4-sided-cookie" : "circle"}></m3e-shape>
            {#if seen}<m3e-icon name="check"></m3e-icon>{:else}<span>{i + 1}</span>{/if}
          </span>
          <div class="art"><Art title={it.title} onclick={() => openTitle(it.title)} /></div>
          <div class="text">
            <div class="top">
              <button class="name" onclick={() => openTitle(it.title)}>{it.title.name}</button>
              {#if it.label}<span class="tag">{it.label}</span>{/if}
              {#if it.optional}<span class="tag soft">Optional</span>{/if}
            </div>
            <span class="year">{it.title.year ?? ""}</span>
            {#if it.note}<p>{it.note}</p>{/if}
          </div>
        </li>
      {/each}
    </ol>
  </div>
</m3e-card>

<style>
  .order {
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
    align-items: baseline;
    justify-content: space-between;
  }
  .mode {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    position: relative;
  }
  li {
    position: relative;
    display: grid;
    grid-template-columns: 36px 52px minmax(0, 1fr);
    gap: 14px;
    align-items: start;
    padding-bottom: 16px;
  }
  /* The line joining each step to the next. */
  li:not(:last-child)::before {
    content: "";
    position: absolute;
    left: 17px;
    top: 38px;
    bottom: 2px;
    width: 2px;
    background: var(--md-sys-color-outline-variant);
  }
  li.optional {
    opacity: 0.75;
  }
  .step {
    --m3e-shape-size: 36px;
    --m3e-shape-container-color: var(--md-sys-color-surface-container-highest);
    font-weight: 700;
    font-size: 14px;
    --m3e-icon-size: 18px;
  }
  .step.seen {
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
  }
  .step > span,
  .step > m3e-icon {
    z-index: 1;
  }
  .art {
    width: 52px;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    padding-top: 4px;
  }
  .top {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
  }
  .name {
    all: unset;
    cursor: pointer;
    font-weight: 700;
    font-size: 15px;
  }
  .name:hover {
    color: var(--md-sys-color-primary);
  }
  .tag {
    font-size: 11px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--md-sys-color-tertiary-container);
    color: var(--md-sys-color-on-tertiary-container);
  }
  .tag.soft {
    background: var(--md-sys-color-surface-container-highest);
    color: var(--md-sys-color-on-surface-variant);
  }
  .year {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  p {
    margin: 2px 0 0;
    font-size: 13px;
    line-height: 1.45;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
