<script lang="ts">
  import { CONTACT_EMAIL } from "../lib/contact";
  import { inview } from "./inview";
  import Note from "./Note.svelte";
  import PageShell from "./PageShell.svelte";
  import Related from "./Related.svelte";
  import Toc from "./Toc.svelte";

  const TEMPLATE = `To: ${CONTACT_EMAIL}
Subject: Copyright notice

1. The work: [what the copyrighted work is, with a link to an authorised copy if you can]
2. The material: [exactly where the material is, with its address (URL). If it is on Velmora Social, give the profile, review or list link]
3. My details: [your name, company, address, email and phone number]
4. Good faith: I have a good-faith belief that the use described above is not authorised by the copyright owner, its agent or the law.
5. Accuracy: The information in this notice is accurate, and I am the copyright owner or authorised to act on the owner's behalf.
6. Signature: [your full name, typed as a signature]`;

  let copied = $state(false);
  async function copy() {
    try {
      await navigator.clipboard.writeText(TEMPLATE);
      copied = true;
      setTimeout(() => (copied = false), 2200);
    } catch {
      // Clipboard blocked: the text is on the page to copy by hand.
    }
  }

  const CREDITS = [
    ["TMDB", "This product uses the TMDB API but is not endorsed or certified by TMDB."],
    ["AniList and Trakt", "Used for optional sync, through their public APIs. Velmora isn't affiliated with either."],
    ["YouTube and Google", "Music uses YouTube. Velmora isn't affiliated with or endorsed by Google or YouTube."],
    ["mpv and FFmpeg", "The video player and the download tools are open-source projects, fetched on first launch and checked with SHA-256."],
  ];
</script>

<PageShell
  eyebrow="Disclaimer and DMCA"
  title="What Velmora does,"
  accent="and doesn't."
  crumb="Disclaimer and DMCA"
  lead="Velmora hosts no video. It searches third-party sites and plays what they serve. This page says that plainly, and says how to send a copyright notice."
  spots={[
    { x: "92%", y: "190px", size: 108, color: "#ff8a65", spin: 24, opacity: 0.5 },
    { x: "5%", y: "380px", size: 52, color: "#8fb4ff", spin: -14, offset: 5, opacity: 0.65 },
  ]}
