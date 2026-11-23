// The numbers behind the website's stats page: which sites get used, which
// player runs, how many installs report in. Totals only.
//
//   GET /api/stats/summary?days=14

import { perDay, totals } from "../_lib/stats.js";

export default async function handler(req, res) {
  const days = Math.min(Math.max(parseInt(req.query.days, 10) || 14, 1), 30);
  try {
    const [sources, engines, quality, versions, daily] = await Promise.all([
      totals("src", days, 20),
      totals("eng", days, 4),
      totals("q", days, 8),
      totals("ver", days, 8),
      perDay("inst", "all", days),
    ]);
    res.setHeader("Cache-Control", "public, s-maxage=1800, stale-while-revalidate=3600");
    return res.status(200).json({ days, sources, engines, quality, versions, daily });
  } catch (e) {
    console.error("stats summary:", e);
    res.setHeader("Cache-Control", "no-store");
    return res.status(503).json({ ok: false });
  }
}
