// Velmora Social, server side. One function for both actions, because the
// Hobby plan caps how many functions a deployment may have.
//
//   GET /api/social/max?ids=<uuid>,<uuid>   -> { max: [<uuid>, ...] }
//       Which of these people have Velmora Max on their account (for the badge).
//
//   GET /api/social/titles?ks=movie/603,tv/1396 -> { titles: { "movie/603": {...} | null } }
//       Names, posters and genres from TMDB, kept in Redis for a week. Needs
//       TMDB_READ_TOKEN (or TMDB_API_KEY) in Vercel's environment variables.
//
//   GET /api/social/songs?ids=<videoId>,...   -> { songs: { "<id>": { title, artist } | null } }
//       Song names from YouTube's public oEmbed, kept for a month.
//
//   GET /api/social/page?u=<username>        -> the social page's HTML
//       Profile links (/@name) are rewritten here so that link previews in
//       chats and on social sites show the person's name, bio and picture.
//       The page itself is the normal single-page app.

import { redis } from "../_lib/license.js";
import { SUPABASE_KEY, SUPABASE_URL } from "../_lib/config.js";
import { cached, tmdb, TITLE_KEY, WEEK } from "../_lib/tmdb.js";

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const NAME = /^[A-Za-z0-9_]{3,20}$/;
const SOCIAL = "https://social.getvelmora.dpdns.org";

// An account has Max when it holds a key that isn't revoked.
const MAX_SCRIPT = `
local out = {}
for _, uid in ipairs(ARGV) do
  local id = redis.call('HGET', 'acct:' .. uid, 'id')
  if id and redis.call('SISMEMBER', 'lic:revoked', id) == 0 then out[#out + 1] = uid end
end
return out`;

async function max(req, res) {
  const ids = [...new Set(String(req.query.ids ?? "").split(",").map((s) => s.trim().toLowerCase()))].filter((s) => UUID.test(s)).slice(0, 60);
  if (!ids.length) return res.status(200).json({ max: [] });
  const found = await redis("EVAL", MAX_SCRIPT, "0", ...ids);
  res.setHeader("Cache-Control", "public, s-maxage=600, stale-while-revalidate=3600");
  return res.status(200).json({ max: Array.isArray(found) ? found : [] });
}

// ── Titles from TMDB ──

// ── Titles from TMDB (cached in Redis by _lib/tmdb.js) ──

async function titles(req, res) {
  const ks = [...new Set(String(req.query.ks ?? "").split(","))].filter((k) => TITLE_KEY.test(k)).slice(0, 40);
  if (!ks.length) return res.status(200).json({ titles: {} });
  const found = await cached("tm:", ks, tmdb, WEEK);
  res.setHeader("Cache-Control", "public, s-maxage=3600, stale-while-revalidate=86400");
  return res.status(200).json({ titles: found });
}

// ── Songs from YouTube ──

const VIDEO = /^[A-Za-z0-9_-]{11}$/;

async function song(id) {
  const r = await fetch(`https://www.youtube.com/oembed?format=json&url=${encodeURIComponent(`https://www.youtube.com/watch?v=${id}`)}`, {
    signal: AbortSignal.timeout(6000),
  });
  if (r.status === 404 || r.status === 401 || r.status === 400) return null;
  if (!r.ok) throw new Error(`oembed ${r.status}`);
  const j = await r.json();
  return { id, title: String(j.title ?? "").slice(0, 200), artist: String(j.author_name ?? "").replace(/ - Topic$/, "").slice(0, 120) };
}

async function songs(req, res) {
  const ids = [...new Set(String(req.query.ids ?? "").split(","))].filter((id) => VIDEO.test(id)).slice(0, 40);
  if (!ids.length) return res.status(200).json({ songs: {} });
  const found = await cached("yt:", ids, song, 30 * 86400);
  res.setHeader("Cache-Control", "public, s-maxage=3600, stale-while-revalidate=86400");
  return res.status(200).json({ songs: found });
}

const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

async function page(req, res) {
  const name = String(req.query.u ?? "");
  const shell = await fetch(`${SOCIAL}/social/`, { signal: AbortSignal.timeout(6000) });
  if (!shell.ok) throw new Error(`shell ${shell.status}`);
  let html = await shell.text();

  if (NAME.test(name)) {
    const q = new URLSearchParams({
      select: "username,display_name,avatar,bio,followers_count",
      username_key: `eq.${name.toLowerCase()}`,
      limit: "1",
    });
    const r = await fetch(`${SUPABASE_URL}/rest/v1/profiles?${q}`, {
      headers: { apikey: SUPABASE_KEY, Authorization: `Bearer ${SUPABASE_KEY}` },
      signal: AbortSignal.timeout(5000),
    }).catch(() => null);
    const [p] = r?.ok ? await r.json().catch(() => []) : [];
    if (p) {
      const title = `${p.display_name} (@${p.username}) on Velmora`;
      const followers = `${p.followers_count} follower${p.followers_count === 1 ? "" : "s"}`;
      const desc = p.bio ? `${p.bio} · ${followers}` : `${followers} on Velmora Social.`;
      const image = /^https:\/\//.test(p.avatar) ? p.avatar : `${SOCIAL}/icon-310.png`;
      const url = `${SOCIAL}/@${p.username}`;
      html = html
        .replace(/<title>[^<]*<\/title>/, `<title>${esc(title)}</title>`)
        .replace(/<meta name="description"[^>]*>/, `<meta name="description" content="${esc(desc)}" />`)
        .replace(/<meta property="og:title"[^>]*>/, `<meta property="og:title" content="${esc(title)}" />`)
        .replace(/<meta property="og:description"[^>]*>/, `<meta property="og:description" content="${esc(desc)}" />`)
        .replace(/<meta property="og:image"[^>]*>/, `<meta property="og:image" content="${esc(image)}" />`)
        .replace("</head>", `<meta property="og:url" content="${esc(url)}" />\n<meta property="og:type" content="profile" />\n<meta name="twitter:card" content="summary" />\n<link rel="canonical" href="${esc(url)}" />\n</head>`);
    }
  }

  res.setHeader("Content-Type", "text/html; charset=utf-8");
  res.setHeader("Cache-Control", "public, s-maxage=300, stale-while-revalidate=3600");
  return res.status(200).send(html);
}

export default async function handler(req, res) {
  if (req.method !== "GET") return res.status(405).json({ ok: false });
  try {
    switch (req.query.action) {
      case "max":
        return await max(req, res);
      case "page":
        return await page(req, res);
      case "titles":
        return await titles(req, res);
      case "songs":
        return await songs(req, res);
      default:
        return res.status(404).json({ ok: false });
    }
  } catch (e) {
    console.error("social:", e);
    if (req.query.action === "page") {
      // Still show the profile, just without the preview details.
      res.setHeader("Cache-Control", "no-store");
      return res.redirect(302, `/social/?at=${encodeURIComponent(`/@${String(req.query.u ?? "")}`)}`);
    }
    res.setHeader("Cache-Control", "no-store");
    return res.status(503).json({ max: [] });
  }
}
