// Velmora Theatre, server side: the daily lineup and the source checks.
// Spec: docs/theatre.md. Database side: supabase/theatre.sql.
//
//   GET  /api/theatre/cron      Vercel's daily cron (Authorization: Bearer CRON_SECRET),
//                               or by hand with the admin token. Finalizes
//                               tomorrow's lineup, drafts the one after, and
//                               on the first run sets up today and tomorrow.
//   GET  /api/theatre/probes    Drafted titles that apps should check for sources.
//   POST /api/theatre/probe     { k, found } from an app: anonymous, counted per title.
//   POST /api/theatre/start     { id, ok, why? } from an app when a screening tried to
//                               play: anonymous, counted per screening (failure rate).
//
// Needs in Vercel: SUPABASE_SERVICE_ROLE_KEY, TMDB_READ_TOKEN, CRON_SECRET
// (and the Upstash variables the site already has).

import crypto from "node:crypto";
import { isAdmin, rateLimited, redis } from "../_lib/license.js";
import { DAY, draft, finalize, isoDay, MOVIE, syncMax } from "../_lib/lineup.js";
import { db } from "../_lib/supabase.js";

async function cron(req, res) {
  const secret = process.env.CRON_SECRET;
  const auth = String(req.headers.authorization ?? "");
  const fromCron = secret && auth.length === secret.length + 7 && crypto.timingSafeEqual(Buffer.from(auth), Buffer.from(`Bearer ${secret}`));
  if (!fromCron && !isAdmin(req)) return res.status(401).json({ ok: false });

  // Paused from the admin site: no new screenings, but Max accounts still sync.
  const [settings] = await db("theatre_settings?select=paused").catch(() => []);
  if (settings?.paused) {
    const max = await syncMax().catch((e) => ({ error: e.message }));
    return res.status(200).json({ ok: true, paused: true, lineup: [], max });
  }

  const t = Date.now();
  const today = isoDay(t);
  const tomorrow = isoDay(t + DAY);
  const after = isoDay(t + 2 * DAY);
  const out = [];
  // First run (or a missed day): set up today and tomorrow straight away.
  out.push(await draft(today, t + 45 * 60_000));
  out.push(await finalize(today, { now: true }));
  out.push(await draft(tomorrow, t + 45 * 60_000));
  out.push(await finalize(tomorrow));
  out.push(await draft(after, t));
  const max = await syncMax().catch((e) => ({ error: e.message }));
  return res.status(200).json({ ok: true, lineup: out, max });
}

// ── Source checks from the apps ──

async function probes(req, res) {
  const rows = await db("theatre_screenings?select=k,name,year&status=eq.draft&proven=eq.false&order=starts_at");
  const seen = new Set();
  const list = rows.filter((r) => !seen.has(r.k) && seen.add(r.k));
  res.setHeader("Cache-Control", "public, s-maxage=600, stale-while-revalidate=1800");
  return res.status(200).json({ titles: list });
}

async function probe(req, res) {
  const body = typeof req.body === "string" ? JSON.parse(req.body || "{}") : (req.body ?? {});
  if (!MOVIE.test(String(body.k)) || typeof body.found !== "boolean") return res.status(400).json({ ok: false });
  if (await rateLimited(req, 30)) return res.status(429).json({ ok: false });
  const key = `th:probe:${body.k}`;
  await redis("HINCRBY", key, body.found ? "found" : "miss", "1");
  await redis("EXPIRE", key, String(4 * 86400));
  return res.status(200).json({ ok: true });
}

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const WHY = new Set(["source", "player", "network", "other"]);

/** Did the movie start? Counted per screening, kept 90 days. */
async function start(req, res) {
  const body = typeof req.body === "string" ? JSON.parse(req.body || "{}") : (req.body ?? {});
  if (!UUID.test(String(body.id)) || typeof body.ok !== "boolean") return res.status(400).json({ ok: false });
  if (await rateLimited(req, 30)) return res.status(429).json({ ok: false });
  const field = body.ok ? "ok" : WHY.has(body.why) ? body.why : "other";
  const key = `th:start:${body.id}`;
  await redis("HINCRBY", key, field, "1");
  await redis("EXPIRE", key, String(90 * 86400));
  return res.status(200).json({ ok: true });
}

export default async function handler(req, res) {
  try {
    switch (req.query.action) {
      case "cron":
        return await cron(req, res);
      case "probes":
        return req.method === "GET" ? await probes(req, res) : res.status(405).end();
      case "probe":
        return req.method === "POST" ? await probe(req, res) : res.status(405).end();
      case "start":
        return req.method === "POST" ? await start(req, res) : res.status(405).end();
      default:
        return res.status(404).json({ ok: false });
    }
  } catch (e) {
    console.error("theatre:", e);
    res.setHeader("Cache-Control", "no-store");
    return res.status(500).json({ ok: false, error: String(e.message ?? e).slice(0, 200) });
  }
}
