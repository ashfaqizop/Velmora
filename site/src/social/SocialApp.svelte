<script lang="ts">
  // Velmora Social. Phase 1 is the shell: sign in on the web, see your own
  // profile card. Profiles, follows, the feed and trending arrive next.
  import { onMount } from "svelte";
  import { fly, fade } from "svelte/transition";
  import mark from "../assets/velmora-mark.webp";
  import Avatar from "../auth/Avatar.svelte";
  import { MAIN_SITE } from "../auth/config";
  import { beginSignIn, restoreSession, signOut, supabase } from "../lib/session";
  import { linkLabel, shortUrl, type ProfileLink } from "../lib/links";

  const SOCIAL_HOST = "social.getvelmora.dpdns.org";

  interface Profile {
    id: string;
    username: string;
    display_name: string;
    avatar: string;
    bio: string;
    links: ProfileLink[];
    verified: boolean;
    followers_count: number;
    following_count: number;
    created_at: string;
  }

  let phase = $state<"loading" | "out" | "in" | "noprofile">("loading");
  let me = $state<Profile | null>(null);
  let visits = $state(0);
  let problem = $state<string | null>(null);
  let busy = $state(false);

  const since = $derived(me ? new Date(me.created_at).toLocaleDateString(undefined, { month: "long", year: "numeric" }) : "");

  onMount(() => {
    const host = location.hostname;
    if (host !== SOCIAL_HOST && host !== "localhost" && host !== "127.0.0.1") {
      location.replace(`https://${SOCIAL_HOST}/social/${location.search}`);
      return;
    }
    load();
  });

  async function load() {
    try {
      const session = await restoreSession();
      if (!session) {
        phase = "out";
        return;
      }
      const { data, error } = await supabase.from("profiles").select("*").eq("id", session.user.id).maybeSingle();
      if (error) throw new Error(error.message);
      if (!data) {
        phase = "noprofile";
        return;
      }
      me = data as Profile;
      const v = await supabase.rpc("visits_this_month", { target: me.id });
      visits = typeof v.data === "number" ? v.data : 0;
      phase = "in";
    } catch (e) {
      problem = (e as Error).message;
      phase = "out";
    }
  }

  async function signIn() {
    busy = true;
    await beginSignIn();
  }

  async function leave() {
    await signOut();
    me = null;
    phase = "out";
  }

  const COMING = [
    { icon: "account_circle", shape: "12-sided-cookie", title: "Profiles", text: "Your picks, lists, reviews and stats on one page at @you." },
    { icon: "person_add", shape: "flower", title: "Follow anyone", text: "Follow people, see who follows you, and watch your profile grow." },
    { icon: "explore", shape: "soft-burst", title: "A feed", text: "What the people you follow are watching, finishing and adding." },
    { icon: "local_fire_department", shape: "sunny", title: "Trending", text: "Movies, shows, anime and music people play most, one click from the app." },
  ];
</script>

