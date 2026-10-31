<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../lib/api";
  import { settings } from "../lib/state.svelte";

  // The folder downloads go to, with Change and Use default.
  // Used in Settings → Downloads and in onboarding.
  let folder = $state("");
  let fallback = $state("");
  let busy = $state(false);

  Promise.all([api.downloadFolder(), api.defaultDownloadFolder()])
    .then(([f, d]) => {
      folder = f;
      fallback = d;
    })
    .catch((e) => console.warn("download folder", e));

  async function change() {
    const picked = await open({ directory: true, multiple: false, defaultPath: folder || undefined, title: "Choose where downloads go" });
    if (typeof picked === "string") await apply(picked);
  }

  async function apply(path: string | null) {
    busy = true;
    try {
      folder = await api.setDownloadDir(path);
      // Keep the cached preferences in step, or the next Settings change would undo this.
      if (settings.prefs) settings.prefs.download_dir = path;
      M3eSnackbar.open(path ? "New downloads will go to the folder you picked" : "Downloads are back in the default folder");
    } catch (e) {
      M3eSnackbar.open(String(e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="folder">
  <span class="on-shape icon"><m3e-icon name="folder"></m3e-icon></span>
  <span class="path" title={folder}>{folder || "…"}</span>
  {#if folder && fallback && folder !== fallback}
    <m3e-button variant="text" size="small" disabled={busy} onclick={() => apply(null)}>Use default</m3e-button>
  {/if}
  <m3e-button variant="tonal" size="small" disabled={busy} onclick={change}>
    <m3e-icon slot="icon" name="drive_folder_upload"></m3e-icon>Change
  </m3e-button>
</div>

<style>
  .folder {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .icon {
    flex: none;
    width: 40px;
    height: 40px;
    border-radius: 12px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .path {
    flex: 1;
    min-width: 0;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
