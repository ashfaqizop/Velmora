<script lang="ts">
  // The first thing a PC without an account sees. Velmora needs an account from
  // 0.6.0 on; only this first sign-in needs the internet. After that the app
  // opens straight away, online or offline (see account.saved).
  import { fade, fly } from "svelte/transition";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Logo from "./Logo.svelte";
  import Avatar from "./Avatar.svelte";
  import { account } from "../lib/account.svelte";
  import { network } from "../lib/state.svelte";
  import { ANIMALS } from "../lib/avatars";
  import { spring } from "../lib/motion.svelte";

  const reason = account.gateReason;
  const win = getCurrentWindow();

  const TEXT = {
    new: {
      title: "Welcome to Velmora",
      body: "Sign in or create a free account to start. It takes a minute in your browser, with Google, Discord or an emailed code.",
    },
    upgrade: {
      title: "From now on, Velmora needs an account",
      body: "Sign in or create a free account on the web to keep going. Your list, your progress and your settings on this PC stay, and move into your account.",
    },
    returning: {
      title: "Sign in to continue",
      body: "You're signed out on this PC. Sign in again and everything comes back.",
    },
  }[reason];

  const PERKS = [
    { icon: "sync", text: "Your list, progress and settings on every PC" },
    { icon: "workspace_premium", text: "Max comes with you to a new PC" },
    { icon: "wifi_off", text: "After this, Velmora opens even offline" },
  ];
</script>

<div class="gate" role="dialog" aria-modal="true" aria-labelledby="gate-title" transition:fade={spring("default-effects")}>
  <div class="glow" aria-hidden="true"></div>
  <div class="drag" data-tauri-drag-region>
    <m3e-icon-button size="extra-small" aria-label="Minimize" onclick={() => win.minimize()}>
      <m3e-icon name="remove"></m3e-icon>
    </m3e-icon-button>
    <m3e-icon-button size="extra-small" class="close" aria-label="Close" onclick={() => win.close()}>
      <m3e-icon name="close"></m3e-icon>
    </m3e-icon-button>
  </div>

  <div class="faces" aria-hidden="true">
    {#each ANIMALS as a, i (a.id)}
      <span class="face" style:--i={i} style:--n={ANIMALS.length}><Avatar value={a.id} size={64} ring={2} /></span>
    {/each}
  </div>

  <main class="card" in:fly={{ y: 24, ...spring("default-spatial") }}>
    <Logo size={44} />
    <m3e-heading id="gate-title" variant="display" size="small" level={1}>{TEXT.title}</m3e-heading>
    <p class="body">{TEXT.body}</p>

    <ul class="perks">
      {#each PERKS as p (p.icon)}
        <li><m3e-icon name={p.icon}></m3e-icon>{p.text}</li>
      {/each}
    </ul>

    {#if !network.online}
      <p class="offline" role="status"><m3e-icon name="wifi_off"></m3e-icon>You're offline. The first sign-in needs the internet; this page carries on once you're connected.</p>
    {/if}
    {#if account.error}<p class="error" role="alert">{account.error}</p>{/if}

    <m3e-button
      class="go"
      variant="filled"
      size="large"
      disabled={!network.online || account.status === "signing-in"}
      onclick={() => account.signIn()}
    >
      <m3e-icon slot="icon" name="open_in_new"></m3e-icon>
      {account.status === "signing-in" ? "Waiting for your browser…" : "Continue on the web"}
    </m3e-button>
    <span class="fine">Opens auth.getvelmora.dpdns.org in your browser, then brings you straight back.</span>
  </main>
</div>

<style>
  .gate {
    position: fixed;
    inset: 0;
    z-index: 1500;
    display: grid;
    place-items: center;
    padding: 48px 24px;
    overflow: hidden;
    background: var(--md-sys-color-surface);
  }
  .glow {
    position: absolute;
    inset: -20%;
    pointer-events: none;
    background:
      radial-gradient(38% 40% at 22% 18%, color-mix(in srgb, var(--md-sys-color-primary) 30%, transparent), transparent 70%),
      radial-gradient(34% 36% at 82% 86%, color-mix(in srgb, #f8e8c8 14%, transparent), transparent 70%);
  }
  .drag {
    position: absolute;
    inset: 0 0 auto;
    height: 48px;
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 2px;
    padding: 0 12px;
    z-index: 2;
  }
  .drag .close:hover {
    --m3e-icon-button-icon-color: var(--md-sys-color-error);
  }
  .faces {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  .face {
    position: absolute;
    left: 50%;
    top: 50%;
    opacity: 0.55;
    --a: calc(360deg / var(--n) * var(--i));
    transform: rotate(var(--a)) translate(min(42vw, 460px)) rotate(calc(-1 * var(--a)));
    animation: orbit 90s linear infinite;
  }
  .card {
    position: relative;
    z-index: 1;
    width: min(520px, 100%);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
    padding: 36px;
    border-radius: 36px;
    background: color-mix(in srgb, var(--md-sys-color-surface-container) 90%, transparent);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 45%, transparent);
    box-shadow: 0 40px 100px -40px rgb(0 0 0 / 75%);
    backdrop-filter: blur(18px);
  }
  .body {
    margin: 0;
    font-size: 16px;
    line-height: 1.6;
    color: var(--md-sys-color-on-surface-variant);
  }
  .perks {
    list-style: none;
    margin: 4px 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .perks li {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 14px;
    --m3e-icon-size: 20px;
  }
  .perks m3e-icon {
    color: var(--md-sys-color-primary);
  }
  .offline,
  .error {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    border-radius: 16px;
    font-size: 14px;
    --m3e-icon-size: 20px;
  }
  .offline {
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .error {
    background: var(--md-sys-color-error-container);
    color: var(--md-sys-color-on-error-container);
  }
  .go {
    width: 100%;
    margin-top: 6px;
  }
  .fine {
    align-self: center;
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  @keyframes orbit {
    to {
      transform: rotate(calc(var(--a) + 360deg)) translate(min(42vw, 460px)) rotate(calc(-1 * var(--a) - 360deg));
    }
  }
  :global(html[data-motion="off"]) .face,
  :global(html[data-motion="reduced"]) .face {
    animation: none;
  }
</style>
