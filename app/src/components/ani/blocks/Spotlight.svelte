<script lang="ts">
  import TitleActions from "./TitleActions.svelte";
  import { openTitle } from "../../../lib/ani/actions";
  import type { RefBook } from "../../../lib/ani/refs";
  import { meta } from "../../../lib/state.svelte";

  // render_spotlight: one pick on its backdrop, like the Home hero.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const t = $derived(refs.title(args.ref as string));
  const art = $derived(t?.backdrop ?? t?.poster);
</script>

{#if t}
  <div class="spot" style:--art={art ? `url("${art}")` : "none"}>
    <div class="inner">
      <div class="copy">
        {#if args.badge}
          <span class="overline"><m3e-icon name="wand_stars" filled></m3e-icon>{args.badge}</span>
        {/if}
        <button class="name display" onclick={() => openTitle(t)}>{t.name}</button>
        <span class="meta">
          {meta(t)}{#if t.rating}<span class="dot">•</span><m3e-icon name="star" filled></m3e-icon>{t.rating.toFixed(1)}{/if}
        </span>
        {#if args.why}<p>{args.why}</p>{/if}
        <TitleActions title={t} />
      </div>
    </div>
  </div>
{/if}

<style>
  /* Full-bleed art; scrims from the left and the bottom keep the copy readable on any image. */
  .spot {
    border-radius: 28px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container);
  }
  .inner {
    position: relative;
    min-height: 340px;
    display: flex;
    align-items: flex-end;
    padding: 32px;
    box-sizing: border-box;
    background:
      linear-gradient(90deg, rgb(10 6 20 / 88%) 0%, rgb(10 6 20 / 62%) 38%, rgb(10 6 20 / 0%) 72%),
      linear-gradient(0deg, rgb(10 6 20 / 85%) 0%, rgb(10 6 20 / 0%) 55%),
      var(--art) center 25% / cover no-repeat;
    text-shadow: 0 1px 12px rgb(0 0 0 / 45%);
    color: #fff;
    /* Buttons and text read on dark art whatever the theme. */
    --md-sys-color-on-surface: #fff;
  }
  .copy {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    max-width: 560px;
  }
  .overline {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary-fixed-dim, #d0bcff);
    --m3e-icon-size: 16px;
  }
  .name {
    all: unset;
    cursor: pointer;
    font-size: 42px;
    line-height: 1;
  }
  .name:focus-visible {
    outline: 2px solid #fff;
  }
  .meta {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: rgb(255 255 255 / 80%);
    --m3e-icon-size: 15px;
  }
  .dot {
    opacity: 0.6;
  }
  p {
    margin: 0;
    font-size: 15px;
    line-height: 1.55;
    color: rgb(255 255 255 / 90%);
  }
</style>
