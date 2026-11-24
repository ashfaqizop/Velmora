# Handoff: where Velmora stands (2026-11-24)

Notes for the next session, so work can carry on without re-reading the whole
history. Memory files in `~/.claude/projects/g--1AA-Velmora/memory/` hold the
same decisions in short form (`accounts-plan.md`, `social-plan.md`,
`improvement-list.md`, `push-allowed.md`).

## House rules

- **No local cargo, rustfmt or Rust builds.** Rust is only checked on GitHub
  Actions (`app.yml` for the app, `universeapi.yml` for the engine). Frontend
  checks run locally: `npx svelte-check --tsconfig ./tsconfig.json` and
  `npm run build` in `app/` and in `site/`.
- **Commit and push freely** (the user allowed it on 2026-11-24). **Never tag
  or publish a release** without the user saying so.
- Commit author `siliconcode-dev`, with only this trailer:
  `Co-Authored-By: ashfaqizop <ashfaqizop@gmail.com>`.
- Write replies, commits and docs in plain language with no AI tells (the
  humanizer style).
- Never write a secret into a file, commit or memory: Supabase service key,
  Upstash token, Mailtrap token, Turnstile secret, Max signing key, GitHub
  tokens. They go into the Supabase or Vercel dashboards by the user. Public
  values are fine in the repo (Supabase URL and publishable key, Turnstile site key).
- UI work uses `@m3e/web` components as much as possible (the user asked for
  "ultra stunning" m3e UI).
- The laptop is slow: don't leave background processes running.

## Stack in one paragraph

Tauri 2 + Svelte 5 (runes) + `@m3e/web` desktop app in `app/`; Rust engine
`universeapi/`; website `site/` (Svelte + Vite, Vercel functions in
`site/api/`, Upstash Redis via `KV_REST_API_URL`/`KV_REST_API_TOKEN`); accounts
on Supabase (project `htvirqckkhfekgfwxops`), SQL in `supabase/`. Hosts:
`getvelmora.dpdns.org` (main site, has ads), `auth.` (sign-in, no ads),
`social.` (Velmora Social, no ads), `admin.` (planned). DNS on Cloudflare,
each host added as a Vercel domain; host routing in `site/vercel.json`
(Vercel serves static files before rewrites, so hosts use redirects to
`/auth/`, `/social/`).

## What shipped since 0.5.0 (all pushed, nothing released)

| Commit | What |
|---|---|
| 5ba852c..346309a | Accounts: web sign-in (Google, Discord, email code + Turnstile), handoff to the app by loopback + PKCE (`app/src-tauri/src/account.rs`, `site/api/auth/{handoff,exchange}.js`), profiles, Settings > Velmora account |
| 847207a | Max on the account (`site/api/license/[action].js` action `account`, `license_account` in `license.rs`) |
| bed7b82 | Sync of My List, progress, settings (`app/src/lib/sync.svelte.ts`, `supabase/sync.sql`) |
| ce8f3ad | Anonymous stats, Popular on Velmora row, Settings > Privacy, `/stats/` page (`site/api/_lib/stats.js`) |
| b6395f9, 9e4bad6 | Engine test fixes for clippy |
| e33a863 | Social phase 1 (below) |

The changelog has a 0.6.0 entry marked `soon` ("In testing") in
`site/src/lib/changes.ts`.

## Social + admin plan (user approved, 5 phases)

1. **Groundwork — done, e33a863.** Nine animal avatars (`animal:<name>`,
   assets in `app/src/assets/avatars` and `site/src/assets/avatars`), bio and
   links (`lib/links.ts` in both, `public.valid_links` in SQL), web sign-in for
   other Velmora sites (`site/src/lib/session.ts`; the auth page takes
   `?return=` for allowed origins), social shell (`site/social/`,
   `site/src/social/SocialApp.svelte`), auth page rebuilt on m3e, **required
   sign-in** (below), `velmora://` deep links (deep-link + single-instance
   plugins, `app/src/lib/deeplink.ts`, website fallback `/open/?to=`), music
   listens in stats (group `mus`, `/api/stats/trending?kind=music`),
   `supabase/social.sql`.
