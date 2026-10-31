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
import { M3eSnackbar } from "@m3e/web/snackbar";
import { applyAccent, DEFAULT_ACCENT } from "./lib/state.svelte";

applyAccent(localStorage.getItem("velmora.accent") ?? DEFAULT_ACCENT);

// Anything that fails without its own handling still tells the user.
window.addEventListener("unhandledrejection", (e) => {
  console.error(e.reason);
  const message = String(e.reason?.message ?? e.reason ?? "Something went wrong");
  M3eSnackbar.open(message.length > 120 ? `${message.slice(0, 117)}...` : message, true);
});

mount(App, { target: document.getElementById("app")! });

// The window starts hidden (it is transparent for mpv); show it maximized on the first painted frame.
requestAnimationFrame(async () => {
  const win = getCurrentWindow();
  await win.maximize().catch(() => {});
  await win.show();
});
