// Admin phase 3a: Max licences, announcements, the app's ad rules and the
// remote config. Wired into api/admin/[action].js, so every action here has
// already passed the sign-in, admin row and TOTP checks.
//
// Environment (Vercel only, never in the repo):
//   LICENSE_SIGNING_KEY   the Max signing key (from `velmora-license secret signing`)
//   CONFIG_GITHUB_TOKEN   fine-grained token, Contents read/write on this repo;
//                         this code only ever writes registry.json on `config`

import crypto from "node:crypto";
import { cleanRules, DEFAULT_RULES, SLOTS } from "./ads-rules.js";
import { audit, must, Refusal, resolveUser, UUID } from "./admin-kit.js";
import { b64url, hash, parseKey, redis } from "./license.js";
import { db, setMax } from "./supabase.js";

const LIC_ID = /^MX-[0-9A-Z]{10}$/;
const now = () => Math.floor(Date.now() / 1000);

// ── Max licences ──────────────────────────────────────────────────────────
// A key is VLM1.<payload>.<signature> over { v, id, n, t, iat }. Signing is
// deterministic, so id + name + iat in Redis is enough to show a key again.

let signer;
function signingKey() {
  const raw = process.env.LICENSE_SIGNING_KEY;
  if (!raw) throw new Refusal("LICENSE_SIGNING_KEY isn't set in Vercel yet, so keys can't be made here.", 503);
  signer ??= crypto.createPrivateKey({ key: b64url.decode(raw.trim()), format: "der", type: "pkcs8" });
  return signer;
}

/** Same bytes as tools/license, so keys made there and here are identical. */
function makeKey(rec) {
  const payload = Buffer.from(JSON.stringify({ v: 1, id: rec.id, n: rec.name, t: "max", iat: rec.iat }));
  const key = `VLM1.${b64url.encode(payload)}.${b64url.encode(crypto.sign(null, payload, signingKey()))}`;
  // A signing key that doesn't match the public key in the app would hand out
  // keys that never work. Check every time; it's cheap.
  if (!parseKey(key)) throw new Refusal("The signing key in Vercel doesn't match the public key in the app. Copy it again from the licence tool.", 503);
  return key;
}

function newId() {
  const ALPHA = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
  return "MX-" + [...crypto.randomBytes(10)].map((b) => ALPHA[b % 32]).join("");
}

const licId = (v) => {
  const id = String(v ?? "").trim().toUpperCase();
  must(LIC_ID.test(id), "That isn't a licence id like MX-ABCDE12345.");
  return id;
};
const cleanName = (v) => {
  const name = String(v ?? "").trim();
  must(name.length >= 1 && name.length <= 40, "Give a name of 1 to 40 characters.");
  return name;
};

async function usernames(ids) {
  const list = [...new Set(ids.filter((x) => UUID.test(x ?? "")))];
  const out = {};
  if (list.length) for (const p of await db(`profiles?id=in.(${list.join(",")})&select=id,username`)) out[p.id] = p.username;
  return out;
}

/** Put a key on an account. An account holds one key, so an old one comes off. */
async function linkAccount(uid, id, key) {
  const before = await redis("HGET", `acct:${uid}`, "id");
  if (before && before !== id) await redis("HDEL", `lic:${before}`, "owner");
  const owner = await redis("HGET", `lic:${id}`, "owner");
  if (owner && owner !== uid && (await redis("HGET", `acct:${owner}`, "id")) === id) {
    await redis("DEL", `acct:${owner}`);
    await setMax(owner, false);
  }
  await redis("HSET", `lic:${id}`, "owner", uid);
  await redis("HSET", `acct:${uid}`, "id", id, "key", key, "linked", String(now()));
  await setMax(uid, true);
}

async function maxList() {
  const [ids, revoked] = await Promise.all([redis("SMEMBERS", "lic:ids"), redis("SMEMBERS", "lic:revoked")]);
  const off = new Set(revoked ?? []);
  const rows = await Promise.all(
    (ids ?? []).map(async (id) => {
      const r = hash(await redis("HGETALL", `lic:${id}`));
      return {
        id,
        name: r.name ?? "",
        note: r.note ?? "",
        issued: Number(r.issued || r.iat) || null,
        activated: Number(r.activated) || null,
        seen: Number(r.seen) || null,
        moved: Number(r.moved) || null,
        active: Boolean(r.device),
        owner: r.owner ?? null,
        revoked: off.has(id),
        showable: Boolean(r.iat),
      };
    }),
  );
  const who = await usernames(rows.map((r) => r.owner));
  for (const r of rows) r.owner_name = who[r.owner] ?? null;
  rows.sort((a, b) => (b.issued ?? 0) - (a.issued ?? 0) || a.id.localeCompare(b.id));
  return { rows, signing: Boolean(process.env.LICENSE_SIGNING_KEY) };
}

