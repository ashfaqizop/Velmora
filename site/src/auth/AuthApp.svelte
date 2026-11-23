<script lang="ts">
  // Sign in to Velmora, in the browser.
  //
  // The desktop app opens this page with ?port=&state=&challenge=. We sign the
  // person in (Google, Discord or an emailed code), make their profile the
  // first time, then hand the finished session back to the app: through a one
  // time code sent to the app's local address, or typed in by hand as a short
  // code if something blocks that. The session never stays in this browser.
  import { onMount } from "svelte";
  import { createClient, type Session, type SupabaseClient } from "@supabase/supabase-js";
  import mark from "../assets/velmora-mark.webp";
  import Avatar from "./Avatar.svelte";
  import { AUTH_HOST, MAIN_SITE, SUPABASE_KEY, SUPABASE_URL } from "./config";
  import { isPhotoUrl, PRESETS, presetId } from "./avatars";

  type View = "loading" | "nocontext" | "signin" | "code" | "setup" | "return" | "done" | "error";
  interface Ctx {
    port: number;
    state: string;
    challenge: string;
  }
  interface Back {
    short: string;
    url: string;
    until: number;
  }

  const CTX = "velmora.auth.ctx";
  const BACK = "velmora.auth.back";
  const STORAGE_KEY = "velmora-auth";

  let sb: SupabaseClient;
  let ctx: Ctx | null = null;

  let view = $state<View>("loading");
  let busy = $state(false);
  let message = $state("");

  // Email
  let email = $state("");
  let token = $state("");
  let cooldown = $state(0);

  // Profile
  let session: Session | null = null;
  let username = $state("");
  let display = $state("");
  let avatar = $state(presetId(1));
  let photo = $state<string | null>(null);
  let nameCheck = $state<"idle" | "checking" | "free" | "taken" | "bad">("idle");
  let checkRun = 0;

  // Back to the app
  let back = $state<Back | null>(null);
  let secondsLeft = $state(0);

  const NAME = /^[A-Za-z0-9_]{3,20}$/;

  const store = {
    get: (k: string) => {
      try {
        return sessionStorage.getItem(k);
      } catch {
        return null;
      }
    },
    set: (k: string, v: string) => {
      try {
        sessionStorage.setItem(k, v);
      } catch {
        // Private windows can refuse; the flow still works in this tab.
      }
    },
  };

  /** Forget the web copy of the sign-in. Not signOut(): that would end the app's session too. */
  function wipeSession() {
    try {
      for (const k of Object.keys(sessionStorage)) if (k.startsWith(STORAGE_KEY)) sessionStorage.removeItem(k);
    } catch {
      // Nothing to wipe.
    }
  }

  function parseCtx(p: URLSearchParams): Ctx | null {
    const port = Number(p.get("port"));
    const state = p.get("state") ?? "";
    const challenge = p.get("challenge") ?? "";
    if (!Number.isInteger(port) || port < 1024 || port > 65535) return null;
    if (!/^[A-Za-z0-9_-]{16,64}$/.test(state) || !/^[A-Za-z0-9_-]{43}$/.test(challenge)) return null;
    return { port, state, challenge };
  }

  onMount(() => {
    // Sessions belong on the auth host only.
    const host = location.hostname;
    if (host !== AUTH_HOST && host !== "localhost" && host !== "127.0.0.1") {
      location.replace(`https://${AUTH_HOST}/${location.search}`);
      return;
    }
    start().catch((e) => fail(e));
  });

  async function start() {
    const params = new URLSearchParams(location.search);

    if (params.get("done")) {
      wipeSession();
      sessionStorage.removeItem(CTX);
      sessionStorage.removeItem(BACK);
      view = "done";
      return;
    }

    sb = createClient(SUPABASE_URL, SUPABASE_KEY, {
      auth: {
        flowType: "pkce",
        persistSession: true,
        autoRefreshToken: false,
        detectSessionInUrl: true,
        storage: window.sessionStorage,
        storageKey: STORAGE_KEY,
      },
    });

    // A fresh request from the app starts over.
    const fresh = parseCtx(params);
    if (fresh) {
      wipeSession();
      sessionStorage.removeItem(BACK);
      store.set(CTX, JSON.stringify(fresh));
      history.replaceState(null, "", location.pathname);
      ctx = fresh;
    } else {
      try {
        ctx = JSON.parse(store.get(CTX) ?? "null");
      } catch {
        ctx = null;
      }
    }

    // Coming back to the page after it already finished: offer the code again.
    const saved = store.get(BACK);
    if (saved && !params.get("code")) {
      try {
        const b = JSON.parse(saved) as Back;
        if (b.until > Date.now()) {
          back = b;
          view = "return";
          tick();
          return;
        }
      } catch {
        // Expired or unreadable; carry on.
      }
      sessionStorage.removeItem(BACK);
    }

    if (!ctx) {
      view = "nocontext";
      return;
    }

    const oauthError = params.get("error_description") ?? new URLSearchParams(location.hash.slice(1)).get("error_description");
    if (oauthError) {
      message = oauthError;
      view = "signin";
      history.replaceState(null, "", location.pathname);
      return;
    }

    const { data, error } = await sb.auth.getSession();
    if (error) {
      message = friendly(error.message);
      view = "signin";
      return;
    }
    if (data.session) await proceed(data.session);
    else view = "signin";
  }

  function fail(e: unknown) {
    console.error(e);
    message = "Something went wrong on our side. Please try again.";
    view = "error";
  }

  function friendly(text: string): string {
    const t = text.toLowerCase();
    if (t.includes("rate limit") || t.includes("too many")) return "Too many tries. Wait a minute and try again.";
    if (t.includes("expired") || t.includes("invalid")) return "That code didn't work. Check it, or ask for a new one.";
    if (t.includes("valid email") || t.includes("unable to validate email")) return "That doesn't look like an email address.";
    return text;
  }

  // ── Signing in ──

  async function oauth(provider: "google" | "discord") {
    busy = true;
    message = "";
    const { error } = await sb.auth.signInWithOAuth({ provider, options: { redirectTo: `${location.origin}/` } });
    if (error) {
      message = friendly(error.message);
      busy = false;
    }
    // On success the browser is already on its way to the provider.
  }

  async function sendCode() {
    const address = email.trim();
    if (!/^\S+@\S+\.\S+$/.test(address)) {
      message = "That doesn't look like an email address.";
      return;
    }
    busy = true;
    message = "";
    const { error } = await sb.auth.signInWithOtp({ email: address, options: { shouldCreateUser: true } });
    busy = false;
    if (error) {
      message = friendly(error.message);
      return;
    }
    email = address;
    token = "";
    view = "code";
    cooldown = 30;
    const timer = setInterval(() => {
      cooldown -= 1;
      if (cooldown <= 0) clearInterval(timer);
    }, 1000);
  }

  async function verify() {
    const code = token.replace(/\s/g, "");
    if (code.length < 6) {
      message = "Type the whole code from the email.";
      return;
    }
    busy = true;
    message = "";
    const { data, error } = await sb.auth.verifyOtp({ email, token: code, type: "email" });
    if (error || !data.session) {
      busy = false;
      message = friendly(error?.message ?? "That code didn't work.");
      return;
    }
    await proceed(data.session);
    busy = false;
  }

  // ── Profile ──

  async function proceed(s: Session) {
    session = s;
    const { data, error } = await sb.from("profiles").select("id").eq("id", s.user.id).maybeSingle();
    if (error) {
      fail(error);
      return;
    }
    if (data) {
      await finish(s);
      return;
    }
    prepareSetup(s);
    view = "setup";
  }

  function prepareSetup(s: Session) {
    const m = (s.user.user_metadata ?? {}) as Record<string, unknown>;
    const text = (v: unknown) => (typeof v === "string" ? v.trim() : "");
    const name = text(m.full_name) || text(m.name) || text((m.custom_claims as Record<string, unknown> | undefined)?.global_name) || text(m.user_name);
    display = (name || (s.user.email ?? "").split("@")[0] || "Fox").slice(0, 40);
    const pic = m.avatar_url ?? m.picture;
    photo = isPhotoUrl(pic) ? pic : null;
    avatar = photo ?? presetId(1 + Math.floor(Math.random() * PRESETS.length));
    suggest(text(m.user_name) || text(m.preferred_username) || name || (s.user.email ?? "").split("@")[0]);
  }

  /** A free username close to what we know about the person. */
  async function suggest(seed: string) {
    let base = seed.replace(/[^A-Za-z0-9_]/g, "").slice(0, 16);
    if (base.length < 3) base = `fox${base}`;
    for (let i = 0; i < 4; i++) {
      const tryName = i === 0 ? base : `${base.slice(0, 16)}${Math.floor(10 + Math.random() * 990)}`;
      const { data } = await sb.rpc("username_available", { candidate: tryName });
      if (data === true) {
        if (!username) username = tryName;
        return;
      }
    }
  }

  $effect(() => {
    const name = username;
    if (view !== "setup") return;
    const run = ++checkRun;
    if (!name) {
      nameCheck = "idle";
      return;
    }
    if (!NAME.test(name)) {
      nameCheck = "bad";
      return;
    }
    nameCheck = "checking";
    const timer = setTimeout(async () => {
      const { data, error } = await sb.rpc("username_available", { candidate: name });
      if (run !== checkRun) return;
      nameCheck = error ? "idle" : data ? "free" : "taken";
    }, 350);
    return () => clearTimeout(timer);
  });

  const nameHint = $derived(
    nameCheck === "free"
      ? "Yours."
      : nameCheck === "taken"
        ? "That name isn't available."
        : nameCheck === "bad"
          ? "3 to 20 letters, numbers or underscores."
          : nameCheck === "checking"
            ? "Checking…"
            : "3 to 20 letters, numbers or underscores. Capital letters don't make it a different name.",
  );

  async function createProfile() {
    if (!session || nameCheck !== "free" || !display.trim()) return;
    busy = true;
    message = "";
    const { error } = await sb
      .from("profiles")
      .insert({ id: session.user.id, username, display_name: display.trim(), avatar });
    if (error) {
      busy = false;
      message =
        error.code === "23505"
          ? "Someone just took that username. Pick another."
          : error.message.includes("username_reserved")
            ? "That name isn't available."
            : "Couldn't save your profile. Try again.";
      if (error.code === "23505") nameCheck = "taken";
      return;
    }
    await finish(session);
    busy = false;
  }

  // ── Back to the app ──

  async function finish(s: Session) {
    if (!ctx) return;
    view = "loading";
    let r: Response;
    try {
      r = await fetch("/api/auth/handoff", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ access_token: s.access_token, refresh_token: s.refresh_token, challenge: ctx.challenge }),
      });
    } catch {
      message = "Couldn't reach Velmora's server. Check your connection and try again.";
      view = "error";
      return;
    }
    const j = await r.json().catch(() => null);
    if (!j?.ok) {
      message = j?.message ?? "Sign-in couldn't finish. Try again.";
      view = "error";
      return;
    }
    wipeSession();
    const url = `http://127.0.0.1:${ctx.port}/callback?code=${encodeURIComponent(j.code)}&state=${encodeURIComponent(ctx.state)}`;
    back = { short: j.short, url, until: Date.now() + (j.expires_in - 5) * 1000 };
    store.set(BACK, JSON.stringify(back));
    view = "return";
    tick();
    // Give the page a moment to show, then go back to the app.
    setTimeout(() => back && location.assign(back.url), 900);
  }

  function tick() {
    const timer = setInterval(() => {
      secondsLeft = Math.max(0, Math.round(((back?.until ?? 0) - Date.now()) / 1000));
      if (secondsLeft <= 0) clearInterval(timer);
    }, 500);
    secondsLeft = Math.max(0, Math.round(((back?.until ?? 0) - Date.now()) / 1000));
  }

  const spaced = (s: string) => `${s.slice(0, 4)} ${s.slice(4)}`;

  function restart() {
    sessionStorage.removeItem(BACK);
    wipeSession();
    back = null;
    message = "";
    view = ctx ? "signin" : "nocontext";
  }
