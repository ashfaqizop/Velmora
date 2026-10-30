import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

// m3e buttons, menu items and nav items are interactive custom elements that
// set their own roles and keyboard handling; Svelte can't see that.
const M3E_A11Y = new Set(["a11y_click_events_have_key_events", "a11y_no_static_element_interactions"]);

export default {
  preprocess: vitePreprocess(),
  compilerOptions: {
    warningFilter: (w) => !(M3E_A11Y.has(w.code) && /<m3e-/.test(w.message)),
  },
};
