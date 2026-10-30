import "@fontsource-variable/roboto-flex/full.css";
import "material-symbols/outlined.css";
import "./lib/m3e";
import "./app.css";

import { mount } from "svelte";
import App from "./App.svelte";

mount(App, { target: document.getElementById("app")! });
