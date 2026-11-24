<script lang="ts">
  // Everything before the admin pages: sign in, "not an admin", and the
  // authenticator code (with a QR code the first time).
  import { fade } from "svelte/transition";
  import mark from "../assets/velmora-mark.webp";
  import { gate, leave, signIn, submitCode } from "./access.svelte";

  let code = $state("");
  let copied = $state(false);

  function typing(e: Event) {
    code = (e.target as HTMLInputElement).value.replace(/\D/g, "").slice(0, 6);
    if (code.length === 6) void submitCode(code).then(() => (code = ""));
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(gate.secret);
      copied = true;
      setTimeout(() => (copied = false), 1600);
    } catch {
      // Clipboard blocked: the secret is on screen to copy by hand.
    }
  }
</script>

<div class="gate">
  <div class="glow" aria-hidden="true"></div>
  {#key gate.phase}
    <div class="card box" in:fade={{ duration: 220 }}>
      <img src={mark} alt="" width="48" height="48" />
      <h1>Velmora <em>Admin</em></h1>

      {#if gate.phase === "loading"}
        <div class="spin"><m3e-loading-indicator></m3e-loading-indicator></div>
      {:else if gate.phase === "out"}
        <p class="muted">Only the owner gets in. You'll sign in on Velmora's sign-in page, then confirm with your authenticator app.</p>
        {#if gate.error}<div class="err">{gate.error}</div>{/if}
        <m3e-button variant="filled" disabled={gate.busy} onclick={signIn}>
          <m3e-icon slot="icon" name="login"></m3e-icon>Sign in
        </m3e-button>
      {:else if gate.phase === "denied"}
        <p class="muted">This account isn't an admin, so there is nothing for it here.</p>
        <m3e-button variant="tonal" onclick={() => leave()}><m3e-icon slot="icon" name="logout"></m3e-icon>Sign out</m3e-button>
      {:else}
        {#if gate.phase === "enroll"}
          <p class="muted">Scan this with an authenticator app (Google Authenticator, Authy, 1Password), then type the 6-digit code it shows.</p>
          <img class="qr" src={gate.qr} alt="QR code for your authenticator app" width="176" height="176" />
          <button class="secret mono" type="button" onclick={copy} title="Copy the setup key">
            {gate.secret}<m3e-icon name={copied ? "check" : "content_copy"}></m3e-icon>
          </button>
        {:else}
          <p class="muted">Type the 6-digit code from your authenticator app.</p>
        {/if}
        <input
          class="code mono"
          inputmode="numeric"
          autocomplete="one-time-code"
          maxlength="6"
          placeholder="000000"
          aria-label="6-digit code"
          value={code}
          oninput={typing}
          disabled={gate.busy}
        />
        {#if gate.error}<div class="err" role="alert">{gate.error}</div>{/if}
        <m3e-button variant="text" onclick={() => leave()}>Use a different account</m3e-button>
      {/if}
    </div>
  {/key}
</div>

<style>
  .gate {
    position: relative;
    min-height: 100vh;
    display: grid;
    place-items: center;
    padding: 24px 16px;
    background: var(--md-sys-color-surface, #0a0d16);
    overflow: hidden;
  }
  .glow {
    position: absolute;
    inset: -20% -10% auto;
    height: 640px;
    pointer-events: none;
    background: radial-gradient(40% 50% at 50% 10%, color-mix(in srgb, var(--md-sys-color-primary) 24%, transparent), transparent 70%);
  }
  .box {
    position: relative;
    width: min(420px, 100%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 32px 28px;
    text-align: center;
    border-radius: 32px;
  }
  .box img:first-child {
    border-radius: 14px;
  }
  h1 {
    margin: 0;
    font-size: 24px;
    font-weight: 800;
  }
  h1 em {
    font-family: var(--display);
    font-style: italic;
    font-weight: 400;
    color: var(--cream);
  }
  p {
    margin: 0;
    font-size: 14px;
    line-height: 1.5;
  }
  .qr {
    border-radius: 16px;
    background: #fff;
    padding: 8px;
  }
  .secret {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    max-width: 100%;
    padding: 8px 14px;
    border: 0;
    border-radius: 12px;
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface-variant);
    cursor: pointer;
    word-break: break-all;
    --m3e-icon-size: 16px;
  }
  .code {
    width: 100%;
    height: 56px;
    border: 0;
    border-radius: 16px;
    outline: 0;
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface);
    text-align: center;
    font-size: 26px;
    letter-spacing: 0.4em;
    padding-left: 0.4em;
    transition: box-shadow 160ms ease;
  }
  .code:focus {
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--md-sys-color-primary) 70%, transparent);
  }
  .err {
    width: 100%;
  }
  .spin {
    padding: 24px;
  }
</style>
