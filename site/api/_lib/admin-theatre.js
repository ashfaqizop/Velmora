// The admin site's Theatre actions (docs/admin.md, phase 2). Every function
// takes (body, adminId) and returns the fields to send back. Writes use the
// service key; each one leaves a line in the audit log.

import { audit, DAYKEY, must, num, quote, Refusal, resolveUser, UUID } from "./admin-kit.js";
import { DAY, draft, finalize, GAP, HOUR, isoDay, MOVIE } from "./lineup.js";
import { loadKnobs } from "./lineup-knobs.js";
import { db } from "./supabase.js";
import { titleDetails, tmdbGet } from "./tmdb.js";

const COLS =
  "id,day,k,name,year,poster,runtime,kind,starts_at,ends_at,status,mode,capacity,expected,booked,waitlisted,proven,opens_at,max_opens_at,special,note,chat_mode,released,settled";
const WHO = "id,username,display_name,avatar";
const iso = (t) => new Date(t).toISOString();
const time = (v) => {
  const t = Date.parse(v);
  must(Number.isFinite(t), "That date and time doesn't look right.");
  return t;
};

// ── The lineup ────────────────────────────────────────────────────────────

async function theatreLineup(b) {
  const back = num(b.back, 0, 14, 1);
  const ahead = num(b.ahead, 1, 21, 7);
  const from = isoDay(Date.now() - back * DAY);
  const to = isoDay(Date.now() + (ahead + 1) * DAY);
  const [rows, [settings]] = await Promise.all([
    db(`theatre_screenings?select=${COLS}&starts_at=gte.${from}T00:00:00Z&starts_at=lt.${to}T00:00:00Z&order=starts_at`),
    db("theatre_settings?select=paused,banner,reminder"),
  ]);
  return { rows, settings: settings ?? { paused: false, banner: "", reminder: "" } };
}

/** Screenings that would run at the same time as [start, end), and what that means for the room. */
async function clashes(start, end, exclude) {
  const lo = iso(start - GAP);
  const hi = iso(end + GAP);
  const rows = await db(`theatre_screenings?select=id,name,starts_at,ends_at,capacity,status&status=neq.cancelled&starts_at=lt.${hi}&ends_at=gt.${lo}&order=starts_at`);
  const others = rows.filter((r) => r.id !== exclude);
  const warnings = [];
  if (others.length >= 3) warnings.push(`${others.length} other screenings run at the same time.`);
  return { others, warnings };
}

async function theatreCheck(b) {
  const start = time(b.starts_at);
  let runtime = num(b.runtime, 20, 600, null);
  let title = null;
  if (b.k !== undefined && b.k !== null && b.k !== "") {
    must(MOVIE.test(String(b.k)), "Pick a movie.");
    const d = (await titleDetails([b.k]))[b.k];
    must(d, "TMDB doesn't have that title.");
    title = { name: d.name, year: d.year, poster: d.poster, runtime: d.runtime };
    runtime = runtime ?? d.runtime;
  }
  must(runtime, "TMDB has no runtime for this one. Enter it yourself (20 to 600 minutes).");
  const end = start + runtime * 60_000;
  const { others, warnings } = await clashes(start, end, UUID.test(b.exclude ?? "") ? b.exclude : null);
  const seats = others.reduce((a, r) => a + r.capacity, 0) + num(b.capacity, 1, 1000, 60);
  const { budget } = await loadKnobs();
  if (seats > budget) warnings.push(`Together these could seat ${seats} people at once. Realtime is sized for about ${budget}.`);
  if (start < Date.now() + 10 * 60_000) warnings.push("This starts in under 10 minutes, so few people will see it in time.");
  return { runtime, ends_at: iso(end), title, overlaps: others, warnings };
}

async function theatreSearch(b) {
  const q = String(b.q ?? "").trim().slice(0, 80);
  must(q.length >= 2, "Type at least two letters.");
  // Staff picks can be shows too; Theatre screenings are movies only.
  const r = await tmdbGet(b.shows ? "/search/multi" : "/search/movie", { query: q, include_adult: "false" });
  const list = (r.results ?? []).filter((m) => !b.shows || m.media_type === "movie" || m.media_type === "tv");
  return {
    results: list.slice(0, 8).map((m) => ({
      k: `${m.media_type === "tv" ? "tv" : "movie"}/${m.id}`,
      name: m.title ?? m.name,
      year: (m.release_date ?? m.first_air_date) ? Number((m.release_date ?? m.first_air_date).slice(0, 4)) || null : null,
      poster: m.poster_path ? `https://image.tmdb.org/t/p/w185${m.poster_path}` : null,
      overview: String(m.overview ?? "").slice(0, 160),
    })),
  };
}

