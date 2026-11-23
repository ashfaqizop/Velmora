// The app's daily batch of anonymous counts.
//
//   POST /api/stats/report { v: 1, day, app, titles: [{k}], sources: [{k, n}], engines: [{k, n}], quality: [{k, n}] }
//
// See _lib/stats.js for what is kept. A batch is one install's finished day.

import { cleanReport, dailyLimited, store } from "../_lib/stats.js";

export default async function handler(req, res) {
  res.setHeader("Cache-Control", "no-store");
  if (req.method !== "POST") return res.status(405).json({ ok: false });
  const report = cleanReport(req.body);
  if (!report) return res.status(400).json({ ok: false });
  try {
    // Generous: many homes share one address.
    if (await dailyLimited(req, 80)) return res.status(429).json({ ok: false });
    await store(report);
    return res.status(200).json({ ok: true });
  } catch (e) {
    console.error("stats report:", e);
    return res.status(503).json({ ok: false });
  }
}
