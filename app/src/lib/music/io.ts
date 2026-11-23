// What the music player talks to: mpv, or the built-in player when mpv
// can't run on this PC. Both take mpv's property and command names, so the
// player's logic is the same either way.
import {
  command as mpvCommand,
  getProperty as mpvGet,
  listenEvents as mpvListen,
  setProperty as mpvSet,
  type MpvEvent,
} from "tauri-plugin-libmpv-api";

export interface PlayerIO {
  readonly kind: "mpv" | "builtin";
  set(name: string, value: string | number | boolean): Promise<void>;
  get<T = unknown>(name: string, format?: "flag" | "double" | "int64" | "string" | "node"): Promise<T>;
  cmd(name: string, args?: (string | number)[]): Promise<void>;
  listen(cb: (ev: MpvEvent) => void): Promise<() => void>;
}

export const mpvIO: PlayerIO = {
  kind: "mpv",
  set: (name, value) => mpvSet(name, value),
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  get: <T>(name: string, format: any = "string") => mpvGet(name, format) as Promise<T>,
  cmd: (name, args = []) => mpvCommand(name, args),
  listen: (cb) => mpvListen(cb),
};
