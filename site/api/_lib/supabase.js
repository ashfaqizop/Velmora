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
