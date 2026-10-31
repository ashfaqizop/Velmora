<script lang="ts">
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../lib/api";
  import { torrentConsent } from "../lib/overlays";
  import { isTorrent, providerName, sourceChips } from "../lib/sources";
  import { downloads, nav, settings, targetLabel } from "../lib/state.svelte";
  import type { SourcesReport, Target, TitleSummary } from "../lib/types";

  let dialog: HTMLElement & { show(): Promise<void>; hide(v?: string): Promise<void> };
  let title = $state<TitleSummary | null>(null);
  let target = $state<Target>({ type: "movie" });
  let report = $state<SourcesReport | null>(null);
  let error = $state<string | null>(null);
  let busy = $state(false);
  let run = 0;

  const downloadable = $derived(report?.sources ?? []);
  /** File names already saved for this title, from any session. */
  let have = $state<string[]>([]);
  const fileName = (p: string) => p.split(/[\\/]/).pop() ?? p;

  export function open(t: TitleSummary, tg: Target) {
    title = t;
    target = tg;
    report = null;
    error = null;
    have = [];
    const mine = ++run;
    dialog.show();
    api
      .downloadExisting(label(t, tg))
      .then((files) => mine === run && (have = files))
      .catch(() => {});
    api
      .sources(t, tg, settings.audioMode)
      .then((r) => mine === run && (report = r))
      .catch((e) => mine === run && (error = String(e)));
  }

  /** File name base: "Dune (2021)" or "Frieren S01E03". */
  function label(t: TitleSummary, tg: Target) {
    if (tg.type === "episode") {
      const pad = (n: number) => String(n).padStart(2, "0");
      return `${t.name} S${pad(tg.season)}E${pad(tg.episode)}`;
    }
    return t.year ? `${t.name} (${t.year})` : t.name;
  }

  async function pick(i: number) {
    const src = downloadable[i];
    if (!src || !title || busy) return;
    if (isTorrent(src) && !(await torrentConsent())) return;
    busy = true;
    try {
      const episode: [number, number] | null = target.type === "episode" ? [target.season, target.episode] : null;
      await api.downloadStart(src, label(title, target), episode);
      downloads.start();
      await dialog.hide();
      M3eSnackbar.open(`${title.name} added to Downloads`, "View", {
        actionCallback: () => nav.go({ name: "downloads" }),
      });
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<m3e-dialog bind:this={dialog} dismissible class="dl">
  <span slot="header">
    Download {title?.name ?? ""}{target.type === "episode" ? ` · ${targetLabel(target)}` : ""}
  </span>
  <div class="body">
    {#if have.length}
      <m3e-callout class="have">
        <m3e-icon slot="icon" name="download_done"></m3e-icon>
        <span slot="label">Already downloaded</span>
        {have.map(fileName).join(", ")} is in your Downloads. Another download saves a second copy beside it.
        <m3e-button
          variant="text"
          size="small"
          onclick={() => {
            dialog.hide();
            nav.go({ name: "downloads" });
          }}>Open Downloads</m3e-button
        >
      </m3e-callout>
    {/if}
    {#if error}
      <p class="error">{error}</p>
    {:else if !report}
      <div class="loading">
        <m3e-loading-indicator></m3e-loading-indicator>
        <span>Finding sources</span>
      </div>
    {:else if !downloadable.length}
      <p class="empty">No downloadable source for this one right now.</p>
    {:else}
      <p class="hint">Best match first. Pick a quality and size that suit you.</p>
      <m3e-list>
        {#each downloadable as s, i (i)}
          <m3e-list-action disabled={busy} onclick={() => pick(i)}>
            <span slot="leading" class="res">{sourceChips(s)[0] ?? "SD"}</span>
            {providerName(s)} · {s.host}
            <span slot="supporting-text">{sourceChips(s).slice(1).join(" · ") || s.label}</span>
            <m3e-icon slot="trailing" name={isTorrent(s) ? "hub" : "download"}></m3e-icon>
          </m3e-list-action>
        {/each}
      </m3e-list>
    {/if}
  </div>
</m3e-dialog>

<style>
  .dl {
    --m3e-dialog-min-width: 480px;
    --m3e-dialog-max-width: 560px;
    --m3e-dialog-max-height: 80vh;
  }
  .body {
    min-height: 120px;
  }
  .have {
    display: block;
    margin-bottom: 16px;
  }
  .loading {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 24px 0;
  }
  .hint,
  .empty {
    margin: 0 0 8px;
    color: var(--md-sys-color-on-surface-variant);
    font-size: 14px;
  }
  .error {
    color: var(--md-sys-color-error);
  }
  .res {
    display: grid;
    place-items: center;
    min-width: 48px;
    height: 32px;
    padding-inline: 6px;
    border-radius: 10px;
    font-size: 12px;
    font-weight: 700;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
</style>
