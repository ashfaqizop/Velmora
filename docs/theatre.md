# Theatre

Velmora's scheduled screenings: every day an algorithm puts on movies that
everyone watches at the same moment, with a live chat that is never saved.
Agreed with the owner on 2026-11-24. This file is the spec; build against it.

## The idea

- Velmora runs the lineup; nobody hosts. Each day has **4 to 12 screenings**,
  chosen by the algorithm: trending movies plus a **Tonight's pick**.
- Each screening starts at **one moment worldwide**, shown in each person's
  own time zone.
- **Movies only.** Never 18+, only titles with a TMDB number.
- The video comes from each person's own Velmora (their own sources). We run
  the schedule, the shared clock, the chat and the reactions. No video passes
  through us.
- It is a small social space: attendee counts, friends going, names in chat
  link to Velmora Social profiles.

## A screening, start to finish

1. **Lineup.** Today's and tomorrow's screenings are visible and bookable
   (drafted 2 days ahead). Cards show poster, local start time, runtime,
   seats left or "open", friends going.
2. **Book.** A ticket goes into My tickets. Cancel any time before start
   (frees the seat). Sold-out screenings have a **waitlist**.
3. **Lobby** opens **30 min before** start, with chat. Shows the backdrop full
   screen, title and runtime, a large countdown, who's here (friends first)
   with a live count, and a source "Ready" indicator. On entering the lobby
   Velmora finds a source and **pre-caches** the movie (first ~10 min, then a
   rolling buffer ahead of live during the show; cache deleted afterwards).
   Velmora Public sees an **ad before airing** (in the lobby, never during
   the movie); Max skips it.
4. **Start.** The countdown's last 5 seconds are an animated 5-4-3-2-1 that
   flows straight into the movie, full screen, with no loading gap.
5. **Live.** A shared clock: joining late starts at the live point. Pause is
   allowed (you fall behind, shown as "2:14 behind"); **Back to live** jumps
   forward; no skipping ahead of live. Your own audio, subtitles and quality,
   switch source (resumes at live), chat layout toggle.
6. **End.** Credits roll; chat stays open **30 min after** the end with a
   "chat closes in" timer; a rate-and-review card (Velmora ratings); the next
   screening with Book; an attendance stamp on your Social profile.
7. After that the screening is gone: **live only**, no replay (the movie is
   still in Velmora to watch normally).

## Chat and reactions

- Text up to ~200 characters, **@mentions**, each message stamped with the
  movie time it was sent at (e.g. 1:12:40). Emoji reactions from a fixed set
  float over the video, with bursts when many react at once.
- **Never saved.** Messages are broadcast and not written to any table.
- Two layouts, switchable live (and hideable, with shortcuts): **overlay**
  (transparent, over the video, old messages fade) and **sidebar** (full
  screen with a side panel).
- Moderation without an admin site: slow mode that tightens as chat gets busy,
  per-person rate limits, length cap, links blocked, repeats blocked, **mute a
  person** for yourself, and the `blocked_words` list. No report button.

## The algorithm

Runs daily (Vercel cron, server code with the Supabase service key), drafting
the day two days ahead.

- **How many and when.** 4 to 12 screenings, scaled by recent activity. Half
  the shows go at Velmora's busiest hours, half spread to cover other time
  zones. At most **3 at once**; shows are spaced by runtime + a buffer.
  (Needs hourly activity, added to the anonymous stats as UTC-hour buckets.)
- **Trending picks.** Mostly from Velmora's own play counts (proof that
  sources exist), topped up from TMDB trending movies. No repeats within a
  set number of days.
- **Tonight's pick.** A hidden gem: high Velmora rating and TMDB score, few
  Velmora plays so far, not screened recently.
- **Availability probes.** Drafted titles not proven by play counts are
  checked by Velmora apps that are open: they quietly search for sources and
  report found / not found anonymously. Titles that fail are swapped out before
  booking opens.
