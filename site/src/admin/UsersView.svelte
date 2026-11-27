<script lang="ts">
  // Find an account by name, username, email or id; open it for actions.
  import { onMount, untrack } from "svelte";
  import Avatar from "../auth/Avatar.svelte";
  import { call, sayError } from "./api";
  import { ago, count } from "./format";
  import { go, route } from "./route.svelte";
  import UserPanel from "./UserPanel.svelte";

  interface Row {
    id: string;
    email: string | null;
    username: string | null;
    display_name: string | null;
    avatar: string | null;
    verified: boolean;
    hidden: boolean;
    banned: boolean;
    followers_count: number;
    created_at: string;
    last_sign_in_at: string | null;
    reports_open: number;
    total: number;
  }

  let q = $state("");
  let page = $state(0);
  let rows = $state<Row[]>([]);
  let total = $state(0);
  let size = $state(30);
  let loading = $state(true);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;

  async function load() {
    const mine = ++seq;
    loading = true;
    try {
      const r = await call<{ rows: Row[]; total: number; size: number }>("users", { q: q.trim(), page });
      if (mine !== seq) return;
      rows = r.rows;
      total = r.total;
      size = r.size;
    } catch (e) {
      if (mine === seq) sayError(e);
    } finally {
      if (mine === seq) loading = false;
    }
  }

  function typing() {
    clearTimeout(timer);
    timer = setTimeout(() => {
      page = 0;
      void load();
    }, 300);
  }

  function turn(by: number) {
    page += by;
    void load();
  }

  onMount(() => {
    untrack(load);
    return () => clearTimeout(timer);
  });

  const pages = $derived(Math.max(1, Math.ceil(total / size)));
</script>

<div class="adm-page">
  <div class="adm-head">
    <div class="grow">
      <h1>People</h1>
      <p>Every account, newest first. Search by name, @username, email or account id.</p>
    </div>
    <label class="field find">
      <m3e-icon name="search"></m3e-icon>
      <input type="search" bind:value={q} oninput={typing} placeholder="Search people" aria-label="Search people" autocomplete="off" spellcheck="false" />
    </label>
  </div>

  <div class="card list" class:busy={loading}>
    {#if !rows.length && !loading}
      <div class="empty">{q ? "No one matches that." : "No accounts yet."}</div>
    {:else}
      {#each rows as r (r.id)}
        <button class="person" onclick={() => go("users", r.id)}>
          <Avatar value={r.avatar ?? "animal:panda"} size={40} />
          <span class="who">
            <strong>{r.display_name ?? "No profile yet"}</strong>
            <span class="muted">{r.username ? `@${r.username}` : "—"} · {r.email ?? "no email"}</span>
          </span>
          <span class="flags">
            {#if r.reports_open}<span class="chip warn"><m3e-icon name="flag"></m3e-icon>{r.reports_open}</span>{/if}
            {#if r.banned}<span class="chip bad">Banned</span>{:else if r.hidden}<span class="chip warn">Hidden</span>{/if}
            {#if r.verified}<span class="chip info"><m3e-icon name="verified"></m3e-icon>Verified</span>{/if}
          </span>
          <span class="when muted">
            <span>Joined {ago(r.created_at)}</span>
            <span>Seen {ago(r.last_sign_in_at)}</span>
          </span>
        </button>
      {/each}
    {/if}
  </div>

  <div class="pager">
    <m3e-button variant="text" disabled={page === 0 || loading} onclick={() => turn(-1)}><m3e-icon slot="icon" name="arrow_back"></m3e-icon>Newer</m3e-button>
    <span class="muted">{count(total)} {total === 1 ? "account" : "accounts"} · page {page + 1} of {pages}</span>
    <m3e-button variant="text" disabled={page + 1 >= pages || loading} onclick={() => turn(1)}>Older<m3e-icon slot="trailing-icon" name="arrow_forward"></m3e-icon></m3e-button>
  </div>
</div>

{#if route.id}
  <UserPanel id={route.id} onclose={() => go("users")} onchange={load} />
{/if}

<style>
  .find {
    width: min(340px, 100%);
  }
  .list {
    padding: 8px;
    display: flex;
    flex-direction: column;
    transition: opacity 160ms ease;
  }
  .list.busy {
    opacity: 0.6;
  }
  .person {
    display: grid;
    grid-template-columns: 40px minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border: 0;
    border-radius: 16px;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: background-color 140ms ease;
  }
  .person:hover,
  .person:focus-visible {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
    outline: 0;
  }
  .who {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .who strong {
    font-size: 15px;
  }
  .who .muted {
    font-size: 12.5px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .flags {
    display: flex;
    gap: 6px;
  }
  .when {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    font-size: 12px;
    min-width: 110px;
  }
  .pager {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    font-size: 13px;
  }
  @media (max-width: 700px) {
    .person {
      grid-template-columns: 40px minmax(0, 1fr);
    }
    .flags,
    .when {
      display: none;
    }
  }
</style>
