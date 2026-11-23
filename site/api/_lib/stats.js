// Anonymous usage statistics, kept as daily counters in Redis. Nothing here
// says who: no ID, no account, no IP address (the per-address limit below keeps
// only a short hash for a day). The app sends one batch per day of counts.
//
// Redis keys, all sorted sets that expire after 40 days:
//   an:<day>:title   member "movie/603" or "tv/1396" (TMDB) -> people who watched it
//   an:<day>:src     member provider id                      -> plays
//   an:<day>:eng     member "mpv" | "builtin"                -> plays
//   an:<day>:q       member resolution                       -> plays
//   an:<day>:ver     member app version                      -> people
//   an:<day>:inst    member "all"                            -> people (reporting that day)
//
// "People" counts one per batch, so it is the number of installs, not plays.

import crypto from "node:crypto";
import { redis } from "./license.js";

export const KEEP_DAYS = 40;
/** A title only shows up once this many different installs watched it. */
export const MIN_PEOPLE = 3;
export const GROUPS = ["title", "src", "eng", "q", "ver", "inst"];

const DAY = /^\d{4}-\d{2}-\d{2}$/;
const TITLE = /^(movie|tv)\/\d{1,9}$/;
const WORD = /^[a-z0-9_]{2,24}$/;
const ENGINES = new Set(["mpv", "builtin"]);
const QUALITY = new Set(["480", "720", "1080", "1440", "2160", "other"]);
const VERSION = /^\d{1,2}\.\d{1,3}\.\d{1,3}$/;

export const today = () => new Date().toISOString().slice(0, 10);

export function daysBack(n) {
  const out = [];
  const t = Date.now();
  for (let i = 0; i < n; i++) out.push(new Date(t - i * 86_400_000).toISOString().slice(0, 10));
  return out;
}

/** The pairs in `list` that are valid, as [member, count] (capped). */
function pairs(list, ok, cap = 40) {
  if (!Array.isArray(list)) return [];
  const out = [];
  for (const it of list.slice(0, cap)) {
    if (!it || typeof it.k !== "string" || !ok(it.k)) continue;
    const n = Number.isInteger(it.n) ? Math.min(Math.max(it.n, 1), 200) : 1;
    out.push([it.k, n]);
  }
  return out;
}

/** A report body checked and cleaned, or null if it isn't one. */
export function cleanReport(body) {
  if (!body || typeof body !== "object" || body.v !== 1) return null;
  if (typeof body.day !== "string" || !DAY.test(body.day) || !daysBack(9).includes(body.day) || body.day === today()) return null;
  if (typeof body.app !== "string" || !VERSION.test(body.app)) return null;
  const titles = pairs(body.titles, (k) => TITLE.test(k), 60).map(([k]) => [k, 1]);
  const src = pairs(body.sources, (k) => WORD.test(k));
  const eng = pairs(body.engines, (k) => ENGINES.has(k), 2);
  const q = pairs(body.quality, (k) => QUALITY.has(k), 6);
  if (!titles.length && !src.length) return null;
  return { day: body.day, groups: [titles, src, eng, q, [[body.app, 1]], [["all", 1]]] };
}

const STORE_SCRIPT = `
local ttl = tonumber(ARGV[1]); local p = 2
for k = 1, #KEYS do
  local n = tonumber(ARGV[p]); p = p + 1
  for _ = 1, n do
    redis.call('ZINCRBY', KEYS[k], tonumber(ARGV[p + 1]), ARGV[p]); p = p + 2
  end
  redis.call('EXPIRE', KEYS[k], ttl)
end
return 1`;

/** One command adds a whole report. */
export async function store(report) {
  const keys = GROUPS.map((g) => `an:${report.day}:${g}`);
  const args = [String(KEEP_DAYS * 86400)];
  for (const group of report.groups) {
    args.push(String(group.length));
    for (const [member, n] of group) args.push(member, String(n));
  }
  await redis("EVAL", STORE_SCRIPT, String(keys.length), ...keys, ...args);
}

const MERGE_SCRIPT = `
local totals = {}; local order = {}
for _, key in ipairs(KEYS) do
  local r = redis.call('ZREVRANGE', key, 0, 299, 'WITHSCORES')
  for i = 1, #r, 2 do
    local m = r[i]
    if totals[m] == nil then totals[m] = 0; order[#order + 1] = m end
    totals[m] = totals[m] + tonumber(r[i + 1])
  end
end
local out = {}
for _, m in ipairs(order) do out[#out + 1] = m; out[#out + 1] = tostring(totals[m]) end
return out`;

/** Totals for one group over the last `days` days, biggest first. One command. */
export async function totals(group, days, limit = 50) {
  const keys = daysBack(days).map((d) => `an:${d}:${group}`);
  const flat = (await redis("EVAL", MERGE_SCRIPT, String(keys.length), ...keys)) ?? [];
  const rows = [];
  for (let i = 0; i + 1 < flat.length; i += 2) rows.push({ key: flat[i], count: Number(flat[i + 1]) });
  return rows.sort((a, b) => b.count - a.count).slice(0, limit);
}

/** Per-day totals of one member of one group, newest last. One command per day. */
export async function perDay(group, member, days) {
  const list = daysBack(days).reverse();
  const counts = await Promise.all(list.map((d) => redis("ZSCORE", `an:${d}:${group}`, member)));
  return list.map((day, i) => ({ day, count: Number(counts[i] ?? 0) }));
}

/** At most `limit` reports a day from one address (a short hash, kept a day). */
export async function dailyLimited(req, limit) {
  const ip = String(req.headers["x-forwarded-for"] ?? "").split(",")[0].trim() || "unknown";
  const key = `an:ip:${crypto.createHash("sha256").update(`velmora:${ip}`).digest("hex").slice(0, 16)}:${today()}`;
  const n = await redis("INCR", key);
  if (n === 1) await redis("EXPIRE", key, 90_000);
  return n > limit;
}
