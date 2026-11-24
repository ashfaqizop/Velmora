<script lang="ts">
  // Someone's profile: picture, name, badges, counts, bio, links, favourites,
  // and their followers and following. Your own page also shows who visited.
  import { fade, fly } from "svelte/transition";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import Avatar from "../auth/Avatar.svelte";
  import { animalOf } from "../auth/avatars";
  import { beginSignIn } from "../lib/session";
  import { linkLabel, shortUrl } from "../lib/links";
  import Badges from "./Badges.svelte";
  import FollowButton from "./FollowButton.svelte";
  import PeopleList from "./PeopleList.svelte";
  import ReportDialog from "./ReportDialog.svelte";
  import ActivityList from "./ActivityList.svelte";
  import RatingsList from "./RatingsList.svelte";
  import ListsGrid from "./ListsGrid.svelte";
  import StatsPanel from "./StatsPanel.svelte";
  import TheatreStamps from "./TheatreStamps.svelte";
  import { userActivity } from "./activity.svelte";
  import { ago, count, monthYear, plural } from "./format";
  import { go, href, profileHref, profileUrl, titleHref, type Tab } from "./nav.svelte";
  import {
    auth,
    checkFollowing,
    checkFollowsMe,
    followDelta,
    followersOf,
    followingOf,
    followsMe,
    loadMax,
    maxIds,
    profileByName,
    recordVisit,
    visitorsOf,
    visitsThisMonth,
    type Profile,
  } from "./data.svelte";

  let { user, tab }: { user: string; tab: Tab } = $props();

  let loaded = $state<Profile | null>(null);
  let phase = $state<"loading" | "ready" | "missing" | "error">("loading");
  let problem = $state("");
  let visits = $state(0);
  let reporting = $state(false);

  const isMe = $derived(!!auth.me && !!loaded && auth.me.id === loaded.id);
  // Your own page follows your edits straight away.
  const p = $derived(isMe ? auth.me : loaded);
  const tint = $derived(animalOf(p?.avatar)?.color ?? "#3271fe");
  const followers = $derived(p ? Math.max(0, p.followers_count + (isMe ? 0 : (followDelta.get(p.id) ?? 0))) : 0);

  $effect(() => {
    const name = user;
    const signedIn = auth.phase;
    let alive = true;
    phase = "loading";
    profileByName(name)
      .then(async (found) => {
        if (!alive) return;
        if (!found) {
          phase = "missing";
          document.title = "Not found · Velmora Social";
          return;
        }
        followDelta.delete(found.id);
        loaded = found;
        phase = "ready";
        document.title = `${found.display_name} (@${found.username}) · Velmora Social`;
        // The address keeps the username as it's written on the profile.
        if (found.username !== name) go(`/@${found.username}${tab === "activity" ? "" : `/${tab}`}`, { replace: true, keepScroll: true });
        loadMax([found.id]);
        if (signedIn !== "loading") {
          checkFollowing([found.id]);
          checkFollowsMe(found.id);
          recordVisit(found.id).catch(() => {});
        }
        visits = await visitsThisMonth(found.id);
      })
      .catch((e) => {
        if (!alive) return;
        problem = (e as Error).message;
        phase = "error";
      });
    return () => (alive = false);
  });

  function pick(t: Tab) {
    if (!p || t === tab) return;
    go(`/@${p.username}${t === "activity" ? "" : `/${t}`}`, { replace: true, keepScroll: true });
  }

  async function share() {
    if (!p) return;
    const url = profileUrl(p.username);
    try {
      if (navigator.share) {
        await navigator.share({ title: `${p.display_name} on Velmora`, url });
        return;
      }
    } catch {
      return;
    }
    try {
      await navigator.clipboard.writeText(url);
      M3eSnackbar.open("Link copied.");
    } catch {
      M3eSnackbar.open(url);
    }
  }

  function reportIt() {
    if (!auth.me) {
      beginSignIn();
      return;
    }
    reporting = true;
  }

  const TABS = $derived<{ id: Tab; label: string; icon: string }[]>([
    { id: "activity", label: "Activity", icon: "history" },
    { id: "reviews", label: "Reviews", icon: "star" },
    { id: "lists", label: "Lists", icon: "list" },
    { id: "stats", label: "Stats", icon: "insights" },
    { id: "theatre", label: "Theatre", icon: "theaters" },
    { id: "followers", label: "Followers", icon: "group" },
    { id: "following", label: "Following", icon: "person_heart" },
    ...(isMe ? [{ id: "visitors" as Tab, label: "Visitors", icon: "visibility" }] : []),
  ]);
