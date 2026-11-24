<script lang="ts">
  // The daily lineup algorithm, by hand: see what it would pick for a day,
  // run it for a day that has no lineup, or redo a day (only while nobody
  // holds a ticket). Manual airings are never touched.
  import { onMount } from "svelte";
  import { call, say, sayError } from "../api";
  import { local } from "./types";

  let { onclose, onchanged }: { onclose: () => void; onchanged: () => void } = $props();

  interface Pick { k: string; name: string; year: number | null; kind: string; starts_at: string; runtime: number; mode: string; capacity: number; pinned: boolean }

  const days = Array.from({ length: 4 }, (_, i) => {
    const d = new Date(Date.now() + i * 86_400_000);
    return { key: d.toISOString().slice(0, 10), label: i === 0 ? "Today" : i === 1 ? "Tomorrow" : d.toLocaleDateString(undefined, { weekday: "short", day: "numeric", month: "short" }), utc: true };
  });

  let dialog = $state<HTMLElement & { show(): void; hide(): void }>();
  let day = $state(days[0].key);
  let preview = $state<Pick[] | null>(null);
  let result = $state("");
  let busy = $state(false);
  let sure = $state(false);

  onMount(() => dialog?.show());

  async function go(op: "preview" | "run" | "regenerate") {
    busy = true;
    result = "";
    try {
      const r = (await call<Record<string, unknown>>("theatreRun", { day, op })) as unknown as { preview?: Pick[]; drafted?: number; scheduled?: number; dropped?: number; removed?: number; note?: string };
      if (op === "preview") preview = r.preview ?? [];
      else {
        preview = null;
        sure = false;
        result = r.note ?? `${r.removed ? `Removed ${r.removed}. ` : ""}Drafted ${r.drafted ?? 0}, scheduled ${r.scheduled ?? 0}${r.dropped ? `, dropped ${r.dropped} with no sources` : ""}.`;
        say(result);
        onchanged();
      }
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }
</script>

<m3e-dialog bind:this={dialog} dismissible onclosed={onclose}>
  <span slot="header">Lineup algorithm</span>
  <div class="body">
    <p class="muted lead">Days are UTC days, like the lineup itself. Nothing here changes a screening someone has a ticket for.</p>
    <m3e-filter-chip-set aria-label="Day">
      {#each days as d (d.key)}
        <m3e-filter-chip selected={day === d.key} onclick={() => { day = d.key; preview = null; result = ""; sure = false; }}>{d.label}</m3e-filter-chip>
      {/each}
    </m3e-filter-chip-set>

    <div class="row-actions">
      <m3e-button variant="tonal" disabled={busy} onclick={() => go("preview")}><m3e-icon slot="icon" name="preview"></m3e-icon>Preview picks</m3e-button>
      <m3e-button variant="tonal" disabled={busy} onclick={() => go("run")}><m3e-icon slot="icon" name="play_circle"></m3e-icon>Run if empty</m3e-button>
      {#if !sure}
        <m3e-button variant="outlined" disabled={busy} onclick={() => (sure = true)}><m3e-icon slot="icon" name="restart_alt"></m3e-icon>Redo day…</m3e-button>
      {:else}
        <m3e-button variant="filled" disabled={busy} onclick={() => go("regenerate")}>Replace this day's lineup</m3e-button>
        <m3e-button variant="text" onclick={() => (sure = false)}>Cancel</m3e-button>
      {/if}
    </div>

    {#if busy}<m3e-loading-indicator></m3e-loading-indicator>{/if}
    {#if result}<div class="done">{result}</div>{/if}
    {#if preview}
      {#if !preview.length}
        <p class="muted">The algorithm found nothing usable for that day.</p>
      {:else}
        <ul class="pick">
          {#each preview as p (p.k)}
            <li>
              <span class="mono">{local(p.starts_at)}</span>
              <span class="n">{p.name}{p.year ? ` (${p.year})` : ""}</span>
              {#if p.kind === "pick"}<span class="chip info">Tonight's pick</span>{/if}
              {#if p.pinned}<span class="chip warn">Pinned</span>{/if}
              <span class="muted">{p.runtime} min · {p.mode === "fixed" ? "ticketed" : "open"} · {p.capacity} seats</span>
            </li>
          {/each}
        </ul>
        <p class="muted small">A preview is a fresh draw. It won't match what a real run picks.</p>
      {/if}
    {/if}
  </div>
  <div slot="actions" class="actions"><m3e-button variant="text" onclick={() => dialog?.hide()}>Close</m3e-button></div>
</m3e-dialog>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 16px;
    width: min(600px, 80vw);
  }
  .lead,
  p {
    margin: 0;
    font-size: 13.5px;
  }
  .small {
    font-size: 12.5px;
  }
  .done {
    padding: 12px 14px;
    border-radius: 14px;
    background: color-mix(in srgb, #4fd18b 16%, transparent);
    color: #8be8b4;
    font-size: 14px;
  }
  .pick {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
    font-size: 14px;
    max-height: 340px;
    overflow-y: auto;
  }
  .pick li {
    display: flex;
    gap: 8px 10px;
    align-items: center;
    flex-wrap: wrap;
  }
  .n {
    font-weight: 700;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
  }
</style>
