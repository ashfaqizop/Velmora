# Velmora website

The download page for Velmora: one static page built with Svelte 5, Vite and @m3e/web. It has no server and no backend.

- The download buttons read the newest `app-v*` release from the GitHub API, so the page needs no edit when a version ships.
- The app mockup is drawn in CSS. Titles are made up and posters are gradients, so no studio artwork is on the site.
- Ads come from Adsterra and appear only on this website, never in the app. They are configured with environment variables, so a build without them shows no ads.

## Changelog

The `/changelog/` page is written by hand in `src/lib/changes.ts`, not read from GitHub, so each release can have its own summary, highlight cards, color and shape. When you ship a version, add it at the top of `RELEASES`. While it's built but not out yet, set `soon: true` and it shows as "In testing"; remove `soon` when it's released. `NEXT` is the "coming soon" card above the timeline.

## Run it

```sh
npm install
npm run dev      # http://localhost:5173
npm run build    # static files in dist/
```

## Deploy on Vercel

1. Import the GitHub repo in Vercel.
2. Set **Root Directory** to `site`. Vercel reads `vercel.json` for the rest (Vite, `npm run build`, `dist`).
3. Add the Adsterra variables from `.env.example` under Project → Settings → Environment Variables, then redeploy. Vite reads them at build time.
4. Optional: add your domain under Project → Settings → Domains.

Every push to `main` that touches `site/` redeploys. To skip builds for app-only commits, set Project → Settings → Git → Ignored Build Step to `git diff --quiet HEAD^ HEAD -- .`.

## Sports cache proxy

`api/sports/[...path].js` is a small Vercel function that proxies TheSportsDB and football-data.org. It holds the keys (`SPORTSDB_KEY` and `FOOTBALL_DATA_TOKEN`, set in Vercel and never committed), calls each API at most once a minute per URL, and lets Vercel's cache answer everyone else. That keeps the keys out of the app and out of the installer, and stops many users from sharing one free-tier limit.

To switch the app over, edit `registry.json` on the repo's `config` branch (no app release needed):

```json
{
  "sports": {
    "thesportsdb_url": "https://YOUR-SITE/api/sports/thesportsdb",
    "football_data_url": "https://YOUR-SITE/api/sports/football-data"
  }
}
```

With the URLs pointed here the engine needs no keys at all. The function only allows the endpoints Velmora uses, and only GET. It doesn't log anything itself, but Vercel keeps its own request logs, which include the caller's IP, so the privacy policy should say so.

## Ads

The Adsterra codes are public, so they're committed: `src/lib/ads.ts` for the site and `public/ad/` for the app.

| Format | Where it shows |
| --- | --- |
| Native Banner | After the features grid, and on the Ani page |
| Banner 728×90, 468×60 or 320×50 | Above the FAQ and on the changelog, by screen width |
| Banner 160×600 | Fixed to the right of the changelog, on screens 1640 px and wider |
| Social Bar | Every page except privacy and terms, 8 seconds after load |
| Popunder | Once per visit, 3 seconds after the first page loads (`src/lib/traffic.ts`) |

In-page ads stay away from the download buttons, so nobody mistakes an ad for the real download. Nothing loads on `localhost`.

### Ads in the app

`public/ad/<size>.html` and `public/ad/native.html` each show one unit and report back to the app whether they filled (`public/ad/report.js`). Only the app and this site may frame them (see `vercel.json`). `public/ads.json` holds the app's ad rules (on/off, caps, timings, which unit each slot uses), so they can change without an app update. `api/ads/stats` takes anonymous per-slot view counts.

## Max licenses

`api/license/[action].js` activates Max keys, one PC per key, in Upstash Redis. It needs `KV_REST_API_URL`, `KV_REST_API_TOKEN` (from the Upstash integration), `LICENSE_SERVER_KEY` and `LICENSE_ADMIN_TOKEN`. The public keys are in `api/_lib/keys.js`. Keys are made with a local tool that isn't in the repo.

## Icons

Icons load from Google Fonts as a subset. The full Material Symbols font is 4 MB. The list lives in `ICONS` in `vite.config.ts`, which adds the font link to every page; add a name there when you use a new icon.

## Pages

Each page is its own HTML file with its own entry script: `index.html` (landing), `changelog/index.html`, `ani/index.html`, `universeapi/index.html`, and `privacy/index.html` and `terms/index.html` (both use `src/legal.ts`; the text is in `src/lib/legal.ts`). Add new ones to `PAGES` in `vite.config.ts`.
