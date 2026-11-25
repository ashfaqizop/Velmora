# SEO notes (2026-11-25)

Goal: show up for searches like "free Netflix alternative". Search Console is linked by the owner.

Done (site/):
- `public/robots.txt` and `public/sitemap.xml` (main host only; social is behind Cloudflare, whose managed robots.txt blocks AI-training crawlers).
- Canonical, `og:url`, `og:site_name`, absolute 1200x630 `og:image` (`public/og.png`) and `summary_large_image` on the public pages. `/open/`, `/auth/` and admin stay noindex.
- Homepage title and description rewritten around what it is; JSON-LD (WebSite, SoftwareApplication, FAQPage that mirrors the visible FAQ); a `<noscript>` block so crawlers that don't run JS still get an H1 and links.
- New static page `/free-netflix-alternative/` (`free-netflix-alternative/index.html`, styles in `src/seo/page.css`): honest comparison, FAQ with FAQPage JSON-LD, links in the footer.

Claims to keep true: Velmora hosts no content; availability depends on third-party sources; Windows only for now; unsigned installer warning. Update the page if any of those change.

Owner steps after deploy: in Search Console, submit `https://getvelmora.dpdns.org/sitemap.xml` and use URL Inspection > Request indexing for `/` and `/free-netflix-alternative/`.

Not done: prerendering the Svelte homepage (still client-rendered), the free-subdomain question, a real keyword/ranking baseline (OpenSEO plugin not installed), more keyword pages.
