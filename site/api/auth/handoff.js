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
// The Android app also uses this to open Velmora Social signed in:
//
//   POST /api/auth/handoff { access_token, challenge, mint: true }
//
// Here the app sends only its access token, never its refresh token, and gets
// a code for a brand-new session of the same person. The page inside the app
// redeems it. Two separate sessions mean the website refreshing its tokens
// never signs the app out (Supabase rotates refresh tokens on use).
//
// Redis keys: ah:<sha256 of code> (the session, 120 s), ahs:<short> (the code, 120 s).

import crypto from "node:crypto";
import { rateLimited, redis } from "../_lib/license.js";
import { SUPABASE_KEY, SUPABASE_URL } from "../_lib/config.js";
import { authAdmin } from "../_lib/supabase.js";

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
  const { access_token: access, refresh_token: given, challenge, mint } = req.body && typeof req.body === "object" ? req.body : {};
  const minting = mint === true;

  if (typeof access !== "string" || access.length < 20 || access.length > 4000) return fail(res, 400, "bad_session", "That sign-in isn't valid.");
  if (!minting && (typeof given !== "string" || given.length < 6 || given.length > 600)) return fail(res, 400, "bad_session", "That sign-in isn't valid.");
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

    let session = { a: access, r: given };
    if (minting) {
      session = await freshSession(who.body.email);
      if (!session) return fail(res, 409, "no_email", "Velmora Social couldn't sign you in here. Open it in your browser instead.");
    }

    const code = crypto.randomBytes(32).toString("base64url");
    const record = JSON.stringify({ a: session.a, r: session.r, c: challenge });
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

/**
 * A new, separate session for the person with this email: a one-time sign-in
 * link made by the server (nothing is emailed) and used straight away.
 */
async function freshSession(email) {
  if (typeof email !== "string" || !email.includes("@")) return null;
  try {
    const link = await authAdmin("generate_link", { method: "POST", body: { type: "magiclink", email } });
    const hash = link?.hashed_token ?? link?.properties?.hashed_token;
    if (typeof hash !== "string") return null;
    for (const type of ["magiclink", "email"]) {
      const r = await fetch(`${SUPABASE_URL}/auth/v1/verify`, {
        method: "POST",
        headers: { apikey: SUPABASE_KEY, "Content-Type": "application/json" },
        body: JSON.stringify({ type, token_hash: hash }),
        signal: AbortSignal.timeout(8000),
      });
      const j = await r.json().catch(() => null);
      if (r.ok && typeof j?.access_token === "string" && typeof j?.refresh_token === "string") return { a: j.access_token, r: j.refresh_token };
      // A used link can't be tried twice; only retry when the type was refused.
      if (r.status !== 400 || !/type/i.test(JSON.stringify(j ?? ""))) return null;
    }
  } catch (e) {
    console.error("auth handoff mint:", e.message);
  }
  return null;
}
