<script lang="ts">
  import MusicCard from "../../music/MusicCard.svelte";
  import type { RefBook } from "../../../lib/ani/refs";

  // render_music_cards: albums, artists or playlists; tapping one opens its Music page.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const list = $derived(((args.refs as string[]) ?? []).flatMap((r) => refs.music(r) ?? []));
</script>

<div class="block">
  {#if args.heading}<m3e-heading variant="title" size="medium" level={3}>{args.heading as string}</m3e-heading>{/if}
  <div class="row">
    {#each list as item, i (item.id + i)}
      <MusicCard {item} size={150} />
    {/each}
  </div>
</div>

<style>
  .block {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .row {
    display: flex;
    gap: 16px;
    overflow-x: auto;
    scrollbar-width: thin;
    padding-bottom: 6px;
  }
</style>
