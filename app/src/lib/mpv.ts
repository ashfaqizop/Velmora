// mpv lives once per window. Destroy is async, so a remount (next episode)
// must wait for the previous instance to be gone before it calls init,
// otherwise the plugin sees the old instance and skips the new init.
//
// Starting can fail on older PCs, mostly in the graphics driver. So mpv is
// tried several ways, from the best picture to the safest, and the first
// that works is remembered for this PC. Each try writes mpv's own log, so a
// start that fails everywhere can say why.
import { destroy, init, type MpvConfig } from "tauri-plugin-libmpv-api";
import * as api from "./api";
import { playerEngine } from "./engine.svelte";

export interface MpvProfile {
  id: string;
  label: string;
  options: Record<string, string>;
}

/** Best picture first. Later ones turn off hardware decoding and lean on
 *  simpler graphics paths that old Intel and AMD drivers handle. */
export const MPV_PROFILES: MpvProfile[] = [
  { id: "gpu-next", label: "Best (gpu-next)", options: { vo: "gpu-next", "gpu-api": "auto", hwdec: "auto-safe" } },
  { id: "gpu-d3d11", label: "Standard (Direct3D 11)", options: { vo: "gpu", "gpu-api": "d3d11", hwdec: "auto-safe" } },
  { id: "gpu-opengl", label: "OpenGL", options: { vo: "gpu", "gpu-api": "opengl", hwdec: "no" } },
  { id: "direct3d", label: "Direct3D 9", options: { vo: "direct3d", hwdec: "no" } },
  { id: "software", label: "Software (no graphics card)", options: { vo: "gpu", "gpu-api": "d3d11", "d3d11-warp": "yes", hwdec: "no" } },
];

export const profileLabel = (id: string | null) => MPV_PROFILES.find((p) => p.id === id)?.label ?? "Not tried yet";

let pending: Promise<unknown> = Promise.resolve();
const logPaths = new Map<string, string>();

async function logFile(id: string): Promise<string | null> {
  if (!logPaths.has(id)) {
    const path = await api.playerLogPath(id).catch(() => null);
    if (!path) return null;
    logPaths.set(id, path);
  }
  return logPaths.get(id) ?? null;
}

/** The order to try: the one that worked last time, then the rest from the top. */
function order(): MpvProfile[] {
  const known = MPV_PROFILES.find((p) => p.id === playerEngine.profile);
  return known ? [known, ...MPV_PROFILES.filter((p) => p !== known)] : [...MPV_PROFILES];
}

/** The error mpv itself logged, in one line, if there is one. */
function firstError(log: string): string | null {
  const line = log
    .split("\n")
    .find((l) => /\[(e|f)\]/i.test(l) || /error|failed|cannot|could not/i.test(l));
  return line ? line.replace(/^\[\s*[\d.]+\]\s*/, "").trim().slice(0, 200) : null;
}

/** Thrown when every way of starting mpv failed. */
export class MpvUnavailable extends Error {}

/**
 * Start mpv for this window, trying each graphics profile until one works.
 * `video` says the caller draws video, so mpv must open its window now
 * (that's what fails on a bad driver, and failing here lets us move on).
 */
export async function startMpv(config: MpvConfig, opts: { video?: boolean } = {}): Promise<MpvProfile> {
  await pending;
  let lastError: unknown = null;
  for (const profile of order()) {
    const log = await logFile(profile.id);
    const initialOptions: MpvConfig["initialOptions"] = {
      ...config.initialOptions,
      ...profile.options,
      ...(log ? { "log-file": log } : {}),
    };
    if (opts.video) initialOptions["force-window"] = "immediate";
    try {
      await init({ ...config, initialOptions });
      if (playerEngine.profile !== profile.id || playerEngine.mpvDown) playerEngine.remember(profile.id);
      return profile;
    } catch (e) {
      lastError = e;
      console.warn(`mpv didn't start with ${profile.id}`, e);
    }
  }
  // Every way failed. Keep the best one's log: it says what went wrong first.
  const log = await api.playerLog(order()[0].id).catch(() => "");
  const reason = firstError(log) ?? String(lastError ?? "mpv couldn't start");
  playerEngine.down(reason, log);
  throw new MpvUnavailable(reason);
}

export function stopMpv() {
  pending = pending.then(() => destroy()).catch(() => {});
  return pending;
}
