// Small helpers shared by the admin site's server code.

import { db } from "./supabase.js";

export const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
export const TITLE = /^(movie|tv)\/\d{1,9}$/;
export const DAYKEY = /^\d{4}-\d{2}-\d{2}$/;
export const PAGE = 30;

/** An error whose message is safe to show the admin as is. */
export class Refusal extends Error {
  constructor(message, status = 400) {
    super(message);
    this.status = status;
  }
}

export const must = (cond, message = "That doesn't look right.") => {
  if (!cond) throw new Refusal(message);
};
export const num = (v, lo, hi, fallback) => (Number.isInteger(v) && v >= lo && v <= hi ? v : fallback);
export const quote = (s) => encodeURIComponent(String(s));

/** One line in public.audit_log. */
export async function audit(admin, action, target, details = {}) {
  await db("audit_log", { method: "POST", body: { admin_id: admin, action, target: target ?? null, details }, prefer: "return=minimal" });
}

export async function isAdminUser(id) {
  const rows = await db(`admins?user_id=eq.${id}&select=user_id`);
  return rows.length > 0;
}

/** A user id from either an id or a @username. */
export async function resolveUser(value) {
  const v = String(value ?? "").trim().replace(/^@/, "");
  if (UUID.test(v)) return v;
  must(/^[A-Za-z0-9_]{3,20}$/.test(v), "Enter a username or an account id.");
  const rows = await db(`profiles?username_key=eq.${quote(v.toLowerCase())}&select=id`);
  must(rows[0], "No one has that username.");
  return rows[0].id;
}