/** Booking opens for Max now and for everyone either now or a while later, but never after the start. */
function opening(booking, now, start) {
  const gap = Math.min(6 * HOUR, Math.max(0, (start - now) / 2));
  return { max_opens_at: iso(now), opens_at: iso(booking === "now" ? now : now + gap) };
}

async function theatreSave(b, me) {
  const mode = b.mode === "fixed" ? "fixed" : "open";
  const capacity = num(b.capacity, 1, 1000, 60);
  const note = String(b.note ?? "").trim().slice(0, 200);
  const status = b.status === "scheduled" ? "scheduled" : "draft";
  const start = time(b.starts_at);
  const now = Date.now();

  if (b.id) {
    must(UUID.test(b.id));
    const [row] = await db(`theatre_screenings?id=eq.${b.id}&select=${COLS}`);
    must(row, "That screening is gone.");
    must(row.status !== "cancelled", "A cancelled screening can't be edited.");
    must(Date.parse(row.ends_at) > now || row.status === "draft", "That screening is over.");
    const runtime = num(b.runtime, 20, 600, row.runtime);
    if (status === "draft" && row.status === "scheduled") must(row.booked + row.waitlisted === 0, "People have tickets for this. Cancel it instead.");
    const patch = { starts_at: iso(start), ends_at: iso(start + runtime * 60_000), day: isoDay(start), runtime, mode, capacity, note };
    if (row.status === "draft" && status === "scheduled") Object.assign(patch, { status: "scheduled", ...opening(b.booking, now, start) });
    else if (row.status === "scheduled") patch.status = "scheduled";
    await db(`theatre_screenings?id=eq.${b.id}`, { method: "PATCH", body: patch, prefer: "return=minimal" });
    const moved = patch.starts_at !== row.starts_at;
    await audit(me, "theatre.edit", b.id, { name: row.name, moved, starts_at: patch.starts_at, mode, capacity, note });
    const { warnings } = await clashes(start, start + runtime * 60_000, b.id);
    return { id: b.id, warnings: moved && row.booked ? [`${row.booked} people hold tickets. They'll see the new time next time they open Theatre.`, ...warnings] : warnings };
  }

  must(MOVIE.test(String(b.k ?? "")), "Pick a movie.");
  must(start >= now + 5 * 60_000, "Start at least five minutes from now.");
  const d = (await titleDetails([b.k]))[b.k];
  must(d, "TMDB doesn't have that title.");
  const runtime = num(b.runtime, 20, 600, d.runtime ?? null);
  must(runtime, "TMDB has no runtime for this one. Enter it yourself (20 to 600 minutes).");
  const row = {
    day: isoDay(start),
    k: b.k,
    name: d.name,
    year: d.year,
    poster: d.poster,
    backdrop: d.backdrop,
    runtime,
    kind: "pick",
    starts_at: iso(start),
    ends_at: iso(start + runtime * 60_000),
    status,
    mode,
    capacity,
    expected: 0,
    // You chose it, so it skips the source check.
    proven: true,
    special: true,
    created_by: me,
    note,
    ...(status === "scheduled" ? opening(b.booking, now, start) : {}),
  };
  const [made] = await db("theatre_screenings?select=id", { method: "POST", body: row, prefer: "return=representation" });
  await audit(me, "theatre.add", made.id, { name: d.name, starts_at: row.starts_at, status, mode, capacity });
  const { warnings } = await clashes(start, start + runtime * 60_000, made.id);
  return { id: made.id, warnings };
}

async function theatreCancel(b, me) {
  must(UUID.test(b.id ?? ""));
  const [row] = await db(`theatre_screenings?id=eq.${b.id}&select=name,status,booked,waitlisted`);
  must(row, "That screening is gone.");
  must(row.status !== "cancelled", "It's already cancelled.");
  const why = String(b.why ?? "").trim().slice(0, 200);
  await db("rpc/admin_theatre_cancel", { method: "POST", body: { sid: b.id, why } });
  await audit(me, "theatre.cancel", b.id, { name: row.name, booked: row.booked, waitlisted: row.waitlisted, why });
  return { people: row.booked + row.waitlisted };
}

// ── Running the algorithm by hand ─────────────────────────────────────────

