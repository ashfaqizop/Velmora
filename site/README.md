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

## Ads

| Variable | Adsterra format | Where it shows |
| --- | --- | --- |
| `VITE_ADSTERRA_NATIVE` | Native Banner (full `invoke.js` URL) | After the features grid |
| `VITE_ADSTERRA_BANNER_728` | Banner 728×90 | Above the FAQ, on screens 760 px and wider |
| `VITE_ADSTERRA_BANNER_300` / `_320` | Banner 300×250 or 320×50 | Same spot, on phones |

Ads are kept away from the download buttons on purpose, so nobody mistakes an ad for the real download. Popunders and Social Bar are left out for the same reason: on a download page they make the site look like malware.

## Icons

Icons load from Google Fonts as a subset. The full Material Symbols font is 4 MB. The list lives in `ICONS` in `vite.config.ts`, which adds the font link to every page; add a name there when you use a new icon.

## Pages

Each page is its own HTML file with its own entry script: `index.html` (landing), `changelog/index.html`, `ani/index.html`, `universeapi/index.html`. Add new ones to `PAGES` in `vite.config.ts`.
