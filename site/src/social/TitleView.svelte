<script lang="ts">
  // A title on Velmora Social: what it is, the Velmora score, your rating and
  // review, everyone's reviews, and a way to open it in the app.
  import { fade, fly } from "svelte/transition";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import Avatar from "../auth/Avatar.svelte";
  import { beginSignIn } from "../lib/session";
  import Badges from "./Badges.svelte";
  import Stars from "./Stars.svelte";
  import { ago } from "./format";
  import { auth } from "./data.svelte";
  import { kindLabel, loadTitles, openInApp, titles } from "./meta.svelte";
  import { go, href, profileHref } from "./nav.svelte";
  import {
    addToList,
    myRating,
    rate,
    titleReviews,
    titleScore,
    unrate,
    userLists,
    type Rating,
    type Score,
    type TitleList,
  } from "./activity.svelte";

  let { k }: { k: string } = $props();

  const meta = $derived(titles.get(k));
  let score = $state<Score>({ average: null, ratings: 0, reviews: 0 });
  let mine = $state<Rating | null>(null);
  let reviews = $state<Rating[]>([]);
  let more = $state(false);
  let lists = $state<TitleList[]>([]);

  let pick = $state(0);
  let text = $state("");
  let spoiler = $state(false);
  let editing = $state(false);
  let busy = $state(false);
  let revealed = $state(new Set<string>());

  $effect(() => {
    const key = k;
    const signedIn = !!auth.me;
    loadTitles([key]);
    titleScore(key).then((s) => (score = s));
    titleReviews(key, 0)
      .then((r) => {
        reviews = r.items;
        more = r.more;
      })
      .catch(() => {});
    if (signedIn) {
      myRating(key).then((r) => {
        mine = r;
        pick = r?.score ?? 0;
        text = r?.review ?? "";
        spoiler = r?.spoiler ?? false;
      });
      userLists(auth.me!.id)
        .then((l) => (lists = l))
        .catch(() => {});
    }
  });

  $effect(() => {
    if (meta) document.title = `${meta.name}${meta.year ? ` (${meta.year})` : ""} · Velmora Social`;
  });

  async function save() {
    if (!auth.me) return beginSignIn();
    if (!pick) return;
    busy = true;
    try {
      await rate(k, pick, text, spoiler);
      mine = await myRating(k);
      editing = false;
      score = await titleScore(k);
      const r = await titleReviews(k, 0);
      reviews = r.items;
      more = r.more;
      M3eSnackbar.open("Rating saved.");
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    } finally {
      busy = false;
    }
  }

  async function remove() {
    busy = true;
    try {
      await unrate(k);
      mine = null;
      pick = 0;
      text = "";
      spoiler = false;
      score = await titleScore(k);
      reviews = reviews.filter((r) => r.user_id !== auth.me?.id);
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    } finally {
      busy = false;
    }
  }

  async function nextReviews() {
    const r = await titleReviews(k, reviews.length);
    reviews = [...reviews, ...r.items];
    more = r.more;
  }

  async function addTo(list: TitleList) {
    if (!meta) return;
    try {
      await addToList(list, { k, name: meta.name, year: meta.year, poster: meta.poster });
      M3eSnackbar.open(`Added to ${list.name}.`);
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    }
  }

  const showForm = $derived(!!auth.me && (!mine || editing));
</script>

