<script lang="ts">
  // Explore: charts, new releases, and moods and genres.
  import FeedView from "../../components/music/FeedView.svelte";
  import { nav } from "../../lib/state.svelte";

  let { tab = "charts" }: { tab?: "charts" | "new" | "moods" } = $props();

  const TABS = [
    { id: "charts", label: "Charts", icon: "leaderboard", browseId: "FEmusic_charts" },
    { id: "new", label: "New releases", icon: "auto_awesome", browseId: "FEmusic_new_releases" },
    { id: "moods", label: "Moods and genres", icon: "mood", browseId: "FEmusic_moods_and_genres" },
  ] as const;

  const current = $derived(TABS.find((t) => t.id === tab) ?? TABS[0]);
</script>

<div class="explore">
  <h1 class="display">Explore</h1>
  <m3e-segmented-button class="tabs">
    {#each TABS as t (t.id)}
      <m3e-button-segment checked={t.id === current.id} onclick={() => nav.replace({ name: "music-explore", tab: t.id })}>
        <m3e-icon slot="icon" name={t.icon}></m3e-icon>{t.label}
      </m3e-button-segment>
    {/each}
  </m3e-segmented-button>
  {#key current.id}
    <FeedView browseId={current.browseId} />
  {/key}
</div>

<style>
  .explore {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  h1 {
    margin: 8px 0 0;
    font-size: clamp(40px, 4.6vw, 64px);
    font-weight: 400;
    line-height: 1;
  }
  .tabs {
    align-self: flex-start;
  }
</style>
