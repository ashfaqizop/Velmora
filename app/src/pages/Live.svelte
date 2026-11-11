<script lang="ts">
  import { onMount } from "svelte";
  import ChannelCard from "../components/ChannelCard.svelte";
  import AdSlot from "../components/AdSlot.svelte";
  import * as api from "../lib/api";
  import { live, sports } from "../lib/live.svelte";
  import type { Facet, TvChannel } from "../lib/types";

  // Live TV from the public iptv-org playlist. Channels the playlist or the
  // remote hide list removes never reach this page.
  const PAGE = 60;
  /** Category chips shown first, when the playlist has them. */
  const FEATURED = ["sports", "news", "movies", "music", "kids", "documentary", "entertainment"];

  let text = $state("");
  let country = $state(sports.region);
  let category = $state("");
  let favoritesOnly = $state(false);
  let limit = $state(PAGE);

  let items = $state<TvChannel[]>([]);
  let total = $state(0);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let countries = $state<Facet[]>([]);
  let categories = $state<Facet[]>([]);
  let attempt = $state(0);
  let run = 0;

  const chips = $derived(FEATURED.map((id) => categories.find((c) => c.id === id)).filter((c): c is Facet => !!c));
  const favorites = $derived(
    live.favorites.filter((f) => !text.trim() || f.name.toLowerCase().includes(text.trim().toLowerCase())),
  );

  onMount(() => {
    // The first call loads the whole channel list, which can take a few seconds.
    Promise.all([api.tvCountries(), api.tvCategories()])
      .then(([co, ca]) => {
        countries = co.slice(0, 120);
        categories = ca;
      })
      .catch(() => {});
  });

  $effect(() => {
    const query = { country: country || undefined, category: category || undefined, text: text.trim() || undefined, offset: 0, limit };
    void attempt;
    if (favoritesOnly) return;
    const mine = ++run;
    loading = true;
    error = null;
    const timer = setTimeout(
      () => {
        api
          .tvChannels(query)
          .then((p) => {
            if (mine !== run) return;
            items = p.items;
            total = p.total;
          })
          .catch((e) => mine === run && (error = String(e)))
          .finally(() => mine === run && (loading = false));
      },
      query.text ? 250 : 0,
    );
    return () => clearTimeout(timer);
  });

  const selectValue = (e: Event) => (e.currentTarget as HTMLElement & { value: string }).value;
  const pick = (id: string) => (category = category === id ? "" : id);
</script>

<div class="page">
  <header>
    <div>
      <m3e-heading variant="headline" size="large" level={1}>Live TV</m3e-heading>
      <p class="sub">Channels from a public playlist. Velmora hosts none of them, and some may not play.</p>
    </div>
  </header>

  <div class="controls">
    <m3e-form-field class="search" variant="outlined">
      <label slot="label" for="live-search">Search channels</label>
      <input id="live-search" type="search" autocomplete="off" spellcheck="false" bind:value={text} />
    </m3e-form-field>
    <m3e-form-field class="country" variant="outlined">
      <m3e-select
        aria-label="Country"
        value={country}
        onchange={(e: Event) => (country = selectValue(e))}
      >
        <m3e-option value="" selected={country === ""}>All countries</m3e-option>
        {#each countries as c (c.id)}
          <m3e-option value={c.id} selected={country === c.id}>{c.name} ({c.count})</m3e-option>
        {/each}
      </m3e-select>
    </m3e-form-field>
  </div>

  <div class="chips" role="group" aria-label="Categories">
    <m3e-chip-set>
      <m3e-filter-chip selected={favoritesOnly} onclick={() => (favoritesOnly = !favoritesOnly)}>
        <m3e-icon slot="icon" name="favorite" filled></m3e-icon>Favorites ({live.favorites.length})
      </m3e-filter-chip>
      {#each chips as c (c.id)}
        <m3e-filter-chip selected={category === c.id && !favoritesOnly} onclick={() => ((favoritesOnly = false), pick(c.id))}>
          {c.name}
        </m3e-filter-chip>
      {/each}
    </m3e-chip-set>
  </div>

  {#if favoritesOnly}
    {#if favorites.length}
      <div class="grid">
        {#each favorites as f (f.id)}
          <ChannelCard channel={f} onopen={() => live.playById(f.id)} />
        {/each}
      </div>
    {:else}
      <div class="empty">
        <m3e-icon name="favorite"></m3e-icon>
        <m3e-heading variant="title" size="large">No favorites yet</m3e-heading>
        <p>Tap the heart on a channel to keep it here.</p>
      </div>
    {/if}
  {:else if error}
    <div class="empty">
      <m3e-icon name="cloud_off"></m3e-icon>
      <m3e-heading variant="title" size="large">The channel list didn't load</m3e-heading>
      <p>{error}</p>
      <m3e-button variant="tonal" onclick={() => attempt++}>
        <m3e-icon slot="icon" name="refresh"></m3e-icon>Try again
      </m3e-button>
    </div>
  {:else if loading && !items.length}
    <div class="grid">
      {#each Array(12) as _, i (i)}
        <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
      {/each}
    </div>
    <p class="hint">Loading the channel list. The first time can take a few seconds.</p>
  {:else if items.length}
    <p class="count">{total} {total === 1 ? "channel" : "channels"}</p>
    <div class="grid">
      {#each items as c (c.id)}
        <ChannelCard channel={c} onopen={() => live.play(c)} />
      {/each}
    </div>
    {#if items.length >= 24}<AdSlot id="live" />{/if}
    {#if items.length < total}
      <div class="more">
        <m3e-button variant="tonal" onclick={() => (limit += PAGE)}>Show more</m3e-button>
      </div>
    {/if}
  {:else}
    <div class="empty">
      <m3e-icon name="search_off"></m3e-icon>
      <m3e-heading variant="title" size="large">No channels found</m3e-heading>
      <p>Try another country or category, or clear the search.</p>
    </div>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding-bottom: 32px;
  }
  .sub {
    margin: 4px 0 0;
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }
  .search {
    flex: 1 1 280px;
  }
  .country {
    flex: 0 1 260px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 10px;
  }
  .ghost {
    height: 80px;
  }
  .count,
  .hint {
    margin: 0;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .more {
    display: flex;
    justify-content: center;
    padding-top: 8px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 56px 0;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 48px;
  }
  .empty p {
    margin: 0;
    max-width: 420px;
  }
</style>
