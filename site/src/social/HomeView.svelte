<script lang="ts">
  // The front page. Signed out: what Velmora Social is. Signed in: your card,
  // and people worth following.
  import { fly, fade } from "svelte/transition";
  import Avatar from "../auth/Avatar.svelte";
  import { MAIN_SITE } from "../auth/config";
  import { animalOf } from "../auth/avatars";
  import { beginSignIn } from "../lib/session";
  import PersonRow from "./PersonRow.svelte";
  import Badges from "./Badges.svelte";
  import { auth, featuredPeople, mostFollowed, newestPeople, staffPicks, type Person, type StaffPick } from "./data.svelte";
  import { count, plural } from "./format";
  import { href, profileHref } from "./nav.svelte";
  import ActivityList from "./ActivityList.svelte";
  import Poster from "./Poster.svelte";
  import { myFeed, suggestedPeople, trendingTitles, type Trend } from "./activity.svelte";

  let top = $state<Person[] | null>(null);
  let fresh = $state<Person[] | null>(null);
  let busy = $state(false);
  let suggested = $state<Person[] | null>(null);
  let trend = $state<Trend[] | null>(null);
  let featured = $state<Person[]>([]);
  let picks = $state<StaffPick[]>([]);

  $effect(() => {
    document.title = "Velmora Social";
    void auth.phase;
    mostFollowed().then((l) => (top = l)).catch(() => (top = []));
    newestPeople().then((l) => (fresh = l)).catch(() => (fresh = []));
    featuredPeople().then((l) => (featured = l)).catch(() => (featured = []));
    staffPicks().then((l) => (picks = l)).catch(() => (picks = []));
    if (auth.me) suggestedPeople().then((l) => (suggested = l)).catch(() => (suggested = []));
    trendingTitles("movie")
      .then(async (m) => {
        const t = await trendingTitles("tv");
        trend = [...m.slice(0, 6), ...t.slice(0, 6)].sort((a, b) => b.people - a.people).slice(0, 8);
      })
      .catch(() => (trend = []));
  });

  const me = $derived(auth.me);
  const others = (list: Person[] | null) => (list ?? []).filter((p) => p.id !== me?.id);

  async function signIn() {
    busy = true;
    await beginSignIn();
  }

  const FEATURES = [
    { icon: "account_circle", shape: "12-sided-cookie", title: "Your page", text: "A profile at @you with your picture, bio, links and six favourite titles." },
    { icon: "person_add", shape: "flower", title: "Follow anyone", text: "Follow people, see who follows you back, and who dropped by this month." },
    { icon: "explore", shape: "soft-burst", title: "A feed", text: "What the people you follow are watching, finishing and rating, with their reviews." },
    { icon: "local_fire_department", shape: "sunny", title: "Trending", text: "What everyone on Velmora plays most, one click from the app." },
  ];
</script>

