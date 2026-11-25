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
  "keyboard_arrow_down", "keyboard_arrow_up", "leaderboard", "library_music", "lyrics", "link", "lock", "menu", "movie", "palette",
  "person", "person_heart", "person_off", "play_arrow", "play_circle", "psychology", "record_voice_over",
  "schedule", "search", "settings", "star", "sticky_note_2", "swap_horiz", "sync", "toggle_off",
  "travel_explore", "trending_up", "verified", "video_library", "view_carousel", "visibility", "visibility_off",
  "volume_up", "wand_stars", "workspace_premium",
  "groups", "mail", "error", "check", "open_in_new", "priority_high", "logout", "edit", "public", "person_add", "insights", "login", "account_circle", "shield", "favorite", "explore", "music_note", "tv", "local_fire_department", "emoji_events", "link_off", "star_rate", "more_vert", "arrow_back",
  "dashboard", "rate_review", "spellcheck", "block", "monitoring", "health_and_safety", "manage_accounts", "restart_alt", "lock_open", "delete_forever", "task_alt", "undo", "storage", "event_busy", "event", "event_available", "pause_circle", "play_circle", "campaign", "forum", "confirmation_number", "pin_drop", "push_pin", "chat_bubble", "mic_off", "person_remove", "sensors", "calendar_month", "preview", "auto_mode",
  "share", "flag", "content_copy", "group", "person_remove", "how_to_reg", "add_link", "playlist_add", "list", "delete", "delete_sweep", "desktop_windows", "send", "refresh", "theaters", "confirmation_number", "lock_clock", "auto_awesome", "local_activity", "forum",
];

const icons = (): Plugin => ({
  name: "velmora-icons",
  transformIndexHtml: () => [
    { tag: "link", attrs: { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "" }, injectTo: "head" },
    {
      tag: "link",
      attrs: {
        rel: "stylesheet",
        href: `https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0..1,0&icon_names=${[...new Set(ICONS)].sort().join(",")}&display=block`,
      },
      injectTo: "head",
    },
  ],
});

// One HTML file per page.
const PAGES = ["index.html", "changelog/index.html", "ani/index.html", "universeapi/index.html", "privacy/index.html", "terms/index.html", "auth/index.html", "stats/index.html", "social/index.html", "open/index.html", "admin/index.html", "free-netflix-alternative/index.html"];

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
