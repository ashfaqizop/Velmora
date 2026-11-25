<script lang="ts">
  import discord from "../assets/brands/discord.svg";
  import google from "../assets/brands/google.svg";
  import LogoTile from "./compare/LogoTile.svelte";
  import { inview } from "./inview";
  import Note from "./Note.svelte";
  import PageShell from "./PageShell.svelte";
  import Related from "./Related.svelte";
  import Steps from "./Steps.svelte";
  import Toc from "./Toc.svelte";

  type Method = "google" | "discord" | "email";
  let method = $state<Method>("google");

  const METHODS: { id: Method; label: string; icon: string; logo?: string; title: string }[] = [
    { id: "google", label: "Google", icon: "account_circle", logo: google, title: "Sign in with Google" },
    { id: "discord", label: "Discord", icon: "forum", logo: discord, title: "Sign in with Discord" },
    { id: "email", label: "Emailed code", icon: "mail", title: "Sign in with an emailed code" },
  ];

  const HOW: Record<Method, { title: string; text: string }[]> = {
    google: [
      { title: "Choose Continue with Google", text: "Pick your Google account in the window your browser opens. Velmora gets the name, email address and photo you share with it." },
      { title: "Pick a username", text: "You'll be offered one based on your name. Change it if you like. It's how people find you on Velmora Social." },
      { title: "Back to the app", text: "Your browser hands you back to Velmora, which signs in by itself. With Google you can also bring your YouTube Music into the app." },
    ],
    discord: [
      { title: "Choose Continue with Discord", text: "Approve the request in the window your browser opens. Velmora gets your name, email address and avatar from Discord." },
      { title: "Pick a username", text: "You'll be offered one based on your Discord name. Change it if you like." },
      { title: "Back to the app", text: "Your browser hands you back to Velmora, which signs in by itself." },
    ],
    email: [
      { title: "Type your email address", text: "Velmora sends you a code. There's no password to make or remember." },
      { title: "Type the code from the email", text: "If it doesn't arrive in a minute or two, check your spam folder, or use Google or Discord instead." },
      { title: "Pick a username", text: "Choose how you'll appear on Velmora Social, then you're back in the app, signed in." },
    ],
  };

  const FIELDS = [
    { icon: "alternate_email", title: "Username", text: "3 to 20 letters, numbers or underscores. You can change it once every 30 days." },
    { icon: "badge", title: "Display name", text: "Up to 40 characters, and you can change it whenever you like." },
    { icon: "face", title: "Picture", text: "One of nine animals, or your Google or Discord photo. There are no uploads." },
    { icon: "edit_note", title: "Bio and links", text: "A bio of up to 160 characters, and links to your own pages." },
    { icon: "favorite", title: "Favourites", text: "Show up to six titles on your profile, picked from your list." },
  ];
</script>

<PageShell
  eyebrow="Your account"
  title="One free account,"
  accent="every PC."
  crumb="Create your account"
  lead="You make it once, in your browser. Your list, progress and settings follow you, and Velmora keeps working offline."
  spots={[
    { x: "91%", y: "190px", size: 120, color: "#8fb4ff", spin: 22, opacity: 0.55 },
    { x: "5%", y: "420px", size: 54, color: "#f8e8c8", spin: -14, offset: 4, opacity: 0.7 },
  ]}
