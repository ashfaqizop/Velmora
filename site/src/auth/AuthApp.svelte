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
  import { quintOut } from "svelte/easing";
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

  // ── Motion: steps slide the way you're going, and the card's height glides. ──
  const ORDER: Record<View, number> = { loading: 0, nocontext: 1, signin: 1, error: 1, code: 2, setup: 3, return: 4, done: 4 };
  let dir = 1;
  let lastOrder = 0;
  $effect.pre(() => {
    const o = ORDER[view];
    dir = o >= lastOrder ? 1 : -1;
    lastOrder = o;
  });
  const calm = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  function step(_node: Element) {
    if (calm()) return { duration: 160, css: (t: number) => `opacity: ${t}` };
    const d = dir;
    return {
      delay: 70,
      duration: 420,
      easing: quintOut,
      css: (t: number, u: number) => `opacity: ${Math.min(1, t * 1.6)}; transform: translateX(${u * 28 * d}px) scale(${0.985 + 0.015 * t})`,
    };
  }
  let innerH = $state(0);
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
  /** Cloudflare wants a click (shown inline under the email form). */
  let needsCheck = $state(false);
  /** "Email me a code" was pressed before the check finished; send as soon as it does. */
  let waitingCheck = $state(false);

  // Which way you signed in last time on this browser, for the "Last used" badge.
  const LAST = "velmora.auth.last";
  let lastUsed = $state<"google" | "discord" | "email" | null>(null);
  let opening = $state<"google" | "discord" | null>(null);
  function remember(m: "google" | "discord" | "email") {
    try {
      localStorage.setItem(LAST, m);
    } catch {
      // Not remembered; only the badge is lost.
    }
  }

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
        callback: (t: string) => ((captcha = t), (captchaFailed = false), (needsCheck = false)),
        "expired-callback": () => (captcha = null),
        "timeout-callback": () => (captcha = null),
        "error-callback": () => ((captcha = null), (captchaFailed = true), (waitingCheck = false)),
        "before-interactive-callback": () => (needsCheck = true),
        "after-interactive-callback": () => (needsCheck = false),
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

  // The check finished after the button was pressed: carry on.
  $effect(() => {
    if (waitingCheck && captcha) {
      waitingCheck = false;
      sendCode();
    }
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
  const siteIcon = $derived(ctx?.kind === "web" ? (ctx.back.includes("admin.") ? "shield" : "groups") : "desktop_windows");

  // The art leans toward the pointer a little.
  let tiltX = $state(0);
  let tiltY = $state(0);
  function lean(e: PointerEvent) {
    if (matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    tiltX = (e.clientX / innerWidth - 0.5) * 10;
    tiltY = (e.clientY / innerHeight - 0.5) * -10;
  }

  const emailLabel = $derived(
    busy && !opening ? "Sending…" : waitingCheck ? "Checking you're human…" : captchaFailed ? "Security check failed" : "Email me a code",
  );

  onMount(() => {
    try {
      const l = localStorage.getItem(LAST);
      lastUsed = l === "google" || l === "discord" || l === "email" ? l : null;
    } catch {
      lastUsed = null;
    }
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
    opening = provider;
    message = "";
    remember(provider);
    const { error } = await sb.auth.signInWithOAuth({ provider, options: { redirectTo: `${location.origin}/auth/` } });
    if (error) {
      message = friendly(error.message);
      busy = false;
      opening = null;
    }
    // On success the browser is already on its way to the provider.
  }

  async function sendCode() {
    const address = email.trim();
    if (!/^\S+@\S+\.\S+$/.test(address)) {
      message = "That doesn't look like an email address.";
      return;
    }
    if (!captcha) {
      if (captchaFailed) message = "The security check couldn't run. A blocker may be stopping it; try Google or Discord instead.";
      else waitingCheck = true;
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
    remember("email");
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

<svelte:window onpointermove={lean} />

<div class="stage">
  <div class="aurora" aria-hidden="true"><span class="a1"></span><span class="a2"></span><span class="a3"></span></div>

  <aside class="art" aria-hidden="true">
    <div class="frame" style:--tx="{tiltX}deg" style:--ty="{tiltY}deg">
      <img src={mark} alt="" width="512" height="512" />
      <span class="sheen"></span>
    </div>
    <div class="pitch">
      <p class="pitch-title">One account.<br /><em>Every PC.</em></p>
      <ul>
        <li><m3e-icon name="sync"></m3e-icon><span>My List, progress and settings on every PC you sign in on</span></li>
        <li><m3e-icon name="person_heart"></m3e-icon><span>Your page on Velmora Social: follow people, rate and review</span></li>
        <li><m3e-icon name="workspace_premium"></m3e-icon><span>Velmora Max goes wherever you do</span></li>
      </ul>
    </div>
  </aside>

  <main class="card" aria-live="polite">
    <header class="brand">
      <img src={mark} alt="" width="44" height="44" />
      <span class="word">Velmora</span>
      {#if ctx && view !== "done"}<span class="dest"><m3e-icon name={siteIcon}></m3e-icon>{site === "the Velmora app" ? "Velmora app" : site}</span>{/if}
    </header>

    <div class="viewport" style:height={innerH ? `${innerH}px` : "auto"}>
    <div class="inner" bind:clientHeight={innerH}>
    {#if view === "loading"}
      <div class="center" in:step>
        <m3e-loading-indicator></m3e-loading-indicator>
        <p class="muted">One moment…</p>
      </div>
    {:else if view === "nocontext"}
      <section class="view" in:step>
        <h1 class="title">Start where you are</h1>
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
      <section class="view" in:step>
        <div class="head">
          <h1 class="title">Sign in or sign up</h1>
          <p class="muted">One free account for your list, your progress and your profile. New here? This makes your account.</p>
        </div>
        {#if message}<p class="notice" role="alert" in:fly={{ y: -6, duration: 260, easing: quintOut }}><m3e-icon name="error"></m3e-icon>{message}</p>{/if}

        <div class="stack">
          <m3e-button class="provider google" class:last={lastUsed === "google"} variant="filled" size="medium" disabled={busy} onclick={() => oauth("google")}>
            <span slot="icon" class="logo-wrap">
              {#if opening === "google"}<m3e-loading-indicator class="spin"></m3e-loading-indicator>{:else}<svg class="logo" viewBox="0 0 48 48" aria-hidden="true">
                <path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z" />
                <path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z" />
                <path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z" />
                <path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z" />
              </svg>{/if}
            </span>
            {opening === "google" ? "Opening Google…" : "Continue with Google"}
            {#if lastUsed === "google"}<span slot="trailing-icon" class="last-chip">Last used</span>{/if}
          </m3e-button>
          <m3e-button class="provider discord" class:last={lastUsed === "discord"} variant="filled" size="medium" disabled={busy} onclick={() => oauth("discord")}>
            <span slot="icon" class="logo-wrap">
              {#if opening === "discord"}<m3e-loading-indicator class="spin"></m3e-loading-indicator>{:else}<svg class="logo" viewBox="0 0 24 24" aria-hidden="true">
                <path fill="currentColor" d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"
                />
              </svg>{/if}
            </span>
            {opening === "discord" ? "Opening Discord…" : "Continue with Discord"}
            {#if lastUsed === "discord"}<span slot="trailing-icon" class="last-chip">Last used</span>{/if}
          </m3e-button>
        </div>

        <div class="or" role="separator"><span>or with your email</span></div>

        <form class="email" class:last={lastUsed === "email"} onsubmit={(e) => (e.preventDefault(), sendCode())}>
          <m3e-form-field variant="outlined" class="field">
            <m3e-icon slot="prefix" name="mail"></m3e-icon>
            <label slot="label" for="email">Email address</label>
            <input id="email" type="email" autocomplete="email" bind:value={email} disabled={busy} />
            {#if lastUsed === "email"}<span slot="suffix" class="last-chip in-field">Last used</span>{/if}
          </m3e-form-field>
          <m3e-button class="wide send" variant="tonal" size="medium" type="submit" disabled={busy || waitingCheck || !email.trim() || captchaFailed} onclick={sendCode}>
            <span slot="icon" class="logo-wrap">
              {#if waitingCheck || (busy && !opening)}<m3e-loading-indicator class="spin"></m3e-loading-indicator>{:else}<m3e-icon name="send"></m3e-icon>{/if}
            </span>
            {emailLabel}
          </m3e-button>
        </form>
      </section>
    {:else if view === "code"}
      <section class="view" in:step>
        <h1 class="title">Check your email</h1>
        <p class="muted">We sent a code to <strong>{email}</strong>. Type it here.</p>
        {#if message}<p class="notice" role="alert" in:fly={{ y: -6, duration: 260, easing: quintOut }}><m3e-icon name="error"></m3e-icon>{message}</p>{/if}
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
      <section class="view" in:step>
        <h1 class="title">Make it yours</h1>
        <p class="muted">Pick a username and a face. You can change both later.</p>
        {#if message}<p class="notice" role="alert" in:fly={{ y: -6, duration: 260, easing: quintOut }}><m3e-icon name="error"></m3e-icon>{message}</p>{/if}

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
      <section class="view center" in:step>
        <span class="badge ok" aria-hidden="true"><m3e-shape name="12-sided-cookie"></m3e-shape><m3e-icon name="check" filled></m3e-icon></span>
        <h1 class="title small">You're signed in</h1>
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
      <section class="view center" in:step>
        <span class="badge ok" aria-hidden="true"><m3e-shape name="12-sided-cookie"></m3e-shape><m3e-icon name="check" filled></m3e-icon></span>
        <h1 class="title small">All set</h1>
        <p class="muted">You're signed in to Velmora. You can close this tab.</p>
      </section>
    {:else}
      <section class="view center" in:step>
        <span class="badge bad" aria-hidden="true"><m3e-shape name="4-sided-cookie"></m3e-shape><m3e-icon name="priority_high"></m3e-icon></span>
        <h1 class="title small">That didn't work</h1>
        <p class="muted">{message || "Something went wrong."}</p>
        <m3e-button variant="filled" onclick={restart}>Try again</m3e-button>
      </section>
    {/if}
    <div class="check" class:show={needsCheck} aria-hidden={!needsCheck}>
      <div class="check-in">
        <span class="check-label"><m3e-icon name="shield"></m3e-icon>One quick check before we email you</span>
        <div class="ts" bind:this={tsEl}></div>
      </div>
    </div>

    {#if view === "signin"}
      <p class="fine" in:fly={{ y: 6, duration: 300, delay: 160 }}>
        By continuing you agree to the <a href="{MAIN_SITE}/terms/">terms</a> and the <a href="{MAIN_SITE}/privacy/">privacy policy</a>.
      </p>
    {/if}
    </div>
    </div>
  </main>
</div>

<style>
  /* ── Frame: aurora backdrop, brand art on the left, the card on the right ── */
  .stage {
    position: relative;
    min-height: 100vh;
    min-height: 100dvh;
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 480px);
    align-items: center;
    gap: clamp(32px, 6vw, 96px);
    padding: clamp(20px, 4vh, 48px) clamp(20px, 6vw, 96px);
    overflow: hidden;
    isolation: isolate;
    background: #070a14;
  }
  .aurora {
    position: absolute;
    inset: 0;
    z-index: -1;
    overflow: hidden;
  }
  .aurora span {
    position: absolute;
    border-radius: 50%;
    will-change: transform;
  }
  .a1 {
    width: 70vmax;
    height: 70vmax;
    left: -22vmax;
    top: -28vmax;
    background: radial-gradient(closest-side, rgb(50 113 254 / 46%), transparent);
    animation: drift-a 26s ease-in-out infinite alternate;
  }
  .a2 {
    width: 52vmax;
    height: 52vmax;
    left: 18vmax;
    bottom: -30vmax;
    background: radial-gradient(closest-side, rgb(124 58 237 / 26%), transparent);
    animation: drift-b 32s ease-in-out infinite alternate;
  }
  .a3 {
    width: 40vmax;
    height: 40vmax;
    right: -14vmax;
    top: -10vmax;
    background: radial-gradient(closest-side, rgb(248 232 200 / 10%), transparent);
    animation: drift-a 38s ease-in-out infinite alternate-reverse;
  }
  @keyframes drift-a {
    to {
      transform: translate(6vmax, 5vmax) scale(1.08);
    }
  }
  @keyframes drift-b {
    to {
      transform: translate(-7vmax, -4vmax) scale(1.12);
    }
  }

  .art {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: clamp(18px, 3.4vh, 36px);
    justify-self: end;
    width: min(460px, 100%);
    animation: art-in 900ms cubic-bezier(0.05, 0.7, 0.1, 1) both;
  }
  .frame {
    position: relative;
    width: clamp(160px, min(24vw, 38vh), 320px);
    aspect-ratio: 1;
    border-radius: 30%;
    overflow: hidden;
    transform: perspective(900px) rotateY(var(--tx, 0deg)) rotateX(var(--ty, 0deg)) rotate(-4deg);
    transition: transform 600ms cubic-bezier(0.2, 0.8, 0.2, 1);
    box-shadow:
      0 0 0 1px rgb(255 255 255 / 10%),
      0 40px 90px -30px rgb(50 113 254 / 70%),
      0 30px 60px -30px rgb(0 0 0 / 80%);
    animation: float 9s ease-in-out infinite alternate;
  }
  .frame img {
    width: 100%;
    height: 100%;
    display: block;
  }
  .sheen {
    position: absolute;
    inset: 0;
    background: linear-gradient(115deg, transparent 35%, rgb(255 255 255 / 14%) 48%, transparent 60%);
    background-size: 250% 100%;
    animation: sheen 7s ease-in-out infinite;
    mix-blend-mode: overlay;
  }
  @keyframes float {
    to {
      translate: 0 -10px;
    }
  }
  @keyframes sheen {
    0%,
    55% {
      background-position: 130% 0;
    }
    100% {
      background-position: -60% 0;
    }
  }
  @keyframes art-in {
    from {
      opacity: 0;
      transform: translateY(18px);
    }
  }
  .pitch {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .pitch-title {
    margin: 0;
    font-family: var(--display);
    font-size: clamp(34px, min(4.4vw, 7.4vh), 60px);
    line-height: 1;
    letter-spacing: -0.01em;
    color: #eef1fb;
  }
  .pitch-title em {
    color: #f8e8c8;
  }
  .pitch ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .pitch li {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: clamp(13.5px, 2.2vh, 15px);
    line-height: 1.45;
    color: color-mix(in srgb, #eef1fb 78%, transparent);
    --m3e-icon-size: 20px;
  }
  .pitch li m3e-icon {
    flex: none;
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 12px;
    background: rgb(50 113 254 / 18%);
    color: #9db8ff;
  }

  .card {
    position: relative;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 22px;
    padding: clamp(24px, 4vh, 36px) clamp(22px, 3vw, 36px);
    border-radius: 36px;
    background: color-mix(in srgb, #10162a 82%, transparent);
    border: 1px solid rgb(255 255 255 / 8%);
    box-shadow:
      inset 0 1px 0 rgb(255 255 255 / 6%),
      0 40px 100px -40px rgb(0 0 0 / 85%);
    backdrop-filter: blur(24px) saturate(1.2);
    animation: card-in 700ms cubic-bezier(0.05, 0.7, 0.1, 1) 80ms both;
  }
  @keyframes card-in {
    from {
      opacity: 0;
      transform: translateY(16px) scale(0.985);
    }
  }
  .viewport {
    transition: height 460ms cubic-bezier(0.2, 0.8, 0.2, 1);
    overflow: hidden;
    /* Room for focus rings and floating labels that sit on the edge. */
    margin: -8px;
    padding: 8px;
    box-sizing: content-box;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .brand img {
    border-radius: 14px;
    box-shadow: 0 8px 20px -8px rgb(50 113 254 / 70%);
  }
  .word {
    font-size: 20px;
    font-weight: 800;
    letter-spacing: -0.01em;
  }
  .dest {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-left: auto;
    padding: 6px 12px 6px 10px;
    border-radius: 999px;
    font-size: 13px;
    font-weight: 600;
    background: rgb(50 113 254 / 16%);
    color: #b9ccff;
    --m3e-icon-size: 18px;
    white-space: nowrap;
  }
  .head {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 4px;
  }
  .title {
    margin: 0;
    font-family: var(--display);
    font-weight: 400;
    font-size: clamp(34px, 4.4vh, 42px);
    line-height: 1.05;
    letter-spacing: -0.01em;
  }
  .title.small {
    font-size: 32px;
  }

  /* ── Providers ── */
  .provider {
    width: 100%;
    --m3e-button-medium-container-height: 54px;
    --m3e-button-medium-label-text-font-size: 16px;
    --m3e-button-medium-label-text-font-weight: 600;
    transition: transform 220ms cubic-bezier(0.2, 0.8, 0.2, 1.4);
  }
  .provider:hover {
    transform: translateY(-1px);
  }
  .provider:active {
    transform: translateY(0) scale(0.99);
  }
  .google {
    --m3e-filled-button-container-color: #ffffff;
    --m3e-filled-button-label-text-color: #1f1f1f;
    --m3e-filled-button-hover-label-text-color: #1f1f1f;
    --m3e-filled-button-focus-label-text-color: #1f1f1f;
    --m3e-filled-button-hover-state-layer-color: #1f1f1f;
    --m3e-filled-button-focus-state-layer-color: #1f1f1f;
    --m3e-filled-button-pressed-state-layer-color: #1f1f1f;
    --m3e-filled-button-disabled-container-color: #ffffff;
    --m3e-filled-button-disabled-container-opacity: 0.55;
    --m3e-filled-button-disabled-label-text-color: #1f1f1f;
  }
  .discord {
    --m3e-filled-button-container-color: #5865f2;
    --m3e-filled-button-label-text-color: #ffffff;
    --m3e-filled-button-hover-label-text-color: #ffffff;
    --m3e-filled-button-focus-label-text-color: #ffffff;
    --m3e-filled-button-hover-state-layer-color: #ffffff;
    --m3e-filled-button-focus-state-layer-color: #ffffff;
    --m3e-filled-button-pressed-state-layer-color: #ffffff;
    --m3e-filled-button-disabled-container-color: #5865f2;
    --m3e-filled-button-disabled-container-opacity: 0.55;
    --m3e-filled-button-disabled-label-text-color: #ffffff;
    color: #fff;
  }
  .last-chip {
    margin-left: 4px;
    padding: 3px 9px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.02em;
    background: rgb(50 113 254 / 16%);
    color: #3271fe;
  }
  .discord .logo {
    color: #fff;
  }
  .discord .last-chip {
    background: rgb(255 255 255 / 22%);
    color: #fff;
  }
  .last-chip.in-field {
    margin-right: 10px;
    color: #b9ccff;
  }
  .spin {
    --m3e-loading-indicator-size: 22px;
    display: inline-grid;
  }
  .google .spin {
    --m3e-loading-indicator-color: #3271fe;
  }
  .discord .spin {
    --m3e-loading-indicator-color: #fff;
  }

  .or {
    display: flex;
    align-items: center;
    gap: 14px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .or::before,
  .or::after {
    content: "";
    flex: 1;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgb(255 255 255 / 14%), transparent);
  }
  .email {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .send {
    --m3e-button-medium-container-height: 52px;
    --m3e-button-medium-label-text-font-size: 15px;
    --m3e-button-medium-label-text-font-weight: 600;
  }

  /* The Turnstile box unfolds only when Cloudflare needs a click. */
  .check {
    display: grid;
    grid-template-rows: 0fr;
    opacity: 0;
    transition:
      grid-template-rows 420ms cubic-bezier(0.2, 0.8, 0.2, 1),
      opacity 300ms ease;
  }
  .check.show {
    grid-template-rows: 1fr;
    opacity: 1;
  }
  .check-in {
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .check.show .check-in {
    padding: 14px;
    border-radius: 20px;
    background: rgb(255 255 255 / 4%);
    border: 1px solid rgb(255 255 255 / 8%);
  }
  .check-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 600;
    color: #b9ccff;
    --m3e-icon-size: 18px;
  }

  .view {
    /* app.css pads every section for the landing page; not here. */
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
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
  @media (max-width: 960px) {
    .stage {
      grid-template-columns: minmax(0, 1fr);
      justify-items: center;
    }
    .card {
      max-width: 480px;
    }
    .art {
      display: none;
    }
  }
  @media (max-width: 480px) {
    .stage {
      padding: 12px;
      align-items: start;
    }
    .card {
      padding: 22px 18px;
      border-radius: 28px;
    }
    .dest {
      font-size: 12px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .face,
    .frame,
    .viewport,
    .check,
    .provider {
      transition: none;
    }
    .aurora span,
    .frame,
    .sheen,
    .art,
    .card {
      animation: none;
    }
  }
</style>
