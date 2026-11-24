<script lang="ts">
  // How many are watching, and who (friends first). Each person opens a card.
  import { fly } from "svelte/transition";
  import Avatar from "../Avatar.svelte";
  import Who from "./Who.svelte";
  import { room } from "../../lib/room.svelte";
  import { theatre } from "../../lib/theatre.svelte";

  let open = $state(false);
  const count = $derived(room.watchers.length);
</script>

<div class="watchers">
  <button type="button" class="pill" aria-expanded={open} onclick={() => (open = !open)}>
    <span class="faces">
      {#each room.watchers.slice(0, 4) as w (w.uid)}<span class="face"><Avatar value={w.avatar} size={22} /></span>{/each}
    </span>
    <m3e-icon name="visibility"></m3e-icon>
    <strong>{count}</strong><span class="word">watching</span>
  </button>

  {#if open}
    <div class="list" transition:fly={{ y: -6, duration: 200 }}>
      <div class="list-head"><strong>{count} watching</strong><m3e-icon-button size="small" aria-label="Close" onclick={() => (open = false)}><m3e-icon name="close"></m3e-icon></m3e-icon-button></div>
      <div class="rows">
        {#each room.watchers as w (w.uid)}
          <div class="row">
            <Who uid={w.uid} username={w.username}><Avatar value={w.avatar} size={32} /></Who>
            <Who uid={w.uid} username={w.username}>
              <span class="name">{w.display_name}{#if w.verified}<m3e-icon name="verified" filled class="tick"></m3e-icon>{/if}</span>
            </Who>
            {#if theatre.following.has(w.uid)}<span class="tag">Following</span>{/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .watchers {
    position: relative;
  }
  .pill {
    all: unset;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px 5px 6px;
    border-radius: 999px;
    font-size: 13px;
    background: rgb(255 255 255 / 10%);
    color: #fff;
    cursor: pointer;
    --m3e-icon-size: 16px;
    transition: background-color 160ms ease;
  }
  .pill:hover {
    background: rgb(255 255 255 / 16%);
  }
  .pill:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
  }
  .faces {
    display: flex;
  }
  .face {
    display: inline-grid;
    margin-left: -6px;
    border-radius: 50%;
    box-shadow: 0 0 0 2px #10162a;
  }
  .face:first-child {
    margin-left: 0;
  }
  .faces:empty {
    display: none;
  }
  .word {
    opacity: 0.75;
  }
  .list {
    position: absolute;
    z-index: 85;
    top: calc(100% + 8px);
    right: 0;
    width: 280px;
    max-height: 360px;
    display: flex;
    flex-direction: column;
    border-radius: 20px;
    background: color-mix(in srgb, #121a30 96%, transparent);
    border: 1px solid rgb(255 255 255 / 10%);
    box-shadow: 0 24px 60px -20px rgb(0 0 0 / 85%);
    color: #fff;
    overflow: hidden;
  }
  .list-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 8px 6px 16px;
  }
  .rows {
    overflow-y: auto;
    padding: 0 8px 10px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 8px;
    border-radius: 12px;
  }
  .row:hover {
    background: rgb(255 255 255 / 6%);
  }
  .name {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 14px;
    font-weight: 600;
  }
  .tick {
    --m3e-icon-size: 14px;
    color: var(--md-sys-color-primary);
  }
  .tag {
    margin-left: auto;
    font-size: 11px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
</style>
