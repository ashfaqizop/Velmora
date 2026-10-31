<script lang="ts">
  import { openTitle, pickDownload, play, toggleList } from "../../../lib/ani/actions";
  import { library } from "../../../lib/state.svelte";
  import type { TitleSummary } from "../../../lib/types";

  // Play (movies) or Open (series), My List and Download, for a title card.
  let { title, compact = false }: { title: TitleSummary; compact?: boolean } = $props();

  const saved = $derived(library.has(title));
  const movie = $derived(title.kind === "movie");
</script>

<div class="actions">
  {#if movie}
    <m3e-button variant="filled" size={compact ? "extra-small" : "small"} onclick={() => play(title)}>
      <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>Play
    </m3e-button>
  {:else}
    <m3e-button variant="filled" size={compact ? "extra-small" : "small"} onclick={() => openTitle(title)}>
      <m3e-icon slot="icon" name="list"></m3e-icon>Episodes
    </m3e-button>
  {/if}
  <m3e-icon-button
    variant="tonal"
    size={compact ? "extra-small" : "small"}
    toggle
    selected={saved}
    aria-label={saved ? "Remove from My List" : "Add to My List"}
    onclick={() => toggleList(title)}
  >
    <m3e-icon name="bookmark" filled={saved}></m3e-icon>
  </m3e-icon-button>
  {#if movie}
    <m3e-icon-button variant="tonal" size={compact ? "extra-small" : "small"} aria-label="Download" onclick={() => pickDownload(title)}>
      <m3e-icon name="download"></m3e-icon>
    </m3e-icon-button>
  {/if}
  {#if movie && !compact}
    <m3e-button variant="text" size="small" onclick={() => openTitle(title)}>Details</m3e-button>
  {/if}
</div>

<style>
  .actions {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
  }
</style>
