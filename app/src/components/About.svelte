<script lang="ts">
  import Logo from "./Logo.svelte";
  import PoweredBy from "./PoweredBy.svelte";
  import * as api from "../lib/api";
  import type { AppInfo } from "../lib/types";

  let dialog: HTMLElement & { show(): Promise<void> };
  let info = $state<AppInfo | null>(null);

  export function open() {
    api.appInfo().then((i) => (info = i));
    dialog.show();
  }
</script>

<m3e-dialog bind:this={dialog} dismissible class="about">
  <span slot="header">About</span>
  <div class="body">
    <Logo size={40} />
    <p class="version">Version {info?.version ?? "…"}{info ? ` · ${info.platform}` : ""}</p>
    <p>
      Movies, series and anime from many sources, searched and played in one place. Free software under the GPL-3.0
      licence.
    </p>
    <m3e-divider></m3e-divider>
    <div class="engine">
      <span class="label">Streaming engine</span>
      <PoweredBy variant="full" height={64} />
      <p class="small">
        UniverseAPI merges ani-cli, MovieBox, freeflix and HDHub into one engine.
        {#if info && !info.tmdb}Metadata comes from Cinemeta in this build.{/if}
      </p>
    </div>
  </div>
  <div slot="actions" {...{ end: "" }}>
    <m3e-button variant="filled"><m3e-dialog-action>Close</m3e-dialog-action></m3e-button>
  </div>
</m3e-dialog>

<style>
  .about {
    --m3e-dialog-max-width: 480px;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  p {
    margin: 0;
    line-height: 1.5;
  }
  .version {
    color: var(--md-sys-color-on-surface-variant);
    font-size: 13px;
  }
  .engine {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .label {
    font-size: 12px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
  }
  .small {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