async function maxIssue(b, me) {
  const name = cleanName(b.name);
  const note = String(b.note ?? "").trim().slice(0, 200);
  const uid = b.account ? await resolveUser(b.account) : null;
  const rec = { id: newId(), name, iat: now() };
  const key = makeKey(rec);
  await redis("HSET", `lic:${rec.id}`, "name", name, "iat", String(rec.iat), "issued", String(rec.iat), "note", note, "by", me);
  await redis("SADD", "lic:ids", rec.id);
  if (uid) await linkAccount(uid, rec.id, key);
  await audit(me, "max.issue", rec.id, { name, account: uid });
  return { id: rec.id, key };
}

async function maxShow(b, me) {
  const id = licId(b.id);
  const r = hash(await redis("HGETALL", `lic:${id}`));
  must(r.iat, "This key was made before the admin site knew about keys. Import your issued.json to show it here.");
  const key = makeKey({ id, name: r.name ?? "", iat: Number(r.iat) });
  await audit(me, "max.show", id, {});
  return { key };
}

async function maxSet(b, me) {
  const id = licId(b.id);
  const k = `lic:${id}`;
  const owner = await redis("HGET", k, "owner");
  switch (b.op) {
    case "release":
      await redis("HDEL", k, "device");
      break;
    case "revoke":
      await redis("SADD", "lic:revoked", id);
      await redis("HDEL", k, "device");
      if (owner) await setMax(owner, false);
      break;
    case "unrevoke":
      await redis("SREM", "lic:revoked", id);
      if (owner) await setMax(owner, true);
      break;
    case "unlink":
      if (owner) {
        await redis("HDEL", k, "owner");
        if ((await redis("HGET", `acct:${owner}`, "id")) === id) await redis("DEL", `acct:${owner}`);
        await setMax(owner, false);
      }
      break;
    case "note":
      await redis("HSET", k, "note", String(b.note ?? "").trim().slice(0, 200));
      break;
    case "give": {
      const uid = await resolveUser(b.account);
      must(!(await redis("SISMEMBER", "lic:revoked", id)), "Turn the key back on first.");
      const r = hash(await redis("HGETALL", k));
      must(r.iat, "Import your issued.json first, so the key can be rebuilt.");
      await linkAccount(uid, id, makeKey({ id, name: r.name ?? "", iat: Number(r.iat) }));
      await audit(me, "max.give", id, { account: uid });
      return { ok: true };
    }
    default:
      throw new Refusal("Unknown action.");
  }
  await audit(me, `max.${b.op}`, id, { owner });
  return { ok: true };
}

/** The local tool's issued.json: [{ id, name, iat }]. Only records whose key checks out. */
async function maxImport(b, me) {
  must(Array.isArray(b.records) && b.records.length >= 1 && b.records.length <= 500, "That file has no keys in it.");
  let added = 0;
  let known = 0;
  for (const raw of b.records) {
    const id = String(raw?.id ?? "").toUpperCase();
    const name = String(raw?.name ?? "");
    const iat = raw?.iat;
    if (!LIC_ID.test(id) || name.length > 40 || !Number.isInteger(iat)) throw new Refusal(`One record isn't a key record: ${id || "no id"}.`);
    makeKey({ id, name, iat }); // throws if the signing key doesn't match
    const k = `lic:${id}`;
    if (await redis("HGET", k, "iat")) {
      known++;
      continue;
    }
    await redis("HSET", k, "name", name, "iat", String(iat));
    await redis("HSETNX", k, "issued", String(iat));
    await redis("SADD", "lic:ids", id);
    added++;
  }
  await audit(me, "max.import", null, { added, known });
  return { added, known };
}

// ── Announcements ─────────────────────────────────────────────────────────

const KINDS = ["info", "warning", "maintenance"];
const AUDIENCES = ["everyone", "public", "max"];
const PLACES = ["app", "web", "both"];

async function notices() {
  const rows = await db("announcements?order=created_at.desc&limit=100&select=*");
  return { rows, now: new Date().toISOString() };
}

const isoOrNull = (v) => {
  if (v == null || v === "") return null;
  const t = new Date(String(v));
  must(!Number.isNaN(t.getTime()), "That date doesn't look right.");
  return t.toISOString();
};

