// Max license activation. One key, one PC.
//
//   POST /api/license/activate { key, device }  bind the key to this PC, or
//                                               refresh the token if it already is
//   POST /api/license/release  { key, device }  free the key from this PC
//   POST /api/license/admin    { action, id }   developer only (Bearer token):
//                                               info, list, release, revoke, unrevoke
//
// Redis keys: lic:<id> (hash: device, name, activated, seen), lic:ids (set),
// lic:revoked (set), rl:<ip>:<minute> (rate limit).

import { activationToken, hash, isAdmin, parseKey, rateLimited, redis, validDevice } from "../_lib/license.js";

const fail = (res, status, error, message) => res.status(status).json({ ok: false, error, message });

async function activate(body, res) {
  const lic = parseKey(body.key);
  if (!lic) return fail(res, 400, "bad_key", "This isn't a valid Velmora Max key.");
  if (!validDevice(body.device)) return fail(res, 400, "bad_device", "The app sent an unreadable PC ID.");
  if (await redis("SISMEMBER", "lic:revoked", lic.id)) {
    return fail(res, 403, "revoked", "This key has been turned off. Contact the person who gave it to you.");
  }

  const k = `lic:${lic.id}`;
  // HSETNX makes the first PC win, even if two try at the same moment.
  const bound = await redis("HSETNX", k, "device", body.device);
  const device = bound ? body.device : await redis("HGET", k, "device");
  if (device !== body.device) {
    return fail(res, 409, "other_device", "This key is already in use on another PC. Remove it there first, or ask for it to be released.");
  }

  const now = String(Math.floor(Date.now() / 1000));
  const fields = ["name", lic.n ?? "", "seen", now];
  if (bound) fields.push("activated", now);
  await redis("HSET", k, ...fields);
  await redis("SADD", "lic:ids", lic.id);

  const token = activationToken(lic.id, body.device);
  if (!token) return fail(res, 503, "not_configured", "Activation isn't set up on the server yet.");
  return res.status(200).json({ ok: true, token, license: { id: lic.id, name: lic.n ?? "", tier: lic.t ?? "max" } });
}

async function release(body, res) {
  const lic = parseKey(body.key);
  if (!lic) return fail(res, 400, "bad_key", "This isn't a valid Velmora Max key.");
  if (!validDevice(body.device)) return fail(res, 400, "bad_device", "The app sent an unreadable PC ID.");
  const k = `lic:${lic.id}`;
  const device = await redis("HGET", k, "device");
  // Only the PC that holds the key can let it go.
  if (device && device !== body.device) return fail(res, 409, "other_device", "This key belongs to another PC.");
  if (device) await redis("HDEL", k, "device");
  return res.status(200).json({ ok: true });
}

async function admin(req, body, res) {
  if (!isAdmin(req)) return fail(res, 401, "unauthorized", "Wrong admin token.");
  const id = typeof body.id === "string" ? body.id.toUpperCase() : "";
  const needsId = body.action !== "list";
  if (needsId && !/^MX-[0-9A-Z]{10}$/.test(id)) return fail(res, 400, "bad_id", "Give a license ID like MX-ABCDE12345.");

  const info = async (one) => ({
    id: one,
    ...hash(await redis("HGETALL", `lic:${one}`)),
    revoked: !!(await redis("SISMEMBER", "lic:revoked", one)),
  });

  switch (body.action) {
    case "info":
      return res.status(200).json({ ok: true, license: await info(id) });
    case "list": {
      const ids = (await redis("SMEMBERS", "lic:ids")) ?? [];
      return res.status(200).json({ ok: true, licenses: await Promise.all(ids.sort().map(info)) });
    }
    case "release":
      await redis("HDEL", `lic:${id}`, "device");
      return res.status(200).json({ ok: true });
    case "revoke":
      await redis("SADD", "lic:revoked", id);
      await redis("HDEL", `lic:${id}`, "device");
      return res.status(200).json({ ok: true });
    case "unrevoke":
      await redis("SREM", "lic:revoked", id);
      return res.status(200).json({ ok: true });
    default:
      return fail(res, 400, "bad_action", "Unknown admin action.");
  }
}

export default async function handler(req, res) {
  res.setHeader("Cache-Control", "no-store");
  if (req.method !== "POST") return fail(res, 405, "method", "POST only.");
  const body = req.body && typeof req.body === "object" ? req.body : {};

  try {
    if (await rateLimited(req, 30)) return fail(res, 429, "rate_limited", "Too many tries. Wait a minute.");
    switch (req.query.action) {
      case "activate":
        return await activate(body, res);
      case "release":
        return await release(body, res);
      case "admin":
        return await admin(req, body, res);
      default:
        return fail(res, 404, "not_found", "Unknown endpoint.");
    }
  } catch (e) {
    console.error("license:", e);
    return fail(res, 503, "unavailable", "The license server couldn't answer. Try again in a minute.");
  }
}
