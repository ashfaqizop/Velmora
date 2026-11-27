// The admin site's server side (admin.getvelmora.dpdns.org). One function for
// every action, because the free Vercel plan caps how many functions a site
// can have.
//
//   POST /api/admin/<action>   JSON body, Authorization: Bearer <access token>
//
// Every action except "me" needs three things: a real Supabase sign-in, a row
// in public.admins, and a sign-in that passed TOTP (the token says aal2).
// Writes use the service key, which lives only in Vercel's environment, and
// each one leaves a line in public.audit_log.
//
// Database side: supabase/admin.sql (and social.sql for the admin tables).

import { rateLimited, redis } from "../_lib/license.js";
import { totals } from "../_lib/stats.js";
import { audit, isAdminUser, must, num, PAGE, quote, Refusal, TITLE, UUID } from "../_lib/admin-kit.js";
import { theatreActions } from "../_lib/admin-theatre.js";
import { controlActions } from "../_lib/admin-control.js";
import { insightActions } from "../_lib/admin-insights.js";
import { authAdmin, db, verifyUser } from "../_lib/supabase.js";

const NAME = /^[a-z0-9_]{2,20}$/;
const WORD = /^[\p{L}\p{N}' _-]{2,40}$/u;
const REPO = "ashfaqizop/Velmora";
const ADMIN_HOSTS = new Set(["https://admin.getvelmora.dpdns.org", "http://localhost:5173"]);

// Free-tier ceilings the overview measures against.
const LIMITS = {
  dbBytes: 500 * 1024 * 1024,
  mau: 50_000,
  redisBytes: 256 * 1024 * 1024,
  emailsPerDay: 150,
};


function claims(token) {
  try {
    return JSON.parse(Buffer.from(token.split(".")[1], "base64url").toString());
  } catch {
    return {};
  }
}


// ── Overview ──────────────────────────────────────────────────────────────

async function ci() {
  try {
    const cached = await redis("GET", "adm:ci");
    if (cached) return JSON.parse(cached);
  } catch {
    // Fall through and ask GitHub.
  }
  const headers = { Accept: "application/vnd.github+json", "User-Agent": "velmora-admin" };
  const get = async (path) => {
    const r = await fetch(`https://api.github.com/repos/${REPO}/${path}`, { headers, signal: AbortSignal.timeout(8000) });
    if (!r.ok) throw new Error(`github ${r.status}`);
    return r.json();
  };
  let out;
  try {
    const [runs, release] = await Promise.all([get("actions/runs?per_page=8"), get("releases/latest").catch(() => null)]);
    out = {
      ok: true,
      runs: (runs.workflow_runs ?? []).slice(0, 5).map((r) => ({ name: r.name, branch: r.head_branch, status: r.status, conclusion: r.conclusion, at: r.updated_at, url: r.html_url })),
      release: release ? { tag: release.tag_name, at: release.published_at, url: release.html_url } : null,
    };
  } catch (e) {
    out = { ok: false, message: e.message, runs: [], release: null };
  }
  await redis("SET", "adm:ci", JSON.stringify(out), "EX", "300").catch(() => {});
  return out;
}

async function upstash() {
  try {
    const [info, keys] = await Promise.all([redis("INFO"), redis("DBSIZE")]);
    const m = /used_memory:(\d+)/.exec(String(info));
    return { ok: true, bytes: m ? Number(m[1]) : null, keys: Number(keys) };
  } catch (e) {
    return { ok: false, message: e.message };
  }
}

async function theatreToday() {
  const day = new Date().toISOString().slice(0, 10);
  try {
    const rows = await db(`theatre_screenings?select=id,name,starts_at,status&starts_at=gte.${day}T00:00:00Z&starts_at=lt.${day}T23:59:59Z&order=starts_at`);
    return { ok: true, rows };
  } catch {
    return { ok: false, rows: [] };
  }
}

async function overview() {
  const [stats, redisInfo, ciInfo, engines, versions, tonight] = await Promise.all([
    db("rpc/admin_overview", { method: "POST", body: {} }),
    upstash(),
    ci(),
    totals("eng", 7, 4).catch(() => []),
    totals("ver", 7, 6).catch(() => []),
    theatreToday(),
  ]);
  const mpv = engines.find((e) => e.key === "mpv")?.count ?? 0;
  const builtin = engines.find((e) => e.key === "builtin")?.count ?? 0;
  const plays = mpv + builtin;
  return {
    stats,
    meters: [
      { id: "db", label: "Supabase database", used: stats.db_bytes, limit: LIMITS.dbBytes, unit: "bytes" },
      { id: "mau", label: "Active people (30 days)", used: stats.active_30d, limit: LIMITS.mau, unit: "count" },
      { id: "mail", label: "Sign-ups today (each can send an email)", used: stats.signups_today, limit: LIMITS.emailsPerDay, unit: "count", note: "Mailtrap allows 150 emails a day. Sign-ups are the closest number we can see." },
      redisInfo.ok && redisInfo.bytes != null ? { id: "redis", label: "Upstash memory", used: redisInfo.bytes, limit: LIMITS.redisBytes, unit: "bytes" } : null,
    ].filter(Boolean),
    redis: redisInfo,
    ci: ciInfo,
    fallback: { mpv, builtin, rate: plays ? builtin / plays : null },
    versions,
    tonight,
  };
}

// ── Users ─────────────────────────────────────────────────────────────────

async function users(b) {
  const q = typeof b.q === "string" ? b.q.slice(0, 80) : "";
  const page = num(b.page, 0, 10_000, 0);
  const rows = await db("rpc/admin_users", { method: "POST", body: { q, lim: PAGE, off: page * PAGE } });
  return { rows, total: rows[0] ? Number(rows[0].total) : 0, page, size: PAGE };
}

async function user(b) {
  must(UUID.test(b.id ?? ""));
  const u = await db("rpc/admin_user", { method: "POST", body: { uid: b.id } });
  must(u, "No such account.");
  let max = false;
  try {
    max = Boolean(await redis("HGET", `acct:${b.id}`, "id"));
  } catch {
    // Storage hiccup: leave it unknown (false).
  }
  return { user: { ...u, max } };
}

const RESETS = {
  display_name: (p) => ({ display_name: p.username }),
  bio: () => ({ bio: "" }),
  links: () => ({ links: [] }),
  avatar: () => ({ avatar: "animal:panda" }),
  favorites: () => ({ favorites: [] }),
  username: (p) => ({ username: `user${p.id.replace(/-/g, "").slice(0, 8)}` }),
};

async function userSet(b, me) {
  must(UUID.test(b.id ?? ""));
  must(b.id !== me, "Do that to someone else. Your own account is off limits here.");
  must(!(await isAdminUser(b.id)), "That account is an admin.");
  const rows = await db(`profiles?id=eq.${b.id}&select=id,username,display_name`);
  const p = rows[0];
  const patch = async (fields) => db(`profiles?id=eq.${b.id}`, { method: "PATCH", body: fields, prefer: "return=minimal" });
  const note = typeof b.note === "string" ? b.note.slice(0, 200) : "";

  switch (b.op) {
    case "hide":
    case "unhide":
      must(p, "That account has no profile.");
      await patch({ hidden: b.op === "hide" });
      break;
    case "verify":
    case "unverify":
      must(p, "That account has no profile.");
      await patch({ verified: b.op === "verify" });
      break;
    case "ban":
    case "unban":
      if (p) await patch({ banned: b.op === "ban", ...(b.op === "ban" ? { hidden: true } : {}) });
      // Banned accounts also can't sign in again (about a hundred years).
      await authAdmin(`users/${b.id}`, { method: "PUT", body: { ban_duration: b.op === "ban" ? "876000h" : "none" } });
      break;
    case "reset": {
      must(p, "That account has no profile.");
      const fields = Array.isArray(b.fields) ? b.fields.filter((f) => f in RESETS) : [];
      must(fields.length, "Pick something to reset.");
      await patch(Object.assign({}, ...fields.map((f) => RESETS[f]({ ...p, id: b.id }))));
      await audit(me, "user.reset", b.id, { fields, username: p.username, note });
      return { ok: true };
    }
    case "delete":
      must(typeof b.confirm === "string" && b.confirm.toLowerCase() === (p?.username ?? b.id).toLowerCase(), "Type the username to confirm.");
      await authAdmin(`users/${b.id}`, { method: "DELETE" });
      break;
    default:
      throw new Refusal("Unknown action.");
  }
  await audit(me, `user.${b.op}`, b.id, { username: p?.username ?? null, note });
  return { ok: true };
}

// ── Reports ───────────────────────────────────────────────────────────────

async function reports(b) {
  const status = ["open", "dismissed", "actioned"].includes(b.status) ? b.status : "open";
  const page = num(b.page, 0, 10_000, 0);
  const who = "id,username,display_name,avatar,verified,hidden,banned";
  const rows = await db(
    `reports?status=eq.${status}&order=created_at.${status === "open" ? "asc" : "desc"}&limit=${PAGE}&offset=${page * PAGE}` +
      `&select=*,rep:profiles!reports_reporter_fkey(${who}),tgt:profiles!reports_target_fkey(${who},bio,links)`,
  );
  // How many reports each target has open, so repeat offenders stand out.
  const ids = [...new Set(rows.map((r) => r.target))];
  const counts = {};
  if (ids.length) {
    const open = await db(`reports?status=eq.open&target=in.(${ids.join(",")})&select=target`);
    for (const r of open) counts[r.target] = (counts[r.target] ?? 0) + 1;
  }
  return { rows: rows.map((r) => ({ ...r, target_open: counts[r.target] ?? 0 })), page, size: PAGE };
}

async function reportSet(b, me) {
  must(Number.isInteger(b.id) && b.id > 0);
  must(["dismissed", "actioned", "open"].includes(b.status));
  await db(`reports?id=eq.${b.id}`, {
    method: "PATCH",
    body: { status: b.status, handled_at: b.status === "open" ? null : new Date().toISOString(), handled_by: b.status === "open" ? null : me },
    prefer: "return=minimal",
  });
  await audit(me, `report.${b.status}`, String(b.id), {});
  return { ok: true };
}

// ── Reviews ───────────────────────────────────────────────────────────────

async function reviews(b) {
  const page = num(b.page, 0, 10_000, 0);
  const q = typeof b.q === "string" ? b.q.trim().slice(0, 40) : "";
  let filter = "";
  if (TITLE.test(q)) filter = `&k=eq.${q}`;
  else if (q) {
    const p = await db(`profiles?username_key=eq.${quote(q.replace(/^@/, "").toLowerCase())}&select=id`);
    filter = p[0] ? `&user_id=eq.${p[0].id}` : "&user_id=eq.00000000-0000-0000-0000-000000000000";
  }
  const rows = await db(
    `ratings?review=neq.&order=updated_at.desc&limit=${PAGE}&offset=${page * PAGE}${filter}` +
      `&select=*,p:profiles!ratings_user_id_fkey(id,username,display_name,avatar)`,
  );
  return { rows, page, size: PAGE };
}

async function reviewSet(b, me) {
  must(UUID.test(b.user_id ?? "") && TITLE.test(b.k ?? ""));
  const where = `ratings?user_id=eq.${b.user_id}&k=eq.${b.k}`;
  const [row] = await db(`${where}&select=review,score`);
  must(row, "That review is already gone.");
  if (b.op === "delete") await db(where, { method: "DELETE", prefer: "return=minimal" });
  else if (b.op === "clear") await db(where, { method: "PATCH", body: { review: "", spoiler: false }, prefer: "return=minimal" });
  else throw new Refusal("Unknown action.");
  await audit(me, `review.${b.op}`, b.user_id, { k: b.k, text: String(row.review).slice(0, 200), score: row.score });
  return { ok: true };
}

// ── Reserved names and blocked words ──────────────────────────────────────

async function names() {
  const [reserved, words] = await Promise.all([db("reserved_usernames?select=name&order=name"), db("blocked_words?select=word&order=word")]);
  return { reserved: reserved.map((r) => r.name), words: words.map((r) => r.word) };
}

async function namesSet(b, me) {
  const del = b.op === "del";
  must(b.op === "add" || del);
  if (b.list === "reserved") {
    const name = String(b.value ?? "").trim().toLowerCase();
    must(NAME.test(name), "Names are 2 to 20 letters, numbers or underscores.");
    if (del) await db(`reserved_usernames?name=eq.${quote(name)}`, { method: "DELETE", prefer: "return=minimal" });
    else await db("reserved_usernames?on_conflict=name", { method: "POST", body: { name }, prefer: "resolution=ignore-duplicates,return=minimal" });
    await audit(me, `reserved.${b.op}`, name);
    // Someone may already hold a name that was just reserved; say so.
    const holder = del ? [] : await db(`profiles?username_key=eq.${quote(name)}&select=id,username`);
    return { ok: true, holder: holder[0] ?? null };
  }
  if (b.list === "words") {
    const word = String(b.value ?? "").trim().toLowerCase();
    must(WORD.test(word), "A blocked word is 2 to 40 letters or numbers.");
    if (del) await db(`blocked_words?word=eq.${quote(word)}`, { method: "DELETE", prefer: "return=minimal" });
    else await db("blocked_words?on_conflict=word", { method: "POST", body: { word }, prefer: "resolution=ignore-duplicates,return=minimal" });
    await audit(me, `word.${b.op}`, word);
    return { ok: true };
  }
  throw new Refusal("Unknown list.");
}

// ── Audit log ─────────────────────────────────────────────────────────────

async function auditList(b) {
  const page = num(b.page, 0, 10_000, 0);
  const rows = await db(`audit_log?order=id.desc&limit=${PAGE}&offset=${page * PAGE}&select=*`);
  // Show who a person is, not just an id: usernames for the ids that appear.
  const ids = [...new Set(rows.flatMap((r) => [r.admin_id, UUID.test(r.target ?? "") ? r.target : null]).filter(Boolean))];
  const who = {};
  if (ids.length) for (const p of await db(`profiles?id=in.(${ids.join(",")})&select=id,username`)) who[p.id] = p.username;
  return { rows: rows.map((r) => ({ ...r, admin_name: who[r.admin_id] ?? null, target_name: who[r.target] ?? null })), page, size: PAGE };
}

const ACTIONS = { ...theatreActions, ...controlActions, ...insightActions, overview, users, user, userSet, reports, reportSet, reviews, reviewSet, names, namesSet, audit: auditList };

// ── Request ───────────────────────────────────────────────────────────────

export default async function handler(req, res) {
  res.setHeader("Cache-Control", "no-store");
  const send = (status, body) => res.status(status).json(body);
  try {
    if (req.method !== "POST") return send(405, { ok: false, message: "POST only." });
    const origin = req.headers.origin;
    if (origin && !ADMIN_HOSTS.has(origin)) return send(403, { ok: false, message: "Not from here." });
    if (await rateLimited(req, 120)) return send(429, { ok: false, message: "Slow down a little." });

    const token = String(req.headers.authorization ?? "").replace(/^Bearer\s+/i, "");
    const user = await verifyUser(token);
    if (!user) return send(401, { ok: false, message: "Sign in again." });
    const admin = await isAdminUser(user.id);
    const aal = claims(token).aal === "aal2" ? 2 : 1;

    const action = String(req.query.action ?? "");
    // "me" tells the page what to show next and nothing more.
    if (action === "me") return send(200, { ok: true, admin, aal });
    if (!admin) return send(403, { ok: false, message: "This account isn't an admin." });
    if (aal < 2) return send(403, { ok: false, code: "mfa", message: "Confirm your authenticator code first." });

    const fn = Object.hasOwn(ACTIONS, action) ? ACTIONS[action] : null;
    if (!fn) return send(404, { ok: false, message: "Unknown action." });
    const body = typeof req.body === "string" ? JSON.parse(req.body || "{}") : (req.body ?? {});
    return send(200, { ok: true, ...(await fn(body ?? {}, user.id)) });
  } catch (e) {
    if (e instanceof Refusal) return send(e.status, { ok: false, message: e.message });
    console.error("admin:", e.message);
    return send(500, { ok: false, message: e.message?.slice(0, 200) || "Something broke." });
  }
}
