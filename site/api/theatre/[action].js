// Velmora Theatre, server side: the daily lineup and the source checks.
// Spec: docs/theatre.md. Database side: supabase/theatre.sql.
//
//   GET  /api/theatre/cron      Vercel's daily cron (Authorization: Bearer CRON_SECRET),
//                               or by hand with the admin token. Finalizes
//                               tomorrow's lineup, drafts the one after, and
//                               on the first run sets up today and tomorrow.
//   GET  /api/theatre/probes    Drafted titles that apps should check for sources.
//   POST /api/theatre/probe     { k, found } from an app: anonymous, counted per title.
//
// Needs in Vercel: SUPABASE_SERVICE_ROLE_KEY, TMDB_READ_TOKEN, CRON_SECRET
// (and the Upstash variables the site already has).

import crypto from "node:crypto";
import { hash, isAdmin, rateLimited, redis } from "../_lib/license.js";
import { totals } from "../_lib/stats.js";
import { db } from "../_lib/supabase.js";
import { titleDetails, tmdbGet } from "../_lib/tmdb.js";

const HOUR = 3_600_000;
const DAY = 24 * HOUR;
/** People connected at once that Theatre may use (Supabase Realtime free: ~200, leave room). */
const BUDGET = 160;
const MAX_OVERLAP = 3;
/** Minutes between the end of one screening and the next that overlaps its slot. */
const GAP = 15 * 60_000;
const REPEAT_DAYS = 14;
const PICK_REPEAT_DAYS = 60;
const MOVIE = /^movie\/\d{1,9}$/;

const isoDay = (t) => new Date(t).toISOString().slice(0, 10);
const dayStart = (day) => Date.parse(`${day}T00:00:00Z`);
const clamp = (n, lo, hi) => Math.max(lo, Math.min(hi, n));

// When people watch, by UTC hour, until Velmora has its own numbers: evenings
// in South Asia (13-17 UTC), Europe (18-22 UTC) and the Americas (0-3 UTC).
const DEFAULT_HOURS = [6, 5, 4, 3, 1, 1, 1, 1, 1, 1, 2, 3, 4, 6, 7, 8, 7, 6, 6, 7, 7, 6, 5, 5];

// ── What we know about the audience ──

async function audience() {
  const [hourRows, inst] = await Promise.all([totals("hr", 14, 24).catch(() => []), totals("inst", 7, 1).catch(() => [])]);
  const hours = new Array(24).fill(0);
  for (const r of hourRows) {
    const h = Number(r.key);
    if (h >= 0 && h < 24) hours[h] = r.count;
  }
  const sum = hours.reduce((a, b) => a + b, 0);
  const weights = sum >= 50 ? hours : DEFAULT_HOURS;
  const total = weights.reduce((a, b) => a + b, 0);
  // "inst" counts installs reporting each day; over 7 days, divide for a daily figure.
  const daily = Math.round((inst[0]?.count ?? 0) / 7);
  return { share: weights.map((w) => w / total), daily };
}

// ── Picking titles ──

async function history() {
  const since = new Date(Date.now() - PICK_REPEAT_DAYS * DAY).toISOString();
  const rows = await db(`theatre_screenings?select=k,kind,starts_at&starts_at=gte.${since}&status=neq.cancelled`);
  const recent = new Set();
  const recentPicks = new Set();
  for (const r of rows) {
    if (Date.parse(r.starts_at) > Date.now() - REPEAT_DAYS * DAY) recent.add(r.k);
    if (r.kind === "pick") recentPicks.add(r.k);
  }
  return { recent, recentPicks };
}

async function overrides(day) {
  const rows = await db("theatre_overrides?select=action,k,day");
  return {
    blocked: new Set(rows.filter((r) => r.action === "block").map((r) => r.k)),
    pinned: rows.filter((r) => r.action === "pin" && r.day === day && MOVIE.test(r.k)).map((r) => r.k),
  };
}

/** Movies Velmora people played this week (proven to have sources), mixed with TMDB's trending. */
async function trendingPool(skip) {
  const [plays, trend] = await Promise.all([
    totals("title", 7, 300).catch(() => []),
    Promise.all([1, 2].map((page) => tmdbGet("/trending/movie/week", { page }).catch(() => ({ results: [] })))),
  ]);
  const pool = new Map();
  for (const p of plays) if (MOVIE.test(p.key) && !skip.has(p.key)) pool.set(p.key, { k: p.key, plays: p.count, rank: null });
  trend
    .flatMap((t) => t.results ?? [])
    .forEach((m, i) => {
      const k = `movie/${m.id}`;
      if (m.adult || skip.has(k)) return;
      const it = pool.get(k) ?? { k, plays: 0, rank: null };
      it.rank = it.rank ?? i;
      pool.set(k, it);
    });
  // Velmora plays weigh most (people here watch it, and it plays); TMDB fills in.
  return [...pool.values()]
    .map((it) => ({ ...it, score: Math.log1p(it.plays) * 3 + (it.rank === null ? 0 : (40 - Math.min(it.rank, 40)) / 20) + Math.random() * 0.3 }))
    .sort((a, b) => b.score - a.score);
}

