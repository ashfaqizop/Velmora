import "@fontsource-variable/roboto-flex/full.css";
import "material-symbols/outlined.css";
import "./lib/m3e";
import "./app.css";

import { mount } from "svelte";
import App from "./App.svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";

mount(App, { target: document.getElementById("app")! });

// The window starts hidden (it is transparent for mpv); show it on the first painted frame.
requestAnimationFrame(() => getCurrentWindow().show());
