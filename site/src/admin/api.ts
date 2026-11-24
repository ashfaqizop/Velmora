// Calls to /api/admin/<action>. The server checks the token, the admin row and
// TOTP on every one of them; nothing here is trusted.
import { M3eSnackbar } from "@m3e/web/snackbar";
import { supabase } from "../lib/session";

export class ApiError extends Error {
  constructor(
    message: string,
    public code?: string,
    public status = 0,
  ) {
    super(message);
  }
}

export async function call<T = Record<string, unknown>>(action: string, body: Record<string, unknown> = {}): Promise<T & { ok: true }> {
  const { data } = await supabase.auth.getSession();
  const token = data.session?.access_token;
  if (!token) throw new ApiError("Sign in again.", "auth", 401);
  let r: Response;
  try {
    r = await fetch(`/api/admin/${action}`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
      body: JSON.stringify(body),
    });
  } catch {
    throw new ApiError("You seem to be offline.");
  }
  const j = await r.json().catch(() => null);
  if (!j?.ok) throw new ApiError(j?.message ?? "That didn't work.", j?.code, r.status);
  return j;
}

export const say = (msg: string) => M3eSnackbar.open(msg);
export const sayError = (e: unknown) => M3eSnackbar.open(e instanceof Error ? e.message : "Something went wrong.");
