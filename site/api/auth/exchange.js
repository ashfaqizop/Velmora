// The desktop app trades a one-time code for the session the browser made.
//
//   POST /api/auth/exchange { code, verifier } -> { ok, access_token, refresh_token }
//
// `code` is the long code the browser sent to the app's local address, or the
// 8-character `short` code typed in by hand. `verifier` is the secret the app
// made before opening the browser; only its hash was ever sent through the
// browser, so a code seen by anyone else is no use to them.

import crypto from "node:crypto";
import { rateLimited, redis } from "../_lib/license.js";

const fail = (res, status, error, message) => res.status(status).json({ ok: false, error, message });

const sha256 = (text, encoding) => crypto.createHash("sha256").update(text).digest(encoding);

function same(a, b) {
  const x = Buffer.from(String(a));
  const y = Buffer.from(String(b));
  return x.length === y.length && crypto.timingSafeEqual(x, y);
}

export default async function handler(req, res) {
  res.setHeader("Cache-Control", "no-store");
  if (req.method !== "POST") return fail(res, 405, "method", "POST only.");
  const { code, verifier } = req.body && typeof req.body === "object" ? req.body : {};
  if (typeof code !== "string" || typeof verifier !== "string") return fail(res, 400, "bad_request", "Missing code.");
  if (!/^[A-Za-z0-9_-]{43,128}$/.test(verifier)) return fail(res, 400, "bad_request", "Missing code.");

  try {
    if (await rateLimited(req, 30)) return fail(res, 429, "rate_limited", "Too many tries. Wait a minute.");

    let long = code.trim();
    if (/^[A-Za-z2-9]{8}$/.test(long) && !/[ilo01]/i.test(long)) {
      long = (await redis("GET", `ahs:${long.toUpperCase()}`)) ?? "";
    }
    if (!/^[A-Za-z0-9_-]{43}$/.test(long)) return fail(res, 404, "bad_code", "That code isn't right, or it has expired. Sign in again.");

    const key = `ah:${sha256(long, "hex")}`;
    const raw = await redis("GET", key);
    if (!raw) return fail(res, 404, "bad_code", "That code isn't right, or it has expired. Sign in again.");
    const record = JSON.parse(raw);
    if (!same(record.c, sha256(verifier, "base64url"))) return fail(res, 403, "bad_verifier", "This sign-in was started somewhere else.");

    // One use only: if someone else took it first, it's gone.
    if (!(await redis("GETDEL", key))) return fail(res, 404, "bad_code", "That code was already used. Sign in again.");
    return res.status(200).json({ ok: true, access_token: record.a, refresh_token: record.r });
  } catch (e) {
    console.error("auth exchange:", e);
    return fail(res, 503, "unavailable", "Sign-in couldn't finish. Try again in a minute.");
  }
}
