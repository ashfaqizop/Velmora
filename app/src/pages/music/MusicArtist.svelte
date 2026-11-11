<script lang="ts">
  import { onMount } from "svelte";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../../lib/api";
  import MusicBanner from "../../components/music/MusicBanner.svelte";
  import ShelfView from "../../components/music/ShelfView.svelte";
  import TrackRow from "../../components/music/TrackRow.svelte";
  import AdSlot from "../../components/AdSlot.svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import { playable } from "../../lib/music/ui.svelte";
  import type { MusicArtist } from "../../lib/types";

  let { id }: { id: string } = $props();

  let artist = $state<MusicArtist | null>(null);
  let error = $state<string | null>(null);
  let subscribed = $state(false);

  onMount(async () => {
    try {
      artist = await api.musicArtist(id);
      subscribed = !!artist.subscribed;
    } catch (e) {
      error = typeof e === "string" ? e : "This artist couldn't be loaded.";
    }
  });

  // Top songs as rows; everything else as shelves.
  const top = $derived(artist?.shelves.find((s) => s.display === "list" && s.items.some(playable)));
  const rest = $derived(artist?.shelves.filter((s) => s !== top) ?? []);

  const meta = $derived(artist ? [artist.monthlyAudience, artist.subscribers ? `${artist.subscribers} subscribers` : null].filter(Boolean).join(" · ") : "");

  async function toggleSubscribe() {
    if (!artist?.channelId) return;
    const on = !subscribed;
    subscribed = on;
    try {
      await api.musicSubscribe(artist.channelId, on);
      M3eSnackbar.open(on ? `Subscribed to ${artist.name}` : `Unsubscribed from ${artist.name}`);
    } catch (e) {
      subscribed = !on;
      M3eSnackbar.open(String(e));
    }
  }
</script>

{#if artist}
  <div class="page">
    <MusicBanner title={artist.name} thumbs={artist.thumbnails} kicker="Artist" {meta} description={artist.description} round>
      {#snippet actions()}
        {#if artist!.shuffle}
          <m3e-button variant="filled" onclick={() => musicPlayer.playTarget(artist!.shuffle!, artist!.name)}>
            <m3e-icon slot="icon" name="shuffle"></m3e-icon>Shuffle
          </m3e-button>
        {:else if top}
          <m3e-button variant="filled" onclick={() => musicPlayer.playList(top!.items, 0, artist!.name)}>
            <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>Play
          </m3e-button>
        {/if}
        {#if artist!.radio}
          <m3e-button variant="tonal" onclick={() => musicPlayer.playTarget(artist!.radio!, `${artist!.name} radio`)}>
            <m3e-icon slot="icon" name="radio"></m3e-icon>Radio
          </m3e-button>
        {/if}
        {#if artist!.channelId}
          <m3e-button variant={subscribed ? "outlined" : "tonal"} onclick={toggleSubscribe}>
            <m3e-icon slot="icon" name={subscribed ? "check" : "add"}></m3e-icon>{subscribed ? "Subscribed" : "Subscribe"}
          </m3e-button>
        {/if}
      {/snippet}
    </MusicBanner>

    {#if top}
      <section>
        <h2 class="display">{top.title}</h2>
        {#each top.items.slice(0, 5) as t, i (t.id)}
          <TrackRow item={t} index={i} list={top.items} source={artist.name} numbered />
        {/each}
      </section>
    {/if}
    {#each rest as shelf, i (shelf.id)}
      <ShelfView {shelf} />
      {#if i === 1}<AdSlot id="music" />{/if}
    {/each}
  </div>
{:else if error}
  <p class="note">{error}</p>
{:else}
  <m3e-loading-indicator class="spin"></m3e-loading-indicator>
{/if}

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 30px;
    padding-bottom: 120px;
  }
  h2 {
    margin: 0 0 10px;
    font-size: 26px;
    font-weight: 400;
  }
  .spin {
    display: block;
    margin: 80px auto;
  }
  .note {
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