<div class="page">
  <div class="glow" aria-hidden="true"></div>

  <header class="bar">
    <a class="brand" href="/social/" aria-label="Velmora Social">
      <img src={mark} alt="" width="36" height="36" />
      <span>Velmora <em>Social</em></span>
    </a>
    <div class="actions">
      {#if phase === "in" && me}
        <m3e-button id="me-button" variant="text" class="me-button">
          <span slot="icon" class="mini"><Avatar value={me.avatar} size={28} /></span>
          {me.display_name}
          <m3e-icon slot="trailing-icon" name="keyboard_arrow_down"></m3e-icon>
          <m3e-menu-trigger for="me-menu"></m3e-menu-trigger>
        </m3e-button>
        <m3e-menu id="me-menu">
          <m3e-menu-item disabled>
            <m3e-icon slot="icon" name="account_circle"></m3e-icon>@{me.username}
          </m3e-menu-item>
          <m3e-divider></m3e-divider>
          <m3e-menu-item onclick={leave}>
            <m3e-icon slot="icon" name="logout"></m3e-icon>Sign out
          </m3e-menu-item>
        </m3e-menu>
      {:else if phase !== "loading"}
        <m3e-button variant="filled" disabled={busy} onclick={signIn}>
          <m3e-icon slot="icon" name="login"></m3e-icon>Sign in
        </m3e-button>
      {/if}
    </div>
  </header>

  <main class="wrap">
    <section class="hero" in:fade={{ duration: 300 }}>
      <span class="eyebrow">Opening soon</span>
      <h1 class="display">Velmora, <em>together.</em></h1>
      <p class="lead">Profiles, follows and what everyone on Velmora is watching. No messages, no noise: just people and what they love.</p>
    </section>

    {#if phase === "loading"}
      <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
    {:else if phase === "in" && me}
      <article class="profile" in:fly={{ y: 16, duration: 320 }}>
        <div class="cover" style:--tint="var(--md-sys-color-primary)" aria-hidden="true"></div>
        <div class="who">
          <Avatar value={me.avatar} size={112} ring={4} />
          <div class="names">
            <h2>
              {me.display_name}
              {#if me.verified}<m3e-icon name="verified" filled class="tick" aria-label="Verified"></m3e-icon>{/if}
            </h2>
            <span class="handle">@{me.username}</span>
          </div>
        </div>

        <div class="counts">
          <div><strong>{me.followers_count}</strong><span>Followers</span></div>
          <div><strong>{me.following_count}</strong><span>Following</span></div>
          <div><strong>{visits}</strong><span>Visits this month</span></div>
        </div>

        {#if me.bio}<p class="bio">{me.bio}</p>{/if}

        {#if me.links?.length}
          <m3e-chip-set class="links">
            {#each me.links as l (l.url)}
              <m3e-assist-chip href={l.url} target="_blank" rel="noopener noreferrer nofollow">
                <m3e-icon slot="icon" name="link"></m3e-icon>{linkLabel(l.kind)} · {shortUrl(l.url)}
              </m3e-assist-chip>
            {/each}
          </m3e-chip-set>
        {/if}

        <p class="since">Member since {since}. Edit your name, picture, bio and links in the app, under Settings, Velmora account.</p>
      </article>
    {:else if phase === "noprofile"}
      <m3e-card variant="filled" class="note">
        <div slot="content">
          <strong>Your profile isn't finished.</strong>
          <p>Sign in again and complete the last step to choose a username.</p>
          <m3e-button variant="filled" onclick={signIn}>Finish my profile</m3e-button>
        </div>
      </m3e-card>
    {:else}
      <m3e-card variant="filled" class="note" in:fly={{ y: 12, duration: 280 }}>
        <div slot="content">
          <strong>Sign in to see your profile</strong>
          <p>Use the same account as in the Velmora app: Google, Discord or email.</p>
          {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
          <div class="row">
            <m3e-button variant="filled" disabled={busy} onclick={signIn}><m3e-icon slot="icon" name="login"></m3e-icon>Sign in</m3e-button>
            <m3e-button variant="text" href={MAIN_SITE}>Get the app</m3e-button>
          </div>
        </div>
      </m3e-card>
    {/if}

    <section class="coming">
      {#each COMING as c, i (c.title)}
        <m3e-card variant="outlined" class="tile" style:--delay="{i * 70}ms">
          <div slot="content">
            <span class="tile-icon" aria-hidden="true"><m3e-shape name={c.shape}></m3e-shape><m3e-icon name={c.icon}></m3e-icon></span>
            <strong>{c.title}</strong>
            <p>{c.text}</p>
          </div>
        </m3e-card>
      {/each}
    </section>
  </main>

  <footer class="foot">
    <a href={MAIN_SITE}>getvelmora.dpdns.org</a>
    <a href="{MAIN_SITE}/privacy/">Privacy</a>
    <a href="{MAIN_SITE}/terms/">Terms</a>
  </footer>
</div>

<style>
  .page {
    position: relative;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    overflow-x: clip;
    background: var(--md-sys-color-surface, #0a0d16);
    color: var(--md-sys-color-on-surface);
  }
  .glow {
    position: absolute;
    inset: -10% -10% auto;
    height: 720px;
    pointer-events: none;
    background:
      radial-gradient(40% 50% at 20% 10%, color-mix(in srgb, var(--md-sys-color-primary) 32%, transparent), transparent 70%),
      radial-gradient(30% 40% at 85% 0%, color-mix(in srgb, #f8e8c8 14%, transparent), transparent 70%);
  }
  .bar {
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px clamp(16px, 4vw, 40px);
    backdrop-filter: blur(16px);
    background: color-mix(in srgb, var(--md-sys-color-surface) 70%, transparent);
    border-bottom: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 40%, transparent);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    color: inherit;
    text-decoration: none;
    font-weight: 800;
    font-size: 18px;
  }
  .brand img {
    border-radius: 10px;
  }
  .brand em {
    font-family: var(--display);
    font-style: italic;
    font-weight: 400;
    color: #f8e8c8;
  }
  .mini {
    display: inline-grid;
  }
  .me-button {
    --m3e-button-label-text-color: var(--md-sys-color-on-surface);
  }
  .wrap {
    position: relative;
    width: min(980px, 100% - 32px);
    margin: 0 auto;
    padding: 56px 0 72px;
    display: flex;
    flex-direction: column;
    gap: 36px;
    flex: 1;
  }
  .hero {
    display: flex;
    flex-direction: column;
    gap: 14px;
    max-width: 680px;
  }
  .eyebrow {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
  }
  .display {
    margin: 0;
    font-family: var(--display);
    font-weight: 400;
    font-size: clamp(44px, 7vw, 76px);
    line-height: 1;
    letter-spacing: -0.01em;
  }
  .display em {
    color: #f8e8c8;
  }
  .lead {
    margin: 0;
    font-size: 18px;
    line-height: 1.6;
    color: var(--md-sys-color-on-surface-variant);
  }
  .center {
    display: grid;
    place-items: center;
    padding: 40px 0;
  }
  .profile {
    position: relative;
    overflow: hidden;
    border-radius: 32px;
    padding: 0 28px 28px;
    background: var(--md-sys-color-surface-container);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 40%, transparent);
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .cover {
    height: 120px;
    margin: 0 -28px;
    background:
      radial-gradient(60% 120% at 20% 0%, color-mix(in srgb, var(--tint) 55%, transparent), transparent 70%),
      linear-gradient(120deg, color-mix(in srgb, var(--tint) 25%, var(--md-sys-color-surface-container)), var(--md-sys-color-surface-container-high));
  }
  .who {
    display: flex;
    align-items: flex-end;
    gap: 18px;
    margin-top: -64px;
  }
  .names h2 {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 28px;
  }
  .tick {
    color: var(--md-sys-color-primary);
    --m3e-icon-size: 24px;
  }
  .handle {
    color: var(--md-sys-color-on-surface-variant);
  }
  .counts {
    display: flex;
    gap: 28px;
    flex-wrap: wrap;
  }
  .counts div {
    display: flex;
    flex-direction: column;
  }
  .counts strong {
    font-size: 22px;
    font-variant-numeric: tabular-nums;
  }
  .counts span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .bio {
    margin: 0;
    font-size: 16px;
    line-height: 1.6;
    white-space: pre-line;
  }
  .since {
    margin: 0;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .note div[slot="content"] {
    display: flex;
    flex-direction: column;
    gap: 10px;
    align-items: flex-start;
  }
  .note p {
    margin: 0;
    color: var(--md-sys-color-on-surface-variant);
  }
  .problem {
    color: var(--md-sys-color-error) !important;
  }
  .row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .coming {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: 14px;
  }
  .tile {
    animation: rise 520ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
    animation-delay: var(--delay);
  }
  .tile div[slot="content"] {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .tile p {
    margin: 0;
    font-size: 14px;
    line-height: 1.55;
    color: var(--md-sys-color-on-surface-variant);
  }
  .tile-icon {
    display: inline-grid;
    place-items: center;
    width: 52px;
    height: 52px;
    --m3e-shape-size: 52px;
    --m3e-shape-container-color: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
  }
  .tile-icon > * {
    grid-area: 1 / 1;
  }
  .foot {
    display: flex;
    gap: 20px;
    justify-content: center;
    padding: 24px 16px 32px;
    font-size: 13px;
  }
  .foot a {
    color: var(--md-sys-color-on-surface-variant);
    text-decoration: none;
  }
  .foot a:hover {
    color: var(--md-sys-color-on-surface);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(14px);
    }
  }
  @media (max-width: 600px) {
    .who {
      flex-direction: column;
      align-items: flex-start;
    }
    .profile {
      padding: 0 20px 22px;
    }
    .cover {
      margin: 0 -20px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .tile {
      animation: none;
    }
  }
</style>
