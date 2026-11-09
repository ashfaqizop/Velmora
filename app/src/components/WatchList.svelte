<script lang="ts">
  import * as api from "../lib/api";
  import { live, sports } from "../lib/live.svelte";
  import { REGIONS, regionName } from "../lib/sports";
  import type { SportsMatch, WatchOption } from "../lib/types";

  // "Where to watch" for one match: official broadcasters in the chosen
  // country. A Play button shows only for free channels someone listed by hand.
  let { match }: { match: SportsMatch } = $props();

  let options = $state<WatchOption[]>([]);
  let loading = $state(true);
  let failed = $state(false);
  let run = 0;

  $effect(() => {
    const region = sports.region;
    const mine = ++run;
    loading = true;
    failed = false;
    api
      .sportsWatch($state.snapshot(match) as SportsMatch, region)
      .then((o) => mine === run && (options = o))
      .catch(() => mine === run && ((options = []), (failed = true)))
      .finally(() => mine === run && (loading = false));
  });

  const selectValue = (e: Event) => (e.currentTarget as HTMLElement & { value: string }).value;
  const KIND: Record<WatchOption["kind"], string> = { free: "Free", subscription: "Subscription", unknown: "Check the app" };
</script>

<div class="watch">
  <header>
    <div>
      <m3e-heading variant="title" size="large" level={2}>Where to watch</m3e-heading>
      <p class="sub">Official broadcasters in {regionName(sports.region)}.</p>
    </div>
    <m3e-form-field class="region" variant="outlined">
      <m3e-select aria-label="Country" value={sports.region} onchange={(e: Event) => sports.setRegion(selectValue(e))}>
        {#each REGIONS as [code, name] (code)}
          <m3e-option value={code} selected={sports.region === code}>{name}</m3e-option>
        {/each}
      </m3e-select>
    </m3e-form-field>
  </header>

  {#if loading}
    {#each Array(2) as _, i (i)}
      <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
    {/each}
  {:else if options.length}
    <ul>
      {#each options as o (o.name + o.country + o.source)}
        <li>
          <span class="logo">
            <span class="initial">{o.name.trim().charAt(0).toUpperCase()}</span>
            {#if o.logo}<img src={o.logo} alt="" loading="lazy" onerror={(e) => ((e.currentTarget as HTMLImageElement).style.display = "none")} />{/if}
          </span>
          <div class="info">
            <strong>{o.name}</strong>
            <span>
              <span class="chip" class:free={o.kind === "free"}>{KIND[o.kind]}</span>
              {o.country}
            </span>
          </div>
          {#if o.play_channel}
            <m3e-button variant="filled" onclick={() => live.playById(o.play_channel!)}>
              <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>Play
            </m3e-button>
          {:else if o.url}
            <m3e-button variant="tonal" onclick={() => api.openExternal(o.url!)}>
              <m3e-icon slot="icon" name="open_in_new"></m3e-icon>Open
            </m3e-button>
          {/if}
        </li>
      {/each}
    </ul>
  {:else}
    <div class="none">
      <m3e-icon name="tv_off"></m3e-icon>
      <p>
        {failed ? "Couldn't look up broadcasters." : `No official broadcaster is listed for ${regionName(sports.region)} yet.`}
        Try another country.
      </p>
    </div>
  {/if}
  <p class="fine">Velmora lists official broadcasters only. A Play button appears for free channels that carry the match.</p>
</div>

<style>
  .watch {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: space-between;
    gap: 12px;
  }
  .sub {
    margin: 2px 0 0;
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .region {
    flex: 0 1 240px;
  }
  .ghost {
    height: 72px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 14px;
    border-radius: 20px;
    background: var(--md-sys-color-surface-container);
  }
  .logo {
    position: relative;
    flex: none;
    width: 52px;
    height: 52px;
    border-radius: 14px;
    display: grid;
    place-items: center;
    overflow: hidden;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    font-family: var(--app-display-font);
    font-size: 26px;
  }
  .logo img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
    padding: 8px;
    background: var(--md-sys-color-surface-container);
  }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .info strong {
    font-weight: 600;
  }
  .info span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .chip {
    display: inline-block;
    margin-right: 6px;
    padding: 1px 8px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 700;
    background: var(--md-sys-color-surface-container-highest);
  }
  .chip.free {
    background: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
  }
  .none {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    border-radius: 20px;
    background: var(--md-sys-color-surface-container);
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 28px;
  }
  .none p {
    margin: 0;
  }
  .fine {
    margin: 0;
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
    opacity: 0.8;
  }
</style>
