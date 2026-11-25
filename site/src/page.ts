// Client entry for the content pages (download, features, faq, docs, contact...).
// The server already rendered them (scripts/prerender.mjs), so this hydrates.
import "@fontsource-variable/plus-jakarta-sans/wght.css";
import "@fontsource/instrument-serif/400.css";
import "@fontsource/instrument-serif/400-italic.css";
import "./lib/m3e";
import "@m3e/web/breadcrumb";
import "@m3e/web/carousel";
import "@m3e/web/toc";
import "@m3e/web/split-button";
import "@m3e/web/button-group";
import "./app.css";
import "./pages/pages.css";

import { hydrate, mount } from "svelte";
import { PAGES } from "./pages/registry";

const target = document.getElementById("app")!;
const def = PAGES[document.body.dataset.page ?? ""];
if (def) (target.firstElementChild ? hydrate : mount)(def.component, { target });
