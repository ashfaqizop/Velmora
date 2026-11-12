// Music on Windows' media controls (media.rs): the volume flyout tile, the
// lock screen, and the media keys. The player's state goes out when it
// changes (and the position every few seconds, for the tile's scrubber); the
// buttons come back as `media-control` events.
import { listen } from "@tauri-apps/api/event";
import { untrack } from "svelte";
import * as api from "../api";
import { musicPlayer } from "./player.svelte";
import { art } from "./ui.svelte";

let started = false;

export function startMediaControls() {
  if (started) return;
  started = true;
  let last = "";

  $effect.root(() => {
    $effect(() => {
      const cur = musicPlayer.current;
      const yielded = musicPlayer.yielded;
      const paused = !musicPlayer.playing;
      const duration = Math.round(musicPlayer.duration);
      // Position only every 5 s (and on a seek, which lands in another slot).
      const slot = Math.floor(musicPlayer.position / 5);
      untrack(() => {
        if (!cur || yielded) {
          if (last !== "clear") api.mediaClear().catch(() => {});
          last = "clear";
          return;
        }
        const key = `${cur.id}|${paused}|${duration}|${slot}`;
        if (key === last) return;
        last = key;
        api
          .mediaUpdate({
            title: cur.title,
            artist: cur.artists.map((a) => a.name).join(", ") || cur.subtitle || "",
            album: cur.album ?? "",
            cover: art(cur.thumbnails, 544) ?? "",
            duration: musicPlayer.duration,
            position: musicPlayer.position,
            paused,
          })
          .catch(() => {});
      });
    });
  });

  listen<{ action: string; position?: number }>("media-control", ({ payload }) => {
    // A movie has the player; the keys aren't Music's then.
    if (!musicPlayer.active || musicPlayer.yielded) return;
    switch (payload.action) {
      case "play":
        if (!musicPlayer.playing) musicPlayer.toggle();
        break;
      case "pause":
      case "stop":
        if (musicPlayer.playing) musicPlayer.toggle();
        break;
      case "toggle":
        musicPlayer.toggle();
        break;
      case "next":
        musicPlayer.next();
        break;
      case "previous":
        musicPlayer.prev();
        break;
      case "seek":
        if (typeof payload.position === "number") musicPlayer.seek(payload.position);
        break;
    }
  });
}
