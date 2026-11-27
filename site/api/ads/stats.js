// The app's ads, both ways.
//
//   GET  /ads.json (rewritten here)              the ad rules the app follows
//   POST /api/ads/stats { counts: { home: 3 } }  anonymous view counts
//
// Rules: Redis cfg:ads, saved from the admin site, else DEFAULT_RULES.
// Counts: Redis ads:views:<YYYY-MM-DD> (hash: slot -> count), kept 400 days.
// Nothing about who or which PC, just totals.

import { DEFAULT_RULES, SLOTS } from "../_lib/ads-rules.js";
import { rateLimited, redis } from "../_lib/license.js";

const COUNTED = new Set(SLOTS);

async function rules(res) {
  let body = DEFAULT_RULES;
  try {
    const saved = await redis("GET", "cfg:ads");
    if (saved) body = JSON.parse(saved);
  } catch {
    // Storage hiccup: the defaults are a safe answer.
  }
  // Edge-cached for a minute, so a change reaches apps within about five.
  res.setHeader("Cache-Control", "public, max-age=300, s-maxage=60, stale-while-revalidate=600");
  res.setHeader("Access-Control-Allow-Origin", "*");
  return res.status(200).json(body);
}

export default async function handler(req, res) {
  if (req.method === "GET") return rules(res);
  res.setHeader("Cache-Control", "no-store");
  if (req.method !== "POST") return res.status(405).json({ ok: false });
  const counts = req.body?.counts;
  if (!counts || typeof counts !== "object") return res.status(400).json({ ok: false });

  try {
    if (await rateLimited(req, 10)) return res.status(429).json({ ok: false });
    const day = new Date().toISOString().slice(0, 10);
    const key = `ads:views:${day}`;
    for (const [slot, n] of Object.entries(counts)) {
      if (!COUNTED.has(slot) || !Number.isInteger(n) || n < 1 || n > 500) continue;
      await redis("HINCRBY", key, slot, n);
    }
    await redis("EXPIRE", key, 400 * 86400);
    return res.status(200).json({ ok: true });
  } catch {
    return res.status(503).json({ ok: false });
  }
}
