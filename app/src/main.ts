import "@fontsource/redaction/400.css";
import "@fontsource/redaction/400-italic.css";
import "@fontsource/redaction/700.css";
import "@fontsource/instrument-serif/400.css";
import "@fontsource/instrument-serif/400-italic.css";
import "material-symbols/outlined.css";
import "./lib/m3e";
import "./app.css";

import { mount } from "svelte";
import App from "./App.svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { applyAccent, DEFAULT_ACCENT } from "./lib/state.svelte";

applyAccent(localStorage.getItem("velmora.accent") ?? DEFAULT_ACCENT);

mount(App, { target: document.getElementById("app")! });

// The window starts hidden (it is transparent for mpv); show it maximized on the first painted frame.
requestAnimationFrame(async () => {
  const win = getCurrentWindow();
  await win.maximize().catch(() => {});
  await win.show();
});
