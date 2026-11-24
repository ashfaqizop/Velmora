// Who is looking at the admin site: signed in, an admin, and past the
// authenticator code. The server checks all of this again on every call.
import { beginSignIn, restoreSession, signOut, supabase } from "../lib/session";
import { call } from "./api";

export type Phase = "loading" | "out" | "denied" | "enroll" | "challenge" | "ok";

export const gate = $state({
  phase: "loading" as Phase,
  error: "",
  busy: false,
  factorId: "",
  qr: "",
  secret: "",
});

const IDLE_MS = 30 * 60_000;
let idleTimer: ReturnType<typeof setTimeout> | undefined;

function touch() {
  clearTimeout(idleTimer);
  if (gate.phase !== "ok") return;
  idleTimer = setTimeout(() => void leave("You were signed out after 30 minutes without activity."), IDLE_MS);
}

function watchIdle() {
  for (const ev of ["pointerdown", "keydown", "scroll"]) addEventListener(ev, touch, { passive: true });
  touch();
}

/** Work out what to show: sign-in, "not an admin", the code box, or the site. */
export async function boot() {
  try {
    const session = await restoreSession();
    if (!session) {
      gate.phase = "out";
      return;
    }
    await step();
  } catch (e) {
    gate.error = (e as Error).message;
    gate.phase = "out";
  }
}

async function step() {
  const me = await call<{ admin: boolean; aal: number }>("me");
  if (!me.admin) {
    gate.phase = "denied";
    return;
  }
  if (me.aal >= 2) {
    gate.phase = "ok";
    watchIdle();
    return;
  }
  const { data, error } = await supabase.auth.mfa.listFactors();
  if (error) throw new Error(error.message);
  const verified = data.totp.find((f) => f.status === "verified");
  if (verified) {
    gate.factorId = verified.id;
    gate.phase = "challenge";
    return;
  }
  // A half-finished set-up from last time would block a new one: clear it.
  for (const f of data.all.filter((f) => f.factor_type === "totp" && f.status !== "verified")) await supabase.auth.mfa.unenroll({ factorId: f.id });
  const made = await supabase.auth.mfa.enroll({ factorType: "totp", friendlyName: "Velmora admin" });
  if (made.error) throw new Error(made.error.message);
  gate.factorId = made.data.id;
  gate.qr = made.data.totp.qr_code;
  gate.secret = made.data.totp.secret;
  gate.phase = "enroll";
}

export async function submitCode(code: string) {
  if (gate.busy || !/^\d{6}$/.test(code)) return;
  gate.busy = true;
  gate.error = "";
  try {
    const { error } = await supabase.auth.mfa.challengeAndVerify({ factorId: gate.factorId, code });
    if (error) {
      gate.error = /invalid|expired/i.test(error.message) ? "That code didn't match. Wait for a fresh one and try again." : error.message;
      return;
    }
    await step();
  } catch (e) {
    gate.error = (e as Error).message;
  } finally {
    gate.busy = false;
  }
}

export async function signIn() {
  gate.busy = true;
  await beginSignIn();
}

export async function leave(message = "") {
  clearTimeout(idleTimer);
  await signOut();
  gate.phase = "out";
  gate.error = message;
  gate.busy = false;
}
