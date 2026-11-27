<script lang="ts">
  // Max licences, online: make keys, see where each one is, and fix the usual
  // problems (new PC, lost key, give it to an account) without the local tool.
  import { onMount } from "svelte";
  import { call, say, sayError } from "../api";
  import { ago } from "../format";

  interface Lic {
    id: string;
    name: string;
    note: string;
    issued: number | null;
    activated: number | null;
    seen: number | null;
    moved: number | null;
    active: boolean;
    owner: string | null;
    owner_name: string | null;
    revoked: boolean;
    showable: boolean;
  }

  let rows = $state<Lic[]>([]);
  let signing = $state(true);
  let loading = $state(true);
  let busy = $state(false);
  let q = $state("");

  // Give a key
  let name = $state("");
  let note = $state("");
  let account = $state("");

  // The key dialog: shown after issuing, or on "Show key".
  let shown = $state<{ id: string; name: string; key: string } | null>(null);
  let keyDialog = $state<HTMLElement & { show(): void; hide(): void }>();

  // Per-row "put on an account" box.
  let giving = $state<string | null>(null);
  let giveTo = $state("");

  const iso = (s: number | null) => (s ? new Date(s * 1000).toISOString() : null);

  async function load() {
    try {
      const r = await call<{ rows: Lic[]; signing: boolean }>("maxList");
      rows = r.rows;
      signing = r.signing;
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  async function run<T>(fn: () => Promise<T>, done?: string): Promise<T | undefined> {
    busy = true;
    try {
      const out = await fn();
      if (done) say(done);
      return out;
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  function openKey(id: string, who: string, key: string) {
    shown = { id, name: who, key };
    queueMicrotask(() => keyDialog?.show());
  }

  async function issue() {
    const r = await run(() => call<{ id: string; key: string }>("maxIssue", { name: name.trim(), note: note.trim(), account: account.trim() || undefined }));
    if (!r) return;
    openKey(r.id, name.trim(), r.key);
    name = note = account = "";
    await load();
  }

  async function show(l: Lic) {
    const r = await run(() => call<{ key: string }>("maxShow", { id: l.id }));
    if (r) openKey(l.id, l.name, r.key);
  }

  async function act(l: Lic, op: string, extra: Record<string, unknown> = {}, done = "Done.") {
    if (op === "revoke" && !confirm(`Turn off ${l.id} (${l.name})? It stops working on its PC within a week.`)) return;
    const ok = await run(() => call("maxSet", { id: l.id, op, ...extra }), done);
    if (ok) {
      giving = null;
      giveTo = "";
      await load();
    }
  }

  async function editNote(l: Lic) {
    const next = prompt(`Private note for ${l.id}`, l.note);
    if (next === null) return;
    await act(l, "note", { note: next }, "Note saved.");
  }

  async function importFile(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) return;
    let records: unknown;
    try {
      records = JSON.parse(await file.text());
    } catch {
      return sayError(new Error("That file isn't JSON. Pick issued.json from the .velmora-license folder."));
    }
    const r = await run(() => call<{ added: number; known: number }>("maxImport", { records }));
    if (r) {
      say(`${r.added} added, ${r.known} already here.`);
      await load();
    }
  }

  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      say("Copied.");
    } catch {
      say("Couldn't copy. Select the key and copy it by hand.");
    }
  }

  const shownRows = $derived.by(() => {
    const t = q.trim().toLowerCase();
    if (!t) return rows;
    return rows.filter((r) => [r.id, r.name, r.note, r.owner_name ?? ""].some((x) => x.toLowerCase().includes(t)));
  });
  const stats = $derived({
    total: rows.length,
    active: rows.filter((r) => r.active && !r.revoked).length,
    linked: rows.filter((r) => r.owner).length,
    revoked: rows.filter((r) => r.revoked).length,
  });
  const hidden = $derived(rows.filter((r) => !r.showable).length);
</script>

<div class="adm-page">
  <div class="adm-head">
    <div class="grow">
      <h1>Max licences</h1>
      <p>Make a key, see which PC and account each one is on, and free or turn off keys. Keys are gifts: lifetime, one PC at a time.</p>
    </div>
  </div>

  {#if !loading && !signing}
    <m3e-callout variant="secondary">
      <m3e-icon slot="icon" name="key"></m3e-icon>
      <span slot="label">The signing key isn't in Vercel yet</span>
      Everything here works except making and showing keys. On your PC run
      <span class="mono">node tools/license/velmora-license.mjs secret signing</span>, paste it into Vercel as
      <span class="mono">LICENSE_SIGNING_KEY</span>, and redeploy.
    </m3e-callout>
  {/if}

  <div class="grid cols-4 stats">
    <div class="card stat"><strong>{stats.total}</strong><span class="muted">keys</span></div>
    <div class="card stat"><strong>{stats.active}</strong><span class="muted">on a PC</span></div>
    <div class="card stat"><strong>{stats.linked}</strong><span class="muted">on an account</span></div>
    <div class="card stat"><strong>{stats.revoked}</strong><span class="muted">turned off</span></div>
  </div>

  <div class="grid cols-2 top">
    <section class="card">
      <h2><m3e-icon name="workspace_premium"></m3e-icon>Give a key</h2>
      <div class="form">
        <label class="lab">
          <span>Who it's for</span>
          <span class="field"><input bind:value={name} maxlength="40" placeholder="Ahmed" /></span>
        </label>
        <label class="lab">
          <span>Put it on their account (optional)</span>
          <span class="field"><m3e-icon name="alternate_email"></m3e-icon><input bind:value={account} placeholder="username" autocomplete="off" spellcheck="false" /></span>
        </label>
        <label class="lab">
          <span>Private note (optional)</span>
          <span class="field"><input bind:value={note} maxlength="200" placeholder="Beta tester, friend from uni…" /></span>
        </label>
        <div class="row-actions">
          <m3e-button variant="filled" disabled={busy || !signing || !name.trim()} onclick={issue}><m3e-icon slot="icon" name="key"></m3e-icon>Make key</m3e-button>
        </div>
        <p class="muted small">With an account, Max turns on the next time they open the app signed in. Without one, send them the key privately.</p>
      </div>
    </section>

    <section class="card">
      <h2><m3e-icon name="upload"></m3e-icon>Bring in keys from your PC</h2>
      <p class="muted small">
        Keys you made with the local tool are already working, but the site can only show them again once it knows their details. Pick
        <span class="mono">issued.json</span> from <span class="mono">%USERPROFILE%\.velmora-license</span>. It holds ids, names and dates, no secrets, and every record is checked against the signing key.
      </p>
      {#if hidden}<p class="small warn-text">{hidden} {hidden === 1 ? "key" : "keys"} below can't be shown yet.</p>{/if}
      <label class="pick" class:off={busy || !signing}>
        <input type="file" accept="application/json,.json" onchange={importFile} disabled={busy || !signing} />
        <m3e-icon name="folder_open"></m3e-icon>Choose issued.json
      </label>
    </section>
  </div>

  <div class="card list">
    <div class="list-head">
      <h2><m3e-icon name="list"></m3e-icon>Every key</h2>
      <label class="field find">
        <m3e-icon name="search"></m3e-icon>
        <input type="search" bind:value={q} placeholder="Id, name, note or @account" aria-label="Filter keys" />
      </label>
    </div>

    {#if loading}
      <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
    {:else if !shownRows.length}
      <div class="empty">{q ? "No key matches that." : "No keys yet."}</div>
    {:else}
      {#each shownRows as l (l.id)}
        <div class="lic" class:revoked={l.revoked}>
          <div class="who">
            <strong>{l.name || "No name"}</strong>
            <span class="mono muted">{l.id}</span>
            {#if l.note}<span class="muted small">{l.note}</span>{/if}
          </div>
          <div class="flags">
            {#if l.revoked}<span class="chip bad"><m3e-icon name="block"></m3e-icon>Turned off</span>
            {:else if l.active}<span class="chip ok"><m3e-icon name="desktop_windows"></m3e-icon>On a PC</span>
            {:else}<span class="chip">Not activated</span>{/if}
            {#if l.owner}<span class="chip info"><m3e-icon name="person"></m3e-icon>{l.owner_name ? `@${l.owner_name}` : "account"}</span>{/if}
          </div>
          <div class="when muted small">
            <span>Made {ago(iso(l.issued))}</span>
            <span>Seen {ago(iso(l.seen))}</span>
          </div>
          <div class="acts">
            <m3e-icon-button title="Show key" aria-label="Show key" disabled={busy || !l.showable || !signing} onclick={() => show(l)}><m3e-icon name="visibility"></m3e-icon></m3e-icon-button>
            <m3e-icon-button title="Free it from its PC" aria-label="Free it from its PC" disabled={busy || !l.active} onclick={() => act(l, "release", {}, "Freed. It can go on a new PC.")}><m3e-icon name="restart_alt"></m3e-icon></m3e-icon-button>
            {#if l.owner}
              <m3e-icon-button title="Take off the account" aria-label="Take off the account" disabled={busy} onclick={() => act(l, "unlink", {}, "Taken off the account.")}><m3e-icon name="link_off"></m3e-icon></m3e-icon-button>
            {:else}
              <m3e-icon-button title="Put on an account" aria-label="Put on an account" disabled={busy || l.revoked || !l.showable} onclick={() => { giving = giving === l.id ? null : l.id; giveTo = ""; }}><m3e-icon name="add_link"></m3e-icon></m3e-icon-button>
            {/if}
            <m3e-icon-button title="Private note" aria-label="Private note" disabled={busy} onclick={() => editNote(l)}><m3e-icon name="edit_note"></m3e-icon></m3e-icon-button>
            {#if l.revoked}
              <m3e-icon-button title="Turn back on" aria-label="Turn back on" disabled={busy} onclick={() => act(l, "unrevoke", {}, "Back on.")}><m3e-icon name="undo"></m3e-icon></m3e-icon-button>
            {:else}
              <m3e-icon-button title="Turn off" aria-label="Turn off" disabled={busy} onclick={() => act(l, "revoke", {}, "Turned off.")}><m3e-icon name="block"></m3e-icon></m3e-icon-button>
            {/if}
          </div>
          {#if giving === l.id}
            <div class="give">
              <span class="field"><m3e-icon name="alternate_email"></m3e-icon><input bind:value={giveTo} placeholder="username" aria-label="Username" autocomplete="off" spellcheck="false" /></span>
              <m3e-button variant="tonal" disabled={busy || !giveTo.trim()} onclick={() => act(l, "give", { account: giveTo.trim() }, "It's on their account now.")}>Put it there</m3e-button>
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<m3e-dialog bind:this={keyDialog} dismissible onclosed={() => (shown = null)}>
  <span slot="header">Max key for {shown?.name}</span>
  {#if shown}
    <div class="body">
      <p class="muted small">{shown.id}. Send it privately: anyone with it can use it on one PC.</p>
      <code class="keybox">{shown.key}</code>
    </div>
  {/if}
  <div slot="actions" class="actions">
    <m3e-button variant="text" onclick={() => keyDialog?.hide()}>Close</m3e-button>
    <m3e-button variant="filled" onclick={() => shown && copy(shown.key)}><m3e-icon slot="icon" name="content_copy"></m3e-icon>Copy key</m3e-button>
  </div>
</m3e-dialog>

<style>
  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 16px 20px;
  }
  .stat strong {
    font-family: var(--display);
    font-weight: 400;
    font-size: 36px;
    line-height: 1.1;
  }
  .top {
    align-items: start;
  }
  .form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .small {
    font-size: 13px;
    line-height: 1.5;
    margin: 0;
  }
  .warn-text {
    color: #ffd08a;
    margin: 10px 0 0;
  }
  .pick {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin-top: 14px;
    height: 40px;
    padding: 0 18px;
    border-radius: 999px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    font-weight: 600;
    font-size: 14px;
    cursor: pointer;
    --m3e-icon-size: 20px;
  }
  .pick.off {
    opacity: 0.5;
    pointer-events: none;
  }
  .pick input {
    display: none;
  }
  .list {
    padding: 8px;
  }
  .list-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
    padding: 12px 12px 8px;
  }
  .list-head h2 {
    margin: 0;
    font-size: 15px;
    display: flex;
    align-items: center;
    gap: 8px;
    --m3e-icon-size: 20px;
  }
  .list-head h2 m3e-icon {
    color: var(--md-sys-color-primary);
  }
  .find {
    width: min(320px, 100%);
    height: 40px;
  }
  .lic {
    display: grid;
    grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: 18px;
    transition: background-color 160ms ease;
  }
  .lic:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, transparent);
  }
  .lic.revoked .who strong {
    text-decoration: line-through;
    opacity: 0.7;
  }
  .who {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .who .mono {
    font-size: 12.5px;
  }
  .flags {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .when {
    display: flex;
    flex-direction: column;
    text-align: right;
    white-space: nowrap;
  }
  .acts {
    display: flex;
  }
  .give {
    grid-column: 1 / -1;
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .give .field {
    flex: 1;
    height: 40px;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  @media (max-width: 860px) {
    .lic {
      grid-template-columns: minmax(0, 1fr) auto;
    }
    .when {
      text-align: left;
    }
  }
</style>
