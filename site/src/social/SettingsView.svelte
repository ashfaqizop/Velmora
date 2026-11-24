<script lang="ts">
  // Edit your profile on the web: picture, names, bio, links and favourites.
  // The same fields can be changed in the app (Settings, Velmora account).
  import { fly, scale } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import Avatar from "../auth/Avatar.svelte";
  import NameStatus from "../auth/NameStatus.svelte";
  import { ANIMALS } from "../auth/avatars";
  import { beginSignIn } from "../lib/session";
  import { LINK_KINDS, MAX_BIO, MAX_LINKS, linkLabel, validLink, type LinkKind, type ProfileLink } from "../lib/links";
  import { auth, myListFavorites, saveProfile, usernameFree, type Favorite } from "./data.svelte";
  import { go } from "./nav.svelte";
  import { clearActivity, setSharing } from "./activity.svelte";

  const NAME = /^[A-Za-z0-9_]{3,20}$/;
  const MAX_FAVS = 6;
  const COOLDOWN_DAYS = 30;

  let avatar = $state("");
  let display = $state("");
  let username = $state("");
  let bio = $state("");
  let links = $state<ProfileLink[]>([]);
  let favorites = $state<Favorite[]>([]);
  let ready = $state(false);
  let busy = $state(false);
  let message = $state("");

  let pool = $state<Favorite[] | null>(null);
  let poolProblem = $state("");

  let nameCheck = $state<"idle" | "same" | "checking" | "free" | "taken" | "bad">("same");
  let checkRun = 0;

  // Fill the form once the profile is known.
  $effect(() => {
    const me = auth.me;
    if (!me || ready) return;
    avatar = me.avatar;
    display = me.display_name;
    username = me.username;
    bio = me.bio;
    links = me.links.map((l) => ({ ...l }));
    favorites = me.favorites.map((f) => ({ ...f }));
    ready = true;
    document.title = "Edit profile · Velmora Social";
    myListFavorites()
      .then((l) => (pool = l))
      .catch((e) => {
        poolProblem = (e as Error).message;
        pool = [];
      });
  });

  const nextChange = $derived.by(() => {
    const at = auth.me?.username_changed_at;
    if (!at) return null;
    const next = new Date(new Date(at).getTime() + COOLDOWN_DAYS * 86400000);
    return next > new Date() ? next : null;
  });

  $effect(() => {
    const name = username;
    const current = auth.me?.username ?? "";
    const run = ++checkRun;
    if (name.toLowerCase() === current.toLowerCase()) {
      nameCheck = "same";
      return;
    }
    if (!NAME.test(name)) {
      nameCheck = /^[A-Za-z0-9_]{0,2}$/.test(name) ? "idle" : "bad";
      return;
    }
    nameCheck = "checking";
    const timer = setTimeout(async () => {
      const free = await usernameFree(name);
      if (run !== checkRun) return;
      nameCheck = free === null ? "idle" : free ? "free" : "taken";
    }, 350);
    return () => clearTimeout(timer);
  });

  const cooldown = $derived(nextChange ? `You can pick a new username after ${nextChange.toLocaleDateString(undefined, { day: "numeric", month: "long" })}.` : "");

  const nameHint = $derived(
    nameCheck !== "same" && nameCheck !== "idle" && cooldown
      ? cooldown
      : nameCheck === "free"
        ? "Yours, if you save now."
        : nameCheck === "taken"
          ? "That name isn't available."
          : nameCheck === "bad"
            ? "3 to 20 letters, numbers or underscores."
            : nameCheck === "checking"
              ? "Checking…"
              : cooldown || "You can change it once every 30 days.",
  );

  const selectValue = (e: Event) => (e.currentTarget as HTMLElement & { value: string }).value;
  const cleanLinks = (): ProfileLink[] => links.filter((l) => l.url.trim()).map((l) => ({ kind: l.kind, url: l.url.trim() }));
  const linksOk = $derived(links.every((l) => !l.url.trim() || validLink({ kind: l.kind, url: l.url.trim() })));

  const patch = $derived.by(() => {
    const me = auth.me;
    if (!me || !ready) return {};
    const out: Record<string, unknown> = {};
    if (avatar !== me.avatar) out.avatar = avatar;
    if (display.trim() !== me.display_name) out.display_name = display.trim();
    if (username !== me.username) out.username = username;
    if (bio.trim() !== me.bio) out.bio = bio.trim();
    const nl = cleanLinks();
    if (JSON.stringify(nl) !== JSON.stringify(me.links)) out.links = nl;
    if (JSON.stringify(favorites) !== JSON.stringify(me.favorites)) out.favorites = favorites;
    return out;
  });

  const dirty = $derived(Object.keys(patch).length > 0);
  const nameOk = $derived(nameCheck === "same" || (nameCheck === "free" && !nextChange));
  const canSave = $derived(dirty && !busy && nameOk && linksOk && !!display.trim() && bio.trim().length <= MAX_BIO);

  const picked = (k: string) => favorites.some((f) => f.k === k);

  function toggleFav(f: Favorite) {
    if (picked(f.k)) favorites = favorites.filter((x) => x.k !== f.k);
    else if (favorites.length < MAX_FAVS) favorites = [...favorites, f];
    else M3eSnackbar.open(`Six is the most. Remove one first.`);
  }

  function moveFav(i: number, by: number) {
    const j = i + by;
    if (j < 0 || j >= favorites.length) return;
    const next = [...favorites];
    [next[i], next[j]] = [next[j], next[i]];
    favorites = next;
  }

  let sharingBusy = $state(false);

  async function share(on: boolean) {
    sharingBusy = true;
    try {
      await setSharing(on);
      M3eSnackbar.open(on ? "Sharing is on. What you watch shows on your profile and in followers' feeds." : "Sharing is off. Your activity is hidden.");
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    } finally {
      sharingBusy = false;
    }
  }

  async function wipe() {
    if (!confirm("Clear all your activity? Ratings, reviews and lists stay.")) return;
    try {
      await clearActivity();
      M3eSnackbar.open("Your activity is cleared.");
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    }
  }

  async function save() {
    if (!canSave) return;
    busy = true;
    message = "";
    try {
      await saveProfile(patch);
      links = auth.me!.links.map((l) => ({ ...l }));
      M3eSnackbar.open("Profile saved.");
      go(`/@${auth.me!.username}`);
    } catch (e) {
      message = (e as Error).message;
      if (message.includes("username")) nameCheck = "taken";
    } finally {
      busy = false;
    }
  }