{#if meta === null}
  <section class="gone">
    <m3e-heading variant="headline" size="medium" level={1}>We couldn't find this title</m3e-heading>
    <p>It may not be on TMDB, or the title lookup isn't set up yet.</p>
    <m3e-button variant="filled" href={href("/trending")}>See what's trending</m3e-button>
  </section>
{:else}
  <article class="hero" in:fade={{ duration: 260 }}>
    {#if meta?.backdrop}<img class="backdrop" src={meta.backdrop} alt="" />{/if}
    <div class="shade"></div>
    <div class="inner">
      <div class="poster" in:fly={{ y: 16, duration: 420 }}>
        {#if meta?.poster}<img src={meta.poster} alt="" />{:else}<m3e-skeleton class="psk"></m3e-skeleton>{/if}
      </div>
      <div class="info">
        {#if meta}
          <span class="eyebrow">{kindLabel(k, meta)}{meta.year ? ` · ${meta.year}` : ""}{meta.runtime ? ` · ${meta.runtime} min` : ""}{meta.seasons ? ` · ${meta.seasons} season${meta.seasons === 1 ? "" : "s"}` : ""}</span>
          <h1>{meta.name}</h1>
          {#if meta.genres.length}<div class="genres">{#each meta.genres as g (g)}<span>{g}</span>{/each}</div>{/if}
          <div class="scores">
            <div class="score">
              <strong>{score.average != null ? (score.average / 2).toFixed(1) : "–"}</strong>
              <div><Stars value={Math.round(score.average ?? 0)} size={16} /><span>{score.ratings} Velmora {score.ratings === 1 ? "rating" : "ratings"}</span></div>
            </div>
            {#if meta.tmdb}<div class="score small"><strong>{meta.tmdb.toFixed(1)}</strong><span>TMDB</span></div>{/if}
          </div>
          <div class="cta">
            <m3e-button variant="filled" size="medium" href={openInApp(`title/${k}`)} target="_blank"><m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>Open in Velmora</m3e-button>
            {#if auth.me}
              <m3e-button id="add-list" variant="tonal" size="medium">
                <m3e-icon slot="icon" name="playlist_add"></m3e-icon>Add to a list
                <m3e-menu-trigger for="list-menu"></m3e-menu-trigger>
              </m3e-button>
              <m3e-menu id="list-menu">
                {#each lists as l (l.id)}
                  <m3e-menu-item onclick={() => addTo(l)}><m3e-icon slot="icon" name={l.items.some((i) => i.k === k) ? "check" : "list"}></m3e-icon>{l.name}</m3e-menu-item>
                {/each}
                {#if lists.length}<m3e-divider></m3e-divider>{/if}
                <m3e-menu-item onclick={() => go("/list/new")}><m3e-icon slot="icon" name="add"></m3e-icon>New list</m3e-menu-item>
              </m3e-menu>
            {/if}
          </div>
        {:else}
          <m3e-skeleton class="tsk"></m3e-skeleton>
        {/if}
      </div>
    </div>
  </article>

  {#if meta?.overview}<p class="overview">{meta.overview}</p>{/if}

  <section class="card rate">
    <h2>Your rating</h2>
    {#if !auth.me}
      <p class="muted">Sign in to rate and review.</p>
      <m3e-button variant="filled" onclick={() => beginSignIn()}><m3e-icon slot="icon" name="login"></m3e-icon>Sign in</m3e-button>
    {:else if showForm}
      <div class="pick">
        <Stars value={pick} size={34} onpick={(s) => (pick = s)} />
        <span class="pick-num">{pick ? `${(pick / 2).toFixed(1)} / 5` : "Tap a star"}</span>
      </div>
      <m3e-form-field variant="outlined" class="field">
        <label slot="label" for="review">Review (optional)</label>
        <textarea id="review" rows="3" maxlength="500" bind:value={text} disabled={busy}></textarea>
        <span slot="hint">{text.length}/500 · Public, under your name.</span>
      </m3e-form-field>
      <label class="spoil"><m3e-switch checked={spoiler} onchange={(e: Event) => (spoiler = (e.currentTarget as HTMLElement & { checked: boolean }).checked)}></m3e-switch>It has spoilers</label>
      <div class="row">
        <m3e-button variant="filled" disabled={!pick || busy} onclick={save}><m3e-icon slot="icon" name="check"></m3e-icon>{busy ? "Saving…" : mine ? "Update" : "Post"}</m3e-button>
        {#if mine}<m3e-button variant="text" onclick={() => (editing = false)}>Cancel</m3e-button>{/if}
      </div>
    {:else if mine}
      <div class="mine">
        <Stars value={mine.score} size={24} />
        <strong>{(mine.score / 2).toFixed(1)}</strong>
        <span class="muted">· {ago(mine.updated_at)}</span>
      </div>
      {#if mine.review}<p class="review">{mine.review}</p>{/if}
      <div class="row">
        <m3e-button variant="tonal" onclick={() => (editing = true)}><m3e-icon slot="icon" name="edit"></m3e-icon>Edit</m3e-button>
        <m3e-button variant="text" disabled={busy} onclick={remove}>Remove</m3e-button>
      </div>
    {/if}
  </section>

  <section class="card">
    <h2>Reviews <span class="muted">{score.reviews}</span></h2>
    {#if !reviews.length}
      <p class="muted">No reviews yet. Yours could be the first.</p>
    {:else}
      <div class="reviews">
        {#each reviews as r, i (r.user_id)}
          <article class="rv" in:fly={{ y: 8, duration: 240, delay: Math.min(i, 8) * 30 }}>
            <a href={profileHref(r.p!.username)} class="rv-pic"><Avatar value={r.p!.avatar} size={40} /></a>
            <div class="rv-body">
              <div class="rv-head">
                <a href={profileHref(r.p!.username)} class="rv-name">{r.p!.display_name}</a>
                <Badges id={r.user_id} verified={r.p!.verified} size={15} />
                <Stars value={r.score} size={14} />
                <span class="muted small">{ago(r.updated_at)}</span>
              </div>
              {#if r.spoiler && !revealed.has(r.user_id)}
                <button type="button" class="spoiler" onclick={() => (revealed = new Set([...revealed, r.user_id]))}>Contains spoilers. Show the review</button>
              {:else}
                <p class="review">{r.review}</p>
              {/if}
            </div>
          </article>
        {/each}
      </div>
      {#if more}<div class="more"><m3e-button variant="tonal" onclick={nextReviews}>Show more</m3e-button></div>{/if}
    {/if}
  </section>
{/if}

<style>
  .hero {
    position: relative;
    overflow: hidden;
    border-radius: 36px;
    min-height: 360px;
    display: flex;
    align-items: flex-end;
    background: var(--md-sys-color-surface-container);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 40%, transparent);
  }
  .backdrop {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0.55;
    animation: settle 1.2s cubic-bezier(0.2, 0.8, 0.2, 1) both;
  }
  .shade {
    position: absolute;
    inset: 0;
    background:
      linear-gradient(0deg, var(--md-sys-color-surface-container) 8%, transparent 70%),
      linear-gradient(90deg, color-mix(in srgb, var(--md-sys-color-surface-container) 85%, transparent), transparent 75%);
  }
  .inner {
    position: relative;
    display: flex;
    align-items: flex-end;
    gap: clamp(16px, 3vw, 32px);
    padding: clamp(20px, 4vw, 36px);
    width: 100%;
  }
  .poster {
    flex: none;
    width: clamp(110px, 18vw, 190px);
    aspect-ratio: 2 / 3;
    border-radius: 22px;
    overflow: hidden;
    box-shadow: 0 30px 60px -24px rgb(0 0 0 / 90%);
    background: var(--md-sys-color-surface-container-highest);
  }
  .poster img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .psk {
    width: 100%;
    height: 100%;
  }
  .tsk {
    width: 260px;
    max-width: 100%;
    height: 32px;
  }
  .info {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .eyebrow {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
  }
  h1 {
    margin: 0;
    font-family: var(--display);
    font-weight: 400;
    font-size: clamp(34px, 5.5vw, 60px);
    line-height: 1;
    overflow-wrap: anywhere;
  }
  .genres {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .genres span {
    padding: 4px 12px;
    border-radius: 999px;
    font-size: 13px;
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 10%, transparent);
  }
  .scores {
    display: flex;
    gap: 22px;
    align-items: center;
    flex-wrap: wrap;
  }
  .score {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .score strong {
    font-size: 34px;
    font-variant-numeric: tabular-nums;
    color: #f8d27a;
  }
  .score div {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .score.small strong {
    font-size: 22px;
    color: var(--md-sys-color-on-surface);
  }
  .score.small span {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .cta {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .overview {
    margin: 0;
    max-width: 72ch;
    font-size: 16px;
    line-height: 1.65;
    color: color-mix(in srgb, var(--md-sys-color-on-surface) 85%, transparent);
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 22px clamp(16px, 3vw, 26px);
    border-radius: 28px;
    background: var(--md-sys-color-surface-container-low);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 30%, transparent);
  }
  .card h2 {
    margin: 0;
    font-size: 18px;
  }
  .muted {
    margin: 0;
    color: var(--md-sys-color-on-surface-variant);
    font-weight: 400;
  }
  .small {
    font-size: 12px;
  }
  .pick {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .pick-num {
    font-weight: 700;
    color: #f8d27a;
  }
  .field {
    width: 100%;
  }
  .spoil {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 14px;
  }
  .row {
    display: flex;
    gap: 8px;
  }
  .mine {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .mine strong {
    font-size: 20px;
    color: #f8d27a;
  }
  .review {
    margin: 0;
    line-height: 1.6;
    white-space: pre-line;
    overflow-wrap: anywhere;
  }
  .reviews {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .rv {
    display: flex;
    gap: 12px;
  }
  .rv-pic {
    flex: none;
    display: inline-grid;
  }
  .rv-body {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }
  .rv-head {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px 8px;
  }
  .rv-name {
    color: inherit;
    font-weight: 700;
    text-decoration: none;
  }
  .spoiler {
    align-self: flex-start;
    padding: 8px 14px;
    border: 1px dashed var(--md-sys-color-outline);
    border-radius: 14px;
    background: none;
    color: var(--md-sys-color-on-surface-variant);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .more {
    display: flex;
    justify-content: center;
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
  @keyframes settle {
    from {
      transform: scale(1.08);
      opacity: 0;
    }
  }
  @media (max-width: 600px) {
    .inner {
      flex-direction: column;
      align-items: flex-start;
    }
    .hero {
      min-height: 0;
    }
  }
</style>