>
  <div class="pg-wide pg-split body">
    <Toc target="doc" />
    <article id="doc" class="pg-prose">
      <section class="pg-section first" id="why">
        <h2 class="pg-title" use:inview>Why there's <em>an account</em></h2>
        <p>From Velmora 0.6.0 you need a free account. It gives you a username and profile, keeps your list, progress and settings in step across PCs, and brings Velmora Max with you to a new PC. You make it once. Only that first sign-in needs the internet. After it, Velmora opens offline too.</p>
      </section>

      <section class="pg-section" id="make">
        <h2 class="pg-title" use:inview>Make <em>yours</em></h2>
        <p>Open Velmora and choose to sign in. Your browser opens Velmora's sign-in page at <code>auth.getvelmora.dpdns.org</code>. Pick how you'd like to sign in:</p>
        <m3e-tabs class="tabs" aria-label="Sign-in method">
          {#each METHODS as m (m.id)}
            <m3e-tab selected={method === m.id} onclick={() => (method = m.id)}>
              {#if m.logo}<span slot="icon" class="tabLogo"><LogoTile src={m.logo} name={m.label} size={26} /></span>{:else}<m3e-icon slot="icon" name={m.icon}></m3e-icon>{/if}{m.label}
            </m3e-tab>
          {/each}
        </m3e-tabs>
        {#each METHODS as m (m.id)}
          <div class="panel" class:on={method === m.id} hidden={method !== m.id}>
            <div class="ptitle">
              {#if m.logo}<LogoTile src={m.logo} name={m.label} size={48} />{:else}<span class="mailTile"><m3e-icon name="mail" filled></m3e-icon></span>{/if}
              <h3>{m.title}</h3>
            </div>
            <Steps steps={HOW[m.id]} />
          </div>
        {/each}
      </section>

      <section class="pg-section" id="profile">
        <h2 class="pg-title" use:inview>Your <em>profile</em></h2>
        <p>These are the things you can set. Everything except your email is shown on your public profile on <a href="https://social.getvelmora.dpdns.org" target="_blank" rel="noopener">Velmora Social</a>.</p>
        <div class="fields">
          {#each FIELDS as f, i (f.title)}
            <div class="pg-card" use:inview={i * 0.05}>
              <m3e-icon name={f.icon}></m3e-icon>
              <h3>{f.title}</h3>
              <p>{f.text}</p>
            </div>
          {/each}
        </div>
      </section>

      <section class="pg-section" id="sync">
        <h2 class="pg-title" use:inview>What <em>syncs</em></h2>
        <p>Your list, your progress in every episode, and your settings follow your account. Sign in on a new PC and they're there. AniList and Trakt are connected separately, in Settings.</p>
        <p>If you have Velmora Max, linking the key to your account means signing in on a new PC brings Max along. The old PC drops back to Public.</p>
      </section>

      <section class="pg-section" id="privacy">
        <h2 class="pg-title" use:inview>Your <em>privacy</em></h2>
        <p>Your account stores your email, username, display name and picture, bio and links, and the list, progress and settings that sync. Velmora Social shows what you watch to the people who follow you, and that is on by default. One switch in Settings, under Privacy, turns it off, and adult titles are never posted.</p>
        <p>The <a href="/privacy/">privacy policy</a> has the full list of what's kept and why.</p>
        <Note label="Copy or delete everything" icon="delete_sweep" variant="info">
          In Velmora, open Settings, then Velmora account. You can copy everything Velmora holds about you, or delete the account and all of it. Deleting can't be undone.
        </Note>
      </section>

      <section class="pg-section" id="help">
        <h2 class="pg-title" use:inview>If you <em>can't sign in</em></h2>
        <p>If the emailed code doesn't arrive, check your spam folder, or sign in with Google or Discord. If something else is wrong, <a href="/contact/">tell us</a> what you saw.</p>
      </section>
    </article>
  </div>

  <Related
    items={[
      { href: "/docs/install/", icon: "menu_book", title: "Install guide", text: "Get Velmora onto your PC first." },
      { href: "/privacy/", icon: "privacy_tip", title: "Privacy policy", text: "What's kept, who gets it and why." },
      { href: "/faq/", icon: "help", title: "FAQ", text: "Short answers about accounts, Max and sync." },
    ]}
  />
</PageShell>

<style>
  .body {
    margin-top: 12px;
  }
  section {
    scroll-margin-top: 96px;
  }
  section.first {
    padding-top: 44px;
  }
  h2 {
    margin-bottom: 14px;
  }
  p {
    margin: 0 0 14px;
  }
  code {
    padding: 2px 8px;
    border-radius: 8px;
    background: rgb(255 255 255 / 8%);
    color: var(--cream);
    font-size: 0.9em;
  }
  .tabs {
    margin-top: 18px;
  }
  .panel {
    animation: swap 0.45s cubic-bezier(0.2, 0.7, 0.2, 1) both;
  }
  .ptitle {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-top: 26px;
  }
  .ptitle h3 {
    margin: 0;
    font-size: 21px;
    color: var(--ink);
  }
  .tabLogo {
    display: inline-grid;
  }
  .mailTile {
    display: grid;
    place-items: center;
    width: 48px;
    height: 48px;
    border-radius: 14px;
    background: color-mix(in srgb, var(--blue) 25%, var(--bg));
    color: var(--cream);
    --m3e-icon-size: 26px;
  }
  .panel[hidden] {
    display: none;
  }
  @keyframes swap {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
  }
  .fields {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 14px;
    margin-top: 22px;
  }
  .pg-card m3e-icon {
    color: var(--blue);
    --m3e-icon-size: 26px;
  }
  @media (prefers-reduced-motion: reduce) {
    .panel {
      animation: none;
    }
  }
</style>
