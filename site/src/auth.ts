// The sign-in page. It runs on its own origin (auth.getvelmora.dpdns.org) and
// deliberately does not load ./lib/traffic: no ads, no third party scripts.
import "@fontsource-variable/plus-jakarta-sans/wght.css";
import "@fontsource/instrument-serif/400.css";
import "./lib/m3e";
import "./base.css";

import { mount } from "svelte";
import AuthApp from "./auth/AuthApp.svelte";

mount(AuthApp, { target: document.getElementById("app")! });
