<script lang="ts">
  import { chats } from "../../../lib/ani/chat.svelte";

  // render_choices: tappable replies. A tap sends the option as the next message,
  // so they only work on the latest reply.
  let { args, live }: { args: Record<string, unknown>; live: boolean } = $props();

  const options = $derived((args.options as string[]) ?? []);
</script>

<div class="choices">
  {#if args.question}<span class="q">{args.question}</span>{/if}
  <div class="row">
    {#each options as o (o)}
      <m3e-suggestion-chip disabled={!live || chats.busy} onclick={() => live && chats.send(o)}>{o}</m3e-suggestion-chip>
    {/each}
  </div>
</div>

<style>
  .choices {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .q {
    font-size: 14px;
    font-weight: 600;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
</style>