- **Seats and tickets.** Decided per screening from expected demand
  (bookings so far, title popularity, time slot). Likely to exceed what we can
  handle: **fixed seats, ticket required**, can sell out, waitlist. Comfortably
  under: **open**, drop in without a ticket. Can switch to fixed if bookings
  surge. Seat budgets are shared across overlapping screenings so the total
  stays within the Supabase Realtime free limit (~200 connections at once).
- **No-shows.** Seats not taken 10 minutes after start go to the waitlist in
  order, with a notification. Repeat no-shows get lower booking priority.

## When things go wrong

- Your Velmora finds no source at start: you stay in the screening with chat
  while it keeps searching (torrents too if allowed) and joins at the live
  point when it finds one; it also offers to move your ticket to another
  screening.

## In the app

- **Theatre** section in the nav (lineup, My tickets, live now), with a LIVE
  pulse while a booked lobby or screening is open.
- **Tonight in Theatre** row on Home.
- Title pages say "Screening in Theatre at 9:30 PM" with Book.
- Windows notifications: lobby opens (30 min), starting soon (5 min),
  waitlist seat freed.

## On Velmora Social

- A Theatre page with the lineup, times, seats left, friends going, Book
  (watching itself happens in the app). A countdown banner / tab title near
  showtime for ticket holders.
- "Going to ..." and "N people you follow booked this" (respects
  share-activity). Feed events: booked / attended. Profile stamps: screenings
  attended.

## Velmora Max

No pre-show ad, early booking for fixed-seat screenings (a few hours before
everyone else), front of waitlists. Nothing else differs.

## Technical notes

- Live position = server now - starts_at. Clients sync their clock to the
  server once (a few round trips), then compute the position locally: no
  per-second messages.
- Chat goes through an RPC that checks the ticket, the chat window, limits and
  blocked words, then calls `realtime.send` on a private channel; nothing is
  stored except per-person rate-limit timestamps. Reactions are lighter
  client broadcasts, throttled. Presence gives who's here.
- Realtime Authorization (RLS on realtime.messages) limits channels to
  people allowed in the screening.
- Timed transitions (lobby open, live, ended, waitlist release) run in
  Supabase `pg_cron` every minute.
- Pre-cache via mpv's cache (cache-secs / demuxer limits on a temp cache dir)
  started paused in the lobby; deleted after the screening.
- The owner has to: put the Supabase service key into Vercel's environment,
  and enable pg_cron in Supabase (Database > Extensions).

## Final decisions (round 2, 2026-11-24)

- **Chat full in an open screening:** quiet mode. Late joiners still watch
  in sync (the clock needs no connection), chat shows "Chat is full" and
  turns on as soon as a slot frees.
- **Source probing:** on by default, anonymous, only while Velmora is open
  and idle; a switch in Settings > Privacy turns it off.
- **Pre-show ad:** an ad slot in the lobby for the last few minutes before
  the countdown; the countdown is never covered. Max skips it.
- **Look:** plain Velmora m3e (shapes, motion, colours), no cinema theme.
- **Who's here:** everyone in a screening sees the attendee list (friends
  first); people outside see only the count.
- **Runtime:** no limit; the scheduler spaces screenings by runtime.
- **Manual override:** a Supabase table where the owner can pin a title to a
  day or block a title; the algorithm respects it (admin site UI later).
- **No-shows:** gentle. After 3 no-shows in 30 days, fixed-seat booking opens
  for you 1 hour later than for others; clears after 30 days.
- **Max early booking:** 6 hours before everyone else.
- **Pre-cache:** at most 2 GB on disk; buffer as many minutes ahead as fit,
  drop played parts, delete everything when the screening ends.
- **Watchers:** a live count and a list of who's watching (lobby and during
  the show). Hovering a name or avatar, in the list or in chat, shows a small
  profile card (picture, name, badges, bio, followers) with a link to their
  Velmora Social profile.

## Build status

