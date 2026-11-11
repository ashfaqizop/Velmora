// Max licenses: key format, signatures and storage.
//
// A Max key is `VLM1.<payload>.<signature>`, both parts base64url. The payload
// is JSON { v, id, n (name), t (tier), iat } and the signature is Ed25519 over
// the payload bytes, made by the signing key that lives only on the
// developer's PC (tools/license). This file holds only the public half, so it
// can check keys but never make them.
//
// Activation ties a key to one PC. The app sends a SHA-256 of its machine ID
// (never the ID itself), this binds it in Redis, and answers with an
// activation token signed by the server key (LICENSE_SERVER_KEY). The app
// checks that token offline until it expires.

import crypto from "node:crypto";
import { SIGNING_PUBLIC_KEY } from "./keys.js";

export const TOKEN_DAYS = 30;
const DEVICE = /^[0-9a-f]{64}$/;

export const b64url = {
  encode: (buf) => Buffer.from(buf).toString("base64url"),
  decode: (s) => Buffer.from(s, "base64url"),
};

/** A raw 32-byte Ed25519 public key (base64url) as a Node key object. */
const publicKey = (x) => crypto.createPublicKey({ key: { kty: "OKP", crv: "Ed25519", x }, format: "jwk" });

let signingKey;
let serverKey;

/** Checks a Max key and returns its payload, or null. */
export function parseKey(key) {
  const pub = SIGNING_PUBLIC_KEY || process.env.LICENSE_SIGNING_PUBLIC_KEY;
  if (typeof key !== "string" || key.length > 600 || !pub) return null;
  const parts = key.trim().split(".");
  if (parts.length !== 3 || parts[0] !== "VLM1") return null;
  try {
    const payload = b64url.decode(parts[1]);
    const sig = b64url.decode(parts[2]);
    signingKey ??= publicKey(pub);
    if (!crypto.verify(null, payload, signingKey, sig)) return null;
    const data = JSON.parse(payload.toString("utf8"));
    if (data.v !== 1 || typeof data.id !== "string" || !/^MX-[0-9A-Z]{10}$/.test(data.id)) return null;
    return data;
  } catch {
    return null;
  }
}

export const validDevice = (d) => typeof d === "string" && DEVICE.test(d);

/** The activation token the app keeps: proof this key is bound to this PC. */
export function activationToken(id, device) {
  const raw = process.env.LICENSE_SERVER_KEY;
  if (!raw) return null;
  serverKey ??= crypto.createPrivateKey({ key: b64url.decode(raw), format: "der", type: "pkcs8" });
  const now = Math.floor(Date.now() / 1000);
  const payload = Buffer.from(JSON.stringify({ v: 1, id, d: device, iat: now, exp: now + TOKEN_DAYS * 86400 }));
  return `${b64url.encode(payload)}.${b64url.encode(crypto.sign(null, payload, serverKey))}`;
}

// Upstash Redis over REST. Vercel's Upstash integration sets these two.
export async function redis(...command) {
  const url = process.env.KV_REST_API_URL;
  const token = process.env.KV_REST_API_TOKEN;
  if (!url || !token) throw new Error("storage is not configured");
  const r = await fetch(url, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
    body: JSON.stringify(command),
    signal: AbortSignal.timeout(8000),
  });
  const body = await r.json();
  if (!r.ok || body.error) throw new Error(body.error || `storage error ${r.status}`);
  return body.result;
}

/** HGETALL comes back as a flat [k, v, k, v] list. */
export const hash = (list) => {
  const out = {};
  for (let i = 0; i + 1 < (list?.length ?? 0); i += 2) out[list[i]] = list[i + 1];
  return out;
};

/** At most `limit` requests a minute from one address. */
export async function rateLimited(req, limit) {
  const ip = String(req.headers["x-forwarded-for"] ?? "").split(",")[0].trim() || "unknown";
  const key = `rl:${ip}:${Math.floor(Date.now() / 60000)}`;
  const n = await redis("INCR", key);
  if (n === 1) await redis("EXPIRE", key, 120);
  return n > limit;
}

/** Constant-time check of the admin token. */
export function isAdmin(req) {
  const want = process.env.LICENSE_ADMIN_TOKEN;
  const got = String(req.headers.authorization ?? "").replace(/^Bearer\s+/i, "");
  if (!want || !got) return false;
  const a = crypto.createHash("sha256").update(want).digest();
  const b = crypto.createHash("sha256").update(got).digest();
  return crypto.timingSafeEqual(a, b);
}
