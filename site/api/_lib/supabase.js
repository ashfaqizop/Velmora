// Checking who a request is from. The app and the sign-in page send the
// person's Supabase access token; Supabase itself says whether it is real.

import { SUPABASE_KEY, SUPABASE_URL } from "./config.js";

/** The signed-in user for an access token, or null. */
export async function verifyUser(accessToken) {
  if (typeof accessToken !== "string" || accessToken.length < 20 || accessToken.length > 4000) return null;
  try {
    const r = await fetch(`${SUPABASE_URL}/auth/v1/user`, {
      headers: { apikey: SUPABASE_KEY, Authorization: `Bearer ${accessToken}` },
      signal: AbortSignal.timeout(8000),
    });
    if (r.status !== 200) return null;
    const user = await r.json();
    return typeof user?.id === "string" ? { id: user.id, email: user.email ?? null } : null;
  } catch {
    return null;
  }
}

/**
 * The Supabase database as the server itself (service key), for jobs that
 * write what people can't: the Theatre lineup, who has Max. The key lives
 * only in Vercel's environment (SUPABASE_SERVICE_ROLE_KEY).
 */
export async function db(path, { method = "GET", body, prefer } = {}) {
  const key = process.env.SUPABASE_SERVICE_ROLE_KEY || process.env.SUPABASE_SECRET_KEY;
  if (!key) throw new Error("SUPABASE_SERVICE_ROLE_KEY is not set");
  const headers = { apikey: key, "Content-Type": "application/json" };
  // Legacy service keys are JWTs and go in Authorization too; new sb_secret_ keys don't.
  if (key.startsWith("eyJ")) headers.Authorization = `Bearer ${key}`;
  if (prefer) headers.Prefer = prefer;
  const r = await fetch(`${SUPABASE_URL}/rest/v1/${path}`, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
    signal: AbortSignal.timeout(10000),
  });
  const text = await r.text();
  if (!r.ok) throw new Error(`supabase ${r.status}: ${text.slice(0, 300)}`);
  return text ? JSON.parse(text) : null;
}

/** Mirror "this account has Max" into the database (for Theatre's booking rules). Never fails the caller. */
export async function setMax(userId, on) {
  try {
    if (on) await db("max_accounts?on_conflict=user_id", { method: "POST", body: { user_id: userId }, prefer: "resolution=ignore-duplicates,return=minimal" });
    else await db(`max_accounts?user_id=eq.${userId}`, { method: "DELETE", prefer: "return=minimal" });
  } catch (e) {
    console.warn("max mirror:", e.message);
  }
}

/** Supabase Auth's admin API (ban, delete) as the server. Returns the parsed body. */
export async function authAdmin(path, { method = "GET", body } = {}) {
  const key = process.env.SUPABASE_SERVICE_ROLE_KEY || process.env.SUPABASE_SECRET_KEY;
  if (!key) throw new Error("SUPABASE_SERVICE_ROLE_KEY is not set");
  const headers = { apikey: key, "Content-Type": "application/json" };
  if (key.startsWith("eyJ")) headers.Authorization = `Bearer ${key}`;
  const r = await fetch(`${SUPABASE_URL}/auth/v1/admin/${path}`, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
    signal: AbortSignal.timeout(10000),
  });
  const text = await r.text();
  if (!r.ok) throw new Error(`auth ${r.status}: ${text.slice(0, 300)}`);
  return text ? JSON.parse(text) : null;
}
