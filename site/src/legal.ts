import "@fontsource-variable/plus-jakarta-sans/wght.css";
import "@fontsource/instrument-serif/400.css";
import "@fontsource/instrument-serif/400-italic.css";
import "./lib/m3e";
import "./app.css";

// The privacy and terms pages share this entry; no ads load on them.
import { mount } from "svelte";
import LegalPage from "./LegalPage.svelte";

const page = location.pathname.startsWith("/terms") ? "terms" : "privacy";
mount(LegalPage, { target: document.getElementById("app")!, props: { page } });
