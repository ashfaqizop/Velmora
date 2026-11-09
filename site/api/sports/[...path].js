// Caching proxy for the two sports APIs.
//
// The free tiers are tiny and shared by everyone who uses one key, so Velmora
// can point at this instead: it holds the keys (Vercel environment variables),
// asks each API at most once a minute per URL, and lets Vercel's cache answer
// everyone else. The app then needs no key at all, and the keys never ship in
// the installer.
//
//   /api/sports/thesportsdb/<anything>/eventsday.php?d=2026-11-09&s=Soccer
//   /api/sports/football-data/matches?dateFrom=2026-11-09&dateTo=2026-11-10
//
// The engine appends its own key segment for TheSportsDB; it is ignored here.
// Only the endpoints Velmora uses are allowed, and only GET.

const THESPORTSDB = "https://www.thesportsdb.com/api/v1/json";
const FOOTBALL_DATA = "https://api.football-data.org/v4";

const SPORTSDB_ENDPOINTS = new Set(["eventsday.php", "lookupevent.php", "eventstv.php"]);
const FOOTBALL_DATA_PATHS = [/^matches$/, /^matches\/\d+$/];

/** Live scores move, so a minute is the longest worth caching. */
const CACHE = "public, s-maxage=60, stale-while-revalidate=120";

function bad(res, status, message) {
  res.setHeader("Cache-Control", "no-store");
  res.status(status).json({ error: message });
}

export default async function handler(req, res) {
  if (req.method !== "GET") return bad(res, 405, "GET only");

  const parts = Array.isArray(req.query.path) ? req.query.path : [req.query.path].filter(Boolean);
  const [source, ...rest] = parts;

  // Everything the client sent except Vercel's own `path` parameter.
  const query = new URLSearchParams();
  for (const [k, v] of Object.entries(req.query)) {
    if (k === "path") continue;
    for (const one of Array.isArray(v) ? v : [v]) query.append(k, one);
  }
  const qs = query.toString() ? `?${query}` : "";

  let upstream;
  let headers = { Accept: "application/json", "User-Agent": "velmora-sports-proxy" };

  if (source === "thesportsdb") {
    const endpoint = rest.at(-1);
    if (!SPORTSDB_ENDPOINTS.has(endpoint)) return bad(res, 404, "unknown endpoint");
    const key = process.env.SPORTSDB_KEY || "123"; // 123 is TheSportsDB's public test key
    upstream = `${THESPORTSDB}/${key}/${endpoint}${qs}`;
  } else if (source === "football-data") {
    const path = rest.join("/");
    if (!FOOTBALL_DATA_PATHS.some((re) => re.test(path))) return bad(res, 404, "unknown endpoint");
    const token = process.env.FOOTBALL_DATA_TOKEN;
    if (!token) return bad(res, 503, "football-data token is not configured");
    upstream = `${FOOTBALL_DATA}/${path}${qs}`;
    headers = { ...headers, "X-Auth-Token": token };
  } else {
    return bad(res, 404, "unknown source");
  }

  try {
    const r = await fetch(upstream, { headers, signal: AbortSignal.timeout(15_000) });
    const body = await r.text();
    // Only good answers are cached; an error should be retried, not remembered.
    res.setHeader("Cache-Control", r.ok ? CACHE : "no-store");
    res.setHeader("Content-Type", r.headers.get("content-type") ?? "application/json");
    res.status(r.status).send(body);
  } catch {
    bad(res, 502, "the sports source did not answer");
  }
}