{#if auth.phase === "in" && me}
  <section class="welcome" in:fly={{ y: 14, duration: 320 }} style:--tint={animalOf(me.avatar)?.color ?? "#3271fe"}>
    <a class="mine" href={profileHref(me.username)}>
      <Avatar value={me.avatar} size={72} ring={3} />
      <span class="mine-text">
        <span class="hello">Welcome back</span>
        <span class="mine-name">{me.display_name}<Badges id={me.id} verified={me.verified} size={20} /></span>
        <span class="mine-counts">{count(me.followers_count)} {plural(me.followers_count, "follower")} · {count(me.following_count)} following</span>
      </span>
      <m3e-icon name="arrow_forward" class="go"></m3e-icon>
    </a>
    <div class="quick">
      <m3e-button variant="filled" href={href("/search")}><m3e-icon slot="icon" name="search"></m3e-icon>Find people</m3e-button>
      <m3e-button variant="tonal" href={href("/settings")}><m3e-icon slot="icon" name="edit"></m3e-icon>Edit profile</m3e-button>
    </div>
  </section>
{:else if auth.phase === "noprofile"}
  <m3e-card variant="filled" class="note">
    <div slot="content">
      <strong>Your profile isn't finished.</strong>
      <p>Sign in again and complete the last step to choose a username.</p>
      <m3e-button variant="filled" onclick={signIn}>Finish my profile</m3e-button>
    </div>
  </m3e-card>
{:else}
  <section class="hero" in:fade={{ duration: 300 }}>
    <span class="eyebrow">Velmora Social</span>
    <h1 class="display">Velmora, <em>together.</em></h1>
    <p class="lead">Profiles, follows and what everyone on Velmora is watching. No messages and no ads, just people and what they love.</p>
    {#if auth.problem}<p class="problem" role="alert">{auth.problem}</p>{/if}
    <div class="row">
      <m3e-button variant="filled" size="medium" disabled={busy} onclick={signIn}><m3e-icon slot="icon" name="login"></m3e-icon>Sign in</m3e-button>
      <m3e-button variant="tonal" size="medium" href={href("/search")}><m3e-icon slot="icon" name="search"></m3e-icon>Find people</m3e-button>
      <m3e-button variant="text" size="medium" href={MAIN_SITE}>Get the app</m3e-button>
    </div>
  </section>

  <section class="features">
    {#each FEATURES as c, i (c.title)}
      <m3e-card variant="outlined" class="tile" style:--delay="{i * 70}ms">
        <div slot="content">
          <span class="tile-icon" aria-hidden="true"><m3e-shape name={c.shape}></m3e-shape><m3e-icon name={c.icon}></m3e-icon></span>
          <strong>{c.title}</strong>
          <p>{c.text}</p>
        </div>
      </m3e-card>
    {/each}
  </section>
{/if}

{#if trend?.length}
  <section class="trend">
    <header>
      <m3e-icon name="local_fire_department"></m3e-icon><h2>Trending this week</h2>
      <m3e-button variant="text" href={href("/trending")}>See all<m3e-icon slot="trailing-icon" name="arrow_forward"></m3e-icon></m3e-button>
    </header>
    <div class="strip">
      {#each trend as t, i (t.k)}<Poster k={t.k} rank={i + 1} {i} note="{t.people} watching" />{/each}
    </div>
  </section>
{/if}

{#if picks.length}
  <section class="trend">
    <header><m3e-icon name="auto_awesome"></m3e-icon><h2>Staff picks</h2></header>
    <div class="strip">
      {#each picks as p, i (p.k)}<Poster k={p.k} {i} note={p.note} />{/each}
    </div>
  </section>
{/if}

{#if others(featured).length}
  <section class="block featured">
    <header><m3e-icon name="verified"></m3e-icon><h2>Featured people</h2></header>
    <div class="featured-grid">
      {#each others(featured) as p, i (p.id)}
        <div in:fly={{ y: 8, duration: 240, delay: i * 30 }}><PersonRow person={p} /></div>
      {/each}
    </div>
  </section>
{/if}

{#if auth.phase === "in" && me}
  <div class="home-grid">
    <section class="block feed">
      <header><m3e-icon name="explore"></m3e-icon><h2>Your feed</h2></header>
      <ActivityList load={(before) => myFeed(before)}>
        {#snippet empty()}
          <div class="feed-empty">
            <p>Your feed fills up with what the people you follow watch, finish, add and rate. Follow a few people to get started.</p>
            <m3e-button variant="tonal" href={href("/search")}><m3e-icon slot="icon" name="search"></m3e-icon>Find people</m3e-button>
          </div>
        {/snippet}
      </ActivityList>
    </section>
    <aside class="side">
      {#if suggested?.length}
        <section class="block">
          <header><m3e-icon name="person_add"></m3e-icon><h2>Suggested for you</h2></header>
          {#each suggested as p, i (p.id)}
            <div in:fly={{ y: 8, duration: 240, delay: i * 30 }}><PersonRow person={p} /></div>
          {/each}
        </section>
      {/if}
      <section class="block">
        <header><m3e-icon name="emoji_events"></m3e-icon><h2>Most followed</h2></header>
        {#if top === null}
          <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
        {:else if !others(top).length}
          <p class="empty">No one here yet.</p>
        {:else}
          {#each others(top).slice(0, 6) as p (p.id)}<PersonRow person={p} />{/each}
        {/if}
        <m3e-button variant="text" class="lb" href={href("/leaderboards")}><m3e-icon slot="icon" name="leaderboard"></m3e-icon>Leaderboards</m3e-button>
      </section>
    </aside>
  </div>
{:else}
<div class="columns">
  <section class="block">
    <header><m3e-icon name="emoji_events"></m3e-icon><h2>Most followed</h2></header>
    {#if top === null}
      <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
    {:else if !others(top).length}
      <p class="empty">No one here yet. Be the first.</p>
    {:else}
      {#each others(top) as p, i (p.id)}
        <div in:fly={{ y: 8, duration: 240, delay: i * 30 }}><PersonRow person={p} /></div>
      {/each}
    {/if}
  </section>
  <section class="block">
    <header><m3e-icon name="wand_stars"></m3e-icon><h2>New on Velmora</h2></header>
    {#if fresh === null}
      <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
    {:else if !others(fresh).length}
      <p class="empty">No one new yet.</p>
    {:else}
      {#each others(fresh) as p, i (p.id)}
        <div in:fly={{ y: 8, duration: 240, delay: i * 30 }}><PersonRow person={p} /></div>
      {/each}
    {/if}
  </section>
</div>
{/if}

<style>
  .featured-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 4px 12px;
  }
  .hero {
    display: flex;
    flex-direction: column;
    gap: 14px;
    max-width: 680px;
    padding-top: 12px;
  }
  .eyebrow {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
  }
  .display {
    margin: 0;
    font-family: var(--display);
    font-weight: 400;
    font-size: clamp(44px, 7vw, 76px);
    line-height: 1;
    letter-spacing: -0.01em;
  }
  .display em {
    color: #f8e8c8;
  }
  .lead {
    margin: 0;
    font-size: 18px;
    line-height: 1.6;
    color: var(--md-sys-color-on-surface-variant);
  }
  .problem {
    margin: 0;
    color: var(--md-sys-color-error);
  }
  .row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    padding-top: 6px;
  }
  .welcome {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    padding: 22px clamp(18px, 3vw, 28px);
    border-radius: 32px;
    background:
      radial-gradient(80% 140% at 0% 0%, color-mix(in srgb, var(--tint) 28%, transparent), transparent 70%),
      var(--md-sys-color-surface-container);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 40%, transparent);
  }
  .mine {
    display: flex;
    align-items: center;
    gap: 16px;
    color: inherit;
    text-decoration: none;
    min-width: 0;
  }
  .mine-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .hello {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .mine-name {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 24px;
    font-weight: 800;
  }
  .mine-counts {
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .go {
    transition: transform 200ms ease;
    color: var(--md-sys-color-on-surface-variant);
  }
  .mine:hover .go {
    transform: translateX(4px);
  }
  .quick {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .note div[slot="content"] {
    display: flex;
    flex-direction: column;
    gap: 10px;
    align-items: flex-start;
  }
  .note p {
    margin: 0;
    color: var(--md-sys-color-on-surface-variant);
  }
  .features {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: 14px;
  }
  .tile {
    animation: rise 520ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
    animation-delay: var(--delay);
  }
  .tile div[slot="content"] {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .tile p {
    margin: 0;
    font-size: 14px;
    line-height: 1.55;
    color: var(--md-sys-color-on-surface-variant);
  }
  .tile-icon {
    display: inline-grid;
    place-items: center;
    width: 52px;
    height: 52px;
    --m3e-shape-size: 52px;
    --m3e-shape-container-color: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
  }
  .tile-icon > * {
    grid-area: 1 / 1;
  }
  .trend {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .trend header {
    display: flex;
    align-items: center;
    gap: 10px;
    color: #ff9a5a;
  }
  .trend h2 {
    margin: 0;
    flex: 1;
    font-size: 18px;
    color: var(--md-sys-color-on-surface);
  }
  .strip {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: minmax(130px, 150px);
    gap: 16px;
    overflow-x: auto;
    padding: 4px 4px 12px;
    scroll-snap-type: x mandatory;
  }
  .strip > :global(*) {
    scroll-snap-align: start;
  }
  .home-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 340px;
    gap: 16px;
    align-items: start;
  }
  .side {
    display: flex;
    flex-direction: column;
    gap: 16px;
    position: sticky;
    top: 84px;
  }
  .feed-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 24px 12px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  .feed-empty p {
    margin: 0;
    max-width: 44ch;
  }
  .lb {
    align-self: flex-start;
    margin-left: 4px;
  }
  @media (max-width: 860px) {
    .home-grid {
      grid-template-columns: 1fr;
    }
    .side {
      position: static;
    }
  }
  .columns {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(320px, 100%), 1fr));
    gap: 16px;
  }
  .block {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 18px 10px 14px;
    border-radius: 32px;
    background: var(--md-sys-color-surface-container-low);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 30%, transparent);
  }
  .block header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 12px 8px;
    color: var(--md-sys-color-primary);
  }
  .block h2 {
    margin: 0;
    font-size: 18px;
    color: var(--md-sys-color-on-surface);
  }
  .center {
    display: grid;
    place-items: center;
    padding: 24px 0;
  }
  .empty {
    margin: 0;
    padding: 20px 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(14px);
    }
  }
  @media (max-width: 400px) {
    .trend {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .trend header {
    display: flex;
    align-items: center;
    gap: 10px;
    color: #ff9a5a;
  }
  .trend h2 {
    margin: 0;
    flex: 1;
    font-size: 18px;
    color: var(--md-sys-color-on-surface);
  }
  .strip {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: minmax(130px, 150px);
    gap: 16px;
    overflow-x: auto;
    padding: 4px 4px 12px;
    scroll-snap-type: x mandatory;
  }
  .strip > :global(*) {
    scroll-snap-align: start;
  }
  .home-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 340px;
    gap: 16px;
    align-items: start;
  }
  .side {
    display: flex;
    flex-direction: column;
    gap: 16px;
    position: sticky;
    top: 84px;
  }
  .feed-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 24px 12px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  .feed-empty p {
    margin: 0;
    max-width: 44ch;
  }
  .lb {
    align-self: flex-start;
    margin-left: 4px;
  }
  @media (max-width: 860px) {
    .home-grid {
      grid-template-columns: 1fr;
    }
    .side {
      position: static;
    }
  }
  .columns {
      grid-template-columns: 1fr;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .tile {
      animation: none;
    }
  }
</style>
