<script lang="ts">
  // Settings → License. Public is the full app with a few quiet ads; a Max
  // key turns them off on this PC. The checks run in Rust (license.rs).
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../lib/api";
  import { license } from "../lib/license.svelte";

  let draft = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let confirmRemove = $state(false);
  /** Showing the key field while Max is on, to switch to another key. */
  let changing = $state(false);

  const s = $derived(license.status);
  const until = $derived(s.validUntil ? new Date(s.validUntil * 1000).toLocaleDateString() : "");

  const PROBLEM: Record<string, { icon: string; title: string; text: string }> = {
    revoked: { icon: "block", title: "This key was turned off", text: "Ads are back. Ask the person who gave you the key." },
    other_device: {
      icon: "devices",
      title: "This key is active on another PC",
      text: "A Max key works on one PC at a time. Remove it there, or ask for it to be released.",
    },
    expired: {
      icon: "sync_problem",
      title: "Max couldn't be checked for 30 days",
      text: "Ads are back until Velmora can reach the license server again.",
    },
    pending: { icon: "sync_problem", title: "Max isn't active yet", text: "Velmora couldn't reach the license server to activate it." },
  };

  async function activate() {
    const key = draft.trim();
    if (!key || busy) return;
    busy = true;
    error = null;
    try {
      await license.activate(key);
      draft = "";
      changing = false;
      M3eSnackbar.open(`Velmora Max is on${license.status.name ? `, ${license.status.name}` : ""}. No more ads on this PC.`);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function retry() {
    busy = true;
    try {
      license.status = await api.licenseRefresh();
      if (!license.max) M3eSnackbar.open("Still couldn't activate Max. Try again later.");
    } catch (e) {
      M3eSnackbar.open(String(e));
    } finally {
      busy = false;
    }
  }

  async function remove() {
    if (!confirmRemove) {
      confirmRemove = true;
      setTimeout(() => (confirmRemove = false), 4000);
      return;
    }
    busy = true;
    try {
      await license.remove();
      confirmRemove = false;
      M3eSnackbar.open("Max removed from this PC. The key can be used on another one.");
    } catch (e) {
      M3eSnackbar.open(String(e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="license">
  {#if license.max}
    <div class="state">
      <span class="on-shape badge max" aria-hidden="true">
        <m3e-shape name="12-sided-cookie"></m3e-shape>
        <m3e-icon name="workspace_premium" filled></m3e-icon>
      </span>
      <div class="text">
        <strong>Velmora Max{s.name ? `, licensed to ${s.name}` : ""}</strong>
        <span>No ads on this PC. Key {s.id}{until ? `, checked until ${until}` : ""}.</span>
      </div>
      {#if !changing}
        <m3e-button variant="text" disabled={busy} onclick={() => (changing = true)}>Change key</m3e-button>
      {/if}
      <m3e-button variant="text" disabled={busy} onclick={remove}>
        {confirmRemove ? "Click again to remove" : "Remove from this PC"}
      </m3e-button>
    </div>
    {#if changing}{@render entry()}{/if}
  {:else if s.state !== "none" && PROBLEM[s.state]}
    {@const p = PROBLEM[s.state]}
    <div class="state">
      <span class="on-shape badge warn" aria-hidden="true">
        <m3e-shape name="4-sided-cookie"></m3e-shape>
        <m3e-icon name={p.icon}></m3e-icon>
      </span>
      <div class="text">
        <strong>{p.title}</strong>
        <span>{p.text}</span>
      </div>
      {#if s.state === "expired" || s.state === "pending"}
        <m3e-button variant="tonal" disabled={busy} onclick={retry}>Try again</m3e-button>
      {/if}
      <m3e-button variant="text" disabled={busy} onclick={remove}>
        {confirmRemove ? "Click again to remove" : "Remove key"}
      </m3e-button>
    </div>
  {:else}
    <div class="state">
      <span class="on-shape badge" aria-hidden="true">
        <m3e-shape name="circle"></m3e-shape>
        <m3e-icon name="verified"></m3e-icon>
      </span>
      <div class="text">
        <strong>Velmora Public</strong>
        <span>The full app, free, with a few quiet ads that keep it going. A Max key turns them off.</span>
      </div>
    </div>
    {@render entry()}
  {/if}
</div>

{#snippet entry()}
    <div class="entry">
      <m3e-form-field class="field" variant="outlined" error={!!error}>
        <label slot="label" for="max-key-input">Max license key</label>
        <input
          id="max-key-input"
          type="password"
          autocomplete="off"
          spellcheck="false"
          bind:value={draft}
          oninput={() => (error = null)}
          onkeydown={(e) => e.key === "Enter" && activate()}
        />
        <span slot="hint">
          {changing ? "The current key is freed once the new one activates." : "Starts with VLM1. It works on one PC at a time."}
        </span>
        <span slot="error">{error}</span>
      </m3e-form-field>
      <m3e-button variant="filled" disabled={busy || !draft.trim()} onclick={activate}>
        {busy ? "Activating…" : "Activate"}
      </m3e-button>
      {#if changing}
        <m3e-button variant="text" onclick={() => ((changing = false), (error = null), (draft = ""))}>Cancel</m3e-button>
      {/if}
    </div>
{/snippet}

<style>
  .license {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-width: 0;
  }
  .state {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .badge {
    --m3e-shape-size: 40px;
    --m3e-shape-container-color: var(--md-sys-color-surface-container-highest);
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 20px;
    flex: none;
  }
  .badge.max {
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
  }
  .badge.warn {
    --m3e-shape-container-color: var(--md-sys-color-error-container);
    color: var(--md-sys-color-on-error-container);
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  .text strong {
    font-weight: 700;
  }
  .text span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .entry {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }
  .entry m3e-button {
    margin-top: 8px;
  }
  .field {
    flex: 1;
    min-width: 0;
  }
  .field input {
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
  }
</style>