async function theatreRun(b, me) {
  must(DAYKEY.test(b.day ?? ""), "Pick a day.");
  const today = isoDay(Date.now());
  must(b.day >= today && b.day <= isoDay(Date.now() + 3 * DAY), "Pick today or one of the next three days.");
  const earliest = Date.now() + 45 * 60_000;
  const isToday = b.day === today;

  if (b.op === "preview") return { ...(await draft(b.day, earliest, { force: true, dry: true })) };

  if (b.op === "run") {
    const made = await draft(b.day, earliest);
    const out = made.drafted ? await finalize(b.day, { now: isToday }) : made;
    await audit(me, "theatre.run", b.day, { drafted: made.drafted });
    return { ...out, drafted: made.drafted };
  }

  must(b.op === "regenerate", "Unknown action.");
  const old = await db(`theatre_screenings?select=id,name,starts_at,booked,waitlisted&day=eq.${b.day}&special=eq.false&status=in.(draft,scheduled)`);
  const stuck = old.filter((r) => r.booked + r.waitlisted > 0 || Date.parse(r.starts_at) < Date.now() + 5 * 60_000);
  must(!stuck.length, `Can't redo this day: ${stuck.map((r) => r.name).slice(0, 3).join(", ")}${stuck.length > 3 ? " and more" : ""} already has tickets or is about to start. Cancel those by hand first.`);
  if (old.length) await db(`theatre_screenings?id=in.(${old.map((r) => r.id).join(",")})`, { method: "DELETE", prefer: "return=minimal" });
  const made = await draft(b.day, earliest);
  const out = made.drafted ? await finalize(b.day, { now: isToday }) : made;
  await audit(me, "theatre.regenerate", b.day, { removed: old.length, drafted: made.drafted });
  return { ...out, removed: old.length, drafted: made.drafted };
}

// ── Pins and blocks ───────────────────────────────────────────────────────

async function theatreOverrides() {
  const rows = await db("theatre_overrides?select=*&order=id.desc&limit=200");
  const names = rows.length ? await titleDetails([...new Set(rows.map((r) => r.k))]) : {};
  return { rows: rows.map((r) => ({ ...r, name: names[r.k]?.name ?? null, year: names[r.k]?.year ?? null })) };
}

async function theatreOverrideSet(b, me) {
  if (b.op === "del") {
    must(Number.isInteger(b.id) && b.id > 0);
    await db(`theatre_overrides?id=eq.${b.id}`, { method: "DELETE", prefer: "return=minimal" });
    await audit(me, "theatre.override.del", String(b.id));
    return {};
  }
  must(b.op === "add");
  must(b.action === "pin" || b.action === "block", "Pin or block?");
  must(MOVIE.test(String(b.k ?? "")), "Pick a movie.");
  const d = (await titleDetails([b.k]))[b.k];
  must(d, "TMDB doesn't have that title.");
  let day = null;
  if (b.action === "pin") {
    must(DAYKEY.test(b.day ?? "") && b.day >= isoDay(Date.now()), "Pick a day that hasn't passed.");
    day = b.day;
  }
  const note = String(b.note ?? "").trim().slice(0, 200);
  await db("theatre_overrides", { method: "POST", body: { action: b.action, k: b.k, day, note }, prefer: "return=minimal" });
  await audit(me, `theatre.${b.action}`, b.k, { name: d.name, day, note });
  return { name: d.name };
}

// ── Tickets ───────────────────────────────────────────────────────────────

async function theatreTickets(b) {
  const id = await resolveUser(b.user);
  const [[profile], tickets, noshows] = await Promise.all([
    db(`profiles?id=eq.${id}&select=${WHO}`),
    db(`theatre_tickets?user_id=eq.${id}&order=created_at.desc&limit=40&select=screening_id,status,is_max,created_at,joined_at,s:theatre_screenings(name,starts_at,ends_at,status)`),
    db(`theatre_tickets?user_id=eq.${id}&status=eq.noshow&created_at=gt.${iso(Date.now() - 30 * DAY)}&select=screening_id`),
  ]);
  must(profile, "That account has no profile.");
  return { profile, tickets, noshows: noshows.length };
}

async function theatreTicketSet(b, me) {
  must(UUID.test(b.sid ?? ""));
  const uid = await resolveUser(b.user);
  const [s] = await db(`theatre_screenings?id=eq.${b.sid}&select=name,ends_at`);
  must(s, "That screening is gone.");
  const where = `theatre_tickets?screening_id=eq.${b.sid}&user_id=eq.${uid}`;
  switch (b.op) {
    case "grant":
      await db("rpc/admin_theatre_grant", { method: "POST", body: { sid: b.sid, uid } });
      break;
    case "revoke":
      await db("rpc/admin_theatre_revoke", { method: "POST", body: { sid: b.sid, uid } });
      break;
    case "noshow":
    case "attended":
    case "clear": {
      must(Date.parse(s.ends_at) < Date.now(), "Only after the screening is over.");
      const [t] = await db(`${where}&select=status`);
      must(t, "They have no ticket for that screening.");
      if (b.op === "clear") must(t.status === "noshow", "That ticket isn't a no-show.");
      await db(where, { method: "PATCH", body: { status: b.op === "noshow" ? "noshow" : b.op === "attended" ? "attended" : "cancelled" }, prefer: "return=minimal" });
      break;
    }
    default:
      throw new Refusal("Unknown action.");
  }
  await audit(me, `theatre.ticket.${b.op}`, uid, { screening: s.name, sid: b.sid });
  return {};
}

