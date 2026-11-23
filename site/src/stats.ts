import "@fontsource-variable/plus-jakarta-sans/wght.css";
import "@fontsource/instrument-serif/400.css";
import "@fontsource/instrument-serif/400-italic.css";
import "./lib/m3e";
import "./app.css";

// The stats page: public anonymous totals. No ads load on it.
import { mount } from "svelte";
import StatsPage from "./StatsPage.svelte";

mount(StatsPage, { target: document.getElementById("app")! });
