// The Theatre lineup: how the daily screenings are picked, placed and
// published. Shared by the daily cron (api/theatre) and the admin site
// (api/admin), which can also preview and redo a day.
//
// Manual airings (special = true) are never touched here: the algorithm
// doesn't count them as "already drafted", publish them or move them.

import { hash, redis } from "./license.js";
import { totals } from "./stats.js";
import { db } from "./supabase.js";
import { DEFAULT_KNOBS, loadKnobs } from "./lineup-knobs.js";
import { titleDetails, tmdbGet } from "./tmdb.js";

export const HOUR = 3_600_000;
export const DAY = 24 * HOUR;
/** Default seat budget; the live value is a knob (lineup-knobs.js). */
export const BUDGET = DEFAULT_KNOBS.budget;
/** Minutes between the end of one screening and the next that overlaps its slot. */
export const GAP = 15 * 60_000;
export const MOVIE = /^movie\/\d{1,9}$/;

export const isoDay = (t) => new Date(t).toISOString().slice(0, 10);
export const dayStart = (day) => Date.parse(`${day}T00:00:00Z`);
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

async function history(K) {
  const since = new Date(Date.now() - Math.max(K.pickRepeatDays, K.repeatDays) * DAY).toISOString();
  const rows = await db(`theatre_screenings?select=k,kind,starts_at&starts_at=gte.${since}&status=neq.cancelled`);
  const recent = new Set();
  const recentPicks = new Set();
  for (const r of rows) {
    const t = Date.parse(r.starts_at);
    if (t > Date.now() - K.repeatDays * DAY) recent.add(r.k);
    if (r.kind === "pick" && t > Date.now() - K.pickRepeatDays * DAY) recentPicks.add(r.k);
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
async function trendingPool(skip, K) {
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
    .map((it) => ({ ...it, score: Math.log1p(it.plays) * K.playsWeight + (it.rank === null ? 0 : ((40 - Math.min(it.rank, 40)) / 20) * K.trendWeight) + Math.random() * 0.3 }))
    .sort((a, b) => b.score - a.score);
}

/** Tonight's pick: a highly rated film few Velmora people have seen. */
async function hiddenGem(skip, K) {
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
    .filter((m) => !m.adult && m.vote_count >= K.gemMinVotes && m.vote_average >= K.gemMinRating && !skip.has(`movie/${m.id}`))
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

function place(items, day, share, earliest, K) {
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
      (t) => overlaps(placed, t, t + len) < K.maxOverlap && placed.every((p) => Math.abs(p.start - t) >= HOUR),
    );
    if (start !== undefined) placed.push({ ...it, start, end: start + len });
  });
  return placed.sort((a, b) => a.start - b.start);
}

/** Expected audience, seats from the shared budget, and open vs fixed. */
function seat(placed, share, daily, K) {
  for (const p of placed) {
    const hourShare = share[new Date(p.start).getUTCHours()] * 24; // 1 = an average hour
    const pull = p.kind === "pick" ? 1 : 0.8 + Math.log1p(p.plays) / 3;
    p.expected = Math.max(2, Math.round(daily * 0.12 * hourShare * pull));
  }
  for (const p of placed) {
    const group = placed.filter((q) => q.start < p.end && p.start < q.end);
    const sum = group.reduce((a, q) => a + q.expected, 0);
    p.capacity = clamp(Math.floor((K.budget * p.expected) / sum), 10, K.budget);
    p.mode = p.expected >= p.capacity * 0.8 ? "fixed" : "open";
  }
}

// ── Draft and finalize ──

