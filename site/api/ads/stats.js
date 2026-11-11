// Anonymous ad view counts from the app: how many ads each slot showed, per
// day. Nothing about who or which PC, just totals.
//
//   POST /api/ads/stats { counts: { home: 3, sidebar: 5 } }
//
// Stored in Redis as ads:views:<YYYY-MM-DD> (hash: slot -> count), kept 400 days.

import { rateLimited, redis } from "../_lib/license.js";

const SLOTS = new Set(["home", "search", "grid", "title", "downloads", "sidebar", "sports", "live", "pause", "end", "music", "nowplaying"]);

export default async function handler(req, res) {
  res.setHeader("Cache-Control", "no-store");
  if (req.method !== "POST") return res.status(405).json({ ok: false });
  const counts = req.body?.counts;
  if (!counts || typeof counts !== "object") return res.status(400).json({ ok: false });

  try {
    if (await rateLimited(req, 10)) return res.status(429).json({ ok: false });
    const day = new Date().toISOString().slice(0, 10);
    const key = `ads:views:${day}`;
    for (const [slot, n] of Object.entries(counts)) {
      if (!SLOTS.has(slot) || !Number.isInteger(n) || n < 1 || n > 500) continue;
      await redis("HINCRBY", key, slot, n);
    }
    await redis("EXPIRE", key, 400 * 86400);
    return res.status(200).json({ ok: true });
  } catch {
    return res.status(503).json({ ok: false });
  }
}
