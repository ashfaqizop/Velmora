// After `vite build`: render the Svelte pages that opted in to static HTML,
// so crawlers, link previews and slow phones get the real content before any
// JavaScript runs. The browser then hydrates the same markup.
//
// Pages are listed in src/pages/registry.ts. Each one's built file in dist/
// has an empty <div id="app"></div>; this fills it and adds the page's
// structured data.
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { render } from "svelte/server";
import { createServer } from "vite";

const root = resolve(import.meta.dirname, "..");

// The server render sees assets by their source path (/src/assets/x.webp);
// the built site has hashed copies. Map one to the other.
const manifest = JSON.parse(readFileSync(resolve(root, "dist/.vite/manifest.json"), "utf8"));
const assets = Object.entries(manifest)
  .filter(([key]) => key.startsWith("src/assets/"))
  .map(([key, v]) => ["/" + key, "/" + v.file]);
const fixAssets = (html) => assets.reduce((h, [from, to]) => h.split(from).join(to), html);
process.env.NODE_ENV = "production";
// Production mode keeps Svelte on its normal server runtime (dev mode would load a second copy).
const vite = await createServer({ root, mode: "production", appType: "custom", logLevel: "error", server: { middlewareMode: true }, optimizeDeps: { noDiscovery: true, include: [] } });

try {
  const { PAGES } = await vite.ssrLoadModule("/src/pages/registry.ts");
  let count = 0;
  for (const [name, def] of Object.entries(PAGES)) {
    const file = resolve(root, "dist", def.path.replace(/^\/|\/$/g, ""), "index.html");
    let html = readFileSync(file, "utf8");
    if (!html.includes('<div id="app"></div>')) throw new Error(`${def.path}: no empty #app to fill`);
    const { body: raw } = render(def.component, {});
    const body = fixAssets(raw);
    html = html.replace('<div id="app"></div>', `<div id="app">${body}</div>`);
    const ld = def.jsonld?.() ?? [];
    if (ld.length) {
      const json = JSON.stringify({ "@context": "https://schema.org", "@graph": ld }).replace(/</g, "\u003c");
      html = html.replace("</head>", `<script type="application/ld+json">${json}</script>\n  </head>`);
    }
    writeFileSync(file, html);
    count++;
    console.log(`prerendered ${def.path} (${name}, ${(body.length / 1024).toFixed(0)} KB)`);
  }
  console.log(`prerender: ${count} pages`);
} finally {
  await vite.close();
}
