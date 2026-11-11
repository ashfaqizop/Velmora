<script lang="ts">
  // Add songs to one of your YouTube Music playlists, or to a new one.
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../../lib/api";
  import Art from "./Art.svelte";
  import { musicUi } from "../../lib/music/ui.svelte";

  let dialog = $state<HTMLElement & { show(): Promise<void>; hide(): Promise<void> }>();
  let creating = $state("");
  let busy = $state(false);

  const songs = $derived(musicUi.addTarget);

  $effect(() => {
    if (songs?.length) {
      dialog?.show();
      musicUi.loadPlaylists();
    }
  });

  function done() {
    musicUi.addTarget = null;
    creating = "";
    dialog?.hide();
  }

  async function add(playlistId: string, title: string) {
    if (!songs?.length || busy) return;
    busy = true;
    try {
      await api.musicPlaylistAdd(playlistId, songs.map((s) => s.id));
      M3eSnackbar.open(`Added to ${title}`);
      done();
    } catch (e) {
      M3eSnackbar.open(String(e));
    } finally {
      busy = false;
    }
  }

  async function create() {
    const title = creating.trim();
    if (!title || !songs?.length || busy) return;
    busy = true;
    try {
      await api.musicPlaylistCreate(title, songs.map((s) => s.id));
      M3eSnackbar.open(`Made a new playlist, ${title}`);
      musicUi.loadPlaylists();
      done();
    } catch (e) {
      M3eSnackbar.open(String(e));
    } finally {
      busy = false;
    }
  }
</script>

<m3e-dialog bind:this={dialog} dismissible onclosed={() => (musicUi.addTarget = null)}>
  <span slot="header">Add to playlist</span>
  <div class="body">
    <div class="new">
      <m3e-form-field variant="outlined" class="field">
        <label slot="label" for="new-playlist">New playlist</label>
        <input id="new-playlist" bind:value={creating} onkeydown={(e) => e.key === "Enter" && create()} />
      </m3e-form-field>
      <m3e-button variant="filled" disabled={!creating.trim() || busy} onclick={create}>Create</m3e-button>
    </div>
    <div class="list">
      {#each musicUi.playlists as p (p.id)}
        <button onclick={() => add(p.id, p.title)} disabled={busy}>
          <Art thumbs={p.thumbnails} size={44} alt="" />
          <span>
            <strong>{p.title}</strong>
            {#if p.subtitle}<small>{p.subtitle}</small>{/if}
          </span>
        </button>
      {:else}
        <p class="empty">No playlists of your own yet. Name one above.</p>
      {/each}
    </div>
  </div>
</m3e-dialog>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: min(440px, 80vw);
  }
  .new {
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }
  .new m3e-button {
    margin-top: 8px;
  }
  .field {
    flex: 1;
  }
  .list {
    display: flex;
    flex-direction: column;
    max-height: 50vh;
    overflow-y: auto;
  }
  .list button {
    all: unset;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    border-radius: 14px;
    cursor: pointer;
  }
  .list button:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 7%, transparent);
  }
  .list span {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  small {
    color: var(--md-sys-color-on-surface-variant);
  }
  .empty {
    color: var(--md-sys-color-on-surface-variant);
    font-size: 14px;
  }
</style>