/** `force` drafts even when the day has a lineup; `dry` returns what it would pick and writes nothing. */
export async function draft(day, earliest, { force = false, dry = false } = {}) {
  if (!force) {
    const existing = await db(`theatre_screenings?select=id&day=eq.${day}&special=eq.false&limit=1`);
    if (existing.length) return { day, drafted: 0, note: "already drafted" };
  }

  const K = await loadKnobs();
  const [{ share, daily }, hist, over] = await Promise.all([audience(), history(K), overrides(day)]);
  const skip = new Set([...hist.recent, ...over.blocked]);
  const count = clamp(K.minPerDay + Math.floor(daily / K.perInstalls), K.minPerDay, K.maxPerDay);

  const [trend, gems] = await Promise.all([trendingPool(skip, K), K.pick ? hiddenGem(new Set([...skip, ...hist.recentPicks]), K) : []]);
  const wanted = [
    ...over.pinned.filter((k) => !over.blocked.has(k)).map((k) => ({ k, plays: 0, kind: "trending", pinned: true })),
    ...trend.map((t) => ({ ...t, kind: "trending" })),
  ];
  const keys = [...new Set([...wanted.map((w) => w.k), ...gems.map((g) => g.k)])].slice(0, 60);
  const details = await titleDetails(keys);
  const excluded = new Set(K.excludeGenres);
  const usable = (k) => {
    const d = details[k];
    return d && d.runtime && d.runtime >= K.minRuntime && d.poster && !(d.genres ?? []).some((g) => excluded.has(g));
  };
  const chosen = [];
  // Genre mix: at most maxPerGenre titles share a main genre (pins don't count against it).
  const perGenre = new Map();
  const mainGenre = (k) => details[k]?.genres?.[0] ?? "";
  const roomFor = (k) => !K.maxPerGenre || !mainGenre(k) || (perGenre.get(mainGenre(k)) ?? 0) < K.maxPerGenre;
  const take = (item) => {
    chosen.push(item);
    const g = mainGenre(item.k);
    if (g) perGenre.set(g, (perGenre.get(g) ?? 0) + 1);
  };

  const pick = gems.find((g) => usable(g.k));
  if (pick) take({ ...pick, kind: "pick" });
  for (const w of wanted) {
    if (chosen.length >= count) break;
    if (chosen.some((c) => c.k === w.k)) continue;
    if (w.pinned ? details[w.k]?.runtime && details[w.k]?.poster : usable(w.k) && roomFor(w.k)) take(w);
  }
  const items = chosen.map((c) => ({ ...c, runtime: details[c.k].runtime }));
  const placed = place(items, day, share, earliest, K);
  seat(placed, share, daily, K);

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
  if (dry) return { day, drafted: 0, preview: rows.map((r) => ({ k: r.k, name: r.name, year: r.year, kind: r.kind, starts_at: r.starts_at, runtime: r.runtime, mode: r.mode, capacity: r.capacity, pinned: Boolean(chosen.find((c) => c.k === r.k)?.pinned) })) };
  if (rows.length) await db("theatre_screenings", { method: "POST", body: rows, prefer: "return=minimal" });
  return { day, drafted: rows.length };
}

export async function finalize(day, { now = false } = {}) {
  const K = await loadKnobs();
  const drafts = await db(`theatre_screenings?select=id,k,proven&day=eq.${day}&status=eq.draft&special=eq.false`);
  if (!drafts.length) return { day, scheduled: 0 };
  const dropped = [];
  for (const d of drafts) {
    if (d.proven) continue;
    const p = hash(await redis("HGETALL", `th:probe:${d.k}`));
    const found = Number(p.found) || 0;
    const miss = Number(p.miss) || 0;
    if (found + miss >= K.probeMinChecks && found / (found + miss) < K.probeMinFound) dropped.push(d.id);
  }
  if (dropped.length) await db(`theatre_screenings?id=in.(${dropped.join(",")})`, { method: "PATCH", body: { status: "cancelled" }, prefer: "return=minimal" });
  const t = Date.now();
  await db(`theatre_screenings?day=eq.${day}&status=eq.draft&special=eq.false`, {
    method: "PATCH",
    body: { status: "scheduled", max_opens_at: new Date(t).toISOString(), opens_at: new Date(now ? t : t + 6 * HOUR).toISOString() },
    prefer: "return=minimal",
  });
  return { day, scheduled: drafts.length - dropped.length, dropped: dropped.length };
}

/** Keep max_accounts in step with the license server (catches revokes and old links). */
export async function syncMax() {
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