</script>

{#if auth.phase === "out"}
  <section class="gate" in:fly={{ y: 12, duration: 260 }}>
    <m3e-heading variant="headline" size="medium" level={1}>Sign in to edit your profile</m3e-heading>
    <p>Use the same account as in the Velmora app.</p>
    <m3e-button variant="filled" onclick={() => beginSignIn()}><m3e-icon slot="icon" name="login"></m3e-icon>Sign in</m3e-button>
  </section>
{:else if auth.phase === "noprofile"}
  <section class="gate">
    <m3e-heading variant="headline" size="medium" level={1}>Finish your profile first</m3e-heading>
    <m3e-button variant="filled" onclick={() => beginSignIn()}>Finish my profile</m3e-button>
  </section>
{:else if auth.me && ready}
  <section class="edit" in:fly={{ y: 12, duration: 280 }}>
    <header class="top">
      <m3e-icon-button aria-label="Back to your profile" onclick={() => go(`/@${auth.me!.username}`)}><m3e-icon name="arrow_back"></m3e-icon></m3e-icon-button>
      <m3e-heading variant="headline" size="large" level={1}>Edit profile</m3e-heading>
    </header>

    {#if message}<p class="notice" role="alert"><m3e-icon name="error"></m3e-icon>{message}</p>{/if}

    <div class="card">
      <h2>Picture</h2>
      <div class="pic-row">
        {#key avatar}<span class="current" in:scale={{ start: 0.85, duration: 240 }}><Avatar value={avatar} size={104} ring={4} /></span>{/key}
        <div class="grid" role="radiogroup" aria-label="Profile picture">
          {#if auth.photo}
            <button type="button" class="face" class:on={avatar === auth.photo} role="radio" aria-checked={avatar === auth.photo} aria-label="Your photo" onclick={() => (avatar = auth.photo!)}>
              <Avatar value={auth.photo} size={56} />
            </button>
          {/if}
          {#each ANIMALS as a (a.id)}
            <button type="button" class="face" class:on={avatar === a.id} role="radio" aria-checked={avatar === a.id} aria-label={a.label} title={a.label} style:--tint={a.color} onclick={() => (avatar = a.id)}>
              <Avatar value={a.id} size={56} />
            </button>
          {/each}
        </div>
      </div>
    </div>

    <div class="card">
      <h2>Names</h2>
      <m3e-form-field variant="outlined" class="field">
        <label slot="label" for="ed-display">Display name</label>
        <input id="ed-display" maxlength="40" bind:value={display} disabled={busy} />
        <span slot="hint">{display.length}/40</span>
      </m3e-form-field>
      <m3e-form-field variant="outlined" class="field" error={nameCheck === "taken" || nameCheck === "bad" || (nameCheck === "free" && !!nextChange)}>
        <span slot="prefix" class="at">@</span>
        <label slot="label" for="ed-username">Username</label>
        <input id="ed-username" maxlength="20" autocomplete="off" autocapitalize="off" spellcheck="false" bind:value={username} disabled={busy} />
        <span slot="suffix" class="tail"><NameStatus state={nameCheck} /></span>
        <span slot="error-icon"></span>
        <span slot="hint">{nameHint}</span>
        <span slot="error">{nameHint}</span>
      </m3e-form-field>
    </div>

    <div class="card">
      <h2>About you</h2>
      <m3e-form-field variant="outlined" class="field" error={bio.trim().length > MAX_BIO}>
        <label slot="label" for="ed-bio">Bio</label>
        <textarea id="ed-bio" rows="3" maxlength={MAX_BIO + 20} bind:value={bio} disabled={busy}></textarea>
        <span slot="hint">{bio.trim().length}/{MAX_BIO}</span>
        <span slot="error">{bio.trim().length}/{MAX_BIO}: a little shorter, please.</span>
      </m3e-form-field>

      <fieldset class="links">
        <legend>Links</legend>
        {#each links as l, i (i)}
          {@const bad = !!l.url.trim() && !validLink({ kind: l.kind, url: l.url.trim() })}
          <div class="link-row" transition:fly={{ y: 6, duration: 180 }}>
            <m3e-form-field class="kind" variant="outlined">
              <m3e-select aria-label="Site" value={l.kind} onchange={(e: Event) => (links[i].kind = selectValue(e) as LinkKind)}>
                {#each LINK_KINDS as k (k.kind)}
                  <m3e-option value={k.kind} selected={l.kind === k.kind}>{k.label}</m3e-option>
                {/each}
              </m3e-select>
            </m3e-form-field>
            <m3e-form-field class="url" variant="outlined" error={bad}>
              <input aria-label="{linkLabel(l.kind)} address" placeholder={LINK_KINDS.find((k) => k.kind === l.kind)?.example} bind:value={links[i].url} spellcheck="false" disabled={busy} />
              <span slot="error">That isn't a {linkLabel(l.kind)} address starting with https://</span>
            </m3e-form-field>
            <m3e-icon-button aria-label="Remove link" onclick={() => links.splice(i, 1)}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
          </div>
        {/each}
        {#if links.length < MAX_LINKS}
          <m3e-button variant="text" onclick={() => links.push({ kind: "instagram", url: "" })}>
            <m3e-icon slot="icon" name="add_link"></m3e-icon>Add a link
          </m3e-button>
        {/if}
      </fieldset>
    </div>

    <div class="card">
      <h2>Favourites <span class="sub">{favorites.length}/{MAX_FAVS}</span></h2>
      <p class="help">Shown on your profile in this order. Pick them from My List; anything you add in the app shows up here once it syncs.</p>

      {#if favorites.length}
        <ol class="chosen">
          {#each favorites as f, i (f.k)}
            <li animate:flip={{ duration: 260 }} in:scale={{ start: 0.9, duration: 200 }}>
              <span class="mini-poster">{#if f.poster}<img src={f.poster} alt="" />{:else}<m3e-icon name="movie"></m3e-icon>{/if}</span>
              <span class="chosen-name">{f.name}{#if f.year}<span class="yr"> · {f.year}</span>{/if}</span>
              <span class="chosen-actions">
                <m3e-icon-button size="extra-small" aria-label="Move up" disabled={i === 0} onclick={() => moveFav(i, -1)}><m3e-icon name="arrow_upward"></m3e-icon></m3e-icon-button>
                <m3e-icon-button size="extra-small" aria-label="Move down" disabled={i === favorites.length - 1} onclick={() => moveFav(i, 1)}><m3e-icon name="arrow_downward"></m3e-icon></m3e-icon-button>
                <m3e-icon-button size="extra-small" aria-label="Remove {f.name}" onclick={() => toggleFav(f)}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
              </span>
            </li>
          {/each}
        </ol>
      {/if}

      {#if pool === null}
        <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
      {:else if poolProblem}
        <p class="help" role="alert">{poolProblem}</p>
      {:else if !pool.length}
        <p class="help">Your My List is empty, or hasn't synced yet. Add titles in the Velmora app and they'll appear here.</p>
      {:else}
        <div class="pool">
          {#each pool as f (f.k)}
            {@const on = picked(f.k)}
            <button type="button" class="pick" class:on aria-pressed={on} title={f.name} onclick={() => toggleFav(f)}>
              <span class="pick-poster">
                {#if f.poster}<img src={f.poster} alt="" loading="lazy" />{:else}<m3e-icon name="movie"></m3e-icon>{/if}
                {#if on}<span class="tick" in:scale={{ start: 0.5, duration: 200 }}><m3e-icon name="check" filled></m3e-icon></span>{/if}
              </span>
              <span class="pick-name">{f.name}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="card">
      <h2>Activity</h2>
      <label class="share">
        <span class="share-text">
          <strong>Share what I watch</strong>
          <span>Titles you start, finish, add to My List and rate show on your profile, in your followers' feeds and in your stats. 18+ titles are never shared.</span>
        </span>
        <m3e-switch checked={auth.me.share_activity !== false} disabled={sharingBusy} onchange={(e: Event) => share((e.currentTarget as HTMLElement & { checked: boolean }).checked)}></m3e-switch>
      </label>
      <div><m3e-button variant="text" onclick={wipe}><m3e-icon slot="icon" name="delete_sweep"></m3e-icon>Clear my activity</m3e-button></div>
    </div>

    <div class="savebar" class:show={dirty}>
      <span>{dirty ? "You have changes that aren't saved." : "Everything is saved."}</span>
      <m3e-button variant="text" disabled={busy} onclick={() => go(`/@${auth.me!.username}`)}>Cancel</m3e-button>
      <m3e-button variant="filled" disabled={!canSave} onclick={save}><m3e-icon slot="icon" name="check"></m3e-icon>{busy ? "Saving…" : "Save"}</m3e-button>
    </div>
  </section>
{:else}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{/if}

<style>
  .edit {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding-bottom: 90px;
  }
  .top {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .notice {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    padding: 12px 16px;
    border-radius: 16px;
    background: var(--md-sys-color-error-container);
    color: var(--md-sys-color-on-error-container);
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 22px clamp(16px, 3vw, 26px);
    border-radius: 28px;
    background: var(--md-sys-color-surface-container);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 35%, transparent);
  }
  .card h2 {
    margin: 0;
    font-size: 18px;
    display: flex;
    align-items: baseline;
    gap: 10px;
  }
  .sub {
    font-size: 14px;
    font-weight: 500;
    color: var(--md-sys-color-on-surface-variant);
  }
  .help {
    margin: 0;
    font-size: 14px;
    line-height: 1.5;
    color: var(--md-sys-color-on-surface-variant);
  }
  .field {
    width: 100%;
  }
  .at {
    color: var(--md-sys-color-on-surface-variant);
    padding-left: 4px;
  }
  .tail {
    display: inline-grid;
    padding-right: 6px;
  }
  .pic-row {
    display: flex;
    align-items: center;
    gap: 24px;
    flex-wrap: wrap;
  }
  .current {
    display: inline-grid;
  }
  .grid {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    flex: 1;
    min-width: 240px;
  }
  .face {
    padding: 3px;
    border: 0;
    border-radius: 50%;
    background: transparent;
    cursor: pointer;
    outline: 2px solid transparent;
    outline-offset: 1px;
    transition:
      transform 200ms cubic-bezier(0.2, 0.8, 0.2, 1.4),
      outline-color 160ms ease;
  }
  .face:hover {
    transform: scale(1.08);
  }
  .face.on {
    outline-color: var(--tint, var(--md-sys-color-primary));
    transform: scale(1.04);
  }
  .face:focus-visible {
    outline-color: var(--md-sys-color-primary);
  }
  .links {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 0;
    padding: 0;
    border: 0;
  }
  .links legend {
    padding: 0 0 8px;
    font-weight: 600;
  }
  .link-row {
    display: flex;
    gap: 8px;
    align-items: flex-start;
  }
  .kind {
    width: 160px;
    flex: none;
  }
  .url {
    flex: 1;
    min-width: 0;
  }
  .chosen {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .chosen li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 8px;
    border-radius: 16px;
    background: var(--md-sys-color-surface-container-high);
  }
  .mini-poster {
    display: grid;
    place-items: center;
    width: 34px;
    height: 50px;
    flex: none;
    border-radius: 8px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
    --m3e-icon-size: 18px;
  }
  .mini-poster img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .chosen-name {
    flex: 1;
    min-width: 0;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .yr {
    font-weight: 400;
    color: var(--md-sys-color-on-surface-variant);
  }
  .chosen-actions {
    display: flex;
    flex: none;
  }
  .pool {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
    gap: 12px;
    max-height: 520px;
    overflow-y: auto;
    padding: 4px;
  }
  .pick {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 0;
    border: 0;
    background: none;
    color: inherit;
    text-align: left;
    cursor: pointer;
    font: inherit;
  }
  .pick-poster {
    position: relative;
    display: grid;
    place-items: center;
    aspect-ratio: 2 / 3;
    border-radius: 14px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
    outline: 3px solid transparent;
    outline-offset: -3px;
    transition:
      outline-color 160ms ease,
      border-radius 240ms ease,
      transform 240ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .pick-poster img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .pick:hover .pick-poster {
    transform: translateY(-2px);
  }
  .pick.on .pick-poster {
    outline-color: var(--md-sys-color-primary);
    border-radius: 22px;
  }
  .pick:focus-visible .pick-poster {
    outline-color: var(--md-sys-color-tertiary);
  }
  .tick {
    position: absolute;
    top: 8px;
    right: 8px;
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 18px;
  }
  .pick-name {
    font-size: 12px;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .share {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .share-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .share-text span {
    font-size: 14px;
    line-height: 1.5;
    color: var(--md-sys-color-on-surface-variant);
  }
  .savebar {
    position: fixed;
    left: 50%;
    bottom: 20px;
    z-index: 20;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 10px 10px 20px;
    width: max-content;
    max-width: calc(100% - 32px);
    border-radius: 999px;
    background: var(--md-sys-color-surface-container-highest);
    box-shadow: 0 20px 50px -20px rgb(0 0 0 / 80%);
    transform: translate(-50%, 140%);
    transition: transform 360ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .savebar.show {
    transform: translate(-50%, 0);
  }
  .savebar span {
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
    margin-right: 6px;
  }
  .gate {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 64px 16px;
    text-align: center;
  }
  .gate p {
    margin: 0;
    color: var(--md-sys-color-on-surface-variant);
  }
  .center {
    display: grid;
    place-items: center;
    padding: 32px 0;
  }
  @media (max-width: 560px) {
    .link-row {
      flex-wrap: wrap;
    }
    .kind {
      width: calc(100% - 48px);
    }
    .url {
      flex-basis: 100%;
      order: 3;
    }
    .savebar span {
      display: none;
    }
    .savebar {
      bottom: 82px;
    }
  }
</style>
