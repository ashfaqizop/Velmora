<script lang="ts">
  import TrackRow from "../../music/TrackRow.svelte";
  import { musicPlayer } from "../../../lib/music/player.svelte";
  import type { RefBook } from "../../../lib/ani/refs";

  // render_songs: playable song rows, with Play all. Nothing plays until the user presses play.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const list = $derived(((args.refs as string[]) ?? []).flatMap((r) => refs.music(r) ?? []));
  const heading = $derived((args.heading as string) || "Songs");
</script>

<div class="block">
  <div class="head">
    <m3e-heading variant="title" size="medium" level={3}>{heading}</m3e-heading>
    {#if list.length > 1}
      <m3e-button variant="tonal" size="small" onclick={() => musicPlayer.playList(list, 0, `Ani: ${heading}`)}>
        <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>Play all
      </m3e-button>
    {/if}
  </div>
  <div class="rows">
    {#each list as s, i (s.id + i)}
      <TrackRow item={s} index={i} {list} source={`Ani: ${heading}`} showAlbum={false} />
    {/each}
  </div>
</div>

<style>
  .block {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .rows {
    display: flex;
    flex-direction: column;
  }
</style>
