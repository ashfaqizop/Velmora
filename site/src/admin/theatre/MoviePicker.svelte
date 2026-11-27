<script lang="ts">
  // Search TMDB for a movie and pick one.
  import { call, sayError } from "../api";
  import type { Movie } from "./types";

  let { onpick, label = "Search for a movie", shows = false }: { onpick: (m: Movie) => void; label?: string; shows?: boolean } = $props();

  let q = $state("");
  let results = $state<Movie[]>([]);
  let loading = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;

  function typing() {
    clearTimeout(timer);
    if (q.trim().length < 2) {
      results = [];
      return;
    }
    timer = setTimeout(async () => {
      const mine = ++seq;
      loading = true;
      try {
        const r = await call<{ results: Movie[] }>("theatreSearch", { q, shows });
        if (mine === seq) results = r.results;
      } catch (e) {
        if (mine === seq) sayError(e);
      } finally {
        if (mine === seq) loading = false;
      }
    }, 350);
  }

  function pick(m: Movie) {
    results = [];
    q = "";
    onpick(m);
  }
</script>

<div class="picker">
  <label class="field">
    <m3e-icon name="search"></m3e-icon>
    <input bind:value={q} oninput={typing} placeholder={label} aria-label={label} autocomplete="off" spellcheck="false" />
  </label>
  {#if results.length}
    <ul class="results">
      {#each results as m (m.k)}
        <li>
          <button type="button" onclick={() => pick(m)}>
            {#if m.poster}<img src={m.poster} alt="" width="36" height="54" loading="lazy" />{:else}<span class="blank"></span>{/if}
            <span class="t"><strong>{m.name}</strong><span class="muted">{m.year ?? "Year unknown"}</span></span>
          </button>
        </li>
      {/each}
    </ul>
  {:else if loading}
    <span class="muted hint">Searching…</span>
  {/if}
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .results {
    list-style: none;
    margin: 0;
    padding: 6px;
    display: flex;
    flex-direction: column;
    border-radius: 20px;
    background: var(--md-sys-color-surface-container-high);
    max-height: 300px;
    overflow-y: auto;
  }
  .results button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 8px;
    border: 0;
    border-radius: 14px;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: background-color 140ms ease;
  }
  .results button:hover,
  .results button:focus-visible {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
    outline: 0;
  }
  img,
  .blank {
    width: 36px;
    height: 54px;
    border-radius: 6px;
    object-fit: cover;
    background: var(--md-sys-color-surface-container-highest);
    flex: none;
  }
  .t {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .t .muted {
    font-size: 12.5px;
  }
  .hint {
    font-size: 13px;
    padding-left: 8px;
  }
</style>