/** Tonight's pick: a highly rated film few Velmora people have seen. */
async function hiddenGem(skip) {
  const pages = [1 + Math.floor(Math.random() * 10), 11 + Math.floor(Math.random() * 15)];
  const [lists, plays, rated] = await Promise.all([
    Promise.all(pages.map((page) => tmdbGet("/movie/top_rated", { page }).catch(() => ({ results: [] })))),
    totals("title", 30, 300).catch(() => []),
    db("rpc/top_rated", { method: "POST", body: { lim: 60, least_ratings: 1 } }).catch(() => []),
  ]);
  const played = new Map(plays.map((p) => [p.key, p.count]));
  const velmora = new Map((rated ?? []).map((r) => [r.k, Number(r.average)]));
  const pool = lists
    .flatMap((l) => l.results ?? [])
    .filter((m) => !m.adult && m.vote_count >= 800 && m.vote_average >= 7.4 && !skip.has(`movie/${m.id}`))
    .map((m) => {
      const k = `movie/${m.id}`;
      const v = velmora.get(k);
      const score = m.vote_average + (v ? (v / 2 - 3) * 0.5 : 0) - Math.log1p(played.get(k) ?? 0) * 0.6 + Math.random() * 0.8;
      return { k, plays: played.get(k) ?? 0, score };
    })
    .sort((a, b) => b.score - a.score);
  return pool.slice(0, 5);
}

// ── Placing them in the day ──

function overlaps(placed, start, end) {
  return placed.filter((p) => p.start < end + GAP && start < p.end + GAP).length;
}

/** Half-hour start times for a UTC day, after `earliest`. */
function slotsOf(day, earliest) {
  const base = dayStart(day);
  const out = [];
  for (let i = 0; i < 48; i++) {
    const t = base + i * 30 * 60_000;
    if (t >= earliest) out.push(t);
  }
  return out;
}

function place(items, day, share, earliest) {
  const slots = slotsOf(day, earliest);
  const placed = [];
  const busyN = Math.ceil(items.length / 2);
  const byBusy = [...slots].sort((a, b) => share[new Date(b).getUTCHours()] - share[new Date(a).getUTCHours()] || a - b);
  items.forEach((it, i) => {
    const len = it.runtime * 60_000;
    let order;
    if (i < busyN) order = byBusy;
    else {
      // Spread: the start furthest (around the clock) from the others.
      const dist = (t) => {
        if (!placed.length) return 24;
        const h = (t / HOUR) % 24;
        return Math.min(...placed.map((p) => {
          const d = Math.abs(h - (p.start / HOUR) % 24);
          return Math.min(d, 24 - d);
        }));
      };
      order = [...slots].sort((a, b) => dist(b) - dist(a) || share[new Date(b).getUTCHours()] - share[new Date(a).getUTCHours()]);
    }
    const start = order.find(
      (t) => overlaps(placed, t, t + len) < MAX_OVERLAP && placed.every((p) => Math.abs(p.start - t) >= HOUR),
    );
    if (start !== undefined) placed.push({ ...it, start, end: start + len });
  });
  return placed.sort((a, b) => a.start - b.start);
}

/** Expected audience, seats from the shared budget, and open vs fixed. */
function seat(placed, share, daily) {
  for (const p of placed) {
    const hourShare = share[new Date(p.start).getUTCHours()] * 24; // 1 = an average hour
    const pull = p.kind === "pick" ? 1 : 0.8 + Math.log1p(p.plays) / 3;
    p.expected = Math.max(2, Math.round(daily * 0.12 * hourShare * pull));
  }
  for (const p of placed) {
    const group = placed.filter((q) => q.start < p.end && p.start < q.end);
    const sum = group.reduce((a, q) => a + q.expected, 0);
    p.capacity = clamp(Math.floor((BUDGET * p.expected) / sum), 10, BUDGET);
    p.mode = p.expected >= p.capacity * 0.8 ? "fixed" : "open";
  }
}

// ── Draft and finalize ──

