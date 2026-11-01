import { resolve } from "node:path";
import { defineConfig, type Plugin } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Material Symbols the site uses. The full font is 4 MB, so Google Fonts
// serves just these. Keep the list sorted; add a name when you use a new icon.
const ICONS = [
  "add", "add_circle", "android", "arrow_downward", "arrow_forward", "arrow_outward", "arrow_upward",
  "back_hand", "badge", "bedtime", "bookmark", "build", "calendar_month", "check_circle", "checklist",
  "close", "cloud_off", "code", "compare_arrows", "donut_large", "download", "edit_note", "equalizer",
  "folder_open", "format_list_numbered", "graphic_eq", "history", "home", "info", "key",
  "keyboard_arrow_down", "keyboard_arrow_up", "leaderboard", "link", "lock", "menu", "movie", "palette",
  "person", "person_heart", "person_off", "play_arrow", "play_circle", "psychology", "record_voice_over",
  "schedule", "search", "settings", "star", "sticky_note_2", "swap_horiz", "sync", "toggle_off",
  "travel_explore", "trending_up", "video_library", "view_carousel", "visibility", "visibility_off",
  "volume_up", "wand_stars",
];

const icons = (): Plugin => ({
  name: "velmora-icons",
  transformIndexHtml: () => [
    { tag: "link", attrs: { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "" }, injectTo: "head" },
    {
      tag: "link",
      attrs: {
        rel: "stylesheet",
        href: `https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0..1,0&icon_names=${[...ICONS].sort().join(",")}&display=block`,
      },
      injectTo: "head",
    },
  ],
});

// One HTML file per page.
const PAGES = ["index.html", "changelog/index.html", "ani/index.html", "universeapi/index.html"];

export default defineConfig({
  plugins: [svelte(), icons()],
  build: {
    target: "es2022",
    chunkSizeWarningLimit: 2000,
    rollupOptions: {
      input: Object.fromEntries(PAGES.map((p) => [p.split("/")[0].replace(".html", "") || "main", resolve(import.meta.dirname, p)])),
    },
  },
});
