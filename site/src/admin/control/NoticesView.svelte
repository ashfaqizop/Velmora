<script lang="ts">
  // Announcements and the maintenance message. Live ones show as a banner at
  // the top of the app (next release), the website and Social.
  import { onMount } from "svelte";
  import { call, say, sayError } from "../api";
  import { when } from "../format";

  type Kind = "info" | "warning" | "maintenance";
  interface Notice {
    id: number;
    kind: Kind;
    title: string;
    body: string;
    link_url: string;
    link_label: string;
    audience: "everyone" | "public" | "max";
    place: "app" | "web" | "both";
    active: boolean;
    starts_at: string;
    ends_at: string | null;
    created_at: string;
  }

  const KINDS: { id: Kind; label: string; icon: string }[] = [
    { id: "info", label: "News", icon: "campaign" },
    { id: "warning", label: "Heads-up", icon: "warning" },
    { id: "maintenance", label: "Maintenance", icon: "construction" },
  ];
  const AUDIENCES = [
    { id: "everyone", label: "Everyone" },
    { id: "public", label: "Public only" },
    { id: "max", label: "Max only" },
  ] as const;
  const PLACES = [
    { id: "both", label: "App and web" },
    { id: "app", label: "App only" },
    { id: "web", label: "Web only" },
  ] as const;

  let rows = $state<Notice[]>([]);
  let loading = $state(true);
  let busy = $state(false);
  let dialog = $state<HTMLElement & { show(): void; hide(): void }>();

  const blank = () => ({
    id: null as number | null,
    kind: "info" as Kind,
    title: "",
    body: "",
    link_url: "",
    link_label: "",
    audience: "everyone" as Notice["audience"],
    place: "both" as Notice["place"],
    active: true,
    start: "",
    end: "",
  });
  let form = $state(blank());

  /** ISO to the value a datetime-local input wants, in this PC's time. */
  const toLocal = (iso: string | null) => {
    if (!iso) return "";
    const d = new Date(iso);
    return new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString().slice(0, 16);
  };
  const fromLocal = (v: string) => (v ? new Date(v).toISOString() : null);

  async function load() {
    try {
      rows = (await call<{ rows: Notice[] }>("notices")).rows;
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  function status(n: Notice): { label: string; cls: string } {
    const t = Date.now();
    if (!n.active) return { label: "Off", cls: "" };
    if (Date.parse(n.starts_at) > t) return { label: "Scheduled", cls: "info" };
    if (n.ends_at && Date.parse(n.ends_at) <= t) return { label: "Ended", cls: "" };
    return { label: "Live", cls: "ok" };
  }

  function edit(n?: Notice) {
    form = n
      ? { id: n.id, kind: n.kind, title: n.title, body: n.body, link_url: n.link_url, link_label: n.link_label, audience: n.audience, place: n.place, active: n.active, start: toLocal(n.starts_at), end: toLocal(n.ends_at) }
      : blank();
    queueMicrotask(() => dialog?.show());
  }

  async function save() {
    busy = true;
    try {
      const { start, end, ...rest } = form;
      await call("noticeSave", { ...rest, id: form.id ?? undefined, starts_at: fromLocal(start), ends_at: fromLocal(end) });
      say(form.id ? "Saved." : "Announcement is up.");
      dialog?.hide();
      await load();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  async function toggle(n: Notice) {
    busy = true;
    try {
      await call("noticeSave", { ...n, active: !n.active });
      say(n.active ? "Switched off." : "Switched on.");
      await load();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  async function remove(n: Notice) {
    if (!confirm(`Delete "${n.title}"?`)) return;
    busy = true;
    try {
      await call("noticeDelete", { id: n.id });
      say("Deleted.");
      await load();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  /** One click: a maintenance banner that's live now and ends in two hours. */
  function maintenance() {
    form = { ...blank(), kind: "maintenance", title: "Velmora is under maintenance", body: "Some things may not work for a little while. We'll be back soon.", end: toLocal(new Date(Date.now() + 2 * 3600_000).toISOString()) };
    queueMicrotask(() => dialog?.show());
  }

  const icon = (k: Kind) => KINDS.find((x) => x.id === k)?.icon ?? "campaign";
  const live = $derived(rows.filter((n) => status(n).label === "Live"));
</script>

<div class="adm-page">
  <div class="adm-head">
    <div class="grow">
      <h1>Announcements</h1>
      <p>A banner at the top of the app, the website and Social. Maintenance messages are red and can't be dismissed; the others can.</p>
    </div>
    <div class="row-actions">
      <m3e-button variant="tonal" disabled={busy} onclick={maintenance}><m3e-icon slot="icon" name="construction"></m3e-icon>Maintenance message</m3e-button>
      <m3e-button variant="filled" disabled={busy} onclick={() => edit()}><m3e-icon slot="icon" name="add"></m3e-icon>New announcement</m3e-button>
    </div>
  </div>

  {#if live.length}
    <section class="card">
      <h2><m3e-icon name="sensors"></m3e-icon>Showing right now</h2>
      <div class="previews">
        {#each live as n (n.id)}
          <div class="banner {n.kind}">
            <m3e-icon name={icon(n.kind)}></m3e-icon>
            <div class="text"><strong>{n.title}</strong>{#if n.body}<span>{n.body}</span>{/if}</div>
            {#if n.link_url}<span class="link">{n.link_label || "Learn more"}</span>{/if}
            {#if n.kind !== "maintenance"}<m3e-icon name="close" class="x"></m3e-icon>{/if}
          </div>
        {/each}
      </div>
      <p class="muted small">The app and site check about every 10 minutes, so a change can take that long to show.</p>
    </section>
  {/if}

  <div class="card list">
    {#if loading}
      <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
    {:else if !rows.length}
      <div class="empty">No announcements yet.</div>
    {:else}
      {#each rows as n (n.id)}
        {@const s = status(n)}
        <div class="row">
          <m3e-icon class="kind {n.kind}" name={icon(n.kind)}></m3e-icon>
          <div class="who">
            <strong>{n.title}</strong>
            <span class="muted small">
              {AUDIENCES.find((a) => a.id === n.audience)?.label} · {PLACES.find((p) => p.id === n.place)?.label} · from {when(n.starts_at)}{n.ends_at ? ` to ${when(n.ends_at)}` : ", no end"}
            </span>
          </div>
          <span class="chip {s.cls}">{s.label}</span>
          <div class="acts">
            <m3e-switch checked={n.active} disabled={busy} onchange={() => toggle(n)} aria-label="On or off"></m3e-switch>
            <m3e-icon-button title="Edit" aria-label="Edit" disabled={busy} onclick={() => edit(n)}><m3e-icon name="edit"></m3e-icon></m3e-icon-button>
            <m3e-icon-button title="Delete" aria-label="Delete" disabled={busy} onclick={() => remove(n)}><m3e-icon name="delete"></m3e-icon></m3e-icon-button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<m3e-dialog bind:this={dialog} dismissible>
  <span slot="header">{form.id ? "Edit announcement" : "New announcement"}</span>
  <div class="body">
    <div class="lab">
      <span>Kind</span>
      <m3e-filter-chip-set aria-label="Kind">
        {#each KINDS as k (k.id)}
          <m3e-filter-chip selected={form.kind === k.id} onclick={() => (form.kind = k.id)}><m3e-icon slot="icon" name={k.icon}></m3e-icon>{k.label}</m3e-filter-chip>
        {/each}
      </m3e-filter-chip-set>
    </div>
    <label class="lab">
      <span>Title</span>
      <span class="field"><input bind:value={form.title} maxlength="80" placeholder="Velmora 0.7 is out" /></span>
    </label>
    <label class="lab">
      <span>Text (optional)</span>
      <textarea class="area" bind:value={form.body} maxlength="400" rows="3" placeholder="One or two short sentences."></textarea>
    </label>
    <div class="two">
      <label class="lab">
        <span>Link (optional)</span>
        <span class="field"><input bind:value={form.link_url} placeholder="https://…" inputmode="url" /></span>
      </label>
      <label class="lab">
        <span>Button label</span>
        <span class="field"><input bind:value={form.link_label} maxlength="30" placeholder="Learn more" /></span>
      </label>
    </div>
    <div class="lab">
      <span>Who sees it</span>
      <m3e-filter-chip-set aria-label="Who sees it">
        {#each AUDIENCES as a (a.id)}
          <m3e-filter-chip selected={form.audience === a.id} onclick={() => (form.audience = a.id)}>{a.label}</m3e-filter-chip>
        {/each}
      </m3e-filter-chip-set>
    </div>
    <div class="lab">
      <span>Where</span>
      <m3e-filter-chip-set aria-label="Where">
        {#each PLACES as p (p.id)}
          <m3e-filter-chip selected={form.place === p.id} onclick={() => (form.place = p.id)}>{p.label}</m3e-filter-chip>
        {/each}
      </m3e-filter-chip-set>
      {#if form.audience !== "everyone" && form.place !== "app"}<span class="muted small">The website can't tell Max from Public, so there it shows to everyone.</span>{/if}
    </div>
    <div class="two">
      <label class="lab">
        <span>Starts (empty = now)</span>
        <span class="field"><input type="datetime-local" bind:value={form.start} /></span>
      </label>
      <label class="lab">
        <span>Ends (empty = until you switch it off)</span>
        <span class="field"><input type="datetime-local" bind:value={form.end} /></span>
      </label>
    </div>
    <label class="on">
      <m3e-switch checked={form.active} onchange={(e: Event) => (form.active = (e.currentTarget as HTMLInputElement).checked)} aria-label="Switched on"></m3e-switch>
      <span>Switched on</span>
    </label>

    <div class="banner {form.kind}">
      <m3e-icon name={icon(form.kind)}></m3e-icon>
      <div class="text"><strong>{form.title || "Title"}</strong>{#if form.body}<span>{form.body}</span>{/if}</div>
      {#if form.link_url}<span class="link">{form.link_label || "Learn more"}</span>{/if}
      {#if form.kind !== "maintenance"}<m3e-icon name="close" class="x"></m3e-icon>{/if}
    </div>
  </div>
  <div slot="actions" class="actions">
    <m3e-button variant="text" onclick={() => dialog?.hide()}>Cancel</m3e-button>
    <m3e-button variant="filled" disabled={busy || !form.title.trim()} onclick={save}>{form.id ? "Save" : "Post it"}</m3e-button>
  </div>
</m3e-dialog>

<style>
  .small {
    font-size: 13px;
    line-height: 1.5;
    margin: 0;
  }
  .previews {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 12px;
  }
  .banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 16px;
    background: color-mix(in srgb, var(--md-sys-color-primary) 16%, var(--md-sys-color-surface-container-high));
    color: var(--md-sys-color-on-surface);
    font-size: 14px;
    --m3e-icon-size: 22px;
  }
  .banner > m3e-icon:first-child {
    color: var(--md-sys-color-primary);
  }
  .banner.warning {
    background: color-mix(in srgb, var(--amber) 20%, var(--md-sys-color-surface-container-high));
  }
  .banner.warning > m3e-icon:first-child {
    color: #ffd08a;
  }
  .banner.maintenance {
    background: color-mix(in srgb, #ff5449 22%, var(--md-sys-color-surface-container-high));
  }
  .banner.maintenance > m3e-icon:first-child {
    color: #ffb4ab;
  }
  .banner .text {
    flex: 1;
    display: flex;
    flex-wrap: wrap;
    gap: 2px 8px;
    min-width: 0;
  }
  .banner .text span {
    color: var(--md-sys-color-on-surface-variant);
  }
  .banner .link {
    font-weight: 700;
    color: var(--md-sys-color-primary);
    white-space: nowrap;
  }
  .banner .x {
    opacity: 0.6;
    --m3e-icon-size: 18px;
  }
  .list {
    padding: 8px;
  }
  .row {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: 18px;
  }
  .row:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, transparent);
  }
  .kind {
    color: var(--md-sys-color-primary);
  }
  .kind.warning {
    color: #ffd08a;
  }
  .kind.maintenance {
    color: #ffb4ab;
  }
  .who {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .acts {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 14px;
    width: min(620px, 80vw);
  }
  .on {
    display: flex;
    align-items: center;
    gap: 12px;
    font-weight: 600;
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  @media (max-width: 700px) {
    .row {
      grid-template-columns: auto minmax(0, 1fr);
    }
  }
</style>
