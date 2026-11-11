// Public keys only. These are safe to publish: they can check signatures but
// not make them. The local license tool's `init` writes both to public.json;
// they go here and into the app (app/src-tauri/src/license.rs).

/** Checks Max keys. Its private half stays on the developer's PC. */
export const SIGNING_PUBLIC_KEY = "FnlbeKNPEBeONzf_j6oj74xGFKFTWl1qAyIA9QJNS5c";

/** Checks activation tokens. Its private half is LICENSE_SERVER_KEY in Vercel. */
export const SERVER_PUBLIC_KEY = "yiN3UCjCmt1qBkAC90Elq0yQ8JBnJ6lO7UxtCRBxgGs";
