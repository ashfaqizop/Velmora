<script lang="ts">
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../lib/api";
  import { ani } from "../lib/state.svelte";

  // The Gemini key Ani runs on. Used in onboarding and Settings → Ani.
  // Rust checks the key with Gemini before saving it and never hands it back.
  let draft = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let replacing = $state(false);

  ani.loadKey().catch((e) => console.warn("ani key status", e));

  async function save() {
    const key = draft.trim();
    if (!key || busy) return;
    busy = true;
    error = null;
    try {
      await ani.saveKey(key);
      draft = "";
      replacing = false;
      M3eSnackbar.open("Key saved. Ani is ready.");
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function remove() {
    busy = true;
    try {
      await ani.saveKey(null);
      M3eSnackbar.open("Key removed. Ani won't answer until you add one.");
    } catch (e) {
      M3eSnackbar.open(String(e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="ani-key">
  {#if ani.key?.set && !replacing}
    <div class="saved">
      <span class="on-shape badge" aria-hidden="true">
        <m3e-shape name="4-sided-cookie"></m3e-shape>
        <m3e-icon name="key" filled></m3e-icon>
      </span>
      <div class="text">
        <strong>Gemini key saved</strong>
        <span>Ends in {ani.key.hint?.replace("…", "")}. It stays on this PC.</span>
      </div>
      <m3e-button variant="text" disabled={busy} onclick={() => (replacing = true)}>Replace</m3e-button>
      <m3e-button variant="text" disabled={busy} onclick={remove}>Remove</m3e-button>
    </div>
  {:else}
    <div class="entry">
      <m3e-form-field class="field" variant="outlined" error={!!error}>
        <label slot="label" for="ani-key-input">Gemini API key</label>
        <input
          id="ani-key-input"
          type="password"
          autocomplete="off"
          spellcheck="false"
          bind:value={draft}
          oninput={() => (error = null)}
          onkeydown={(e) => e.key === "Enter" && save()}
        />
        <span slot="hint">Ani uses your own key. It stays on this PC.</span>
        <span slot="error">{error}</span>
      </m3e-form-field>
      <m3e-button variant="filled" disabled={busy || !draft.trim()} onclick={save}>
        {busy ? "Checking…" : "Save"}
      </m3e-button>
      {#if replacing}
        <m3e-button variant="text" onclick={() => ((replacing = false), (error = null))}>Cancel</m3e-button>
      {/if}
    </div>
    <m3e-button variant="text" size="small" class="get" onclick={() => api.openExternal("https://aistudio.google.com/apikey")}>
      <m3e-icon slot="icon" name="open_in_new"></m3e-icon>Get a free key from Google AI Studio
    </m3e-button>
  {/if}
</div>

<style>
  .ani-key {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }
  .saved {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .badge {
    --m3e-shape-size: 40px;
    --m3e-shape-container-color: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
    --m3e-icon-size: 20px;
    flex: none;
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
  .get {
    align-self: flex-start;
    margin-left: -12px;
  }
</style>
