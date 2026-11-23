// Passing a web sign-in back to the desktop app.
//
//   POST /api/auth/handoff { access_token, refresh_token, challenge }
//     -> { ok, code, short }
//
// The sign-in happens in the browser, on the auth site. When it is done, the
// page posts the finished session here. We check it with Supabase, keep it for
// two minutes under a one-time code, and the page sends that code to the app
// (a local address the app is listening on, or typed in by hand as `short`).
// The app trades the code for the session at /api/auth/exchange, proving it
// started the sign-in with the secret that `challenge` is a hash of (PKCE).
//
// Redis keys: ah:<sha256 of code> (the session, 120 s), ahs:<short> (the code, 120 s).

import crypto from "node:crypto";
import { rateLimited, redis } from "../_lib/license.js";
import { SUPABASE_KEY, SUPABASE_URL } from "../_lib/config.js";

const TTL = 120;
const SHORT_ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const CHALLENGE = /^[A-Za-z0-9_-]{43}$/;

const fail = (res, status, error, message) => res.status(status).json({ ok: false, error, message });

const shortCode = () => Array.from({ length: 8 }, () => SHORT_ALPHABET[crypto.randomInt(SHORT_ALPHABET.length)]).join("");

async function supabase(path, token) {
  const r = await fetch(`${SUPABASE_URL}${path}`, {
    headers: { apikey: SUPABASE_KEY, Authorization: `Bearer ${token}` },
    signal: AbortSignal.timeout(8000),
  });
  return { status: r.status, body: await r.json().catch(() => null) };
}

export default async function handler(req, res) {
  res.setHeader("Cache-Control", "no-store");
  if (req.method !== "POST") return fail(res, 405, "method", "POST only.");
  const { access_token: access, refresh_token: refresh, challenge } = req.body && typeof req.body === "object" ? req.body : {};

  if (typeof access !== "string" || access.length < 20 || access.length > 4000) return fail(res, 400, "bad_session", "That sign-in isn't valid.");
  if (typeof refresh !== "string" || refresh.length < 6 || refresh.length > 600) return fail(res, 400, "bad_session", "That sign-in isn't valid.");
  if (typeof challenge !== "string" || !CHALLENGE.test(challenge)) return fail(res, 400, "bad_challenge", "The app's request looks wrong. Start again from the app.");

  try {
    if (await rateLimited(req, 20)) return fail(res, 429, "rate_limited", "Too many tries. Wait a minute.");

    // Only a session Supabase itself accepts, for someone with a finished profile.
    const who = await supabase("/auth/v1/user", access);
    if (who.status !== 200 || !who.body?.id) return fail(res, 401, "bad_session", "Your sign-in expired. Please sign in again.");
    const profile = await supabase(`/rest/v1/profiles?id=eq.${encodeURIComponent(who.body.id)}&select=id`, access);
    if (profile.status !== 200 || !Array.isArray(profile.body) || !profile.body.length) {
      return fail(res, 409, "no_profile", "Finish setting up your profile first.");
    }

    const code = crypto.randomBytes(32).toString("base64url");
    const record = JSON.stringify({ a: access, r: refresh, c: challenge });
    await redis("SET", `ah:${crypto.createHash("sha256").update(code).digest("hex")}`, record, "EX", TTL);

    let short = "";
    for (let i = 0; i < 4 && !short; i++) {
      const candidate = shortCode();
      if ((await redis("SET", `ahs:${candidate}`, code, "NX", "EX", TTL)) === "OK") short = candidate;
    }
    return res.status(200).json({ ok: true, code, short, expires_in: TTL });
  } catch (e) {
    console.error("auth handoff:", e);
    return fail(res, 503, "unavailable", "Sign-in couldn't finish. Try again in a minute.");
  }
}
