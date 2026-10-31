<script lang="ts">
  let dialog: HTMLElement & { show(): Promise<void>; returnValue?: string };
  let resolve: ((ok: boolean) => void) | null = null;

  export function ask(): Promise<boolean> {
    dialog.show();
    return new Promise((r) => (resolve = r));
  }

  function closed() {
    resolve?.(dialog.returnValue === "ok");
    resolve = null;
  }
</script>

<m3e-dialog bind:this={dialog} onclosed={closed} class="notice">
  <span slot="header">Before you use torrents</span>
  <div class="body">
    <m3e-icon name="vpn_lock" class="icon"></m3e-icon>
    <p>
      Torrent sources download from other people, and your IP address is visible to them while it runs. In many
      countries that's enough to get a letter from your internet provider.
    </p>
    <p>Use a VPN if you aren't sure. Velmora stops uploading once a download finishes unless you turn seeding on in Settings.</p>
  </div>
  <div slot="actions" {...{ end: "" }}>
    <m3e-button><m3e-dialog-action return-value="cancel">Not now</m3e-dialog-action></m3e-button>
    <m3e-button variant="filled"><m3e-dialog-action return-value="ok">I understand</m3e-dialog-action></m3e-button>
  </div>
</m3e-dialog>

<style>
  .notice {
    --m3e-dialog-max-width: 460px;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .icon {
    --m3e-icon-size: 36px;
    color: var(--md-sys-color-primary);
  }
  p {
    margin: 0;
    line-height: 1.5;
  }
</style>
