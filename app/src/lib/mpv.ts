// mpv lives once per window. Destroy is async, so a remount (next episode)
// must wait for the previous instance to be gone before it calls init,
// otherwise the plugin sees the old instance and skips the new init.
import { destroy, init, type MpvConfig } from "tauri-plugin-libmpv-api";

let pending: Promise<unknown> = Promise.resolve();

export async function startMpv(config: MpvConfig) {
  await pending;
  await init(config);
}

export function stopMpv() {
  pending = pending.then(() => destroy()).catch(() => {});
  return pending;
}
