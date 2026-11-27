// Admin phase 3b: platform analytics, Social curation, and Theatre tuning and
// insight. Wired into api/admin/[action].js (sign-in, admin row and TOTP are
// already checked by the time anything here runs).

import { audit, DAYKEY, must, num, Refusal, resolveUser, TITLE, UUID } from "./admin-kit.js";
import { hash, redis } from "./license.js";
import { cleanKnobs, DEFAULT_KNOBS, GENRES, loadKnobs } from "./lineup-knobs.js";
import { daysBack, MIN_PEOPLE, perDay, totals } from "./stats.js";
import { db } from "./supabase.js";
import { titleDetails } from "./tmdb.js";

// ── Platform analytics ────────────────────────────────────────────────────

const share = (rows) => {
  const sum = rows.reduce((a, r) => a + r.count, 0) || 1;
  return rows.map((r) => ({ ...r, share: r.count / sum }));
};

async function named(rows) {
  const keys = rows.map((r) => r.key).filter((k) => TITLE.test(k));
  const info = keys.length ? await titleDetails(keys).catch(() => ({})) : {};
  return rows.map((r) => ({ ...r, name: info[r.key]?.name ?? r.key, year: info[r.key]?.year ?? null, poster: info[r.key]?.poster ?? null }));
}

async function analytics(b) {
  const days = [7, 14, 30].includes(b.days) ? b.days : 14;
  const [installs, versions, engines, quality, sources, hours, titles, music, growth, adDays] = await Promise.all([
    perDay("inst", "all", 30),
    totals("ver", days, 8),
    totals("eng", days, 4),
    totals("q", days, 8),
    totals("src", days, 20),
    totals("hr", days, 24),
    totals("title", days, 20),
    totals("mus", days, 10),
    db("rpc/admin_growth", { method: "POST", body: { days: 30 } }).catch(() => []),
    Promise.all(daysBack(30).reverse().map(async (day) => {
      const h = hash(await redis("HGETALL", `ads:views:${day}`));
      return { day, count: Object.values(h).reduce((a, n) => a + (Number(n) || 0), 0) };
    })),
  ]);
  const byHour = new Array(24).fill(0);
  for (const r of hours) {
    const h = Number(r.key);
    if (h >= 0 && h < 24) byHour[h] = r.count;
  }
  return {
    days,
    installs,
    versions: share(versions),
    engines: share(engines),
    quality: share(quality),
    sources: share(sources),
    hours: byHour,
    titles: await named(titles),
    music,
    growth,
    ads: adDays,
    note: `Titles and songs show once ${MIN_PEOPLE} different installs played them. Counts are per install per day, never per person.`,
  };
}

// ── Social curation ───────────────────────────────────────────────────────

async function curation() {
  const [rows, hidden] = await Promise.all([
    db("featured?select=*&order=kind,position,created_at"),
    db("leaderboard_hidden?select=user_id,created_at&order=created_at.desc"),
  ]);
  const ids = [...new Set([...rows.map((r) => r.user_id), ...hidden.map((h) => h.user_id)].filter(Boolean))];
  const people = ids.length ? await db(`profiles?id=in.(${ids.join(",")})&select=id,username,display_name,avatar,verified,hidden,banned,followers_count`) : [];
  const who = Object.fromEntries(people.map((p) => [p.id, p]));
  return {
    people: rows.filter((r) => r.kind === "person").map((r) => ({ ...r, profile: who[r.user_id] ?? null })),
    titles: rows.filter((r) => r.kind === "title"),
    hidden: hidden.map((h) => ({ ...h, profile: who[h.user_id] ?? null })),
  };
}

