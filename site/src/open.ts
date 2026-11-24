// "Open in Velmora": hands a link to the installed app (velmora://...), and
// offers the download when nothing answers. No ads on this page.
import "@fontsource-variable/plus-jakarta-sans/wght.css";
import "@fontsource/instrument-serif/400.css";
import "./lib/m3e";
import "./app.css";

import { mount } from "svelte";
import OpenPage from "./OpenPage.svelte";

mount(OpenPage, { target: document.getElementById("app")! });
