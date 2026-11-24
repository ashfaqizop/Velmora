// Velmora Admin. Its own origin (admin.getvelmora.dpdns.org): no ads, no
// third-party scripts, and every action is checked again on the server.
import "@fontsource-variable/plus-jakarta-sans/wght.css";
import "@fontsource/instrument-serif/400.css";
import "@fontsource/instrument-serif/400-italic.css";
import "./lib/m3e";
import "@m3e/web/dialog";
import "./base.css";
import "./admin/admin.css";

import { mount } from "svelte";
import AdminApp from "./admin/AdminApp.svelte";

mount(AdminApp, { target: document.getElementById("app")! });