async function draft(day, earliest) {
  const existing = await db(`theatre_screenings?select=id&day=eq.${day}&limit=1`);
  if (existing.length) return { day, drafted: 0, note: "already drafted" };

  const [{ share, daily }, hist, over] = await Promise.all([audience(), history(), overrides(day)]);
  const skip = new Set([...hist.recent, ...over.blocked]);
  const count = clamp(4 + Math.floor(daily / 30), 4, 12);

  const [trend, gems] = await Promise.all([trendingPool(skip), hiddenGem(new Set([...skip, ...hist.recentPicks]))]);
  const wanted = [
    ...over.pinned.filter((k) => !over.blocked.has(k)).map((k) => ({ k, plays: 0, kind: "trending", pinned: true })),
    ...trend.map((t) => ({ ...t, kind: "trending" })),
  ];
  const keys = [...new Set([...wanted.map((w) => w.k), ...gems.map((g) => g.k)])].slice(0, 60);
  const details = await titleDetails(keys);
  const usable = (k) => {
    const d = details[k];
    return d && d.runtime && d.runtime >= 40 && d.poster;
  };

  const chosen = [];
  const pick = gems.find((g) => usable(g.k));
  if (pick) chosen.push({ ...pick, kind: "pick" });
  for (const w of wanted) {
    if (chosen.length >= count) break;
    if (usable(w.k) && !chosen.some((c) => c.k === w.k)) chosen.push(w);
  }
  const items = chosen.map((c) => ({ ...c, runtime: details[c.k].runtime }));
  const placed = place(items, day, share, earliest);
  seat(placed, share, daily);

  const rows = placed.map((p) => {
    const d = details[p.k];
    return {
      day,
      k: p.k,
      name: d.name,
      year: d.year,
      poster: d.poster,
      backdrop: d.backdrop,
      runtime: d.runtime,
      kind: p.kind,
      starts_at: new Date(p.start).toISOString(),
      ends_at: new Date(p.end).toISOString(),
      status: "draft",
      mode: p.mode,
      capacity: p.capacity,
      expected: p.expected,
      proven: p.plays > 0 || !!p.pinned,
    };
  });
  if (rows.length) await db("theatre_screenings", { method: "POST", body: rows, prefer: "return=minimal" });
  return { day, drafted: rows.length };
}

async function finalize(day, { now = false } = {}) {
  const drafts = await db(`theatre_screenings?select=id,k,proven&day=eq.${day}&status=eq.draft`);
  if (!drafts.length) return { day, scheduled: 0 };
  const dropped = [];
  for (const d of drafts) {
    if (d.proven) continue;
    const p = hash(await redis("HGETALL", `th:probe:${d.k}`));
    const found = Number(p.found) || 0;
    const miss = Number(p.miss) || 0;
    if (found + miss >= 3 && found / (found + miss) < 0.5) dropped.push(d.id);
  }
  if (dropped.length) await db(`theatre_screenings?id=in.(${dropped.join(",")})`, { method: "PATCH", body: { status: "cancelled" }, prefer: "return=minimal" });
  const t = Date.now();
  await db(`theatre_screenings?day=eq.${day}&status=eq.draft`, {
    method: "PATCH",
    body: { status: "scheduled", max_opens_at: new Date(t).toISOString(), opens_at: new Date(now ? t : t + 6 * HOUR).toISOString() },
    prefer: "return=minimal",
  });
  return { day, scheduled: drafts.length - dropped.length, dropped: dropped.length };
}

/** Keep max_accounts in step with the license server (catches revokes and old links). */
async function syncMax() {
  const users = new Set();
  let cursor = "0";
  do {
    const [next, keys] = await redis("SCAN", cursor, "MATCH", "acct:*", "COUNT", "500");
    cursor = next;
    for (const key of keys) {
      const id = await redis("HGET", key, "id");
      if (id && !(await redis("SISMEMBER", "lic:revoked", id))) users.add(key.slice(5));
    }
  } while (cursor !== "0");
  const have = new Set((await db("max_accounts?select=user_id")).map((r) => r.user_id));
  const add = [...users].filter((u) => !have.has(u)).map((user_id) => ({ user_id }));
  const gone = [...have].filter((u) => !users.has(u));
  if (add.length) await db("max_accounts?on_conflict=user_id", { method: "POST", body: add, prefer: "resolution=ignore-duplicates,return=minimal" }).catch((e) => console.warn("max add:", e.message));
  if (gone.length) await db(`max_accounts?user_id=in.(${gone.join(",")})`, { method: "DELETE", prefer: "return=minimal" });
  return { max: users.size, added: add.length, removed: gone.length };
}

async function cron(req, res) {
  const secret = process.env.CRON_SECRET;
  const auth = String(req.headers.authorization ?? "");
  const fromCron = secret && auth.length === secret.length + 7 && crypto.timingSafeEqual(Buffer.from(auth), Buffer.from(`Bearer ${secret}`));
  if (!fromCron && !isAdmin(req)) return res.status(401).json({ ok: false });

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

export default async function handler(req, res) {
  try {
    switch (req.query.action) {
      case "cron":
        return await cron(req, res);
      case "probes":
        return req.method === "GET" ? await probes(req, res) : res.status(405).end();
      case "probe":
        return req.method === "POST" ? await probe(req, res) : res.status(405).end();
      default:
        return res.status(404).json({ ok: false });
    }
  } catch (e) {
    console.error("theatre:", e);
    res.setHeader("Cache-Control", "no-store");
    return res.status(500).json({ ok: false, error: String(e.message ?? e).slice(0, 200) });
  }
}