>
  <div class="pg-wide pg-split body">
    <Toc target="doc" />
    <article id="doc" class="pg-prose">
      <section class="pg-section first" id="summary">
        <Note label="In short" icon="gavel" variant="primary">
          Velmora doesn't host, upload or store any movies, shows, channels or streams. It's responsible for the app and for what people post on Velmora Social. The sites it searches are responsible for what they serve, and you're responsible for what you watch and for following the law where you live.
        </Note>
      </section>

      <section class="pg-section" id="what">
        <h2 class="pg-title" use:inview>What Velmora <em>is</em></h2>
        <p>Velmora is a free, open-source app. It searches third-party websites the way a browser does, ranks the links it finds, and plays one in its built-in player. It has no catalogue of its own and doesn't control what those websites offer, so what you can watch varies, and some links won't play.</p>
        <p>Sports listings name official broadcasters only. Live TV channels come from a public playlist that Velmora doesn't control, and some may not play.</p>
      </section>

      <section class="pg-section" id="you">
        <h2 class="pg-title" use:inview>Your <em>responsibility</em></h2>
        <p>You're responsible for what you watch and download, and for following the law where you live. Copyright rules differ from country to country, and Velmora can't tell you what is allowed where you are. Please use it only for content you have the right to watch. The <a href="/terms/">terms of use</a> cover the rest, including the no-warranty clause.</p>
      </section>

      <section class="pg-section" id="credits">
        <h2 class="pg-title" use:inview>Names and <em>credits</em></h2>
        <p>All trademarks, names and logos belong to their owners. Velmora isn't affiliated with, endorsed by or sponsored by any streaming service, studio or broadcaster, and movie and show names on this website are for illustration.</p>
        <dl>
          {#each CREDITS as [name, text] (name)}
            <div use:inview><dt>{name}</dt><dd>{text}</dd></div>
          {/each}
        </dl>
      </section>

      <section class="pg-section" id="dmca">
        <h2 class="pg-title" use:inview>Copyright <em>notices</em></h2>
        <p>If you own a copyright and believe something connected to Velmora infringes it, tell us. First, it helps to know where the material actually is:</p>
        <ul>
          <li><strong>A video on a third-party website.</strong> Velmora doesn't host it, so the website that does is the one that can remove it. Please send your notice to that website or its host. If you tell us about the site, we'll review it, and we may stop using it as a source.</li>
          <li><strong>Something on Velmora Social.</strong> Profiles, pictures, bios, links, reviews and lists are posted by members. We can remove them, and we can ban an account that keeps posting infringing material.</li>
          <li><strong>Something on this website or in the app.</strong> Our own pages, text and images. Tell us and we'll look at it.</li>
        </ul>

        <h3>What to include</h3>
        <ol>
          <li>The copyrighted work you say is infringed.</li>
          <li>Exactly where the material is, with its address (URL).</li>
          <li>Your name, address, email and phone number.</li>
          <li>A statement that you have a good-faith belief the use isn't authorised by the copyright owner, its agent or the law.</li>
          <li>A statement that the information in the notice is accurate, and that you're the owner or authorised to act for the owner.</li>
          <li>Your signature, typed or written.</li>
        </ol>

        <div class="send" use:inview>
          <div>
            <strong>Send it to</strong>
            <a class="mail" href="mailto:{CONTACT_EMAIL}?subject=Copyright%20notice">{CONTACT_EMAIL}</a>
          </div>
          <m3e-button variant="tonal" onclick={copy}>
            <m3e-icon slot="icon" name={copied ? "check" : "content_copy"}></m3e-icon>{copied ? "Copied" : "Copy a notice template"}
          </m3e-button>
        </div>
        <pre class="tpl">{TEMPLATE}</pre>

        <h3>What happens next</h3>
        <p>We read every notice that has the details above. If it's about something we control, we'll remove it and tell the person who posted it. If you think something was removed by mistake, write to the same address and explain, and we'll look again. Notices that are missing details, or that are about content on other sites, may get a reply asking for more, or pointing you to the site that hosts it.</p>
        <Note label="This page isn't legal advice" icon="info" variant="info">
          It explains how Velmora handles notices. It doesn't say what your rights or ours are in your country. If you're unsure, ask a lawyer.
        </Note>
      </section>
    </article>
  </div>

  <Related
    items={[
      { href: "/terms/", icon: "description", title: "Terms of use", text: "The rules for using Velmora, including no warranty." },
      { href: "/privacy/", icon: "privacy_tip", title: "Privacy policy", text: "What's kept, who gets it and why." },
      { href: "/contact/", icon: "forum", title: "Contact", text: "Bugs, ideas, security reports and how to help." },
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
    padding-top: 36px;
  }
  h2 {
    margin-bottom: 14px;
  }
  h3 {
    margin: 28px 0 10px;
    font-size: 18px;
    color: var(--ink);
  }
  p {
    margin: 0 0 14px;
  }
  ul,
  ol {
    padding-left: 22px;
    display: grid;
    gap: 10px;
    margin: 0 0 14px;
  }
  dl {
    display: grid;
    gap: 10px;
    margin: 20px 0 0;
  }
  dl div {
    padding: 16px 20px;
    border-radius: 20px;
    border: 1px solid var(--line);
    background: var(--glass);
  }
  dt {
    font-weight: 700;
    color: var(--ink);
    margin-bottom: 4px;
  }
  dd {
    margin: 0;
    color: var(--muted);
    font-size: 15px;
    line-height: 1.6;
  }
  .send {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    margin: 26px 0 14px;
    padding: 20px 24px;
    border-radius: 24px;
    border: 1px solid color-mix(in srgb, var(--blue) 40%, transparent);
    background: color-mix(in srgb, var(--blue) 10%, transparent);
  }
  .send strong {
    display: block;
    font-size: 13px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 4px;
  }
  .mail {
    font-size: 19px;
    font-weight: 700;
    color: var(--cream);
    text-decoration: none;
    word-break: break-all;
  }
  .tpl {
    margin: 0 0 14px;
    padding: 20px 22px;
    border-radius: 20px;
    border: 1px solid var(--line);
    background: rgb(0 0 0 / 28%);
    color: var(--muted);
    font-family: "JetBrains Mono Variable", ui-monospace, monospace;
    font-size: 13px;
    line-height: 1.65;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
