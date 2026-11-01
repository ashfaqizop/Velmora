import "@fontsource-variable/plus-jakarta-sans/wght.css";
import "@fontsource/instrument-serif/400.css";
import "@fontsource/instrument-serif/400-italic.css";
import "./lib/m3e";
import "./app.css";

import { mount } from "svelte";
import AniPage from "./AniPage.svelte";

mount(AniPage, { target: document.getElementById("app")! });