</script>

{#if phase === "loading"}
  <div class="profile skeleton" aria-busy="true">
    <div class="cover"></div>
    <div class="head">
      <m3e-skeleton shape="circle" class="sk-pic"></m3e-skeleton>
      <div class="sk-lines"><m3e-skeleton class="sk-1"></m3e-skeleton><m3e-skeleton class="sk-2"></m3e-skeleton></div>
    </div>
  </div>
{:else if phase === "missing"}
  <section class="gone" in:fly={{ y: 12, duration: 280 }}>
    <span class="gone-icon" aria-hidden="true"><m3e-shape name="soft-burst"></m3e-shape><m3e-icon name="person_off"></m3e-icon></span>
    <m3e-heading variant="headline" size="medium" level={1}>No one here</m3e-heading>
    <p>There's no profile called @{user}. It may have been renamed or removed.</p>
    <m3e-button variant="filled" href={href("/search")}><m3e-icon slot="icon" name="search"></m3e-icon>Find people</m3e-button>
  </section>
{:else if phase === "error"}
  <section class="gone">
    <m3e-heading variant="headline" size="medium" level={1}>Couldn't load this profile</m3e-heading>
    <p role="alert">{problem}</p>
  </section>
{:else if p}
  <article class="profile" style:--tint={tint} in:fade={{ duration: 220 }}>
    <div class="cover" aria-hidden="true">
      <span class="orb a"></span><span class="orb b"></span>
    </div>

    <div class="head">
      <div class="pic" in:fly={{ y: 14, duration: 420 }}>
        <Avatar value={p.avatar} size={128} ring={5} />
      </div>
      <div class="actions">
        {#if isMe}
          <m3e-button variant="tonal" href={href("/settings")}><m3e-icon slot="icon" name="edit"></m3e-icon>Edit profile</m3e-button>
        {:else}
          <FollowButton id={p.id} size="medium" wide />
        {/if}
        <m3e-icon-button variant="tonal" aria-label="Share profile" onclick={share}><m3e-icon name="share"></m3e-icon></m3e-icon-button>
        {#if !isMe}
          <m3e-icon-button id="profile-more" aria-label="More">
            <m3e-icon name="more_vert"></m3e-icon>
            <m3e-menu-trigger for="profile-menu"></m3e-menu-trigger>
          </m3e-icon-button>
          <m3e-menu id="profile-menu">
            <m3e-menu-item onclick={share}><m3e-icon slot="icon" name="content_copy"></m3e-icon>Copy link</m3e-menu-item>
            <m3e-menu-item onclick={reportIt}><m3e-icon slot="icon" name="flag"></m3e-icon>Report @{p.username}</m3e-menu-item>
          </m3e-menu>
        {/if}
      </div>
    </div>

    <div class="names" in:fly={{ y: 10, duration: 380, delay: 60 }}>
      <h1>
        <span>{p.display_name}</span>
        <Badges id={p.id} verified={p.verified} size={26} />
      </h1>
      <div class="handle">
        <span>@{p.username}</span>
        {#if !isMe && followsMe.has(p.id)}<span class="pill">Follows you</span>{/if}
      </div>
    </div>

    <div class="counts" in:fly={{ y: 10, duration: 380, delay: 110 }}>
      <a class="stat" href={profileHref(p.username, "followers")}><strong>{count(followers)}</strong><span>{plural(followers, "Follower")}</span></a>
      <a class="stat" href={profileHref(p.username, "following")}><strong>{count(p.following_count)}</strong><span>Following</span></a>
      <svelte:element this={isMe ? "a" : "div"} class="stat" href={isMe ? profileHref(p.username, "visitors") : undefined}>
        <strong>{count(visits)}</strong><span>{plural(visits, "Visit")} this month</span>
      </svelte:element>
    </div>

    {#if p.bio}<p class="bio" in:fly={{ y: 10, duration: 380, delay: 150 }}>{p.bio}</p>{/if}

    {#if p.links.length}
      <m3e-chip-set class="links">
        {#each p.links as l (l.url)}
          <m3e-assist-chip href={l.url} target="_blank" rel="noopener noreferrer nofollow ugc">
            <m3e-icon slot="icon" name="link"></m3e-icon>{linkLabel(l.kind)}<span class="url">&nbsp;· {shortUrl(l.url)}</span>
          </m3e-assist-chip>
        {/each}
      </m3e-chip-set>
    {/if}

    <div class="meta">
      <span><m3e-icon name="calendar_month"></m3e-icon>Joined {monthYear(p.created_at)}</span>
      {#if maxIds.has(p.id)}<span class="max"><m3e-icon name="workspace_premium" filled></m3e-icon>Velmora Max</span>{/if}
      {#if p.verified}<span class="ver"><m3e-icon name="verified" filled></m3e-icon>Verified</span>{/if}
    </div>
  </article>

  {#if p.favorites.length || isMe}
    <section class="showcase">
      <h2><m3e-icon name="favorite" filled></m3e-icon>Favourites</h2>
      {#if p.favorites.length}
        <div class="favs">
          {#each p.favorites as f, i (f.k)}
            <a class="fav" href={titleHref(f.k)} style:--i={i} title={f.name}>
              <span class="poster">
                {#if f.poster}<img src={f.poster} alt="" loading="lazy" decoding="async" />{:else}<m3e-icon name="movie"></m3e-icon>{/if}
                <span class="play" aria-hidden="true"><m3e-icon name="play_arrow" filled></m3e-icon></span>
              </span>
              <span class="fav-name">{f.name}</span>
              {#if f.year}<span class="fav-year">{f.year}</span>{/if}
            </a>
          {/each}
        </div>
      {:else if isMe}
        <div class="empty-cta">
          <p>Show off up to six favourites from your My List.</p>
          <m3e-button variant="tonal" href={href("/settings")}><m3e-icon slot="icon" name="favorite"></m3e-icon>Pick favourites</m3e-button>
        </div>
      {:else}
        <p class="empty">@{p.username} hasn't picked favourites yet.</p>
      {/if}
    </section>
  {/if}

  <section class="tabs-wrap">
    <m3e-tabs class="tabs">
      {#each TABS as t (t.id)}
        <m3e-tab selected={tab === t.id} onclick={() => pick(t.id)}>
          <m3e-icon slot="icon" name={t.icon}></m3e-icon>{t.label}
        </m3e-tab>
      {/each}
    </m3e-tabs>

    <div class="panel">
      {#key `${p.id}:${tab}`}
        <div in:fade={{ duration: 180 }}>
          {#if tab === "activity"}
            <ActivityList load={(before) => userActivity(p.id, before)}>
              {#snippet empty()}
                <p class="empty">{isMe ? (auth.me?.share_activity === false ? "Sharing is off, so nothing you watch shows here. Turn it on in Edit profile." : "Nothing yet. What you watch, finish, add and rate in Velmora shows up here.") : `Nothing from @${p.username} yet.`}</p>
              {/snippet}
            </ActivityList>
          {:else if tab === "reviews"}
            <RatingsList userId={p.id} empty={isMe ? "Rate titles here or in the app and they show up here." : `@${p.username} hasn't rated anything yet.`} />
          {:else if tab === "lists"}
            <ListsGrid userId={p.id} mine={isMe} name={`@${p.username}`} />
          {:else if tab === "stats"}
            <StatsPanel id={p.id} mine={isMe} name={`@${p.username}`} />
          {:else if tab === "theatre"}
            <TheatreStamps userId={p.id} mine={isMe} name={`@${p.username}`} />
          {:else if tab === "followers"}
            <PeopleList load={(from) => followersOf(p.id, from)} empty={isMe ? "No followers yet. Share your profile to get some." : `No one follows @${p.username} yet.`} />
          {:else if tab === "following"}
            <PeopleList load={(from) => followingOf(p.id, from)} empty={isMe ? "You don't follow anyone yet." : `@${p.username} doesn't follow anyone yet.`} />
          {:else if tab === "visitors"}
            {#if isMe}
              <p class="private"><m3e-icon name="lock"></m3e-icon>Only you can see this list. Visitors who aren't signed in are counted but not shown.</p>
              <PeopleList load={(from) => visitorsOf(p.id, from)} empty="No signed-in visitors yet." note={(v) => ago(v.at)} />
            {:else}
              <p class="empty">Only @{p.username} can see who visited.</p>
            {/if}
          {/if}
        </div>
      {/key}
    </div>
  </section>

  {#if !isMe}<ReportDialog target={p.id} username={p.username} bind:open={reporting} />{/if}
{/if}

<style>
  .profile {
    position: relative;
    overflow: hidden;
    border-radius: 36px;
    padding: 0 clamp(20px, 4vw, 36px) 28px;
    background: var(--md-sys-color-surface-container);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 40%, transparent);
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .cover {
    position: relative;
    height: clamp(120px, 20vw, 180px);
    margin: 0 calc(-1 * clamp(20px, 4vw, 36px));
    overflow: hidden;
    background:
      linear-gradient(180deg, transparent 55%, color-mix(in srgb, var(--md-sys-color-surface-container) 70%, transparent)),
      linear-gradient(135deg, color-mix(in srgb, var(--tint) 40%, #0a0d16), color-mix(in srgb, var(--tint) 14%, #0a0d16));
  }
  .orb {
    position: absolute;
    border-radius: 50%;
    pointer-events: none;
    animation: drift 14s ease-in-out infinite alternate;
  }
  .orb.a {
    width: 70%;
    height: 220%;
    left: -15%;
    top: -90%;
    background: radial-gradient(closest-side, color-mix(in srgb, var(--tint) 75%, transparent), transparent);
  }
  .orb.b {
    width: 55%;
    height: 180%;
    right: -10%;
    top: -60%;
    background: radial-gradient(closest-side, color-mix(in srgb, var(--tint) 35%, #ffffff 12%), transparent);
    animation-duration: 18s;
  }
  @keyframes drift {
    to {
      transform: translate(8%, 12%) scale(1.08);
    }
  }
  .head {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    margin-top: -72px;
    flex-wrap: wrap;
  }
  .pic {
    border-radius: 50%;
    padding: 5px;
    background: var(--md-sys-color-surface-container);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-bottom: 6px;
  }
  .names h1 {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: clamp(26px, 4vw, 34px);
    line-height: 1.15;
    letter-spacing: -0.01em;
    overflow-wrap: anywhere;
  }
  .handle {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 4px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .pill {
    font-size: 12px;
    font-weight: 600;
    padding: 3px 10px;
    border-radius: 999px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .counts {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .stat {
    display: flex;
    flex-direction: column;
    padding: 8px 14px;
    border-radius: 18px;
    color: inherit;
    text-decoration: none;
    transition: background-color 160ms ease;
  }
  a.stat:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
  }
  .stat:first-child {
    padding-left: 0;
  }
  a.stat:first-child:hover {
    padding-left: 14px;
    margin-left: -14px;
  }
  .stat strong {
    font-size: 22px;
    font-variant-numeric: tabular-nums;
  }
  .stat span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .bio {
    margin: 0;
    font-size: 16px;
    line-height: 1.6;
    white-space: pre-line;
    overflow-wrap: anywhere;
    max-width: 62ch;
  }
  .url {
    color: var(--md-sys-color-on-surface-variant);
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 18px;
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .meta span {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    --m3e-icon-size: 18px;
  }
  .meta .max {
    color: #f8d27a;
  }
  .meta .ver {
    color: var(--md-sys-color-primary);
  }
  .showcase {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .showcase h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 4px;
    font-size: 18px;
    --m3e-icon-size: 20px;
  }
  .showcase h2 m3e-icon {
    color: #ff7aa2;
  }
  .tabs-wrap {
    border-radius: 32px;
    background: var(--md-sys-color-surface-container-low);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 30%, transparent);
    overflow: hidden;
  }
  .tabs {
    --m3e-tabs-container-color: transparent;
  }
  .panel {
    padding: 16px clamp(8px, 2vw, 20px) 22px;
  }
  .favs {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 18px;
    padding: 6px 4px;
  }
  .fav {
    display: flex;
    flex-direction: column;
    gap: 6px;
    color: inherit;
    text-decoration: none;
    animation: rise 480ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
    animation-delay: calc(var(--i) * 60ms);
  }
  .poster {
    position: relative;
    display: grid;
    place-items: center;
    aspect-ratio: 2 / 3;
    border-radius: 20px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
    box-shadow: 0 14px 30px -18px rgb(0 0 0 / 80%);
    transition:
      transform 320ms cubic-bezier(0.2, 0.8, 0.2, 1),
      border-radius 320ms ease;
    --m3e-icon-size: 40px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .poster img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .play {
    position: absolute;
    inset: auto 10px 10px auto;
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 26px;
    opacity: 0;
    transform: scale(0.6);
    transition:
      opacity 200ms ease,
      transform 260ms cubic-bezier(0.2, 0.8, 0.2, 1.4);
  }
  .fav:hover .poster,
  .fav:focus-visible .poster {
    transform: translateY(-4px) rotate(-1deg);
    border-radius: 28px;
  }
  .fav:hover .play,
  .fav:focus-visible .play {
    opacity: 1;
    transform: none;
  }
  .fav-name {
    font-weight: 600;
    font-size: 14px;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .fav-year {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    margin-top: -4px;
  }
  .empty {
    margin: 0;
    padding: 28px 12px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  .empty-cta {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 24px 12px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  .empty-cta p {
    margin: 0;
  }
  .private {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 12px 10px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 18px;
  }
  .gone {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 64px 16px;
    text-align: center;
  }
  .gone p {
    margin: 0;
    color: var(--md-sys-color-on-surface-variant);
  }
  .gone-icon {
    display: inline-grid;
    place-items: center;
    width: 88px;
    height: 88px;
    --m3e-shape-size: 88px;
    --m3e-shape-container-color: var(--md-sys-color-tertiary-container);
    --m3e-icon-size: 40px;
    color: var(--md-sys-color-on-tertiary-container);
  }
  .gone-icon > * {
    grid-area: 1 / 1;
  }
  .skeleton .head {
    align-items: center;
    justify-content: flex-start;
  }
  .sk-pic {
    width: 128px;
    height: 128px;
  }
  .sk-lines {
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex: 1;
    padding-top: 60px;
  }
  .sk-1 {
    width: 220px;
    max-width: 70%;
    height: 22px;
  }
  .sk-2 {
    width: 140px;
    height: 14px;
  }
  .skeleton .cover {
    background: var(--md-sys-color-surface-container-high);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(14px);
    }
  }
  @media (max-width: 560px) {
    .head {
      margin-top: -60px;
    }
    .head :global(.avatar) {
      width: 104px !important;
      height: 104px !important;
    }
    .actions {
      width: 100%;
    }
    .favs {
      grid-template-columns: repeat(auto-fill, minmax(104px, 1fr));
      gap: 12px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .orb,
    .fav {
      animation: none;
    }
  }
</style>