**Phase 1 (groundwork): done.**
- `supabase/theatre.sql`: `theatre_screenings`, `theatre_tickets`,
  `theatre_overrides` (pin/block, edit in the Table Editor), `max_accounts`
  (mirrored from the license server), `theatre_now`, `theatre_book`,
  `theatre_cancel`, `theatre_enter`, `theatre_say` (chat via
  `realtime.send`, nothing stored), Realtime Authorization policies for
  `theatre:<id>` (presence, reactions) and `theatre-chat:<id>` (listen only),
  and `theatre_tick` scheduled every minute with pg_cron (surge to fixed,
  no-show release at +10 min with waitlist promotion, settle attended /
  noshow after the chat closes).
- `site/api/theatre/[action].js`: `cron` (daily at 00:10 UTC via Vercel
  cron; finalizes tomorrow, drafts the day after; on the first run also sets
  up today and tomorrow; re-syncs Max), `probes` (titles to check),
  `probe` (anonymous found/miss counts in Redis `th:probe:<k>`).
- `site/api/_lib/tmdb.js` (shared TMDB + cache), `_lib/supabase.js` `db()` and
  `setMax()` (service key), license `account` link/restore/unlink mirror Max.
- Anonymous stats gained `hr` (plays per UTC hour); app sends it.

**Setup (owner):**
1. Supabase: enable `pg_cron` (Database > Extensions), then run
   `supabase/theatre.sql`.
2. Vercel env: `SUPABASE_SERVICE_ROLE_KEY` (Supabase > Settings > API keys,
   the secret/service_role key), `CRON_SECRET` (any long random string),
   `TMDB_READ_TOKEN` if not set yet. Redeploy.
3. First lineup without waiting for midnight:
   `curl -H "Authorization: Bearer <LICENSE_ADMIN_TOKEN>" https://getvelmora.dpdns.org/api/theatre/cron`

**Next: phase 2** (the app's Theatre section, booking, reminders, probing).

**Phase 2 (the app's Theatre section): done.**
- Sidebar: Live TV and Sports merged into one **Live** item (a TV channels /
  Sports switch on both pages, remembers the last one); **Theatre** takes the
  freed slot, with a LIVE pulse while a booked screening's lobby or show is on.
- `app/src/lib/theatre.svelte.ts`: lineup, your tickets, people you follow
  going, server clock sync (`theatre_now`), book / cancel with readable
  errors, reminders (lobby opens, 5 minutes before, waitlist seat freed) as
  Windows notifications (tauri-plugin-notification) plus snackbars, and the
  anonymous source checks (`theatre_probes` / `theatre_probe` Rust commands,
  Settings > Privacy switch).
- `pages/Theatre.svelte` (Lineup / My tickets, featured live-or-next
  screening, days grouped in local time), `components/theatre/ScreeningCard`,
  `TheatreCard` (Home "Tonight in Theatre" row), title pages show "In
  Theatre today, 9:30 PM".
- Enter / Join currently shows a note; the lobby and live player are phase 3.

**Phase 3 (lobby and live player): done.**
- `app/src/pages/Screening.svelte` (route `screening`, mounted full window
  like the player): `theatre_enter`, then the **lobby** (backdrop, title,
  countdown, source readiness "Ready · N min loaded ahead", the `lobby` ad
  slot in the last 5 minutes for Velmora Public), the **final 5 seconds** as
  big numbers on morphing m3e shapes, then the lobby lifts away into the
  movie, full screen. Late joiners go straight to the live point with a
  "Joined live, N in" toast. No source: "Still looking" while the player
  retries every minute, plus **Move my ticket** to another screening. Not
  allowed: lobby not open yet / waitlist / needs a ticket (Book here) / over.
  End card once the movie ends.
- `Player.svelte` gained a `theatre` prop (`TheatreLive`): loads paused at
  0:00 while `hold` (the lobby) with a disk cache up to ~10 min / 2 GB ahead
  (`cache-on-disk`, deleted by mpv on close), starts at the live point when
  the hold lifts (and goes full screen), follows live (drift over 6 s snaps
  back), pause = fall behind with "N behind" + **Back to live**, the seek bar
  can't pass live, failed sources resume at live, no pause/end ads, no up
  next, lobby keys ignored while held.
- New ad slot `lobby` (300x250), only on the screening route; other ads are
  quiet there.