2. **Social core (next).** Public profile pages `/@username` on the social
   host: avatar, name, bio, links, follower/following/visit counts, badges
   (Max, verified, member since), favourites showcase; follow/unfollow (open to
   all); follower and following lists; visits (count public via
   `visits_this_month`, visitor list visible only to the owner via
   `profile_visitors`; record with `record_visit`); edit profile on the web;
   user search; report button (`reports` table). No ads, m3e UI.
3. **Discovery + activity.** Trending movies / shows+anime / music pages with
   "Open in Velmora" (`/open/?to=`); activity from the app, shared by default
   with an off switch (`profiles.share_activity`), **18+ titles never shared**;
   public lists; stats (hours, top genres); ratings and short reviews; feed of
   people you follow; suggested people; leaderboards.
4. **Admin part 1** at `admin.getvelmora.dpdns.org`: admin role (`admins`
   table) + Supabase TOTP MFA, own origin, all writes through server code with
   the service key (user pastes it into Vercel), overview with free-tier meters
   (Supabase, Upstash, Mailtrap 150/day) and health (CI, providers, mpv
   fallback rate), users (hide/ban/reset fields/verify/delete, no private
   history), moderation queue, reserved names, blocked words, audit log.
5. **Admin part 2:** Max fully online (signing key moves into Vercel env, user
   agreed), ads controls, remote config on the `config` branch via a GitHub
   token, in-app announcements + maintenance message, analytics, social
   curation (featured, staff picks, leaderboard control).

Only the user is admin; no moderator role. No DMs, no chat. Theatre (watch
together) comes after these phases.

## Required sign-in (decided 2026-11-24)

Velmora needs an account from 0.6.0. `account.saved` is true when a refresh
token is in Credential Manager; `account.needsSignIn` shows
`components/SignInGate.svelte` only when nothing is saved. With a saved
sign-in the app opens straight in, **online or offline** (the user stressed
this twice). `account.gateReason` picks the wording: `new`, `upgrade` (was
onboarded on 0.5.0: "From now on, Velmora needs an account"), `returning`.

## Waiting on the user

- Run `supabase/social.sql` (after `schema.sql`, `sync.sql`). Until then,
  saving animal avatars, bio or links fails.
- Cloudflare CNAME `social` (proxy off) + Vercel domain
  `social.getvelmora.dpdns.org`.
- Test from the installed CI build: fresh install shows the gate; offline
  launch goes straight in; Max link/restore; sync between two PCs; Privacy page.
- The user said they finished: `sync.sql`, Mailtrap SMTP fix, Turnstile secret
  in Supabase. Email sign-in hasn't been re-tested in this session.

## Known problems

- **Engine lint job fails on rustfmt**: about 200 formatting diffs across the
  engine crate, starting from the 0.5.0 release commit (probably a new stable
  rustfmt; `universeapi/rustfmt.toml` is `max_width = 120`,
  `use_small_heuristics = "Max"`). Can't run rustfmt locally. Proposed: a
  manual `workflow_dispatch` GitHub Action that runs `cargo fmt` and commits.
  Not done yet; ask or just do it.
- The "Android check (impersonate, informational)" engine job always fails
  (BoringSSL); it's informational.
- CI for e33a863 (deep-link + single-instance plugins, first compile) was
  still running at handoff. Check `gh run list` first.

## Useful facts

- Supabase publishable key and URL: `app/src/lib/account.svelte.ts`,
  `site/src/auth/config.ts`, `site/api/_lib/config.js`.
- Avatars must agree in three places: `app/src/lib/avatars.ts`,
  `site/src/auth/avatars.ts`, the `profiles_avatar_ok` constraint.
- `profiles_guard` (social.sql version) protects `verified`, `hidden`,
  `banned` and the follow counters from non-service writes; counters move via
  the `follows_count` trigger using the `velmora.counting` setting.
- Stats live in Redis zsets `an:<day>:<group>` (groups title, src, eng, q,
  ver, inst, mus), one EVAL per report, `MIN_PEOPLE = 3` for trending.
- Site Material Symbols are subset: add new icon names to `ICONS` in
  `site/vite.config.ts`. New site pages go into `PAGES` there.
- Android port is paused.
