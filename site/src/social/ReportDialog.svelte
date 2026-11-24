<script lang="ts">
  // Report a profile. Only the Velmora admin reads reports.
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import { report, type ReportReason } from "./data.svelte";

  let { target, username, open = $bindable(false) }: { target: string; username: string; open?: boolean } = $props();

  const REASONS: { id: ReportReason; label: string }[] = [
    { id: "username", label: "Username" },
    { id: "display_name", label: "Display name" },
    { id: "avatar", label: "Profile picture" },
    { id: "bio", label: "Bio" },
    { id: "links", label: "Links" },
    { id: "spam", label: "Spam or a fake account" },
    { id: "other", label: "Something else" },
  ];

  let reason = $state<ReportReason | null>(null);
  let note = $state("");
  let busy = $state(false);
  let dialog = $state<HTMLElement & { show(): void; hide(): void }>();

  $effect(() => {
    if (!dialog) return;
    if (open) dialog.show();
    else dialog.hide();
  });

  async function send() {
    if (!reason) return;
    busy = true;
    try {
      await report(target, reason, note);
      open = false;
      reason = null;
      note = "";
      M3eSnackbar.open("Thanks. The report is in, and only the Velmora team sees it.");
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    } finally {
      busy = false;
    }
  }
</script>

<m3e-dialog bind:this={dialog} dismissible onclosed={() => (open = false)}>
  <span slot="header">Report @{username}</span>
  <div class="body">
    <p class="lead">What's wrong with this profile? @{username} won't know who reported them.</p>
    <m3e-filter-chip-set class="reasons" aria-label="Reason">
      {#each REASONS as r (r.id)}
        <m3e-filter-chip selected={reason === r.id} onclick={() => (reason = r.id)}>{r.label}</m3e-filter-chip>
      {/each}
    </m3e-filter-chip-set>
    <m3e-form-field variant="outlined" class="field">
      <label slot="label" for="report-note">More details (optional)</label>
      <textarea id="report-note" rows="3" maxlength="500" bind:value={note}></textarea>
      <span slot="hint">{note.length}/500</span>
    </m3e-form-field>
  </div>
  <div slot="actions" class="actions">
    <m3e-button variant="text" onclick={() => (open = false)}>Cancel</m3e-button>
    <m3e-button variant="filled" disabled={!reason || busy} onclick={send}>
      <m3e-icon slot="icon" name="flag"></m3e-icon>{busy ? "Sending…" : "Send report"}
    </m3e-button>
  </div>
</m3e-dialog>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 460px;
  }
  .lead {
    margin: 0;
    color: var(--md-sys-color-on-surface-variant);
  }
  .field {
    width: 100%;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
