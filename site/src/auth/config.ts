// Public settings for accounts. The publishable key is public by design: it
// can only do what the row level security rules in supabase/schema.sql allow.
// Keep in step with site/api/_lib/config.js and app/src/lib/account.svelte.ts.

export const SUPABASE_URL = "https://htvirqckkhfekgfwxops.supabase.co";
export const SUPABASE_KEY = "sb_publishable_YSf01mWc6H2QFTxcBQcCMA_lElqBvgQ";

/** The only place sign-in pages run. Its own origin keeps the main site's ad scripts away from sessions. */
export const AUTH_HOST = "auth.getvelmora.dpdns.org";
export const MAIN_SITE = "https://getvelmora.dpdns.org";