async function curationSet(b, me) {
  const note = String(b.note ?? "").trim().slice(0, 140);
  switch (b.op) {
    case "addPerson": {
      const uid = await resolveUser(b.value);
      const [p] = await db(`profiles?id=eq.${uid}&select=username,hidden,banned`);
      must(p, "That account has no profile yet.");
      must(!p.hidden && !p.banned, "That account is hidden or banned.");
      const [last] = await db("featured?kind=eq.person&select=position&order=position.desc&limit=1");
      await db("featured", { method: "POST", body: { kind: "person", user_id: uid, note, position: (last?.position ?? 0) + 1 }, prefer: "return=minimal" }).catch(() => {
        throw new Refusal("They're already featured.");
      });
      await audit(me, "curation.person", uid, { username: p.username });
      return { ok: true };
    }
    case "addTitle": {
      const k = String(b.value ?? "").trim();
      must(TITLE.test(k), "Pick a movie or show.");
      const d = (await titleDetails([k]))[k];
      must(d, "TMDB doesn't have that title.");
      const [last] = await db("featured?kind=eq.title&select=position&order=position.desc&limit=1");
      await db("featured", { method: "POST", body: { kind: "title", k, name: d.name ?? "", poster: d.poster ?? "", note, position: (last?.position ?? 0) + 1 }, prefer: "return=minimal" }).catch(() => {
        throw new Refusal("That's already a staff pick.");
      });
      await audit(me, "curation.title", k, { name: d.name });
      return { ok: true };
    }
    case "note":
      must(Number.isInteger(b.id));
      await db(`featured?id=eq.${b.id}`, { method: "PATCH", body: { note }, prefer: "return=minimal" });
      return { ok: true };
    case "move": {
      must(Number.isInteger(b.id) && (b.by === -1 || b.by === 1));
      const [row] = await db(`featured?id=eq.${b.id}&select=kind`);
      must(row, "That one is gone.");
      const list = await db(`featured?kind=eq.${row.kind}&select=id&order=position,created_at`);
      const i = list.findIndex((r) => r.id === b.id);
      const j = i + b.by;
      if (j < 0 || j >= list.length) return { ok: true };
      [list[i], list[j]] = [list[j], list[i]];
      await Promise.all(list.map((r, pos) => db(`featured?id=eq.${r.id}`, { method: "PATCH", body: { position: pos }, prefer: "return=minimal" })));
      return { ok: true };
    }
    case "remove": {
      must(Number.isInteger(b.id));
      const [row] = await db(`featured?id=eq.${b.id}&select=kind,user_id,k,name`);
      must(row, "That one is already gone.");
      await db(`featured?id=eq.${b.id}`, { method: "DELETE", prefer: "return=minimal" });
      await audit(me, `curation.remove`, row.user_id ?? row.k, { kind: row.kind, name: row.name });
      return { ok: true };
    }
    case "hide":
    case "unhide": {
      const uid = UUID.test(b.value ?? "") ? b.value : await resolveUser(b.value);
      if (b.op === "hide") await db("leaderboard_hidden?on_conflict=user_id", { method: "POST", body: { user_id: uid }, prefer: "resolution=ignore-duplicates,return=minimal" });
      else await db(`leaderboard_hidden?user_id=eq.${uid}`, { method: "DELETE", prefer: "return=minimal" });
      await audit(me, `leaderboard.${b.op}`, uid, {});
      return { ok: true };
    }
    default:
      throw new Refusal("Unknown action.");
  }
}

// ── Theatre tuning ────────────────────────────────────────────────────────

async function theatreKnobs() {
  const meta = hash(await redis("HGETALL", "cfg:lineup:meta"));
  return { knobs: await loadKnobs(), defaults: DEFAULT_KNOBS, genres: GENRES, saved_at: Number(meta.at) || null, custom: Boolean(await redis("EXISTS", "cfg:lineup")) };
}

async function theatreKnobsSet(b, me) {
  if (b.reset) {
    await redis("DEL", "cfg:lineup");
  } else {
    const { knobs, error } = cleanKnobs(b.knobs);
    if (error) throw new Refusal(error);
    await redis("SET", "cfg:lineup", JSON.stringify(knobs));
  }
  await redis("HSET", "cfg:lineup:meta", "at", String(Math.floor(Date.now() / 1000)), "by", me);
  await audit(me, b.reset ? "theatre.knobs.reset" : "theatre.knobs", null, b.reset ? {} : { knobs: b.knobs });
  return { ok: true };
}

// ── Theatre insight ───────────────────────────────────────────────────────

