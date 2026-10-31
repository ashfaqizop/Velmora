<script lang="ts">
  import Poster from "../../Poster.svelte";
  import type { RefBook } from "../../../lib/ani/refs";

  // render_person: photo, role, Ani's short bio and what they're known for.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const e = $derived(refs.get(args.ref as string));
  const p = $derived(e?.type === "person" ? e.person : null);
  const known = $derived(((args.known_for as string[]) ?? []).flatMap((r) => refs.title(r) ?? []));
</script>

{#if p}
  <m3e-card variant="filled" class="person">
    <div slot="content" class="inner">
      <div class="top">
        <div class="photo">
          {#if p.photo}<img src={p.photo.replace("/w185/", "/h632/")} alt="" />{:else}<m3e-icon name="person"></m3e-icon>{/if}
        </div>
        <div class="text">
          <strong class="display">{p.name}</strong>
          {#if p.role}<span class="role">{p.role}</span>{/if}
          {#if args.bio}<p>{args.bio}</p>{/if}
        </div>
      </div>
      {#if known.length}
        <span class="label">Known for</span>
        <div class="row">
          {#each known as t (t.origin.provider + t.origin.id)}<Poster title={t} />{/each}
        </div>
      {/if}
    </div>
  </m3e-card>
{/if}

<style>
  .person {
    --m3e-card-shape: 24px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 20px;
  }
  .top {
    display: grid;
    grid-template-columns: 120px minmax(0, 1fr);
    gap: 20px;
    align-items: start;
  }
  .photo {
    aspect-ratio: 2 / 3;
    border-radius: 18px;
    overflow: hidden;
    display: grid;
    place-items: center;
    background: var(--md-sys-color-secondary-container);
    --m3e-icon-size: 40px;
  }
  .photo img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .text strong {
    font-size: 32px;
    font-weight: 400;
    line-height: 1.05;
  }
  .role {
    font-size: 13px;
    color: var(--md-sys-color-primary);
    font-weight: 700;
  }
  p {
    margin: 4px 0 0;
    font-size: 14.5px;
    line-height: 1.55;
  }
  .label {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
  }
  .row {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 128px;
    gap: 12px;
    overflow-x: auto;
    scrollbar-width: none;
  }
</style>
