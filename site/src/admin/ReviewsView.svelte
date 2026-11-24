<script lang="ts">
  // Every written review, newest first. Remove the words and keep the score,
  // or delete the rating altogether.
  import { onMount } from "svelte";
  import Avatar from "../auth/Avatar.svelte";
  import { call, say, sayError } from "./api";
  import { ago } from "./format";

  interface Review {
    user_id: string;
    k: string;
    score: number;
    review: string;
    spoiler: boolean;
    updated_at: string;
    p: { id: string; username: string; display_name: string; avatar: string } | null;
  }

  let q = $state("");
  let rows = $state<Review[]>([]);
  let page = $state(0);
  let size = $state(30);
  let loading = $state(true);
  let asking = $state<string | null>(null);
  let busy = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;

  const key = (r: Review) => `${r.user_id}:${r.k}`;

  async function load() {
    const mine = ++seq;
    loading = true;
    try {
      const r = await call<{ rows: Review[]; size: number }>("reviews", { q: q.trim(), page });
      if (mine !== seq) return;
      rows = r.rows;
      size = r.size;
    } catch (e) {
      if (mine === seq) sayError(e);
    } finally {
      if (mine === seq) loading = false;
    }
  }
  onMount(() => {
    void load();
    return () => clearTimeout(timer);
  });

  function typing() {
    clearTimeout(timer);
    timer = setTimeout(() => {
      page = 0;
      void load();
    }, 300);
  }

  async function act(r: Review, op: "clear" | "delete") {
    busy = true;
    try {
      await call("reviewSet", { user_id: r.user_id, k: r.k, op });
      say(op === "clear" ? "Review text removed." : "Rating deleted.");
      rows = rows.filter((x) => key(x) !== key(r));
      asking = null;
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="adm-page">
  <div class="adm-head">
    <div class="grow">
      <h1>Reviews</h1>
      <p>Search by @username or a title key such as <span class="mono">movie/603</span>. Removing the text leaves the person's score alone.</p>
    </div>
    <label class="field find">
      <m3e-icon name="search"></m3e-icon>
      <input type="search" bind:value={q} oninput={typing} placeholder="@username or movie/603" aria-label="Search reviews" autocomplete="off" spellcheck="false" />
    </label>
  </div>

  {#if loading && !rows.length}
    <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
  {:else if !rows.length}
    <div class="card empty">{q ? "No reviews match." : "No reviews yet."}</div>
  {:else}
    <div class="list">
      {#each rows as r (key(r))}
        <article class="card rev">
          <header>
            {#if r.p}<Avatar value={r.p.avatar} size={32} /><strong>{r.p.display_name}</strong><span class="muted">@{r.p.username}</span>{/if}
            <a class="chip info" href="https://social.getvelmora.dpdns.org/title/{r.k}" target="_blank" rel="noopener">{r.k}</a>
            <span class="chip">{r.score}/10</span>
            {#if r.spoiler}<span class="chip warn">Spoiler</span>{/if}
            <span class="muted when">{ago(r.updated_at)}</span>
          </header>
          <p class="text">{r.review}</p>
          {#if asking === key(r)}
            <div class="row-actions">
              <span class="muted">Remove the words, or the whole rating?</span>
              <m3e-button variant="filled" disabled={busy} onclick={() => act(r, "clear")}>Remove text</m3e-button>
              <m3e-button variant="outlined" disabled={busy} onclick={() => act(r, "delete")}>Delete rating</m3e-button>
              <m3e-button variant="text" disabled={busy} onclick={() => (asking = null)}>Cancel</m3e-button>
            </div>
          {:else}
            <div class="row-actions">
              <m3e-button variant="tonal" onclick={() => (asking = key(r))}><m3e-icon slot="icon" name="delete"></m3e-icon>Remove…</m3e-button>
            </div>
          {/if}
        </article>
      {/each}
    </div>
    <div class="pager">
      <m3e-button variant="text" disabled={page === 0 || loading} onclick={() => { page--; void load(); }}><m3e-icon slot="icon" name="arrow_back"></m3e-icon>Newer</m3e-button>
      <m3e-button variant="text" disabled={rows.length < size || loading} onclick={() => { page++; void load(); }}>Older<m3e-icon slot="trailing-icon" name="arrow_forward"></m3e-icon></m3e-button>
    </div>
  {/if}
</div>

<style>
  .find {
    width: min(340px, 100%);
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .rev {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  header {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  header a {
    text-decoration: none;
  }
  .when {
    margin-left: auto;
    font-size: 12.5px;
  }
  .text {
    margin: 0;
    font-size: 14.5px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .pager {
    display: flex;
    justify-content: space-between;
  }
</style>
