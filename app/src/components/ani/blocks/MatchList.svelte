<script lang="ts">
  import MatchCard from "../../MatchCard.svelte";
  import type { RefBook } from "../../../lib/ani/refs";

  // render_matches: scores, fixtures or results as match cards.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const list = $derived(((args.refs as string[]) ?? []).flatMap((r) => refs.match(r) ?? []));
</script>

<div class="block">
  {#if args.heading}<m3e-heading variant="title" size="medium" level={3}>{args.heading as string}</m3e-heading>{/if}
  <div class="grid">
    {#each list as m (m.id)}
      <MatchCard match={m} />
    {/each}
  </div>
  <p class="fine">Scores can run a few minutes behind.</p>
</div>

<style>
  .block {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 10px;
  }
  .fine {
    margin: 0;
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
    opacity: 0.8;
  }
</style>