</script>

<div class="stage">
  <main class="card" aria-live="polite">
    <header class="brand">
      <img src={mark} alt="" width="64" height="64" />
      <span class="eyebrow">Velmora</span>
    </header>

    {#if view === "loading"}
      <div class="center">
        <span class="spinner" aria-hidden="true"></span>
        <p class="muted">One moment…</p>
      </div>
    {:else if view === "nocontext"}
      <h1>Sign in from the app</h1>
      <p class="muted">
        Open Velmora on your PC and choose <strong>Sign in</strong>. It brings you back to this page, and finishes the job when you're done.
      </p>
      <a class="btn ghost" href={MAIN_SITE}>Get Velmora</a>
    {:else if view === "signin"}
      <h1>Sign in or sign up</h1>
      <p class="muted">One account for your list, your progress and what's coming next.</p>
      {#if message}<p class="notice" role="alert">{message}</p>{/if}

      <div class="stack">
        <button class="btn provider" disabled={busy} onclick={() => oauth("google")}>
          <svg viewBox="0 0 48 48" aria-hidden="true">
            <path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z" />
            <path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z" />
            <path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z" />
            <path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z" />
          </svg>
          Continue with Google
        </button>
        <button class="btn provider" disabled={busy} onclick={() => oauth("discord")}>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path
              fill="#5865F2"
              d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"
            />
          </svg>
          Continue with Discord
        </button>
      </div>

      <div class="or"><span>or use your email</span></div>

      <form
        class="stack"
        onsubmit={(e) => {
          e.preventDefault();
          sendCode();
        }}
      >
        <label class="field">
          <span>Email</span>
          <input type="email" autocomplete="email" placeholder="you@example.com" bind:value={email} disabled={busy} />
        </label>
        <button class="btn primary" disabled={busy || !email.trim()}>{busy ? "Sending…" : "Email me a code"}</button>
      </form>

      <p class="fine">
        By continuing you agree to the <a href="{MAIN_SITE}/terms/">terms</a> and the <a href="{MAIN_SITE}/privacy/">privacy policy</a>.
      </p>
    {:else if view === "code"}
      <h1>Check your email</h1>
      <p class="muted">We sent a code to <strong>{email}</strong>. Type it here.</p>
      {#if message}<p class="notice" role="alert">{message}</p>{/if}
      <form
        class="stack"
        onsubmit={(e) => {
          e.preventDefault();
          verify();
        }}
      >
        <label class="field">
          <span>Code</span>
          <input
            class="code"
            inputmode="numeric"
            autocomplete="one-time-code"
            maxlength="10"
            placeholder="123456"
            bind:value={token}
            disabled={busy}
          />
        </label>
        <button class="btn primary" disabled={busy || token.replace(/\s/g, "").length < 6}>{busy ? "Checking…" : "Continue"}</button>
      </form>
      <div class="row">
        <button class="link" disabled={cooldown > 0 || busy} onclick={sendCode}>{cooldown > 0 ? `Send again in ${cooldown}s` : "Send a new code"}</button>
        <button class="link" onclick={() => ((view = "signin"), (message = ""))}>Use a different email</button>
      </div>
    {:else if view === "setup"}
      <h1>Make it yours</h1>
      <p class="muted">Pick a username and a face. You can change both later.</p>
      {#if message}<p class="notice" role="alert">{message}</p>{/if}
      <form
        class="stack"
        onsubmit={(e) => {
          e.preventDefault();
          createProfile();
        }}
      >
        <div class="me">
          <Avatar value={avatar} size={72} />
          <div class="grow">
            <label class="field">
              <span>Display name</span>
              <input maxlength="40" bind:value={display} disabled={busy} />
            </label>
          </div>
        </div>

        <label class="field">
          <span>Username</span>
          <div class="at">
            <i>@</i>
            <input
              maxlength="20"
              autocomplete="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={username}
              disabled={busy}
              aria-describedby="name-hint"
              class:good={nameCheck === "free"}
              class:bad={nameCheck === "taken" || nameCheck === "bad"}
            />
          </div>
          <small id="name-hint" class:good={nameCheck === "free"} class:bad={nameCheck === "taken" || nameCheck === "bad"}>{nameHint}</small>
        </label>

        <fieldset class="faces">
          <legend>Profile picture</legend>
          <div class="grid">
            {#if photo}
              <button type="button" class="face" class:on={avatar === photo} aria-label="Your photo" onclick={() => (avatar = photo!)}>
                <Avatar value={photo} size={48} />
              </button>
            {/if}
            {#each PRESETS as _, i (i)}
              <button type="button" class="face" class:on={avatar === presetId(i + 1)} aria-label="Fox {i + 1}" onclick={() => (avatar = presetId(i + 1))}>
                <Avatar value={presetId(i + 1)} size={48} />
              </button>
            {/each}
          </div>
        </fieldset>

        <button class="btn primary" disabled={busy || nameCheck !== "free" || !display.trim()}>{busy ? "Saving…" : "Finish"}</button>
      </form>
    {:else if view === "return"}
      <div class="center">
        <span class="tick" aria-hidden="true">✓</span>
        <h1>You're signed in</h1>
        <p class="muted">Taking you back to Velmora…</p>
      </div>
      {#if back}
        <div class="manual">
          <a class="btn primary" href={back.url}>Open Velmora</a>
          <p class="fine">
            Nothing happened? In Velmora, choose <strong>Enter a code</strong> and type
            <span class="short">{spaced(back.short)}</span>
            {#if secondsLeft > 0}<span class="muted">(good for {secondsLeft}s)</span>{:else}<span class="muted">(expired)</span>{/if}
          </p>
          {#if secondsLeft <= 0}<button class="link" onclick={restart}>Sign in again</button>{/if}
        </div>
      {/if}
    {:else if view === "done"}
      <div class="center">
        <span class="tick" aria-hidden="true">✓</span>
        <h1>All set</h1>
        <p class="muted">You're signed in to Velmora. You can close this tab.</p>
      </div>
    {:else}
      <h1>That didn't work</h1>
      <p class="muted">{message || "Something went wrong."}</p>
      <button class="btn primary" onclick={restart}>Try again</button>
    {/if}
  </main>
</div>

<style>
  .stage {
    min-height: 100vh;
    display: grid;
    place-items: center;
    padding: 32px 16px;
    background:
      radial-gradient(900px 500px at 15% -10%, color-mix(in srgb, var(--blue) 28%, transparent), transparent 60%),
      radial-gradient(700px 480px at 100% 110%, color-mix(in srgb, var(--cream) 14%, transparent), transparent 60%),
      var(--bg);
  }
  .card {
    width: min(440px, 100%);
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 32px;
    border-radius: 28px;
    background: color-mix(in srgb, #121827 92%, transparent);
    border: 1px solid var(--line);
    box-shadow: 0 30px 80px -30px rgb(0 0 0 / 70%);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .brand img {
    border-radius: 16px;
  }
  .eyebrow {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--cream);
  }
  h1 {
    margin: 4px 0 0;
    font-family: var(--display);
    font-weight: 400;
    font-size: 40px;
    line-height: 1.05;
  }
  p {
    margin: 0;
    line-height: 1.6;
  }
  .muted {
    color: var(--muted);
  }
  .fine {
    font-size: 13px;
    color: var(--muted);
    line-height: 1.6;
  }
  .fine a,
  .link {
    color: var(--violet);
  }
  .stack {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .notice {
    padding: 12px 14px;
    border-radius: 14px;
    font-size: 14px;
    background: color-mix(in srgb, #e5484d 18%, transparent);
    color: #ffc9cb;
  }
  .btn {
    all: unset;
    box-sizing: border-box;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    min-height: 48px;
    padding: 0 20px;
    border-radius: 16px;
    font-weight: 700;
    font-size: 15px;
    cursor: pointer;
    text-align: center;
    transition:
      transform 160ms cubic-bezier(0.2, 0.8, 0.2, 1),
      background 160ms,
      opacity 160ms;
  }
  .btn:focus-visible,
  .link:focus-visible,
  .face:focus-visible {
    outline: 2px solid var(--violet);
    outline-offset: 2px;
  }
  .btn:active:not(:disabled) {
    transform: scale(0.97);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .btn svg {
    width: 20px;
    height: 20px;
  }
  .provider {
    background: rgb(255 255 255 / 7%);
    border: 1px solid var(--line);
    color: var(--ink);
  }
  .provider:hover:not(:disabled) {
    background: rgb(255 255 255 / 12%);
  }
  .primary {
    background: var(--blue);
    color: #fff;
  }
  .primary:hover:not(:disabled) {
    background: color-mix(in srgb, var(--blue) 85%, white);
  }
  .ghost {
    border: 1px solid var(--line);
    color: var(--ink);
  }
  .or {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .or::before,
  .or::after {
    content: "";
    flex: 1;
    height: 1px;
    background: var(--line);
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
    color: var(--muted);
  }
  input {
    all: unset;
    box-sizing: border-box;
    width: 100%;
    min-height: 48px;
    padding: 0 14px;
    border-radius: 14px;
    border: 1px solid var(--line);
    background: rgb(255 255 255 / 5%);
    color: var(--ink);
    font-size: 16px;
  }
  input:focus {
    border-color: var(--violet);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--violet) 30%, transparent);
  }
  input::placeholder {
    color: rgb(255 255 255 / 30%);
  }
  .code {
    font-size: 28px;
    letter-spacing: 0.3em;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .at {
    position: relative;
  }
  .at i {
    position: absolute;
    left: 14px;
    top: 50%;
    translate: 0 -50%;
    font-style: normal;
    color: var(--muted);
  }
  .at input {
    padding-left: 32px;
  }
  .at input.good {
    border-color: #3ecf8e;
  }
  .at input.bad {
    border-color: #ff6b6b;
  }
  small {
    font-weight: 500;
    font-size: 12.5px;
    color: var(--muted);
  }
  small.good {
    color: #3ecf8e;
  }
  small.bad {
    color: #ff8a8a;
  }
  .row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }
  .link {
    all: unset;
    cursor: pointer;
    font-size: 14px;
    font-weight: 600;
    color: var(--violet);
  }
  .link:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .me {
    display: flex;
    align-items: flex-end;
    gap: 16px;
  }
  .grow {
    flex: 1;
    min-width: 0;
  }
  .faces {
    border: 0;
    margin: 0;
    padding: 0;
  }
  .faces legend {
    padding: 0;
    margin-bottom: 8px;
    font-size: 13px;
    font-weight: 600;
    color: var(--muted);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, 52px);
    gap: 8px;
  }
  .face {
    all: unset;
    cursor: pointer;
    display: grid;
    place-items: center;
    width: 52px;
    height: 52px;
    border-radius: 50%;
    transition: transform 160ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .face:hover {
    transform: scale(1.08);
  }
  .face.on {
    box-shadow: 0 0 0 3px var(--bg), 0 0 0 5px var(--violet);
  }
  .center {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px 0;
    text-align: center;
  }
  .tick {
    display: grid;
    place-items: center;
    width: 64px;
    height: 64px;
    border-radius: 50%;
    font-size: 32px;
    background: color-mix(in srgb, #3ecf8e 22%, transparent);
    color: #3ecf8e;
  }
  .spinner {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 3px solid rgb(255 255 255 / 14%);
    border-top-color: var(--violet);
    animation: spin 0.8s linear infinite;
  }
  .manual {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-top: 8px;
    border-top: 1px solid var(--line);
  }
  .short {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 8px;
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-weight: 700;
    letter-spacing: 0.12em;
    background: rgb(255 255 255 / 8%);
    color: var(--ink);
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .spinner {
      animation-duration: 2.4s;
    }
    .btn,
    .face {
      transition: none;
    }
  }
</style>
