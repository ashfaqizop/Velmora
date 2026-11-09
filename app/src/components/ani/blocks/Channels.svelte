<script lang="ts">
  import ChannelCard from "../../ChannelCard.svelte";
  import { live } from "../../../lib/live.svelte";
  import type { RefBook } from "../../../lib/ani/refs";

  // render_channels: live TV channels, each opens in the player.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const list = $derived(((args.refs as string[]) ?? []).flatMap((r) => refs.channel(r) ?? []));
</script>

<div class="block">
  {#if args.heading}<m3e-heading variant="title" size="medium" level={3}>{args.heading as string}</m3e-heading>{/if}
  <div class="grid">
    {#each list as c (c.id)}
      <ChannelCard channel={c} onopen={() => live.play(c)} />
    {/each}
  </div>
  <p class="fine">From a public playlist. Velmora hosts none of these, and some may not play.</p>
</div>

<style>
  .block {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 8px;
  }
  .fine {
    margin: 0;
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
    opacity: 0.8;
  }
</style>
