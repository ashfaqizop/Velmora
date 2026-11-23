<script lang="ts">
  // Shown while the browser has the sign-in. It closes by itself when the
  // browser hands the result back. The typed code is the way out if a
  // firewall stops the browser reaching the app.
  import { fade, scale } from "svelte/transition";
  import { spring } from "../lib/motion.svelte";
  import { account } from "../lib/account.svelte";
  import Logo from "./Logo.svelte";

  let code = $state("");
  let busy = $state(false);
  let manual = $state(false);

  $effect(() => {
    if (account.status !== "signing-in") {
      code = "";
      manual = false;
      busy = false;
    }
  });

  async function redeem() {
    if (busy || code.replace(/\s/g, "").length < 8) return;
    busy = true;
    await account.redeem(code);
    busy = false;
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && account.status === "signing-in" && account.cancel()} />

{#if account.status === "signing-in"}
  <div class="scrim" role="presentation" transition:fade={spring("fast-effects")}>
    <div
      class="card"
      role="dialog"
      aria-modal="true"
      aria-labelledby="signin-title"
      transition:scale={{ start: 0.92, opacity: 0, ...spring("default-spatial") }}
    >
      <Logo size={36} />
      <h2 id="signin-title">Finish signing in</h2>
      <p>
        Velmora opened your browser. Sign in or sign up there, and you'll come straight back here.
      </p>
      <div class="wait"><span class="spinner" aria-hidden="true"></span>Waiting for the browser…</div>

      {#if manual}
        <div class="manual">
          <p class="small">The page shows an 8-character code if it can't open Velmora. Type it here.</p>
          <div class="entry">
            <input
              class="code"
              maxlength="9"
              autocomplete="off"
              spellcheck="false"
              placeholder="ABCD 2345"
              bind:value={code}
              onkeydown={(e) => e.key === "Enter" && redeem()}
              aria-label="Sign-in code"
            />
            <m3e-button variant="filled" disabled={busy || code.replace(/\s/g, "").length < 8} onclick={redeem}>
              {busy ? "Checking…" : "Continue"}
            </m3e-button>
          </div>
        </div>
      {/if}

      {#if account.error}<p class="error" role="alert">{account.error}</p>{/if}

      <div class="actions">
        {#if !manual}
          <m3e-button variant="text" onclick={() => (manual = true)}>Enter a code instead</m3e-button>
        {/if}
        <m3e-button variant="tonal" onclick={() => account.cancel()}>Cancel</m3e-button>
      </div>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 2000;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgb(0 0 0 / 55%);
    backdrop-filter: blur(6px);
  }
  .card {
    width: min(440px, 100%);
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 28px;
    border-radius: 28px;
    background: var(--md-sys-color-surface-container-high);
    box-shadow: 0 24px 64px rgb(0 0 0 / 45%);
  }
  h2 {
    margin: 4px 0 0;
    font-size: 24px;
    font-weight: 700;
  }
  p {
    margin: 0;
    line-height: 1.55;
    color: var(--md-sys-color-on-surface-variant);
  }
  .small {
    font-size: 13px;
  }
  .wait {
    display: flex;
    align-items: center;
    gap: 12px;
    font-weight: 600;
  }
  .spinner {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 3px solid color-mix(in srgb, var(--md-sys-color-primary) 25%, transparent);
    border-top-color: var(--md-sys-color-primary);
    animation: spin 0.8s linear infinite;
  }
  .manual {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-top: 6px;
  }
  .entry {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .code {
    all: unset;
    box-sizing: border-box;
    flex: 1;
    min-width: 0;
    height: 48px;
    padding: 0 14px;
    border-radius: 14px;
    border: 1px solid var(--md-sys-color-outline-variant);
    background: var(--md-sys-color-surface-container);
    color: var(--md-sys-color-on-surface);
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-size: 20px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    text-align: center;
  }
  .code:focus {
    border-color: var(--md-sys-color-primary);
  }
  .error {
    color: var(--md-sys-color-error);
    font-size: 14px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
</style>
