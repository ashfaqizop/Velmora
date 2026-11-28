<script lang="ts">
  import { PLATFORMS, rememberPlatform, type Platform } from "../lib/platform";
  import WindowsIcon from "./WindowsIcon.svelte";

  // A pill switch between Windows and Android, the thumb springing across.
  // The side for the device you're on is marked.
  let { value = $bindable(), detected = null }: { value: Platform; detected?: Platform | null } = $props();

  const LIST: Platform[] = ["windows", "android"];

  function pick(p: Platform) {
    value = p;
    rememberPlatform(p);
  }
</script>

<div class="switch" role="tablist" aria-label="Platform" style="--i: {LIST.indexOf(value)}">
  <span class="thumb {value}" aria-hidden="true"></span>
  {#each LIST as p (p)}
    <button role="tab" aria-selected={value === p} class:on={value === p} onclick={() => pick(p)}>
      {#if p === "windows"}<WindowsIcon size={18} />{:else}<m3e-icon name={PLATFORMS[p].icon}></m3e-icon>{/if}
      {PLATFORMS[p].name}
      {#if detected === p}<span class="you">Your device</span>{/if}
    </button>
  {/each}
</div>

<style>
  .switch {
    position: relative;
    display: inline-grid;
    grid-template-columns: 1fr 1fr;
    margin-top: 30px;
    padding: 6px;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: var(--glass);
    backdrop-filter: blur(14px);
  }
  .thumb {
    position: absolute;
    top: 6px;
    bottom: 6px;
    left: 6px;
    width: calc(50% - 6px);
    border-radius: 999px;
    transform: translateX(calc(var(--i) * 100%));
    transition:
      transform 0.55s cubic-bezier(0.34, 1.45, 0.5, 1),
      background 0.4s,
      box-shadow 0.4s;
    background: linear-gradient(135deg, #5d8eff, var(--blue));
    box-shadow: 0 10px 30px -10px rgb(50 113 254 / 70%);
  }
  .thumb.android {
    background: linear-gradient(135deg, #6ff0a6, #3ddc84);
    box-shadow: 0 10px 30px -10px rgb(61 220 132 / 70%);
  }
  button {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-width: 150px;
    padding: 12px 22px;
    border: 0;
    border-radius: 999px;
    background: none;
    font: inherit;
    font-weight: 600;
    font-size: 15px;
    color: var(--muted);
    cursor: pointer;
    transition: color 0.3s;
    --m3e-icon-size: 20px;
  }
  button.on {
    color: #fff;
  }
  .thumb.android ~ button.on {
    color: #06210f;
  }
  .you {
    padding: 2px 7px;
    border-radius: 999px;
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    background: rgb(255 255 255 / 18%);
  }
  button:not(.on) .you {
    background: rgb(255 255 255 / 8%);
  }
  @media (max-width: 600px) {
    .switch {
      display: grid;
      width: 100%;
    }
    button {
      min-width: 0;
      padding: 11px 12px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .thumb {
      transition: none;
    }
  }
</style>