async function noticeSave(b, me) {
  const title = String(b.title ?? "").trim();
  const body = String(b.body ?? "").trim();
  const link_url = String(b.link_url ?? "").trim();
  const link_label = String(b.link_label ?? "").trim();
  must(title.length >= 1 && title.length <= 80, "The title is 1 to 80 characters.");
  must(body.length <= 400, "Keep the text under 400 characters.");
  must(link_url === "" || /^https:\/\/\S{3,300}$/.test(link_url), "Links must start with https://.");
  must(link_label.length <= 30, "Keep the button label under 30 characters.");
  must(KINDS.includes(b.kind) && AUDIENCES.includes(b.audience) && PLACES.includes(b.place));
  const starts_at = isoOrNull(b.starts_at) ?? new Date().toISOString();
  const ends_at = isoOrNull(b.ends_at);
  must(!ends_at || ends_at > starts_at, "It has to end after it starts.");
  const row = { kind: b.kind, title, body, link_url, link_label, audience: b.audience, place: b.place, active: b.active !== false, starts_at, ends_at, updated_at: new Date().toISOString() };

  if (b.id != null) {
    must(Number.isInteger(b.id) && b.id > 0);
    await db(`announcements?id=eq.${b.id}`, { method: "PATCH", body: row, prefer: "return=minimal" });
    await audit(me, "notice.edit", String(b.id), { title, kind: b.kind, active: row.active });
    return { id: b.id };
  }
  const [made] = await db("announcements", { method: "POST", body: { ...row, created_by: me }, prefer: "return=representation" });
  await audit(me, "notice.create", String(made.id), { title, kind: b.kind });
  return { id: made.id };
}

async function noticeDelete(b, me) {
  must(Number.isInteger(b.id) && b.id > 0);
  const [row] = await db(`announcements?id=eq.${b.id}&select=title`);
  must(row, "That one is already gone.");
  await db(`announcements?id=eq.${b.id}`, { method: "DELETE", prefer: "return=minimal" });
  await audit(me, "notice.delete", String(b.id), { title: row.title });
  return { ok: true };
}

// ── The app's ad rules ────────────────────────────────────────────────────

async function adViews(days) {
  const out = [];
  const totals = Object.fromEntries(SLOTS.map((s) => [s, 0]));
  for (let i = days - 1; i >= 0; i--) {
    const day = new Date(Date.now() - i * 86400000).toISOString().slice(0, 10);
    const h = hash(await redis("HGETALL", `ads:views:${day}`));
    let sum = 0;
    for (const [slot, n] of Object.entries(h)) {
      sum += Number(n) || 0;
      if (slot in totals) totals[slot] += Number(n) || 0;
    }
    out.push({ day, views: sum });
  }
  return { days: out, slots: totals };
}

async function ads() {
  const saved = await redis("GET", "cfg:ads");
  const meta = hash(await redis("HGETALL", "cfg:ads:meta"));
  return {
    rules: saved ? JSON.parse(saved) : DEFAULT_RULES,
    custom: Boolean(saved),
    defaults: DEFAULT_RULES,
    saved_at: Number(meta.at) || null,
    views: await adViews(14),
  };
}

async function adsSave(b, me) {
  if (b.reset) {
    await redis("DEL", "cfg:ads");
    await redis("HSET", "cfg:ads:meta", "at", String(now()), "by", me);
    await audit(me, "ads.reset", null, {});
    return { ok: true };
  }
  const { rules, error } = cleanRules(b.rules);
  if (error) throw new Refusal(error);
  await redis("SET", "cfg:ads", JSON.stringify(rules));
  await redis("HSET", "cfg:ads:meta", "at", String(now()), "by", me);
  await audit(me, "ads.save", null, { enabled: rules.enabled, off: SLOTS.filter((s) => !rules.slots[s].enabled) });
  return { ok: true };
}

// ── Remote config (registry.json on the config branch) ────────────────────

const REPO = "ashfaqizop/Velmora";
const BRANCH = "config";
const FILE = "registry.json";
const PROVIDERS = new Set([
  "cinemeta", "tmdb", "anilist", "hd_hub", "hi_anime", "zoko_anime", "movie_box", "four_k_hd_hub", "dramachi",
  "anime_sama", "french_stream", "coflix", "yts", "eztv", "nyaa", "iptv_org", "youtube_music",
]);
const SECRET_FIELDS = ["tmdb_api_key", "tmdb_read_token", "thesportsdb_key", "football_data_token"];

