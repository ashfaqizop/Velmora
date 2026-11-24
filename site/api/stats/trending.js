// The titles most watched on Velmora lately, for the app's "Popular on Velmora" row.
//
//   GET /api/stats/trending?days=7&limit=20 -> { people: <installs>, titles: [{ k: "movie/603", people: 12 }] }
//   GET /api/stats/trending?kind=music       -> the same, with YouTube video ids as `k`
//
// Only titles watched by at least MIN_PEOPLE different installs are listed.
// Cached for half an hour at the edge, so it costs almost nothing to read.

import { MIN_PEOPLE, totals } from "../_lib/stats.js";

export default async function handler(req, res) {
  const days = Math.min(Math.max(parseInt(req.query.days, 10) || 7, 1), 30);
  const limit = Math.min(Math.max(parseInt(req.query.limit, 10) || 20, 1), 50);
  try {
    const group = req.query.kind === "music" ? "mus" : "title";
    const [titles, inst] = await Promise.all([totals(group, days, 300), totals("inst", days, 1)]);
    res.setHeader("Cache-Control", "public, s-maxage=1800, stale-while-revalidate=3600");
    return res.status(200).json({
      people: inst[0]?.count ?? 0,
      titles: titles.filter((t) => t.count >= MIN_PEOPLE).slice(0, limit).map((t) => ({ k: t.key, people: t.count })),
    });
  } catch (e) {
    console.error("stats trending:", e);
    res.setHeader("Cache-Control", "no-store");
    return res.status(503).json({ titles: [] });
  }
}
