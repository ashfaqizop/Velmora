<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import type { RefBook } from "../../lib/ani/refs";
  import TitleList from "./blocks/TitleList.svelte";
  import Carousel from "./blocks/Carousel.svelte";
  import Spotlight from "./blocks/Spotlight.svelte";
  import TitleCompare from "./blocks/TitleCompare.svelte";
  import Episodes from "./blocks/Episodes.svelte";
  import SeasonOverview from "./blocks/SeasonOverview.svelte";
  import WatchOrder from "./blocks/WatchOrder.svelte";
  import LinkList from "./blocks/LinkList.svelte";
  import LinkCompare from "./blocks/LinkCompare.svelte";
  import Cast from "./blocks/Cast.svelte";
  import PersonCard from "./blocks/PersonCard.svelte";
  import TitleFacts from "./blocks/TitleFacts.svelte";
  import Media from "./blocks/Media.svelte";
  import Schedule from "./blocks/Schedule.svelte";
  import Progress from "./blocks/Progress.svelte";
  import Taste from "./blocks/Taste.svelte";
  import Choices from "./blocks/Choices.svelte";

  // One generative UI block: each render_* function maps to a component.
  // `live` is true on the latest reply, where choice chips can still be tapped.
  let { name, args, refs, live = false }: { name: string; args: Record<string, unknown>; refs: RefBook | null; live?: boolean } = $props();
</script>

{#if refs}
  <div class="block" in:fly={{ y: 12, duration: 380, easing: cubicOut }}>
    {#if name === "render_title_list"}
      <TitleList {args} {refs} />
    {:else if name === "render_carousel"}
      <Carousel {args} {refs} />
    {:else if name === "render_collection"}
      <Carousel {args} {refs} grid />
    {:else if name === "render_spotlight"}
      <Spotlight {args} {refs} />
    {:else if name === "render_title_compare"}
      <TitleCompare {args} {refs} />
    {:else if name === "render_episodes"}
      <Episodes {args} {refs} />
    {:else if name === "render_season_overview"}
      <SeasonOverview {args} {refs} />
    {:else if name === "render_watch_order"}
      <WatchOrder {args} {refs} />
    {:else if name === "render_download_links"}
      <LinkList {args} {refs} mode="download" />
    {:else if name === "render_torrents"}
      <LinkList {args} {refs} mode="torrent" />
    {:else if name === "render_stream_links"}
      <LinkList {args} {refs} mode="stream" />
    {:else if name === "render_link_compare"}
      <LinkCompare {args} {refs} />
    {:else if name === "render_cast"}
      <Cast {args} {refs} />
    {:else if name === "render_person"}
      <PersonCard {args} {refs} />
    {:else if name === "render_ratings"}
      <TitleFacts {args} {refs} mode="ratings" />
    {:else if name === "render_facts"}
      <TitleFacts {args} {refs} mode="facts" />
    {:else if name === "render_trailer"}
      <Media {args} {refs} mode="trailer" />
    {:else if name === "render_gallery"}
      <Media {args} {refs} mode="gallery" />
    {:else if name === "render_schedule"}
      <Schedule {args} {refs} />
    {:else if name === "render_progress"}
      <Progress {args} {refs} />
    {:else if name === "render_taste_profile"}
      <Taste {args} />
    {:else if name === "render_choices"}
      <Choices {args} {live} />
    {/if}
  </div>
{/if}

<style>
  .block {
    min-width: 0;
  }
</style>
