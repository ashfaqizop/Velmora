<script lang="ts">
  // registry.json on the config branch: source mirrors, kill-switches and the
  // rest. The app reads it at launch, so a save reaches people on their next
  // start (GitHub's raw cache can add about five minutes).
  import { onMount } from "svelte";
  import { call, say, sayError } from "../api";
  import { ago } from "../format";

  interface Commit {
    sha: string;
    message: string;
    at: string | null;
    url: string;
  }
  type Registry = { version: number; mirrors?: Record<string, string[]>; disabled?: string[]; [k: string]: unknown };

  const NAMES: Record<string, string> = {
    hd_hub: "HDHub",
    hi_anime: "HiAnime",
    zoko_anime: "ZokoAnime",
    movie_box: "MovieBox",
    four_k_hd_hub: "4KHDHub",
    dramachi: "Dramachi",
    anime_sama: "Anime-Sama",
    french_stream: "French Stream",
    coflix: "Coflix",
    yts: "YTS",
    eztv: "EZTV",
    nyaa: "Nyaa",
    cinemeta: "Cinemeta",
    tmdb: "TMDB",
    anilist: "AniList",
    iptv_org: "iptv-org",
    youtube_music: "YouTube Music",
  };

  let loading = $state(true);
  let busy = $state(false);
  let writable = $state(false);
  let sha = $state("");
  let orig = $state("");
  let text = $state("");
  let history = $state<Commit[]>([]);
  let providers = $state<string[]>([]);
  let tab = $state<"sources" | "raw">("sources");
  let message = $state("");
  let viewing = $state<string | null>(null);

  async function load() {
    loading = true;
    try {
      const r = await call<{ text: string; sha: string; writable: boolean; history: Commit[]; providers: string[] }>("config");
      sha = r.sha;
      orig = text = r.text;
      writable = r.writable;
      history = r.history;
      providers = r.providers;
      viewing = null;
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  const parsed = $derived.by((): { ok: true; reg: Registry } | { ok: false; error: string } => {
    try {
      const reg = JSON.parse(text);
      return reg && typeof reg === "object" && !Array.isArray(reg) ? { ok: true, reg } : { ok: false, error: "The file must be a JSON object." };
    } catch (e) {
      return { ok: false, error: (e as Error).message };
    }
  });
  const base = $derived.by((): Registry | null => {
    try {
      return JSON.parse(orig);
    } catch {
      return null;
    }
  });

  /** Edit the parsed file and write it back as text. */
  function edit(fn: (reg: Registry) => void) {
    if (!parsed.ok) return;
    const reg = structuredClone($state.snapshot(parsed.reg)) as Registry;
    fn(reg);
    text = JSON.stringify(reg, null, 2) + "\n";
  }

  const sources = $derived.by(() => {
    if (!parsed.ok) return [];
    const m = parsed.reg.mirrors ?? {};
    const keys = [...new Set([...Object.keys(m), ...providers.filter((p) => !["cinemeta", "tmdb", "anilist", "iptv_org", "youtube_music"].includes(p))])];
    return keys.map((id) => ({ id, urls: m[id] ?? [], off: (parsed.reg.disabled ?? []).includes(id) }));
  });

  function setUrls(id: string, value: string) {
    const urls = value.split(/\s+/).map((u) => u.trim()).filter(Boolean);
    edit((r) => {
      r.mirrors ??= {};
      if (urls.length) r.mirrors[id] = urls;
      else delete r.mirrors[id];
    });
  }
  function setOff(id: string, off: boolean) {
    edit((r) => {
      const list = new Set(r.disabled ?? []);
      if (off) list.add(id);
      else list.delete(id);
      r.disabled = [...list];
    });
  }

  /** What changed, in words, so a save is never a surprise. */
  const changes = $derived.by(() => {
    if (!parsed.ok || !base) return [];
    const a = base;
    const b = parsed.reg;
    const out: string[] = [];
    const am = a.mirrors ?? {};
    const bm = b.mirrors ?? {};
    for (const id of new Set([...Object.keys(am), ...Object.keys(bm)])) {
      if (JSON.stringify(am[id] ?? []) !== JSON.stringify(bm[id] ?? [])) out.push(`${NAMES[id] ?? id}: mirrors ${(am[id] ?? []).length} → ${(bm[id] ?? []).length}${(am[id] ?? [])[0] !== (bm[id] ?? [])[0] ? ", first one changed" : ""}`);
    }
    const ad = new Set(a.disabled ?? []);
    const bd = new Set(b.disabled ?? []);
    for (const id of bd) if (!ad.has(id)) out.push(`${NAMES[id] ?? id}: switched off`);
    for (const id of ad) if (!bd.has(id)) out.push(`${NAMES[id] ?? id}: switched back on`);
    for (const k of new Set([...Object.keys(a), ...Object.keys(b)])) {
      if (k === "mirrors" || k === "disabled") continue;
      if (JSON.stringify(a[k]) !== JSON.stringify(b[k])) out.push(`${k}: changed`);
    }
    return out;
  });
  const dirty = $derived(text !== orig);

  async function save() {
    busy = true;
    try {
      await call("configSave", { text, sha, message: message.trim() || changes.slice(0, 2).join("; ") });
      say("Saved to the config branch. Apps pick it up on their next launch.");
      message = "";
      await load();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  async function open(c: Commit) {
    busy = true;
    try {
      const r = await call<{ text: string }>("configAt", { sha: c.sha });
      text = r.text;
      viewing = c.sha;
      tab = "raw";
      say("Loaded that version into the editor. Save to put it back.");
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
      <h1>Remote config</h1>
      <p>Source mirrors, kill-switches and engine settings in <span class="mono">registry.json</span> on the <span class="mono">config</span> branch. The app reads it at every launch, so a fix here needs no release.</p>
    </div>
    {#if !loading}
      <div class="row-actions">
        <m3e-button variant="text" disabled={busy || !dirty} onclick={() => { text = orig; viewing = null; }}>Undo edits</m3e-button>
        <m3e-button variant="text" disabled={busy} onclick={load}><m3e-icon slot="icon" name="refresh"></m3e-icon>Reload</m3e-button>
      </div>
    {/if}
  </div>

  {#if !loading && !writable}
    <m3e-callout variant="secondary">
      <m3e-icon slot="icon" name="lock"></m3e-icon>
      <span slot="label">Read only for now</span>
      Saving needs <span class="mono">CONFIG_GITHUB_TOKEN</span> in Vercel: a fine-grained GitHub token for this repo only, with Contents set to read and write. This page only ever writes <span class="mono">registry.json</span> on the config branch.
    </m3e-callout>
  {/if}

  {#if loading}
    <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
  {:else}
    <m3e-filter-chip-set aria-label="Editor">
      <m3e-filter-chip selected={tab === "sources"} onclick={() => (tab = "sources")}>Sources</m3e-filter-chip>
      <m3e-filter-chip selected={tab === "raw"} onclick={() => (tab = "raw")}>Whole file</m3e-filter-chip>
    </m3e-filter-chip-set>

    {#if viewing}
      <div class="err soft">Showing an older version ({viewing.slice(0, 7)}). Save to restore it, or Undo edits to go back.</div>
    {/if}

    {#if tab === "sources"}
      {#if !parsed.ok}
        <div class="err">The file doesn't parse right now ({parsed.error}). Fix it under Whole file first.</div>
      {:else}
        <div class="grid cols-2 sources">
          {#each sources as s (s.id)}
            <section class="card src" class:off={s.off}>
              <div class="src-head">
                <strong>{NAMES[s.id] ?? s.id}</strong>
                <span class="chip {s.off ? 'bad' : 'ok'}">{s.off ? "Switched off" : "On"}</span>
                <m3e-switch checked={!s.off} disabled={busy} onchange={(e: Event) => setOff(s.id, !(e.currentTarget as HTMLInputElement).checked)} aria-label="{NAMES[s.id] ?? s.id} on"></m3e-switch>
              </div>
              <textarea
                class="area mono"
                rows={Math.min(7, Math.max(2, s.urls.length + 1))}
                value={s.urls.join("\n")}
                onchange={(e) => setUrls(s.id, (e.currentTarget as HTMLTextAreaElement).value)}
                placeholder="https://… (one per line, first is tried first)"
                spellcheck="false"
                aria-label="{NAMES[s.id] ?? s.id} mirrors"
              ></textarea>
            </section>
          {/each}
        </div>
        <p class="muted small">One mirror per line; the app tries them in order. Switching a source off is the kill-switch: the app stops asking it for anything.</p>
      {/if}
    {:else}
      <section class="card">
        <textarea class="area mono raw" bind:value={text} spellcheck="false" aria-label="registry.json"></textarea>
        {#if !parsed.ok}<div class="err">Not valid JSON: {parsed.error}</div>{/if}
      </section>
    {/if}

    <section class="card">
      <h2><m3e-icon name="save"></m3e-icon>Save</h2>
      {#if dirty && changes.length}
        <ul class="changes">
          {#each changes as c (c)}<li>{c}</li>{/each}
        </ul>
      {:else if dirty}
        <p class="muted small">Only formatting changed.</p>
      {:else}
        <p class="muted small">Nothing changed yet.</p>
      {/if}
      <div class="save-row">
        <span class="field"><input bind:value={message} maxlength="100" placeholder="What changed and why (goes in the commit)" aria-label="Commit message" /></span>
        <m3e-button variant="filled" disabled={busy || !writable || !dirty || !parsed.ok} onclick={save}><m3e-icon slot="icon" name="cloud_sync"></m3e-icon>Save to config</m3e-button>
      </div>
    </section>

    <section class="card">
      <h2><m3e-icon name="history"></m3e-icon>History</h2>
      {#if !history.length}
        <div class="empty">No history.</div>
      {:else}
        {#each history as c, i (c.sha)}
          <div class="commit">
            <span class="mono muted">{c.sha.slice(0, 7)}</span>
            <span class="msg">{c.message}</span>
            <span class="muted small-nowrap">{ago(c.at)}</span>
            {#if i > 0}
              <m3e-button variant="text" size="small" disabled={busy} onclick={() => open(c)}>Load</m3e-button>
            {:else}
              <span class="chip ok">Current</span>
            {/if}
            <a class="gh" href={c.url} target="_blank" rel="noreferrer" aria-label="Open on GitHub"><m3e-icon name="open_in_new"></m3e-icon></a>
          </div>
        {/each}
      {/if}
    </section>
  {/if}
</div>

<style>
  .small {
    font-size: 13px;
    line-height: 1.5;
    margin: 0;
  }
  .soft {
    background: color-mix(in srgb, var(--amber) 16%, transparent);
    color: #ffd08a;
  }
  .sources {
    align-items: start;
  }
  .src {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
  }
  .src.off {
    border-color: color-mix(in srgb, #ff5449 45%, transparent);
  }
  .src-head {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .src-head strong {
    flex: 1;
  }
  .area.mono {
    font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
    font-size: 12.5px;
    min-height: 0;
  }
  .raw {
    min-height: 480px;
  }
  .changes {
    margin: 0 0 14px;
    padding-left: 20px;
    font-size: 14px;
    line-height: 1.6;
  }
  .save-row {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
    margin-top: 12px;
  }
  .save-row .field {
    flex: 1;
    min-width: 240px;
  }
  .commit {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto auto auto;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
    border-top: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 30%, transparent);
  }
  .commit:first-of-type {
    border-top: 0;
  }
  .msg {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .small-nowrap {
    font-size: 13px;
    white-space: nowrap;
  }
  .gh {
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 18px;
  }
</style>
