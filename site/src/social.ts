// Velmora Social. Its own origin (social.getvelmora.dpdns.org): no ads and no
// third-party scripts, because people are signed in here.
import "@fontsource-variable/plus-jakarta-sans/wght.css";
import "@fontsource/instrument-serif/400.css";
import "@fontsource/instrument-serif/400-italic.css";
import "./lib/m3e";
// Only the social pages need these.
import "@m3e/web/dialog";
import "@m3e/web/search";
import "@m3e/web/skeleton";
import "./app.css";

import { mount } from "svelte";
import SocialApp from "./social/SocialApp.svelte";

mount(SocialApp, { target: document.getElementById("app")! });