const avg = (list) => (list.length ? list.reduce((a, n) => a + n, 0) / list.length : 0);

async function theatreInsights(b) {
  const days = num(b.days, 1, 90, 30);
  const to = DAYKEY.test(b.to ?? "") ? b.to : new Date().toISOString().slice(0, 10);
  const from = new Date(Date.parse(`${to}T00:00:00Z`) - (days - 1) * 86400000).toISOString().slice(0, 10);

  const [rows, drafts, lobbyAds] = await Promise.all([
    db("rpc/admin_theatre_stats", { method: "POST", body: { from_day: from, to_day: to } }),
    db(`theatre_screenings?select=id,k,name,starts_at,proven,status&status=eq.draft&order=starts_at`),
    Promise.all(daysBack(days).map(async (day) => Number(await redis("HGET", `ads:views:${day}`, "lobby")) || 0)),
  ]);

  // Did the movie start? Reports from the apps, per screening.
  const starts = await Promise.all(rows.map(async (r) => hash(await redis("HGETALL", `th:start:${r.id}`))));
  const shows = rows.map((r, i) => {
    const s = starts[i];
    const ok = Number(s.ok) || 0;
    const failed = ["source", "player", "network", "other"].reduce((a, k) => a + (Number(s[k]) || 0), 0);
    return { ...r, start_ok: ok, start_failed: failed, start_why: { source: Number(s.source) || 0, player: Number(s.player) || 0, network: Number(s.network) || 0, other: Number(s.other) || 0 } };
  });

  // Source checks on what is still a draft, with the drop rule as it stands.
  const knobs = await loadKnobs();
  const probes = await Promise.all(
    drafts.map(async (d) => {
      const p = hash(await redis("HGETALL", `th:probe:${d.k}`));
      const found = Number(p.found) || 0;
      const miss = Number(p.miss) || 0;
      const checks = found + miss;
      const verdict = d.proven ? "proven" : checks < knobs.probeMinChecks ? "waiting" : found / checks < knobs.probeMinFound ? "drop" : "keep";
      return { ...d, found, miss, verdict };
    }),
  );

  // Best hours and genres, from screenings that already happened.
  const past = shows.filter((r) => r.status === "scheduled" && Date.parse(r.starts_at) < Date.now());
  const hourly = Array.from({ length: 24 }, () => []);
  for (const r of past) hourly[new Date(r.starts_at).getUTCHours()].push(r.came_in);
  const details = past.length ? await titleDetails([...new Set(past.map((r) => r.k))]).catch(() => ({})) : {};
  const genreMap = new Map();
  for (const r of past) {
    const g = details[r.k]?.genres?.[0];
    if (!g) continue;
    (genreMap.get(g) ?? genreMap.set(g, []).get(g)).push(r.came_in);
  }

  const sum = (f) => past.reduce((a, r) => a + f(r), 0);
  const started = shows.reduce((a, r) => a + r.start_ok, 0);
  const failedStarts = shows.reduce((a, r) => a + r.start_failed, 0);
  return {
    from,
    to,
    shows: shows.reverse(),
    probes,
    rule: { minChecks: knobs.probeMinChecks, minFound: knobs.probeMinFound },
    totals: {
      screenings: past.length,
      cancelled: shows.filter((r) => r.status === "cancelled").length,
      tickets: sum((r) => r.booked),
      came_in: sum((r) => r.came_in),
      noshow: sum((r) => r.noshow),
      max: sum((r) => r.max_tickets),
      social: sum((r) => r.social_tickets),
      started,
      failed: failedStarts,
      fail_rate: started + failedStarts ? failedStarts / (started + failedStarts) : null,
      lobby_ads: lobbyAds.reduce((a, n) => a + n, 0),
    },
    hours: hourly.map((list, h) => ({ hour: h, screenings: list.length, avg: avg(list) })),
    genres: [...genreMap].map(([genre, list]) => ({ genre, screenings: list.length, avg: avg(list) })).sort((a, b) => b.avg - a.avg),
  };
}

export const insightActions = { analytics, curation, curationSet, theatreKnobs, theatreKnobsSet, theatreInsights };
