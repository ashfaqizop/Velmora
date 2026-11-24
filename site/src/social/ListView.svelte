<script lang="ts">
  // A list of titles. Anyone can view a public list; its owner can edit it.
  // /list/new makes one. Titles are picked from your My List.
  import { fade, fly, scale } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import Avatar from "../auth/Avatar.svelte";
  import { beginSignIn } from "../lib/session";
  import Badges from "./Badges.svelte";
  import { auth, myListFavorites, type Favorite } from "./data.svelte";
  import { ago } from "./format";
  import { go, href, profileHref, titleHref } from "./nav.svelte";
  import { deleteList, getList, saveList, type TitleList } from "./activity.svelte";

  let { id }: { id: string } = $props();

  const MAX = 100;
  let list = $state<TitleList | null>(null);
  let phase = $state<"loading" | "ready" | "missing">("loading");
  let editing = $state(false);
  let busy = $state(false);

  let name = $state("");
  let description = $state("");
  let items = $state<Favorite[]>([]);
  let isPublic = $state(true);
  let pool = $state<Favorite[] | null>(null);
  let filter = $state("");

  const isNew = $derived(id === "new");
  const mine = $derived(isNew || (!!auth.me && list?.user_id === auth.me.id));

  $effect(() => {
    const which = id;
    if (which === "new") {
      list = null;
      name = "";
      description = "";
      items = [];
      isPublic = true;
      editing = true;
      phase = "ready";
      document.title = "New list · Velmora Social";
      return;
    }
    phase = "loading";
    getList(which)
      .then((l) => {
        if (!l) {
          phase = "missing";
          return;
        }
        list = l;
        phase = "ready";
        document.title = `${l.name} · Velmora Social`;
      })
      .catch(() => (phase = "missing"));
  });

  $effect(() => {
    if (editing && auth.me && pool === null) myListFavorites().then((p) => (pool = p)).catch(() => (pool = []));
  });

  function startEdit() {
    if (!list) return;
    name = list.name;
    description = list.description;
    items = list.items.map((i) => ({ ...i }));
    isPublic = list.public;
    editing = true;
  }

  const has = (k: string) => items.some((i) => i.k === k);
  function toggle(f: Favorite) {
    if (has(f.k)) items = items.filter((i) => i.k !== f.k);
    else if (items.length < MAX) items = [...items, f];
    else M3eSnackbar.open("A list holds 100 titles at most.");
  }
  function move(i: number, by: number) {
    const j = i + by;
    if (j < 0 || j >= items.length) return;
    const next = [...items];
    [next[i], next[j]] = [next[j], next[i]];
    items = next;
  }

  const shownPool = $derived((pool ?? []).filter((f) => !filter.trim() || f.name.toLowerCase().includes(filter.trim().toLowerCase())));

  async function save() {
    if (!name.trim()) return;
    busy = true;
    try {
      const saved = await saveList({ id: isNew ? undefined : id, name, description, items, public: isPublic });
      editing = false;
      M3eSnackbar.open("List saved.");
      if (isNew) go(`/list/${saved}`, { replace: true });
      else list = await getList(id);
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    } finally {
      busy = false;
    }
  }

  async function remove() {
    if (!list || !confirm(`Delete "${list.name}"? This can't be undone.`)) return;
    busy = true;
    try {
      await deleteList(list.id);
      M3eSnackbar.open("List deleted.");
      go(`/@${auth.me!.username}/lists`);
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    } finally {
      busy = false;
    }
  }

  async function share() {
    const url = `https://social.getvelmora.dpdns.org/list/${id}`;
    try {
      await navigator.clipboard.writeText(url);
      M3eSnackbar.open("Link copied.");
    } catch {
      M3eSnackbar.open(url);
    }
  }
</script>

