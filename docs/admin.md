# Admin site plan

`admin.getvelmora.dpdns.org`. Only the owner is admin, no moderator role.
Deferred on 2026-11-24, restarted the same day. Nothing is built yet.

## Access and safety
- `admins` table in Supabase plus TOTP MFA. Own origin, no third-party scripts, no ads.
- Every write goes through Vercel server code with the service key.
- Audit log of every admin action. Destructive actions ask for a confirm.
- Vercel Hobby allows 12 functions and 10 are used, so admin is one `api/admin/[action].js`.

## Part 1: running the platform
- Overview: free-tier meters (Supabase, Upstash, Mailtrap 150/day), health (CI, providers, mpv fallback rate).
- Users: search, hide, ban, reset fields, verify, delete. No private history.
- Moderation queue (profile reports), reserved usernames, blocked words (shared with chat), rating and review moderation.

## Part 2: product control
- Max licences fully online (signing key into Vercel env).
- Ads controls.
- Remote config on the `config` branch through a GitHub token limited to that branch.
- In-app announcements and a maintenance message.
- Analytics (anonymous stats).
- Social curation: featured, staff picks, leaderboard control.

## Theatre
Lineup
- Tonight's board: screenings with time, runtime, source, status, booked vs seats.
- Pin a title to a day, block a title (`theatre_overrides`, with a form).
- Swap or remove a draft screening; cancel a finalized one (tickets released, bookers notified).
- Run the lineup now (dry-run preview) and regenerate one day.
- Algorithm knobs: min/max per day, genre mix, trending count, recently-aired cooldown (new config table).
- 7-day calendar view.

Add an airing manually (movies only for now)
1. Search any movie (TMDB).
2. Exact start time, shown in local time, saved as the one worldwide moment.
3. Overlap check against the day's timeline (warns, never blocks).
4. Source: pick one or let the probe decide.
5. Seats and waitlist: defaults or per-screening override.
6. Marked "Special": outside the daily 4-12, ignored by the algorithm.
7. Publish now or save as a draft.
8. Edit or cancel later. Same `theatre_screenings` rows, with a flag for who created them.

Seats and tickets
- Capacity, waitlist length, Max early-booking window (6 h) settings.
- Look up a user's tickets (booked, waitlisted, attended, no-shows).
- Clear or add a no-show penalty. Grant or revoke a seat by hand.

Live room (chat is never saved, so no message history)
- Live now panel: lobby and room counts, watchers (Realtime presence).
- Chat kill switch, force slow or quiet mode, mute or remove a person.
- Global blocked words.

Sources and health
- Source probe results per title, "is this playable" check before scheduling.
- Auto-skip titles where every probe fails.
- Start-of-show failure rate (needs a small anonymous event from the app).

Analytics
- Per screening: booked, attended, no-shows, peak watchers, average time watched, rating score.
- Best slots and genres, app vs Social bookings, Max vs Public mix, lobby ad impressions.

Comms
- Edit reminder text, send an extra reminder for a special.
- Banner on the Theatre page and the social countdown.
- Delay or cancel a screening with a message to everyone booked.
- "Pause Theatre" switch: stops new screenings and bookings.

## Needs new storage
Algorithm knobs, capacity settings, the playback-error event, a `created_by` flag and `special` label on screenings.

## Build order
1. Access + MFA, overview, users, moderation, audit log.
2. Theatre: tonight's board, pin/block, manual airing, ticket lookup, live-room kill switch, pause switch.
3. Part 2 (Max online, ads, remote config, announcements, analytics, curation) and the rest of Theatre.
