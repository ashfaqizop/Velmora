<script lang="ts">
  import Hero from "../components/Hero.svelte";
  import Rail from "../components/Rail.svelte";
  import Poster from "../components/Poster.svelte";
  import ContinueCard from "../components/ContinueCard.svelte";
  import { feed, keyOf, library, nav } from "../lib/state.svelte";

  const ICONS: Record<string, string> = {
    trending_movies: "local_fire_department",
    trending_series: "tv",
    trending_anime: "animation",
    airing_today: "event_upcoming",
    hdhub_latest: "new_releases",
    kdrama: "favorite",
    hdhub_bollywood: "theaters",
    hdhub_south: "translate",
  };
</script>

<div class="home">
  <Hero items={feed.spotlight} loading={feed.loading} />

  {#if library.continueWatching.length}
    <Rail title="Continue watching" icon="resume">
      {#each library.continueWatching as entry (keyOf(entry.title))}
        <ContinueCard {entry} />
      {/each}
    </Rail>
  {/if}

  {#if feed.loading}
    <Rail title="Trending movies" icon="local_fire_department" loading />
    <Rail title="Trending series" icon="tv" loading />
  {:else if feed.error || !feed.rails.length}
    <div class="empty">
      <m3e-icon name="cloud_off"></m3e-icon>
      <m3e-heading variant="title" size="large">Nothing to show yet</m3e-heading>
      <p>{feed.error ?? "None of the sources answered. Check your connection and try again."}</p>
      <m3e-button variant="tonal" onclick={() => feed.load()}>
        <m3e-icon slot="icon" name="refresh"></m3e-icon>Try again
      </m3e-button>
    </div>
  {:else}
    {#each feed.rails as rail (rail.id)}
      <Rail title={rail.title} icon={ICONS[rail.id] ?? "movie"} onseeall={() => nav.go({ name: "collection", id: rail.id })}>
        {#each rail.items as t (keyOf(t))}
          <Poster title={t} width={160} />
        {/each}
      </Rail>
    {/each}
  {/if}
</div>

<style>
  .home {
    display: flex;
    flex-direction: column;
    gap: 32px;
    padding-bottom: 32px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 48px 0;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 48px;
  }
  .empty p {
    margin: 0;
    max-width: 420px;
  }
</style>