{#if phase === "loading"}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else if phase === "missing"}
  <section class="gone">
    <m3e-heading variant="headline" size="medium" level={1}>This list isn't here</m3e-heading>
    <p>It may be private, or it was deleted.</p>
    <m3e-button variant="filled" href={href("/social/")}>Go home</m3e-button>
  </section>
{:else if isNew && !auth.me}
  <section class="gone">
    <m3e-heading variant="headline" size="medium" level={1}>Sign in to make a list</m3e-heading>
    <m3e-button variant="filled" onclick={() => beginSignIn()}><m3e-icon slot="icon" name="login"></m3e-icon>Sign in</m3e-button>
  </section>
{:else if editing}
  <section class="edit" in:fade={{ duration: 200 }}>
    <m3e-heading variant="headline" size="large" level={1}>{isNew ? "New list" : "Edit list"}</m3e-heading>
    <div class="card">
      <m3e-form-field variant="outlined" class="field">
        <label slot="label" for="l-name">Name</label>
        <input id="l-name" maxlength="60" bind:value={name} disabled={busy} />
      </m3e-form-field>
      <m3e-form-field variant="outlined" class="field">
        <label slot="label" for="l-desc">Description (optional)</label>
        <textarea id="l-desc" rows="2" maxlength="300" bind:value={description} disabled={busy}></textarea>
        <span slot="hint">{description.length}/300</span>
      </m3e-form-field>
      <label class="switch-row">
        <m3e-switch checked={isPublic} onchange={(e: Event) => (isPublic = (e.currentTarget as HTMLElement & { checked: boolean }).checked)}></m3e-switch>
        <span><strong>Public</strong><br /><small>{isPublic ? "Anyone can see it on your profile." : "Only you can see it."}</small></span>
      </label>
    </div>

    <div class="card">
      <h2>In this list <span class="muted">{items.length}/{MAX}</span></h2>
      {#if !items.length}<p class="muted">Pick titles below.</p>{/if}
      <ol class="chosen">
        {#each items as f, i (f.k)}
          <li animate:flip={{ duration: 240 }} in:scale={{ start: 0.92, duration: 200 }}>
            <span class="n">{i + 1}</span>
            <span class="mini">{#if f.poster}<img src={f.poster} alt="" />{/if}</span>
            <span class="nm">{f.name}{#if f.year}<span class="muted"> · {f.year}</span>{/if}</span>
            <m3e-icon-button size="extra-small" aria-label="Move up" disabled={i === 0} onclick={() => move(i, -1)}><m3e-icon name="arrow_upward"></m3e-icon></m3e-icon-button>
            <m3e-icon-button size="extra-small" aria-label="Move down" disabled={i === items.length - 1} onclick={() => move(i, 1)}><m3e-icon name="arrow_downward"></m3e-icon></m3e-icon-button>
            <m3e-icon-button size="extra-small" aria-label="Remove" onclick={() => toggle(f)}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
          </li>
        {/each}
      </ol>
    </div>

    <div class="card">
      <h2>From My List</h2>
      <input class="filter" type="search" placeholder="Filter" bind:value={filter} aria-label="Filter My List" />
      {#if pool === null}
        <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
      {:else if !pool.length}
        <p class="muted">Your My List is empty or hasn't synced yet. Add titles in the app. You can also add a title to a list from its page here.</p>
      {:else}
        <div class="pool">
          {#each shownPool as f (f.k)}
            {@const on = has(f.k)}
            <button type="button" class="pick" class:on aria-pressed={on} title={f.name} onclick={() => toggle(f)}>
              <span class="pp">{#if f.poster}<img src={f.poster} alt="" loading="lazy" />{/if}{#if on}<span class="tick"><m3e-icon name="check"></m3e-icon></span>{/if}</span>
              <span class="pn">{f.name}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="actions">
      <m3e-button variant="text" disabled={busy} onclick={() => (isNew ? history.back() : (editing = false))}>Cancel</m3e-button>
      <m3e-button variant="filled" disabled={busy || !name.trim()} onclick={save}><m3e-icon slot="icon" name="check"></m3e-icon>{busy ? "Saving…" : "Save list"}</m3e-button>
    </div>
  </section>
{:else if list}
  <section class="view" in:fly={{ y: 12, duration: 280 }}>
    <header class="top">
      <span class="eyebrow">{list.public ? "List" : "Private list"} · {list.items.length} {list.items.length === 1 ? "title" : "titles"}</span>
      <h1 class="display">{list.name}</h1>
      {#if list.description}<p class="desc">{list.description}</p>{/if}
      {#if list.p}
        <a class="by" href={profileHref(list.p.username)}>
          <Avatar value={list.p.avatar} size={28} />
          <span>{list.p.display_name}</span><Badges id={list.p.id} verified={list.p.verified} size={14} />
          <span class="muted">· updated {ago(list.updated_at)}</span>
        </a>
      {/if}
      <div class="row">
        {#if mine}
          <m3e-button variant="tonal" onclick={startEdit}><m3e-icon slot="icon" name="edit"></m3e-icon>Edit</m3e-button>
          <m3e-button variant="text" disabled={busy} onclick={remove}><m3e-icon slot="icon" name="delete"></m3e-icon>Delete</m3e-button>
        {/if}
        {#if list.public}<m3e-icon-button variant="tonal" aria-label="Copy link" onclick={share}><m3e-icon name="share"></m3e-icon></m3e-icon-button>{/if}
      </div>
    </header>
    {#if !list.items.length}
      <p class="muted center-text">Nothing in this list yet.</p>
    {:else}
      <div class="grid">
        {#each list.items as f, i (f.k)}
          <a class="tile" href={titleHref(f.k)} style:--i={i}>
            <span class="poster">{#if f.poster}<img src={f.poster} alt="" loading="lazy" />{:else}<m3e-icon name="movie"></m3e-icon>{/if}<span class="num">{i + 1}</span></span>
            <span class="tn">{f.name}</span>
            {#if f.year}<span class="ty">{f.year}</span>{/if}
          </a>
        {/each}
      </div>
    {/if}
  </section>
{/if}

<style>
  .center {
    display: grid;
    place-items: center;
    padding: 40px 0;
  }
  .gone {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 64px 16px;
    text-align: center;
  }
  .gone p,
  .muted {
    margin: 0;
    color: var(--md-sys-color-on-surface-variant);
    font-weight: 400;
  }
  .center-text {
    text-align: center;
    padding: 32px 0;
  }
  .edit,
  .view {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 22px clamp(16px, 3vw, 26px);
    border-radius: 28px;
    background: var(--md-sys-color-surface-container);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 35%, transparent);
  }
  .card h2 {
    margin: 0;
    font-size: 18px;
  }
  .field {
    width: 100%;
  }
  .switch-row {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .switch-row small {
    color: var(--md-sys-color-on-surface-variant);
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
    gap: 10px;
    padding: 6px 8px;
    border-radius: 16px;
    background: var(--md-sys-color-surface-container-high);
  }
  .n {
    width: 22px;
    text-align: right;
    color: var(--md-sys-color-on-surface-variant);
    font-variant-numeric: tabular-nums;
  }
  .mini {
    width: 32px;
    height: 48px;
    flex: none;
    border-radius: 8px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
  }
  .mini img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .nm {
    flex: 1;
    min-width: 0;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .filter {
    height: 42px;
    padding: 0 16px;
    border: 0;
    border-radius: 999px;
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface);
    font: inherit;
    outline: none;
  }
  .filter:focus {
    box-shadow: 0 0 0 2px var(--md-sys-color-primary);
  }
  .pool {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(92px, 1fr));
    gap: 12px;
    max-height: 480px;
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
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .pp {
    position: relative;
    aspect-ratio: 2 / 3;
    border-radius: 14px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
    outline: 3px solid transparent;
    outline-offset: -3px;
    transition:
      outline-color 160ms ease,
      border-radius 220ms ease;
  }
  .pp img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .pick.on .pp {
    outline-color: var(--md-sys-color-primary);
    border-radius: 22px;
  }
  .tick {
    position: absolute;
    top: 6px;
    right: 6px;
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 18px;
  }
  .pn {
    font-size: 12px;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .actions {
    position: sticky;
    bottom: 16px;
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 10px;
    border-radius: 999px;
    background: var(--md-sys-color-surface-container-highest);
    box-shadow: 0 20px 50px -20px rgb(0 0 0 / 80%);
    align-self: flex-end;
  }
  .top {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .eyebrow {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
  }
  .display {
    margin: 0;
    font-family: var(--display);
    font-weight: 400;
    font-size: clamp(40px, 6vw, 64px);
    line-height: 1;
    overflow-wrap: anywhere;
  }
  .desc {
    margin: 0;
    max-width: 64ch;
    line-height: 1.6;
    white-space: pre-line;
  }
  .by {
    display: flex;
    align-items: center;
    gap: 8px;
    color: inherit;
    text-decoration: none;
    font-weight: 600;
  }
  .row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 20px 16px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: 6px;
    color: inherit;
    text-decoration: none;
    animation: rise 460ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
    animation-delay: calc(min(var(--i), 16) * 35ms);
  }
  .poster {
    position: relative;
    display: grid;
    place-items: center;
    aspect-ratio: 2 / 3;
    border-radius: 18px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
    transition:
      transform 300ms cubic-bezier(0.2, 0.8, 0.2, 1),
      border-radius 300ms ease;
  }
  .poster img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .num {
    position: absolute;
    left: 8px;
    bottom: 2px;
    font-family: var(--display);
    font-size: 40px;
    color: #fff;
    text-shadow: 0 4px 16px rgb(0 0 0 / 80%);
  }
  .tile:hover .poster {
    transform: translateY(-4px);
    border-radius: 26px;
  }
  .tn {
    font-weight: 600;
    font-size: 14px;
    line-height: 1.3;
  }
  .ty {
    margin-top: -4px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
</style>
