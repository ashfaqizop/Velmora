<script lang="ts">
  import { detailOf } from "../../../lib/ani/tools/shape";
  import type { RefBook } from "../../../lib/ani/refs";
  import type { TitleDetail } from "../../../lib/types";

  // render_cast: a strip of cast photos, then the key crew.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const t = $derived(refs.title(args.ref as string));
  let detail = $state<TitleDetail | null>(null);
  $effect(() => {
    if (t) detailOf(t).then((d) => (detail = d)).catch(() => {});
  });
  const cast = $derived(detail?.extra.cast.slice(0, 16) ?? []);
  const crew = $derived(detail?.extra.crew.slice(0, 4) ?? []);
  const initials = (n: string) =>
    n
      .split(/\s+/)
      .slice(0, 2)
      .map((w) => w[0])
      .join("");
</script>

{#if t}
  <div class="block">
    <m3e-heading variant="title" size="medium" level={3}>{(args.heading as string) ?? `Cast of ${t.name}`}</m3e-heading>
    {#if !detail}
      <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
    {:else}
      <div class="strip">
        {#each cast as p, i (p.name + i)}
          <div class="person">
            <div class="photo">
              {#if p.photo}<img src={p.photo} alt="" loading="lazy" />{:else}<span>{initials(p.name)}</span>{/if}
            </div>
            <strong>{p.name}</strong>
            {#if p.role}<span>{p.role}</span>{/if}
          </div>
        {/each}
      </div>
      {#if crew.length}
        <div class="crew">
          {#each crew as p, i (p.name + i)}<span><strong>{p.role}</strong> {p.name}</span>{/each}
        </div>
      {/if}
    {/if}
  </div>
{/if}

<style>
  .block {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }
  .ghost {
    height: 150px;
  }
  .strip {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 104px;
    gap: 14px;
    overflow-x: auto;
    scrollbar-width: thin;
    padding-bottom: 6px;
  }
  .person {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 4px;
    font-size: 12.5px;
  }
  .photo {
    width: 88px;
    height: 88px;
    border-radius: 50%;
    overflow: hidden;
    display: grid;
    place-items: center;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    font-weight: 700;
    margin-bottom: 4px;
  }
  .photo img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .person span {
    color: var(--md-sys-color-on-surface-variant);
    font-size: 12px;
  }
  .crew {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 18px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .crew strong {
    color: var(--md-sys-color-on-surface);
    font-weight: 600;
    margin-right: 4px;
  }
</style>
