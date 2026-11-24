<script lang="ts">
  // Sign in to Velmora, in the browser.
  //
  // Two kinds of caller open this page:
  //  - the desktop app, with ?port=&state=&challenge=: the finished session goes
  //    back through a one-time code sent to the app's local address (or typed in
  //    by hand as a short code if something blocks that);
  //  - a Velmora website (social, admin), with ?return=&state=&challenge=: the
  //    same one-time code goes back to that page, which trades it for the session.
  // We sign the person in (Google, Discord or an emailed code) and make their
  // profile the first time. The session never stays on this origin.
  import { onMount } from "svelte";
  import { createClient, type Session, type SupabaseClient } from "@supabase/supabase-js";
  import mark from "../assets/velmora-mark.webp";
  import Avatar from "./Avatar.svelte";
  import NameStatus from "./NameStatus.svelte";
  import { fly } from "svelte/transition";
  import { AUTH_HOST, MAIN_SITE, SUPABASE_KEY, SUPABASE_URL, TURNSTILE_SITE_KEY } from "./config";
  import { ANIMALS, isPhotoUrl, randomAnimal } from "./avatars";

  type View = "loading" | "nocontext" | "signin" | "code" | "setup" | "return" | "done" | "error";
  type Ctx =
    | { kind: "app"; port: number; state: string; challenge: string }
    | { kind: "web"; back: string; state: string; challenge: string };

  /** Velmora sites that may ask this page for a sign-in. */
  const WEB_ORIGINS = ["https://social.getvelmora.dpdns.org", "https://admin.getvelmora.dpdns.org", "http://localhost:5173"];
  interface Back {
    short: string;
    url: string;
    until: number;
  }

  const CTX = "velmora.auth.ctx";
  const BACK = "velmora.auth.back";
  const STORAGE_KEY = "velmora-auth";

  let sb: SupabaseClient;
  let ctx = $state<Ctx | null>(null);

  let view = $state<View>("loading");
  let busy = $state(false);
  let message = $state("");

  // Cloudflare Turnstile: a quiet check that the email form isn't a bot. It only
  // shows itself when it needs the person to do something.
  interface TurnstileApi {
    render(el: HTMLElement, options: Record<string, unknown>): string;
    reset(id?: string): void;
  }
  let tsEl = $state<HTMLElement | undefined>();
  let tsWidget: string | null = null;
  let captcha = $state<string | null>(null);
  let captchaFailed = $state(false);

  function loadTurnstile(): Promise<TurnstileApi> {
    const w = window as unknown as { turnstile?: TurnstileApi };
    if (w.turnstile) return Promise.resolve(w.turnstile);
    return new Promise((resolve, reject) => {
      const s = document.createElement("script");
      s.src = "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit";
      s.async = true;
      s.onload = () => (w.turnstile ? resolve(w.turnstile) : reject(new Error("turnstile missing")));
      s.onerror = () => reject(new Error("turnstile blocked"));
      document.head.appendChild(s);
    });
  }

  async function mountCaptcha(el: HTMLElement) {
    tsWidget = "pending";
    try {
      const ts = await loadTurnstile();
      tsWidget = ts.render(el, {
        sitekey: TURNSTILE_SITE_KEY,
        theme: "dark",
        appearance: "interaction-only",
        callback: (t: string) => ((captcha = t), (captchaFailed = false)),
        "expired-callback": () => (captcha = null),
        "timeout-callback": () => (captcha = null),
        "error-callback": () => ((captcha = null), (captchaFailed = true)),
      });
    } catch {
      captchaFailed = true;
    }
  }

  /** A token works once, so ask for a fresh one after each use. */
  function freshCaptcha() {
    captcha = null;
    const w = window as unknown as { turnstile?: TurnstileApi };
    if (w.turnstile && tsWidget && tsWidget !== "pending") w.turnstile.reset(tsWidget);
  }

  $effect(() => {
    if ((view === "signin" || view === "code") && tsEl && !tsWidget) mountCaptcha(tsEl);
  });

  // Email
  let email = $state("");
  let token = $state("");
  let cooldown = $state(0);

  // Profile
  let session: Session | null = null;
  let username = $state("");
  let display = $state("");
  let avatar = $state(ANIMALS[0].id);
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
    const state = p.get("state") ?? "";
    const challenge = p.get("challenge") ?? "";
    if (!/^[A-Za-z0-9_-]{16,64}$/.test(state) || !/^[A-Za-z0-9_-]{43}$/.test(challenge)) return null;
    const ret = p.get("return");
    if (ret) {
      let url: URL;
      try {
        url = new URL(ret);
      } catch {
        return null;
      }
      if (!WEB_ORIGINS.includes(url.origin)) return null;
      return { kind: "web", back: `${url.origin}${url.pathname}`, state, challenge };
    }
    const port = Number(p.get("port"));
    if (!Number.isInteger(port) || port < 1024 || port > 65535) return null;
    return { kind: "app", port, state, challenge };
  }

  /** Where the person came from, for the headings. */
  const site = $derived(ctx?.kind === "web" ? (ctx.back.includes("admin.") ? "Velmora admin" : "Velmora Social") : "the Velmora app");

  onMount(() => {
    // Sessions belong on the auth host only.
    const host = location.hostname;
    if (host !== AUTH_HOST && host !== "localhost" && host !== "127.0.0.1") {
      location.replace(`https://${AUTH_HOST}/auth/${location.search}`);
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
        const saved = JSON.parse(store.get(CTX) ?? "null");
        // Saved before web callers existed: those were always the app.
        ctx = saved && !saved.kind ? { ...saved, kind: "app" } : saved;
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
    const { error } = await sb.auth.signInWithOAuth({ provider, options: { redirectTo: `${location.origin}/auth/` } });
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
    const { error } = await sb.auth.signInWithOtp({
      email: address,
      options: { shouldCreateUser: true, captchaToken: captcha ?? undefined },
    });
    freshCaptcha();
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
    avatar = photo ?? randomAnimal();
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
      // Still typing the first letters is not a mistake.
      nameCheck = /^[A-Za-z0-9_]{0,2}$/.test(name) ? "idle" : "bad";
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
            : `Couldn't save your profile. Try again. (${error.code ?? "error"}: ${error.message.slice(0, 120)})`;
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
    if (ctx.kind === "web") {
      // A Velmora website: straight back, it finishes the job itself.
      sessionStorage.removeItem(CTX);
      view = "loading";
      location.replace(`${ctx.back}?vcode=${encodeURIComponent(j.code)}&vstate=${encodeURIComponent(ctx.state)}`);
      return;
    }
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
  <div class="glow" aria-hidden="true"></div>
  <main class="card" aria-live="polite">
    <header class="brand">
      <img src={mark} alt="" width="56" height="56" />
      <div class="brand-text">
        <span class="eyebrow">Velmora</span>
        {#if ctx}<span class="for">for {site}</span>{/if}
      </div>
    </header>

    {#if view === "loading"}
      <div class="center" in:fly={{ y: 8, duration: 220 }}>
        <m3e-loading-indicator></m3e-loading-indicator>
        <p class="muted">One moment…</p>
      </div>
    {:else if view === "nocontext"}
      <section class="view" in:fly={{ y: 8, duration: 220 }}>
        <m3e-heading variant="display" size="small" level={1}>Start where you are</m3e-heading>
        <p class="muted">
          Choose <strong>Sign in</strong> in the Velmora app on your PC, or on Velmora Social. Either one brings you here and finishes the job when you're done.
        </p>
        <div class="row">
          <m3e-button variant="filled" href="https://social.getvelmora.dpdns.org/">
            <m3e-icon slot="icon" name="groups"></m3e-icon>Open Velmora Social
          </m3e-button>
          <m3e-button variant="text" href={MAIN_SITE}>Get the app</m3e-button>
        </div>
      </section>
    {:else if view === "signin"}
      <section class="view" in:fly={{ y: 8, duration: 220 }}>
        <m3e-heading variant="display" size="small" level={1}>Sign in or sign up</m3e-heading>
        <p class="muted">One account for your list, your progress and your profile.</p>
        {#if message}<p class="notice" role="alert"><m3e-icon name="error"></m3e-icon>{message}</p>{/if}

        <div class="stack">
          <m3e-button class="wide" variant="outlined" size="medium" disabled={busy} onclick={() => oauth("google")}>
            <span slot="icon" class="logo-wrap"><svg class="logo" viewBox="0 0 48 48" aria-hidden="true">
              <path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z" />
              <path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z" />
              <path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z" />
              <path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z" />
            </svg></span>
            Continue with Google
          </m3e-button>
          <m3e-button class="wide" variant="outlined" size="medium" disabled={busy} onclick={() => oauth("discord")}>
            <span slot="icon" class="logo-wrap"><svg class="logo" viewBox="0 0 24 24" aria-hidden="true">
              <path
                fill="#5865F2"
                d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"
              />
            </svg></span>
            Continue with Discord
          </m3e-button>
        </div>

        <m3e-divider class="or"></m3e-divider>
        <span class="or-label">or use your email</span>

        <div class="stack">
          <m3e-form-field variant="outlined" class="field">
            <m3e-icon slot="prefix" name="mail"></m3e-icon>
            <label slot="label" for="email">Email</label>
            <input id="email" type="email" autocomplete="email" bind:value={email} disabled={busy} onkeydown={(e) => e.key === "Enter" && captcha && sendCode()} />
            {#if captchaFailed}
              <span slot="hint">The security check couldn't run. A blocker may be stopping it; try Google or Discord instead.</span>
            {/if}
          </m3e-form-field>
          <m3e-button class="wide" variant="filled" size="medium" disabled={busy || !email.trim() || !captcha} onclick={sendCode}>
            {busy ? "Sending…" : captcha ? "Email me a code" : captchaFailed ? "Security check failed" : "Checking…"}
          </m3e-button>
        </div>

        <p class="fine">
          By continuing you agree to the <a href="{MAIN_SITE}/terms/">terms</a> and the <a href="{MAIN_SITE}/privacy/">privacy policy</a>.
        </p>
      </section>
    {:else if view === "code"}
      <section class="view" in:fly={{ y: 8, duration: 220 }}>
        <m3e-heading variant="display" size="small" level={1}>Check your email</m3e-heading>
        <p class="muted">We sent a code to <strong>{email}</strong>. Type it here.</p>
        {#if message}<p class="notice" role="alert"><m3e-icon name="error"></m3e-icon>{message}</p>{/if}
        <m3e-form-field variant="outlined" class="field">
          <label slot="label" for="otp">Code</label>
          <input
            id="otp"
            class="code"
            inputmode="numeric"
            autocomplete="one-time-code"
            maxlength="10"
            bind:value={token}
            disabled={busy}
            onkeydown={(e) => e.key === "Enter" && verify()}
          />
        </m3e-form-field>
        <m3e-button class="wide" variant="filled" size="medium" disabled={busy || token.replace(/\s/g, "").length < 6} onclick={verify}>
          {busy ? "Checking…" : "Continue"}
        </m3e-button>
        <div class="row spread">
          <m3e-button variant="text" disabled={cooldown > 0 || busy || !captcha} onclick={sendCode}>
            {cooldown > 0 ? `Send again in ${cooldown}s` : "Send a new code"}
          </m3e-button>
          <m3e-button variant="text" onclick={() => ((view = "signin"), (message = ""))}>Use a different email</m3e-button>
        </div>
      </section>
    {:else if view === "setup"}
      <section class="view" in:fly={{ y: 8, duration: 220 }}>
        <m3e-heading variant="display" size="small" level={1}>Make it yours</m3e-heading>
        <p class="muted">Pick a username and a face. You can change both later.</p>
        {#if message}<p class="notice" role="alert"><m3e-icon name="error"></m3e-icon>{message}</p>{/if}

        <div class="me">
          {#key avatar}<span class="me-pic" in:fly={{ y: 6, duration: 200 }}><Avatar value={avatar} size={88} ring={3} /></span>{/key}
          <m3e-form-field variant="outlined" class="field grow">
            <label slot="label" for="display">Display name</label>
            <input id="display" maxlength="40" bind:value={display} disabled={busy} />
          </m3e-form-field>
        </div>

        <m3e-form-field variant="outlined" class="field" error={nameCheck === "taken" || nameCheck === "bad"}>
          <span slot="prefix" class="at">@</span>
          <label slot="label" for="username">Username</label>
          <input id="username" maxlength="20" autocomplete="off" autocapitalize="off" spellcheck="false" bind:value={username} disabled={busy} />
          <span slot="suffix" class="tail"><NameStatus state={nameCheck} /></span>
          <span slot="error-icon"></span>
          <span slot="hint">{nameHint}</span>
          <span slot="error">{nameHint}</span>
        </m3e-form-field>

        <fieldset class="faces">
          <legend>Profile picture</legend>
          <div class="grid">
            {#if photo}
              <button type="button" class="face" class:on={avatar === photo} aria-label="Your photo" aria-pressed={avatar === photo} onclick={() => (avatar = photo!)}>
                <Avatar value={photo} size={56} />
              </button>
            {/if}
            {#each ANIMALS as a (a.id)}
              <button type="button" class="face" class:on={avatar === a.id} aria-label={a.label} aria-pressed={avatar === a.id} title={a.label} style:--tint={a.color} onclick={() => (avatar = a.id)}>
                <Avatar value={a.id} size={56} />
              </button>
            {/each}
          </div>
        </fieldset>

        <m3e-button class="wide" variant="filled" size="medium" disabled={busy || nameCheck !== "free" || !display.trim()} onclick={createProfile}>
          <m3e-icon slot="icon" name="check"></m3e-icon>{busy ? "Saving…" : "Finish"}
        </m3e-button>
      </section>
    {:else if view === "return"}
      <section class="view center" in:fly={{ y: 8, duration: 220 }}>
        <span class="badge ok" aria-hidden="true"><m3e-shape name="12-sided-cookie"></m3e-shape><m3e-icon name="check" filled></m3e-icon></span>
        <m3e-heading variant="headline" size="large" level={1}>You're signed in</m3e-heading>
        <p class="muted">Taking you back to Velmora…</p>
      </section>
      {#if back}
        <div class="manual">
          <m3e-button class="wide" variant="filled" href={back.url}><m3e-icon slot="icon" name="open_in_new"></m3e-icon>Open Velmora</m3e-button>
          <p class="fine">
            Nothing happened? In Velmora, choose <strong>Enter a code instead</strong> and type
            <span class="short">{spaced(back.short)}</span>
            {#if secondsLeft > 0}<span class="muted">(good for {secondsLeft}s)</span>{:else}<span class="muted">(expired)</span>{/if}
          </p>
          {#if secondsLeft <= 0}<m3e-button variant="text" onclick={restart}>Sign in again</m3e-button>{/if}
        </div>
      {/if}
    {:else if view === "done"}
      <section class="view center" in:fly={{ y: 8, duration: 220 }}>
        <span class="badge ok" aria-hidden="true"><m3e-shape name="12-sided-cookie"></m3e-shape><m3e-icon name="check" filled></m3e-icon></span>
        <m3e-heading variant="headline" size="large" level={1}>All set</m3e-heading>
        <p class="muted">You're signed in to Velmora. You can close this tab.</p>
      </section>
    {:else}
      <section class="view center" in:fly={{ y: 8, duration: 220 }}>
        <span class="badge bad" aria-hidden="true"><m3e-shape name="4-sided-cookie"></m3e-shape><m3e-icon name="priority_high"></m3e-icon></span>
        <m3e-heading variant="headline" size="large" level={1}>That didn't work</m3e-heading>
        <p class="muted">{message || "Something went wrong."}</p>
        <m3e-button variant="filled" onclick={restart}>Try again</m3e-button>
      </section>
    {/if}
    <div class="ts" bind:this={tsEl}></div>
  </main>
</div>

<style>
  .stage {
    position: relative;
    min-height: 100vh;
    display: grid;
    place-items: center;
    padding: 32px 16px;
    overflow: hidden;
    background: var(--md-sys-color-surface, #0a0d16);
  }
  .glow {
    position: absolute;
    inset: -20%;
    pointer-events: none;
    background:
      radial-gradient(42% 36% at 18% 12%, color-mix(in srgb, var(--md-sys-color-primary) 30%, transparent), transparent 70%),
      radial-gradient(36% 32% at 88% 92%, color-mix(in srgb, #f8e8c8 16%, transparent), transparent 70%);
    filter: blur(10px);
  }
  .card {
    position: relative;
    width: min(460px, 100%);
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 32px;
    border-radius: 32px;
    background: color-mix(in srgb, var(--md-sys-color-surface-container) 92%, transparent);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 50%, transparent);
    box-shadow: 0 30px 80px -30px rgb(0 0 0 / 70%);
    backdrop-filter: blur(18px);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .brand img {
    border-radius: 18px;
  }
  .brand-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .eyebrow {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: #f8e8c8;
  }
  .for {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .view {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  m3e-heading {
    font-family: var(--display);
  }
  p {
    margin: 0;
    line-height: 1.6;
  }
  .muted {
    color: var(--md-sys-color-on-surface-variant);
  }
  .fine {
    font-size: 13px;
    line-height: 1.6;
    color: var(--md-sys-color-on-surface-variant);
  }
  .fine a {
    color: var(--md-sys-color-primary);
  }
  .stack {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .wide {
    width: 100%;
  }
  .logo-wrap {
    display: inline-grid;
  }
  .logo {
    width: 20px;
    height: 20px;
  }
  .row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    align-items: center;
  }
  .spread {
    justify-content: space-between;
  }
  .or {
    margin-top: 6px;
  }
  .or-label {
    align-self: center;
    margin-top: -24px;
    padding: 0 12px;
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
    background: var(--md-sys-color-surface-container);
  }
  .field {
    width: 100%;
  }
  .grow {
    flex: 1;
    min-width: 0;
  }
  .notice {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    border-radius: 16px;
    font-size: 14px;
    background: var(--md-sys-color-error-container);
    color: var(--md-sys-color-on-error-container);
    --m3e-icon-size: 20px;
  }
  .code {
    font-size: 24px;
    letter-spacing: 0.3em;
    font-variant-numeric: tabular-nums;
  }
  .at {
    padding-left: 4px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .tail {
    display: grid;
    padding-right: 8px;
    --status-busy: var(--md-sys-color-primary);
    --status-no: var(--md-sys-color-error);
  }
  .me {
    display: flex;
    align-items: center;
    gap: 18px;
  }
  .me-pic {
    display: inline-grid;
  }
  .faces {
    border: 0;
    margin: 0;
    padding: 0;
  }
  .faces legend {
    padding: 0;
    margin-bottom: 10px;
    font-size: 13px;
    font-weight: 600;
    color: var(--md-sys-color-on-surface-variant);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(60px, 1fr));
    gap: 10px;
    justify-items: center;
  }
  .face {
    all: unset;
    cursor: pointer;
    display: grid;
    place-items: center;
    width: 60px;
    height: 60px;
    border-radius: 50%;
    transition:
      transform 220ms cubic-bezier(0.34, 1.56, 0.64, 1),
      box-shadow 200ms ease;
  }
  .face:hover {
    transform: scale(1.08);
  }
  .face:active {
    transform: scale(0.94);
  }
  .face:focus-visible,
  .face.on {
    box-shadow:
      0 0 0 3px var(--md-sys-color-surface-container),
      0 0 0 6px var(--tint, var(--md-sys-color-primary));
  }
  .center {
    align-items: center;
    text-align: center;
    padding: 8px 0;
  }
  .badge {
    display: inline-grid;
    place-items: center;
    --m3e-shape-size: 72px;
    --m3e-icon-size: 34px;
  }
  .badge > * {
    grid-area: 1 / 1;
  }
  .badge.ok {
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
  }
  .badge.bad {
    --m3e-shape-container-color: var(--md-sys-color-error-container);
    color: var(--md-sys-color-on-error-container);
  }
  .manual {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-top: 14px;
    border-top: 1px solid var(--md-sys-color-outline-variant);
  }
  .short {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 8px;
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-weight: 700;
    letter-spacing: 0.12em;
    background: var(--md-sys-color-surface-container-highest);
    color: var(--md-sys-color-on-surface);
  }
  .ts:empty {
    display: none;
  }
  @media (max-width: 480px) {
    .card {
      padding: 24px 20px;
      border-radius: 28px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .face {
      transition: none;
    }
  }
</style>
