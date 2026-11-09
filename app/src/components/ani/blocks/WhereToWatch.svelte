<script lang="ts">
  import MatchCard from "../../MatchCard.svelte";
  import WatchList from "../../WatchList.svelte";
  import type { RefBook } from "../../../lib/ani/refs";

  // render_where_to_watch: the match, then its official broadcasters.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const match = $derived(refs.match(args.ref as string));
</script>

{#if match}
  <m3e-card variant="filled" class="card">
    <div slot="content" class="inner">
      <MatchCard {match} />
      <WatchList {match} />
    </div>
  </m3e-card>
{/if}

<style>
  .card {
    --m3e-card-shape: 24px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 18px;
  }
</style>
