<script lang="ts">
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import { personalize, type ActivityKind } from "../lib/ani/personalize.svelte";
  import { kindLabel } from "../lib/state.svelte";

  // Everything Ani knows about the user, with a way to remove any of it.
  let dialog: HTMLElement & { show(): Promise<void>; hide(v?: string): Promise<void> };

  export function open() {
    dialog.show();
  }

  const ICON: Record<ActivityKind, string> = {
    search: "search",
    view: "visibility",
    stream: "play_circle",
    finish: "task_alt",
    download: "download",
  };
  const VERB: Record<ActivityKind, string> = {
    search: "Searched",
    view: "Looked at",
    stream: "Watched",
    finish: "Finished",
    download: "Downloaded",
  };

  const stats = $derived(personalize.stats());
  const recent = $derived(personalize.events.slice(0, 60));

  function ago(at: number) {
    const m = Math.round((Date.now() - at) / 60_000);
    if (m < 1) return "just now";
    if (m < 60) return `${m} min ago`;
    const h = Math.round(m / 60);
    if (h < 24) return `${h} h ago`;
    const d = Math.round(h / 24);
    return d === 1 ? "yesterday" : `${d} days ago`;
  }

  function clearAll() {
    personalize.clear();
    M3eSnackbar.open("Personalize cleared. Ani starts fresh.");
  }
</script>

<m3e-dialog bind:this={dialog} dismissible class="pz">
  <span slot="header">What Ani knows about you</span>
  <div class="body">
    <p class="lead">Kept on this PC only. Ani reads it before answering so its picks fit your taste.</p>

    {#if stats.genres.length}
      <section>
        <m3e-heading variant="title" size="small" level={3}>Your taste</m3e-heading>
        <m3e-chip-set class="chips">
          {#each stats.genres as [g] (g)}<m3e-chip>{g}</m3e-chip>{/each}
          {#each stats.kinds as [k] (k)}<m3e-chip><m3e-icon slot="icon" name="category"></m3e-icon>{kindLabel[k]}</m3e-chip>{/each}
          {#each stats.decades as [d] (d)}<m3e-chip><m3e-icon slot="icon" name="history"></m3e-icon>{d}</m3e-chip>{/each}
        </m3e-chip-set>
      </section>
    {/if}

    <section>
      <m3e-heading variant="title" size="small" level={3}>Notes from Ani</m3e-heading>
      {#if personalize.notes.length}
        <m3e-list>
          {#each personalize.notes as n (n.id)}
            <m3e-list-item>
              <m3e-icon slot="leading" name="sticky_note_2"></m3e-icon>
              {n.text}
              <span slot="supporting-text">{ago(n.at)}</span>
              <m3e-icon-button slot="trailing" aria-label="Forget this note" onclick={() => personalize.forgetNote(n.id)}>
                <m3e-icon name="close"></m3e-icon>
              </m3e-icon-button>
            </m3e-list-item>
          {/each}
        </m3e-list>
      {:else}
        <p class="empty">None yet. When you tell Ani something about yourself, like "I prefer dubs", it saves it here.</p>
      {/if}
    </section>

    <section>
      <m3e-heading variant="title" size="small" level={3}>Recent activity</m3e-heading>
      {#if recent.length}
        <m3e-list>
          {#each recent as e (e.at + e.kind)}
            <m3e-list-item>
              <m3e-icon slot="leading" name={ICON[e.kind]}></m3e-icon>
              {VERB[e.kind]}
              {e.kind === "search" ? `“${e.query}”` : e.title?.name}{e.episode ? ` · ${e.episode}` : ""}
              <span slot="supporting-text">{ago(e.at)}{e.source && e.source !== "download" ? ` · ${e.source}` : ""}</span>
              <m3e-icon-button slot="trailing" aria-label="Remove" onclick={() => personalize.forgetEvent(e.at)}>
                <m3e-icon name="close"></m3e-icon>
              </m3e-icon-button>
            </m3e-list-item>
          {/each}
        </m3e-list>
      {:else}
        <p class="empty">
          {personalize.learning ? "Nothing yet. Search, open or watch something and it shows up here." : "Learning is off, so nothing new is recorded."}
        </p>
      {/if}
    </section>
  </div>
  <div slot="actions" class="actions">
    <m3e-button variant="text" disabled={!personalize.events.length && !personalize.notes.length} onclick={clearAll}>
      Clear everything
    </m3e-button>
    <m3e-button variant="filled" onclick={() => dialog.hide()}>Done</m3e-button>
  </div>
</m3e-dialog>

<style>
  .pz {
    --m3e-dialog-min-width: 520px;
    --m3e-dialog-max-width: 620px;
    --m3e-dialog-max-height: 80vh;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  p {
    margin: 0;
    line-height: 1.5;
    color: var(--md-sys-color-on-surface-variant);
  }
  .lead {
    font-size: 14px;
  }
  .empty {
    font-size: 13px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
