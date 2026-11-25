// Writes the HTML file for each content page in src/pages/registry.ts
// (title, description, canonical, social tags). Run it after changing the
// registry: `node scripts/make-pages.mjs`. The files are committed; the build
// (vite build, then scripts/prerender.mjs) fills in the page body.
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { createServer } from "vite";

const root = resolve(import.meta.dirname, "..");
const ORIGIN = "https://getvelmora.dpdns.org";
const esc = (s) => s.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");

const vite = await createServer({ root, appType: "custom", logLevel: "error", server: { middlewareMode: true } });
try {
  const { PAGES } = await vite.ssrLoadModule("/src/pages/registry.ts");
  for (const [name, p] of Object.entries(PAGES)) {
    const url = ORIGIN + p.path;
    const html = `<!doctype html>
<html lang="en" style="background: #0a0d16">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="color-scheme" content="dark" />
    <meta name="theme-color" content="#0a0d16" />
    <title>${esc(p.title)}</title>
    <meta name="description" content="${esc(p.description)}" />
    <link rel="canonical" href="${url}" />
    <meta property="og:type" content="website" />
    <meta property="og:site_name" content="Velmora" />
    <meta property="og:url" content="${url}" />
    <meta property="og:title" content="${esc(p.title)}" />
    <meta property="og:description" content="${esc(p.description)}" />
    <meta property="og:image" content="${ORIGIN}/og.png" />
    <meta property="og:image:width" content="1200" />
    <meta property="og:image:height" content="630" />
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:image" content="${ORIGIN}/og.png" />
    <link rel="icon" href="/favicon.ico" sizes="any" />
    <link rel="apple-touch-icon" href="/icon-310.png" />
  </head>
  <body data-page="${name}">
    <m3e-theme id="theme" color="#3271FE" variant="vibrant" scheme="dark" motion="expressive">
      <div id="app"></div>
    </m3e-theme>
    <script type="module" src="/src/page.ts"></script>
  </body>
</html>
`;
    const file = resolve(root, p.path.replace(/^\/|\/$/g, ""), "index.html");
    mkdirSync(dirname(file), { recursive: true });
    writeFileSync(file, html);
    console.log("wrote", p.path);
  }
} finally {
  await vite.close();
}
