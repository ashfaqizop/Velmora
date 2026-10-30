<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import Logo from "./Logo.svelte";
  import PoweredBy from "./PoweredBy.svelte";
  import * as api from "../lib/api";
  import { markConfigReady } from "../lib/state.svelte";

  let { ondone }: { ondone: () => void } = $props();

  type Step =
    | { kind: "checking" }
    | { kind: "downloading"; version: string; connecting: boolean; percent: number }
    | { kind: "installing"; version: string }
    | { kind: "config" }
    | { kind: "ready"; note: string };

  let shown = $state(false);
  let docked = $state(false);
  let leaving = $state(false);
  let step = $state<Step | null>(null);

  const SHAPES = ["12-sided-cookie", "soft-burst", "flower", "sunny", "puffy-diamond", "9-sided-cookie"] as const;
  let morph = $state(0);
  let shapeA = $derived(SHAPES[morph % SHAPES.length]);
  let shapeB = $derived(SHAPES[(morph + 3) % SHAPES.length]);

  const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
  /** Keep each status on screen long enough to be read. */
  async function atLeast<T>(ms: number, work: Promise<T>): Promise<T> {
    const [value] = await Promise.all([work, sleep(ms)]);
    return value;
  }

  const copy = $derived.by(() => {
    switch (step?.kind) {
      case "checking":
        return { title: "Checking for updates", detail: "Looking for a newer Velmora on GitHub" };
      case "downloading":
        return {
          title: "Downloading update",
          detail: step.connecting ? "Connecting to the download server" : `Velmora ${step.version}`,
        };
      case "installing":
        return { title: "Installing update", detail: "Velmora restarts by itself when it's done" };
      case "config":
        return { title: "Updating config", detail: "Fetching the latest sources and mirrors" };
      case "ready":
        return { title: "All set", detail: step.note };
      default:
        return null;
    }
  });

  async function checkForUpdate(): Promise<boolean> {
    const demo = !api.inTauri && new URLSearchParams(location.search).has("demo-update");
    if (!api.inTauri) {
      await atLeast(1100, Promise.resolve());
      if (demo) await fakeDownload();
      return demo;
    }
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await atLeast(1100, check({ timeout: 8000 })).catch((e) => {
      console.warn("update check failed", e);
      return null;
    });
    if (!update) return false;

    step = { kind: "downloading", version: update.version, connecting: true, percent: 0 };
    let total = 0;
    let received = 0;
    try {
      await update.downloadAndInstall((ev) => {
        if (step?.kind !== "downloading") return;
        if (ev.event === "Started") {
          total = ev.data.contentLength ?? 0;
        } else if (ev.event === "Progress") {
          received += ev.data.chunkLength;
          step.connecting = false;
          step.percent = total ? Math.min(100, (received / total) * 100) : 0;
        } else if (ev.event === "Finished") {
          step = { kind: "installing", version: update.version };
        }
      });
      // Windows exits into the installer above; other platforms relaunch here.
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
      return true;
    } catch (e) {
      console.warn("update failed", e);
      return false;
    }
  }

  /** Browser preview of the download states (`?demo-update`). */
  async function fakeDownload() {
    step = { kind: "downloading", version: "0.2.0", connecting: true, percent: 0 };
    await sleep(1600);
    for (let p = 0; p <= 100; p += 2) {
      if (step.kind !== "downloading") break;
      step.connecting = false;
      step.percent = p;
      await sleep(45);
    }
    step = { kind: "installing", version: "0.2.0" };
    await sleep(1400);
  }

  async function run() {
    await sleep(60);
    shown = true;
    await sleep(1150);
    docked = true;
    await sleep(520);

    step = { kind: "checking" };
    const restarting = await checkForUpdate();
    if (restarting && api.inTauri) return;

    step = { kind: "config" };
    const status = await atLeast(900, api.refreshConfig().catch(() => ({ updated: false, message: "offline" })));
    markConfigReady();
    step = { kind: "ready", note: status.updated ? "Config is up to date" : "Couldn't reach GitHub, using saved config" };

    await sleep(650);
    leaving = true;
    await sleep(700);
    ondone();
  }

  onMount(() => {
    const timer = setInterval(() => morph++, 2600);
    run();
    return () => clearInterval(timer);
  });
</script>

