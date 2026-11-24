// Max license activation. One key, one PC.
//
//   POST /api/license/activate { key, device }  bind the key to this PC, or
//                                               refresh the token if it already is
//   POST /api/license/release  { key, device }  free the key from this PC
//   POST /api/license/account  { action, access_token, device, key? }
//                                               keep Max with a Velmora account:
//                                               link (this PC's key -> the account),
//                                               restore (the account's key -> this PC),
//                                               unlink (forget the link)
//   POST /api/license/admin    { action, id }   developer only (Bearer token):
//                                               info, list, release, revoke, unrevoke, unlink
//
// Redis keys: lic:<id> (hash: device, name, activated, seen, owner, moved),
// lic:ids (set), lic:revoked (set), acct:<user id> (hash: id, key, linked),
// rl:<ip>:<minute> (rate limit).

import { activationToken, hash, isAdmin, parseKey, rateLimited, redis, validDevice } from "../_lib/license.js";
import { setMax, verifyUser } from "../_lib/supabase.js";

/** Moving Max to another PC through the account: at most once in this long. */
const MOVE_COOLDOWN = 12 * 3600;

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

async function account(body, res) {
  const user = await verifyUser(body.access_token);
  if (!user) return fail(res, 401, "bad_session", "Sign in again to use your account.");
  const a = `acct:${user.id}`;
  const now = Math.floor(Date.now() / 1000);

  switch (body.action) {
    case "link": {
      const lic = parseKey(body.key);
      if (!lic) return fail(res, 400, "bad_key", "This isn't a valid Velmora Max key.");
      if (!validDevice(body.device)) return fail(res, 400, "bad_device", "The app sent an unreadable PC ID.");
      if (await redis("SISMEMBER", "lic:revoked", lic.id)) return fail(res, 403, "revoked", "This key has been turned off.");
      const k = `lic:${lic.id}`;
      // Only a PC that really holds the key can attach it to an account.
      if ((await redis("HGET", k, "device")) !== body.device) {
        return fail(res, 409, "not_this_pc", "Activate the key on this PC first, then link it.");
      }
      const owner = await redis("HGET", k, "owner");
      if (owner && owner !== user.id) {
        return fail(res, 409, "other_account", "This key is already linked to a different Velmora account.");
      }
      // An account holds one key; linking another frees the old one.
      const before = await redis("HGET", a, "id");
      if (before && before !== lic.id) await redis("HDEL", `lic:${before}`, "owner");
      await redis("HSET", k, "owner", user.id);
      await redis("HSET", a, "id", lic.id, "key", body.key.trim(), "linked", String(now));
      await setMax(user.id, true);
      return res.status(200).json({ ok: true, result: owner === user.id ? "already" : "linked" });
    }

    case "restore": {
      if (!validDevice(body.device)) return fail(res, 400, "bad_device", "The app sent an unreadable PC ID.");
      const rec = hash(await redis("HGETALL", a));
      if (!rec.key) return res.status(200).json({ ok: true, result: "none" });
      const lic = parseKey(rec.key);
      if (!lic) return res.status(200).json({ ok: true, result: "none" });
      if (await redis("SISMEMBER", "lic:revoked", lic.id)) return fail(res, 403, "revoked", "The key on your account has been turned off.");

      const k = `lic:${lic.id}`;
      const bound = await redis("HGET", k, "device");
      let result = "already";
      if (bound !== body.device) {
        if (bound) {
          // Moving it from another PC. Once in a while is fine; back and forth is not.
          const moved = Number(await redis("HGET", k, "moved")) || 0;
          const wait = moved + MOVE_COOLDOWN - now;
          if (wait > 0) {
            const hours = Math.ceil(wait / 3600);
            return fail(res, 429, "cooldown", `Max was moved recently. You can move it again in about ${hours} hour${hours === 1 ? "" : "s"}.`);
          }
          await redis("HSET", k, "moved", String(now));
        }
        await redis("HSET", k, "device", body.device);
        result = "restored";
      }
      await redis("HSET", k, "name", lic.n ?? "", "seen", String(now));
      await redis("SADD", "lic:ids", lic.id);
      await setMax(user.id, true);
      const token = activationToken(lic.id, body.device);
      if (!token) return fail(res, 503, "not_configured", "Activation isn't set up on the server yet.");
      return res.status(200).json({ ok: true, result, key: rec.key, token });
    }

    case "unlink": {
      const id = await redis("HGET", a, "id");
      if (id && (await redis("HGET", `lic:${id}`, "owner")) === user.id) await redis("HDEL", `lic:${id}`, "owner");
      await redis("DEL", a);
      await setMax(user.id, false);
      return res.status(200).json({ ok: true, result: "unlinked" });
    }

    default:
      return fail(res, 400, "bad_action", "Unknown action.");
  }
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
    case "unlink": {
      // Detach a key from whichever account holds it (the person lost access, say).
      const owner = await redis("HGET", `lic:${id}`, "owner");
      if (owner) {
        await redis("HDEL", `lic:${id}`, "owner");
        if ((await redis("HGET", `acct:${owner}`, "id")) === id) await redis("DEL", `acct:${owner}`);
      }
      return res.status(200).json({ ok: true });
    }
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
      case "account":
        return await account(body, res);
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
