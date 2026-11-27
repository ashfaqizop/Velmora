<script lang="ts">
  // Everything an admin did, newest first. Nothing here can be edited.
  import { onMount } from "svelte";
  import { call, sayError } from "./api";
  import { when } from "./format";

  interface Entry { id: number; admin_id: string | null; admin_name: string | null; action: string; target: string | null; target_name: string | null; details: Record<string, unknown>; at: string }

  let rows = $state<Entry[]>([]);
  let page = $state(0);
  let size = $state(30);
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      const r = await call<{ rows: Entry[]; size: number }>("audit", { page });
      rows = r.rows;
      size = r.size;
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  const detail = (d: Record<string, unknown>) =>
    Object.entries(d)
      .filter(([, v]) => v !== "" && v !== null && !(Array.isArray(v) && !v.length))
      .map(([k, v]) => `${k}: ${Array.isArray(v) ? v.join(", ") : v}`)
      .join(" · ");
</script>

<div class="adm-page">
  <div class="adm-head">
    <div class="grow">
      <h1>Audit log</h1>
      <p>Every change made on this site, who made it and what it touched.</p>
    </div>
  </div>

  {#if loading && !rows.length}
    <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
  {:else if !rows.length}
    <div class="card empty">Nothing has been done yet.</div>
  {:else}
    <div class="card list" class:busy={loading}>
      {#each rows as r (r.id)}
        <div class="entry">
          <span class="chip">{r.action}</span>
          <div class="what">
            <span>{r.target_name ? `@${r.target_name}` : (r.target ?? "—")}{#if r.admin_name} <span class="muted">by @{r.admin_name}</span>{/if}</span>
            {#if detail(r.details)}<span class="muted det">{detail(r.details)}</span>{/if}
          </div>
          <span class="muted at">{when(r.at)}</span>
        </div>
      {/each}
    </div>
    <div class="pager">
      <m3e-button variant="text" disabled={page === 0 || loading} onclick={() => { page--; void load(); }}><m3e-icon slot="icon" name="arrow_back"></m3e-icon>Newer</m3e-button>
      <m3e-button variant="text" disabled={rows.length < size || loading} onclick={() => { page++; void load(); }}>Older<m3e-icon slot="trailing-icon" name="arrow_forward"></m3e-icon></m3e-button>
    </div>
  {/if}
</div>

<style>
  .list {
    padding: 8px 12px;
    transition: opacity 160ms ease;
  }
  .list.busy {
    opacity: 0.6;
  }
  .entry {
    display: grid;
    grid-template-columns: 150px minmax(0, 1fr) auto;
    gap: 12px;
    align-items: start;
    padding: 12px 4px;
    font-size: 14px;
  }
  .entry + .entry {
    border-top: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 35%, transparent);
  }
  .entry .chip {
    justify-self: start;
  }
  .what {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .det {
    font-size: 12.5px;
    word-break: break-word;
  }
  .at {
    font-size: 12.5px;
    white-space: nowrap;
  }
  .pager {
    display: flex;
    justify-content: space-between;
  }
  @media (max-width: 700px) {
    .entry {
      grid-template-columns: minmax(0, 1fr);
      gap: 4px;
    }
  }
</style>