async function gh(path, { method = "GET", body, write = false } = {}) {
  const token = process.env.CONFIG_GITHUB_TOKEN;
  if (write && !token) throw new Refusal("CONFIG_GITHUB_TOKEN isn't set in Vercel yet, so the config can be read but not saved.", 503);
  const headers = { Accept: "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28", "User-Agent": "velmora-admin" };
  if (token) headers.Authorization = `Bearer ${token}`;
  if (body) headers["Content-Type"] = "application/json";
  const r = await fetch(`https://api.github.com/repos/${REPO}/${path}`, { method, headers, body: body ? JSON.stringify(body) : undefined, signal: AbortSignal.timeout(10000) });
  const j = await r.json().catch(() => ({}));
  if (r.status === 409 || r.status === 422) throw new Refusal("The config changed since you opened it. Reload and make your edit again.", 409);
  if (!r.ok) throw new Refusal(`GitHub said ${r.status}${j.message ? `: ${j.message}` : ""}.`, 502);
  return j;
}

/** Problems that would make the engine reject or misread the file. */
function checkRegistry(reg) {
  const bad = [];
  if (!reg || typeof reg !== "object" || Array.isArray(reg)) return ["The file must be a JSON object."];
  if (reg.version !== 1) bad.push("version must be 1.");
  for (const f of SECRET_FIELDS) if (f in reg) bad.push(`${f} is a secret and must never go in this public file.`);
  if (reg.mirrors !== undefined) {
    if (typeof reg.mirrors !== "object" || Array.isArray(reg.mirrors)) bad.push("mirrors must be an object.");
    else
      for (const [p, urls] of Object.entries(reg.mirrors)) {
        if (!PROVIDERS.has(p)) bad.push(`mirrors: "${p}" isn't a provider the engine knows.`);
        if (!Array.isArray(urls) || urls.some((u) => typeof u !== "string" || !/^https?:\/\/[^\s/]+[^\s]*$/.test(u))) bad.push(`mirrors.${p} must be a list of http(s) URLs.`);
      }
  }
  if (reg.disabled !== undefined) {
    if (!Array.isArray(reg.disabled)) bad.push("disabled must be a list.");
    else for (const p of reg.disabled) if (!PROVIDERS.has(p)) bad.push(`disabled: "${p}" isn't a provider the engine knows.`);
  }
  if (reg.host_aliases !== undefined) {
    if (typeof reg.host_aliases !== "object" || Array.isArray(reg.host_aliases) || Object.values(reg.host_aliases).some((v) => typeof v !== "string")) bad.push("host_aliases maps names to names.");
  }
  if (reg.hdhub !== undefined && (typeof reg.hdhub !== "object" || Array.isArray(reg.hdhub))) bad.push("hdhub must be an object.");
  return bad;
}

const decode = (content) => Buffer.from(String(content ?? ""), "base64").toString("utf8");

async function config() {
  const [file, commits] = await Promise.all([
    gh(`contents/${FILE}?ref=${BRANCH}`),
    gh(`commits?sha=${BRANCH}&path=${FILE}&per_page=15`).catch(() => []),
  ]);
  return {
    text: decode(file.content),
    sha: file.sha,
    writable: Boolean(process.env.CONFIG_GITHUB_TOKEN),
    providers: [...PROVIDERS],
    history: (commits ?? []).map((c) => ({ sha: c.sha, message: c.commit?.message?.split("\n")[0] ?? "", at: c.commit?.committer?.date ?? null, url: c.html_url })),
  };
}

async function configAt(b) {
  must(/^[0-9a-f]{40}$/.test(b.sha ?? ""), "That isn't a commit.");
  const file = await gh(`contents/${FILE}?ref=${b.sha}`);
  return { text: decode(file.content) };
}

async function configSave(b, me) {
  const text = String(b.text ?? "");
  must(text.length > 0 && text.length <= 64 * 1024, "The file is empty or too big.");
  let reg;
  try {
    reg = JSON.parse(text);
  } catch (e) {
    throw new Refusal(`That isn't valid JSON: ${e.message}`);
  }
  const bad = checkRegistry(reg);
  if (bad.length) throw new Refusal(bad.slice(0, 4).join(" "));
  must(/^[0-9a-f]{40}$/.test(b.sha ?? ""), "Reload the config first.");
  const message = String(b.message ?? "").replace(/\s+/g, " ").trim().slice(0, 100) || "update registry from the admin site";
  const out = await gh(`contents/${FILE}`, {
    method: "PUT",
    write: true,
    body: { message: `config: ${message}`, content: Buffer.from(JSON.stringify(reg, null, 2) + "\n").toString("base64"), sha: b.sha, branch: BRANCH },
  });
  await audit(me, "config.save", out.commit?.sha?.slice(0, 7) ?? null, { message });
  return { sha: out.content?.sha, commit: out.commit?.sha ?? null };
}

export const controlActions = {
  maxList,
  maxIssue,
  maxShow,
  maxSet,
  maxImport,
  notices,
  noticeSave,
  noticeDelete,
  ads,
  adsSave,
  config,
  configAt,
  configSave,
};
