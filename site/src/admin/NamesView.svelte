<script lang="ts">
  // Usernames nobody may take, and words nobody may write. The blocked list is
  // shared: usernames, display names, bios, reviews, list names and Theatre chat.
  import { onMount } from "svelte";
  import { call, say, sayError } from "./api";

  let reserved = $state<string[]>([]);
  let words = $state<string[]>([]);
  let loading = $state(true);
  let name = $state("");
  let word = $state("");
  let busy = $state(false);

  async function load() {
    try {
      const r = await call<{ reserved: string[]; words: string[] }>("names");
      reserved = r.reserved;
      words = r.words;
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  async function change(list: "reserved" | "words", op: "add" | "del", value: string) {
    value = value.trim();
    if (!value || busy) return;
    busy = true;
    try {
      const r = await call<{ holder: { username: string } | null }>("namesSet", { list, op, value });
      if (list === "reserved") {
        say(op === "add" ? (r.holder ? `Reserved. Note: @${r.holder.username} already has it.` : "Reserved.") : "Released.");
        name = op === "add" ? "" : name;
      } else {
        say(op === "add" ? "Blocked." : "Unblocked.");
        word = op === "add" ? "" : word;
      }
      await load();
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
      <h1>Names &amp; words</h1>
      <p>Reserved usernames can't be picked. Blocked words can't be written on profiles, in reviews or in Theatre chat. Neither changes what people already have.</p>
    </div>
  </div>

  {#if loading}
    <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
  {:else}
    <div class="grid cols-2">
      <section class="card">
        <h2><m3e-icon name="shield"></m3e-icon>Reserved usernames</h2>
        <form class="add" onsubmit={(e) => { e.preventDefault(); void change("reserved", "add", name); }}>
          <label class="field"><input bind:value={name} maxlength="20" placeholder="Reserve a username" aria-label="Reserve a username" autocomplete="off" spellcheck="false" /></label>
          <m3e-button variant="filled" disabled={busy || !name.trim()} onclick={() => change("reserved", "add", name)}>Reserve</m3e-button>
        </form>
        <div class="chips">
          {#each reserved as n (n)}
            <span class="tag">{n}<button type="button" aria-label="Release {n}" disabled={busy} onclick={() => change("reserved", "del", n)}><m3e-icon name="close"></m3e-icon></button></span>
          {/each}
        </div>
      </section>

      <section class="card">
        <h2><m3e-icon name="block"></m3e-icon>Blocked words</h2>
        <form class="add" onsubmit={(e) => { e.preventDefault(); void change("words", "add", word); }}>
          <label class="field"><input bind:value={word} maxlength="40" placeholder="Block a word" aria-label="Block a word" autocomplete="off" spellcheck="false" /></label>
          <m3e-button variant="filled" disabled={busy || !word.trim()} onclick={() => change("words", "add", word)}>Block</m3e-button>
        </form>
        {#if words.length}
          <div class="chips">
            {#each words as w (w)}
              <span class="tag">{w}<button type="button" aria-label="Unblock {w}" disabled={busy} onclick={() => change("words", "del", w)}><m3e-icon name="close"></m3e-icon></button></span>
            {/each}
          </div>
        {:else}
          <p class="muted none">No blocked words yet.</p>
        {/if}
        <p class="muted hint">Words match whole words. In usernames they match anywhere, because a username has no spaces.</p>
      </section>
    </div>
  {/if}
</div>

<style>
  .add {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-bottom: 16px;
  }
  .add .field {
    flex: 1;
    min-width: 0;
  }
  .chips {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    height: 32px;
    padding: 0 4px 0 12px;
    border-radius: 999px;
    background: var(--md-sys-color-surface-container-highest);
    font-size: 13px;
    font-weight: 600;
  }
  .tag button {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border: 0;
    border-radius: 50%;
    background: none;
    color: var(--md-sys-color-on-surface-variant);
    cursor: pointer;
    --m3e-icon-size: 16px;
    transition: background-color 140ms ease;
  }
  .tag button:hover:not(:disabled) {
    background: color-mix(in srgb, #ff5449 28%, transparent);
    color: #ffb4ab;
  }
  .none,
  .hint {
    margin: 12px 0 0;
    font-size: 13px;
  }
</style>
