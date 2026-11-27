<script lang="ts">
  // Reports people filed against profiles. Reporters stay private to everyone
  // but you; the person reported never learns who it was.
  import { onMount } from "svelte";
  import Avatar from "../auth/Avatar.svelte";
  import { call, say, sayError } from "./api";
  import { ago } from "./format";
  import { go } from "./route.svelte";

  interface Who { id: string; username: string; display_name: string; avatar: string; verified: boolean; hidden: boolean; banned: boolean; bio?: string }
  interface Report {
    id: number;
    reason: string;
    note: string;
    status: "open" | "dismissed" | "actioned";
    created_at: string;
    rep: Who | null;
    tgt: Who | null;
    target_open: number;
  }

  const TABS = [
    { id: "open", label: "Open" },
    { id: "actioned", label: "Actioned" },
    { id: "dismissed", label: "Dismissed" },
  ] as const;
  const REASONS: Record<string, string> = { username: "Username", display_name: "Display name", bio: "Bio", links: "Links", avatar: "Picture", review: "Review", spam: "Spam or fake", other: "Something else" };

  let tab = $state<(typeof TABS)[number]["id"]>("open");
  let rows = $state<Report[]>([]);
  let page = $state(0);
  let size = $state(30);
  let loading = $state(true);
  let busy = $state(0);
  let seq = 0;

  async function load() {
    const mine = ++seq;
    loading = true;
    try {
      const r = await call<{ rows: Report[]; size: number }>("reports", { status: tab, page });
      if (mine !== seq) return;
      rows = r.rows;
      size = r.size;
    } catch (e) {
      if (mine === seq) sayError(e);
    } finally {
      if (mine === seq) loading = false;
    }
  }
  onMount(load);

  function pick(t: typeof tab) {
    tab = t;
    page = 0;
    void load();
  }

  async function mark(r: Report, status: Report["status"]) {
    busy = r.id;
    try {
      await call("reportSet", { id: r.id, status });
      say(status === "open" ? "Reopened." : status === "dismissed" ? "Dismissed." : "Marked as actioned.");
      rows = rows.filter((x) => x.id !== r.id);
    } catch (e) {
      sayError(e);
    } finally {
      busy = 0;
    }
  }
</script>

<div class="adm-page">
  <div class="adm-head">
    <div class="grow">
      <h1>Reports</h1>
      <p>Oldest first, so nothing waits forever. Open the account to hide, ban or reset it, then mark the report actioned.</p>
    </div>
  </div>

  <m3e-filter-chip-set aria-label="Status">
    {#each TABS as t (t.id)}
      <m3e-filter-chip selected={tab === t.id} onclick={() => pick(t.id)}>{t.label}</m3e-filter-chip>
    {/each}
  </m3e-filter-chip-set>

  {#if loading && !rows.length}
    <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
  {:else if !rows.length}
    <div class="card empty">{tab === "open" ? "No open reports. Nice." : "Nothing here."}</div>
  {:else}
    <div class="list">
      {#each rows as r (r.id)}
        <article class="card rep">
          <header>
            <span class="chip warn">{REASONS[r.reason] ?? r.reason}</span>
            {#if r.target_open > 1}<span class="chip bad">{r.target_open} open against this account</span>{/if}
            <span class="muted when">{ago(r.created_at)}</span>
          </header>

          {#if r.tgt}
            <div class="target">
              <Avatar value={r.tgt.avatar} size={44} />
              <div>
                <strong>{r.tgt.display_name}</strong>
                <span class="muted">@{r.tgt.username}</span>
                {#if r.tgt.banned}<span class="chip bad">Banned</span>{:else if r.tgt.hidden}<span class="chip warn">Hidden</span>{/if}
                {#if r.tgt.bio}<p class="bio">{r.tgt.bio}</p>{/if}
              </div>
            </div>
          {:else}
            <p class="muted">The reported account no longer exists.</p>
          {/if}

          {#if r.note}<blockquote>{r.note}</blockquote>{/if}
          <p class="muted from">Reported by {r.rep ? `@${r.rep.username}` : "an account that was deleted"}</p>

          <div class="row-actions">
            {#if r.tgt}<m3e-button variant="tonal" onclick={() => go("users", r.tgt!.id)}><m3e-icon slot="icon" name="manage_accounts"></m3e-icon>Open account</m3e-button>{/if}
            {#if r.status === "open"}
              <m3e-button variant="filled" disabled={busy === r.id} onclick={() => mark(r, "actioned")}><m3e-icon slot="icon" name="task_alt"></m3e-icon>Actioned</m3e-button>
              <m3e-button variant="text" disabled={busy === r.id} onclick={() => mark(r, "dismissed")}>Dismiss</m3e-button>
            {:else}
              <m3e-button variant="text" disabled={busy === r.id} onclick={() => mark(r, "open")}><m3e-icon slot="icon" name="undo"></m3e-icon>Reopen</m3e-button>
            {/if}
          </div>
        </article>
      {/each}
    </div>
    <div class="pager">
      <m3e-button variant="text" disabled={page === 0 || loading} onclick={() => { page--; void load(); }}><m3e-icon slot="icon" name="arrow_back"></m3e-icon>Previous</m3e-button>
      <m3e-button variant="text" disabled={rows.length < size || loading} onclick={() => { page++; void load(); }}>Next<m3e-icon slot="trailing-icon" name="arrow_forward"></m3e-icon></m3e-button>
    </div>
  {/if}
</div>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .rep {
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
  .when {
    margin-left: auto;
    font-size: 12.5px;
  }
  .target {
    display: flex;
    gap: 12px;
    align-items: flex-start;
  }
  .target strong {
    margin-right: 6px;
  }
  .bio {
    margin: 6px 0 0;
    font-size: 14px;
  }
  blockquote {
    margin: 0;
    padding: 10px 14px;
    border-left: 3px solid var(--md-sys-color-primary);
    border-radius: 4px 12px 12px 4px;
    background: var(--md-sys-color-surface-container-high);
    font-size: 14px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .from {
    margin: 0;
    font-size: 12.5px;
  }
  .pager {
    display: flex;
    justify-content: space-between;
  }
</style>
