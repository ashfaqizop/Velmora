import "@fontsource-variable/plus-jakarta-sans/wght.css";
import "@fontsource/instrument-serif/400.css";
import "@fontsource/instrument-serif/400-italic.css";
import "@fontsource-variable/jetbrains-mono/wght.css";
import "./lib/m3e";
import "./app.css";

import { mount } from "svelte";
import EnginePage from "./EnginePage.svelte";

mount(EnginePage, { target: document.getElementById("app")! });
