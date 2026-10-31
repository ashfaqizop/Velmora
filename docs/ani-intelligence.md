# Ani Intelligence

Velmora's second flagship feature next to UniverseAPI, and the content of v0.2.0.

## What the user asked for

- Ani is an assistant dedicated to Velmora, built on Gemini models.
  - Default chat model: Gemini 3.5 Flash (`gemini-3.5-flash`, GA). The user also mentioned 3.6 Flash; no such model id was found on 2026-10-31, so it should be selectable in Settings.
  - Tool-calling model: Gemini 3.1 Flash-Lite (`gemini-3.1-flash-lite`).
- BYOK only. The user brings their own Gemini API key, and without one Ani doesn't work.
- Personalize: everything the user searches, watches, streams or downloads is saved there.
  - Ani reads it to personalize answers.
  - Ani can also update it.
- Ani lives only on its own Ani Chat page, which gets an entry in the left sidebar.
- Onboarding asks whether the user wants Ani. If they say yes, the sidebar shows the Ani entry; if no, it doesn't. Settings can switch it on or off later.
- Ani can call anything in the app through tools, such as `tmdb_search` and `universe_search`, with 40 or more tools in total.
- Ani can reply with generative UI such as cards and carousels. The user will send examples of what they want.
- Skills loaded into Ani:
  - `humanizer`: always used when chatting. The user will provide their version.
  - `velmora-tools`: used when tool calling is needed.
- Example flow for "More movies like Saiyaara":
  1. Read the user's Personalize data.
  2. Look up Saiyaara.
  3. Search thoroughly for similar titles.
  4. Reply with text and title cards in the UI.

## Proposed design (to confirm)

- **Key handling:** the Gemini key is stored by the Rust side in the app config folder. The webview never holds it.
  - A Rust command makes the Gemini HTTP call, streaming SSE chunks back through a Tauri `Channel`.
  - Requests reuse the engine's reqwest client, which already has the `stream` feature.
  - The CSP stays unchanged.
- **Two-stage turn:**
  1. Research: Flash-Lite runs the tool loop with the `velmora-tools` skill and the Personalize summary, until it stops calling tools or hits a step cap.
  2. Answer: Flash gets the gathered facts and the humanizer skill, and writes the reply.
- **Generative UI:** the answer model calls `render_*` functions (cards, carousel, and the others the examples show).
  - Gemini returns text parts and function calls in order, so UI blocks can sit between paragraphs.
  - Each render function maps to a Svelte component built on @m3e/web.
- **Gemini 3.5 rule:** every `functionResponse` must carry the `id` and `name` of its `functionCall`.
- **Tools:** thin wrappers over the existing `api.ts` commands, run on the frontend. Examples: search, detail, sources, home rails, My List, history, downloads, preferences, and Personalize read and write.
- **Personalize store:** an event log (search, watch progress, stream started, download) plus notes Ani writes.
  - It's capped and summarised before each request.
  - The user can see and clear it in Settings.
- **Preferences:** a UI-level `ani.enabled` flag, set in onboarding and Settings, controls the sidebar entry.

## Decisions since the first draft

- Ani is a chatbot. It doesn't play, navigate, download or change settings. The buttons on its cards (Play, My List, Download) run only when the user clicks them.
- The humanizer skill is [blader/humanizer](https://github.com/blader/humanizer) (MIT), bundled with its license notice and used on every reply.
- The main generative UI block follows the user's example: a list with a poster on the left, title, a chip badge, a short "why", bullet facts and actions, divided between rows.

## Tools (39)

Search and discovery: `universe_search`, `tmdb_search`, `anilist_search`, `title_detail`, `title_ratings`, `similar_titles`, `discover`, `genre_list`, `trending`, `home_rails`, `tonight_pick`, `list_episodes`, `trailer`, `title_images`, `person_search`, `person_credits`, `collection`, `upcoming`.

Finding links (read-only, UniverseAPI): `find_stream_links`, `find_download_links`, `find_torrents`, `find_season_packs`, `link_details`, `compare_links`, `source_status`, `already_downloaded`, `downloads_library`.

Personalize: `profile_read`, `profile_note`, `profile_forget`, `taste_stats`, `search_history`, `watch_history`, `continue_watching`, `watchlist`, `is_watched`.

App info (read-only): `get_settings`, `sync_status`, `app_info`.

## Generative UI blocks (22)

`render_title_list`, `render_carousel`, `render_spotlight`, `render_title_compare`, `render_episodes`, `render_season_overview`, `render_watch_order`, `render_collection`, `render_download_links`, `render_torrents`, `render_link_compare`, `render_stream_links`, `render_cast`, `render_person`, `render_ratings`, `render_facts`, `render_trailer`, `render_gallery`, `render_schedule`, `render_progress`, `render_taste_profile`, `render_choices`. A passive "Ani noted" line shows every `profile_note`.

## Build plan

Eight steps, two per commit and push. No tag or release without the user's say-so.

1. Rust: key storage, the streaming Gemini call, read-only TMDB and AniList lookups (`ani.rs`, `Tmdb::raw/list`, `Anilist::find/raw`). Done.
2. Ani switch in onboarding and Settings, key entry, model pickers, sidebar entry, the `ani` route. Done.
3. Personalize: recording, plus viewing and clearing it in Settings. Done.
4. Agent core: two-stage loop, bundled skills (humanizer, velmora-tools). Done.
5. The 39 tools. Done.
6. Chat page: streaming, composer, history. Done.
7. Generative UI, titles and links.
8. Generative UI, the rest; polish, docs, version 0.2.0.
