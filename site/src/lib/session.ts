// Signing in on Velmora's own websites (social, admin). The sign-in itself
// happens on the auth site; it sends back a one-time code that only this tab
// can redeem, because only this tab knows the secret the code was tied to (PKCE).
// The session then lives on this site's own origin. These sites load no ads
// and no third-party scripts, so nothing else on the page can read it.
import { createClient, type Session } from "@supabase/supabase-js";
import { AUTH_HOST, SUPABASE_KEY, SUPABASE_URL } from "../auth/config";

const PENDING = "velmora.web.signin";

export const supabase = createClient(SUPABASE_URL, SUPABASE_KEY, {
  auth: { persistSession: true, autoRefreshToken: true, detectSessionInUrl: false, storageKey: "velmora-web" },
});

const b64url = (bytes: Uint8Array) => btoa(String.fromCharCode(...bytes)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
const random = (n: number) => b64url(crypto.getRandomValues(new Uint8Array(n)));

/** Go to the sign-in page; it comes back here when done. */
export async function beginSignIn() {
  const verifier = random(32);
  const challenge = b64url(new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(verifier))));
  const state = random(24);
  sessionStorage.setItem(PENDING, JSON.stringify({ verifier, state, back: location.pathname + location.search }));
  const ret = `${location.origin}${location.pathname}`;
  location.assign(`https://${AUTH_HOST}/auth/?return=${encodeURIComponent(ret)}&state=${state}&challenge=${challenge}`);
}

/**
 * Finish a sign-in that just came back (?vcode=&vstate=), or pick up the
 * session from last time. Resolves to the session, or null when signed out.
 * Throws with a readable message when a sign-in came back but failed.
 */
export async function restoreSession(): Promise<Session | null> {
  const params = new URLSearchParams(location.search);
  const code = params.get("vcode");
  if (code) {
    let pending: { verifier: string; state: string; back: string } | null = null;
    try {
      pending = JSON.parse(sessionStorage.getItem(PENDING) ?? "null");
    } catch {
      pending = null;
    }
    sessionStorage.removeItem(PENDING);
    history.replaceState(null, "", pending?.back ?? location.pathname);
    if (!pending || pending.state !== params.get("vstate")) throw new Error("That sign-in was started in another tab. Try again here.");
    const r = await fetch("/api/auth/exchange", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ code, verifier: pending.verifier }),
    });
    const j = await r.json().catch(() => null);
    if (!j?.ok) throw new Error(j?.message ?? "Sign-in couldn't finish. Try again.");
    const { data, error } = await supabase.auth.setSession({ access_token: j.access_token, refresh_token: j.refresh_token });
    if (error) throw new Error(error.message);
    return data.session;
  }
  const { data } = await supabase.auth.getSession();
  return data.session;
}

/** Sign out of this site only; the app and other sites stay signed in. */
export async function signOut() {
  await supabase.auth.signOut({ scope: "local" }).catch(() => {});
}