// ── The live room ─────────────────────────────────────────────────────────

async function theatreLive() {
  const now = Date.now();
  const rows = await db(
    `theatre_screenings?select=${COLS}&status=eq.scheduled&starts_at=lt.${iso(now + 3 * HOUR)}&ends_at=gt.${iso(now - 30 * 60_000)}&order=starts_at`,
  );
  const entered = {};
  if (rows.length) {
    const joined = await db(`theatre_tickets?screening_id=in.(${rows.map((r) => r.id).join(",")})&joined_at=not.is.null&select=screening_id`);
    for (const t of joined) entered[t.screening_id] = (entered[t.screening_id] ?? 0) + 1;
  }
  return { rows: rows.map((r) => ({ ...r, entered: entered[r.id] ?? 0 })), now: iso(now) };
}

async function theatreRoom(b) {
  must(UUID.test(b.sid ?? ""));
  const [[screening], people, bans] = await Promise.all([
    db(`theatre_screenings?id=eq.${b.sid}&select=${COLS}`),
    db(
      `theatre_tickets?screening_id=eq.${b.sid}&status=in.(booked,attended,released)&order=joined_at.desc.nullslast&limit=300` +
        `&select=user_id,status,joined_at,p:profiles!theatre_tickets_user_id_fkey(${WHO})`,
    ),
    db(`theatre_room_bans?screening_id=eq.${b.sid}&select=user_id,kind`),
  ]);
  must(screening, "That screening is gone.");
  const ban = new Map(bans.map((x) => [x.user_id, x.kind]));
  const known = new Set(people.map((p) => p.user_id));
  // Someone banned who has no ticket (an open screening) still needs a row to undo.
  const extra = bans.filter((x) => !known.has(x.user_id)).map((x) => x.user_id);
  const strays = extra.length ? await db(`profiles?id=in.(${extra.join(",")})&select=${WHO}`) : [];
  return {
    screening,
    people: [
      ...people.map((p) => ({ ...p, ban: ban.get(p.user_id) ?? null })),
      ...strays.map((p) => ({ user_id: p.id, status: "removed", joined_at: null, p, ban: ban.get(p.id) ?? null })),
    ],
    entered: people.filter((p) => p.joined_at).length,
  };
}

async function theatreChat(b, me) {
  must(UUID.test(b.sid ?? ""));
  must(["normal", "slow", "closed"].includes(b.mode), "Pick a chat mode.");
  await db("rpc/admin_theatre_chat", { method: "POST", body: { sid: b.sid, mode: b.mode } });
  await audit(me, "theatre.chat", b.sid, { mode: b.mode });
  return {};
}

async function theatreMember(b, me) {
  must(UUID.test(b.sid ?? ""));
  must(["mute", "remove", "clear"].includes(b.what), "Unknown action.");
  const uid = await resolveUser(b.user);
  await db("rpc/admin_theatre_room", { method: "POST", body: { sid: b.sid, uid, what: b.what } });
  await audit(me, `theatre.room.${b.what}`, uid, { sid: b.sid });
  return {};
}

// ── Switches and words ────────────────────────────────────────────────────

async function theatreSettings() {
  const [settings] = await db("theatre_settings?select=paused,banner,reminder,updated_at");
  return { settings: settings ?? { paused: false, banner: "", reminder: "" } };
}

async function theatreSettingsSet(b, me) {
  const patch = { updated_at: new Date().toISOString() };
  if (typeof b.paused === "boolean") patch.paused = b.paused;
  if (typeof b.banner === "string") patch.banner = b.banner.trim().slice(0, 200);
  if (typeof b.reminder === "string") {
    patch.reminder = b.reminder.trim().slice(0, 160);
    must(!patch.reminder || patch.reminder.includes("{title}"), "Use {title} where the film's name should go.");
  }
  await db("theatre_settings?id=eq.true", { method: "PATCH", body: patch, prefer: "return=minimal" });
  const { updated_at, ...shown } = patch;
  void updated_at;
  await audit(me, "theatre.settings", null, shown);
  return {};
}

export const theatreActions = {
  theatreLineup,
  theatreCheck,
  theatreSearch,
  theatreSave,
  theatreCancel,
  theatreRun,
  theatreOverrides,
  theatreOverrideSet,
  theatreTickets,
  theatreTicketSet,
  theatreLive,
  theatreRoom,
  theatreChat,
  theatreMember,
  theatreSettings,
  theatreSettingsSet,
};