<div class="splash" class:docked class:leaving data-tauri-drag-region>
  <div class="ambient" aria-hidden="true">
    <m3e-shape class="blob a" name={shapeA}></m3e-shape>
    <m3e-shape class="blob b" name={shapeB}></m3e-shape>
  </div>

  <div class="brand">
    <div class="intro" class:shown>
      <Logo size={96} />
    </div>
  </div>

  {#if step && copy}
    <section class="status" in:fade={{ duration: 400 }} aria-live="polite">
      <div class="text">
        {#key copy.title}
          <div class="line" in:fly={{ y: 18, duration: 520, easing: cubicOut }} out:fade={{ duration: 160 }}>
            <m3e-heading variant="display" size="small" level={1}>{copy.title}</m3e-heading>
          </div>
        {/key}
        {#key copy.detail}
          <p class="detail" in:fade={{ duration: 300, delay: 120 }} out:fade={{ duration: 120 }}>{copy.detail}</p>
        {/key}
      </div>

      <div class="indicator">
        {#if step.kind === "downloading"}
          <div class="ring" class:visible={step.connecting}>
            <m3e-circular-progress-indicator variant="wavy" indeterminate></m3e-circular-progress-indicator>
          </div>
          <div class="ring" class:visible={!step.connecting}>
            <m3e-circular-progress-indicator variant="wavy" value={step.percent}>
              <span class="pct">{Math.round(step.percent)}%</span>
            </m3e-circular-progress-indicator>
          </div>
        {:else if step.kind === "installing"}
          <div class="ring visible">
            <m3e-circular-progress-indicator variant="wavy" indeterminate></m3e-circular-progress-indicator>
          </div>
        {:else if step.kind === "ready"}
          <div class="ring visible" in:fade={{ duration: 250 }}>
            <m3e-icon class="done" name="check_circle" filled></m3e-icon>
          </div>
        {:else}
          <div class="ring visible">
            <m3e-loading-indicator variant="contained"></m3e-loading-indicator>
          </div>
        {/if}
      </div>
    </section>

    <footer class="credit" in:fade={{ duration: 600, delay: 200 }}>
      <PoweredBy height={13} />
    </footer>
  {/if}
</div>

<style>
  .splash {
    position: fixed;
    inset: 0;
    z-index: 100;
    overflow: hidden;
    background:
      radial-gradient(120% 80% at 50% 120%, color-mix(in srgb, var(--md-sys-color-primary) 16%, transparent), transparent 60%),
      #0c0a13;
    transition: opacity 650ms var(--md-sys-motion-easing-emphasized, ease), filter 650ms ease;
  }
  .splash.leaving {
    opacity: 0;
    filter: blur(6px);
    pointer-events: none;
  }

  /* Morphing glow behind everything. */
  .ambient {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  .blob {
    position: absolute;
    left: 50%;
    top: 50%;
    --m3e-shape-size: 520px;
    --m3e-shape-transition: 2400ms cubic-bezier(0.34, 0.8, 0.34, 1);
    filter: blur(90px);
    opacity: 0;
    transition:
      opacity 1400ms ease,
      transform 1800ms var(--md-sys-motion-easing-emphasized, ease);
  }
  .blob.a {
    --m3e-shape-container-color: var(--md-sys-color-primary);
    transform: translate(-75%, -60%) rotate(0deg);
    animation: spin 40s linear infinite;
  }
  .blob.b {
    --m3e-shape-container-color: var(--brand-coral);
    transform: translate(-20%, -35%) scale(0.8);
    animation: spin 55s linear infinite reverse;
  }
  .splash:has(.intro.shown) .blob.a {
    opacity: 0.42;
  }
  .splash:has(.intro.shown) .blob.b {
    opacity: 0.22;
  }
  .docked .blob.a {
    transform: translate(-110%, -10%) scale(1.25);
  }
  .docked .blob.b {
    transform: translate(10%, -70%) scale(1.1);
  }
  .leaving .blob {
    opacity: 0 !important;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }

  /* Logo: centred and large, then docks where the shell's logo sits. */
  .brand {
    position: absolute;
    left: 50%;
    top: 50%;
    transform-origin: top left;
    transform: translate(-50%, -50%) scale(1);
    transition:
      left 900ms var(--md-sys-motion-easing-emphasized, ease),
      top 900ms var(--md-sys-motion-easing-emphasized, ease),
      transform 900ms var(--md-sys-motion-easing-emphasized, ease);
  }
  .docked .brand {
    left: 28px;
    top: 20px;
    transform: translate(0, 0) scale(0.3334);
  }
  .intro {
    opacity: 0;
    transform: scale(0.9);
    filter: blur(14px);
    transition:
      opacity 900ms ease,
      transform 1100ms var(--md-sys-motion-easing-emphasized-decelerate, ease-out),
      filter 900ms ease;
  }
  .intro.shown {
    opacity: 1;
    transform: none;
    filter: none;
  }

  /* Status row: title left, indicator right. */
  .status {
    position: absolute;
    left: 50%;
    top: 50%;
    width: min(880px, calc(100% - 96px));
    transform: translate(-50%, -50%);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 48px;
  }
  .text {
    display: grid;
    min-width: 0;
  }
  .text > * {
    grid-area: 1 / 1;
  }
  .line {
    align-self: end;
  }
  .line m3e-heading {
    color: var(--md-sys-color-on-surface);
  }
  .detail {
    grid-area: 2 / 1;
    margin: 10px 0 0;
    font-size: 16px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .indicator {
    position: relative;
    flex: none;
    width: 112px;
    height: 112px;
  }
  .ring {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    opacity: 0;
    transform: scale(0.85);
    transition:
      opacity 450ms ease,
      transform var(--md-sys-motion-spring-default-spatial, 500ms ease);
  }
  .ring.visible {
    opacity: 1;
    transform: none;
  }
  m3e-loading-indicator {
    --m3e-loading-indicator-container-size: 96px;
    --m3e-loading-indicator-active-indicator-size: 72px;
  }
  m3e-circular-progress-indicator {
    --m3e-circular-wavy-progress-indicator-diameter: 104px;
    --m3e-circular-progress-indicator-thickness: 6px;
  }
  .pct {
    font-size: 18px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--md-sys-color-on-surface);
  }
  .done {
    --m3e-icon-size: 72px;
    color: var(--md-sys-color-primary);
  }

  .credit {
    position: absolute;
    left: 50%;
    bottom: 32px;
    transform: translateX(-50%);
    opacity: 0.85;
  }
</style>
