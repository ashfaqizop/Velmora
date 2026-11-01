<script lang="ts">
  import { screen } from "./lib/media.svelte";
  import { onMount } from "svelte";
  import Backdrop from "./components/Backdrop.svelte";
  import Nav from "./components/Nav.svelte";
  import Footer from "./components/Footer.svelte";
  import Ad from "./components/Ad.svelte";
  import Shapes from "./components/Shapes.svelte";
  import DownloadButton from "./components/DownloadButton.svelte";
  import { latest } from "./lib/latest.svelte";
  import { reveal } from "./lib/reveal";
  import Dots from "./components/Dots.svelte";

  onMount(() => latest.load());

  const KEY_URL = "https://aistudio.google.com/apikey";

  const ASK = [
    ["movie", "Something like Low Tide, but shorter?"],
    ["format_list_numbered", "What order should I watch the whole franchise in?"],
    ["download", "Smallest 1080p download of episode 4, with English subtitles."],
    ["calendar_month", "What's airing this week, in my time zone?"],
    ["person", "Who plays the detective, and what's she in next?"],
    ["compare_arrows", "Paper Moons or Glass Orchard for tonight?"],
    ["history", "Where did I stop in Echo Valley?"],
    ["trending_up", "What's trending in anime right now?"],
  ];

  const STAGES = [
    {
      icon: "psychology",
      title: "It thinks it over",
      text: "Ani works out what you're really asking and what it needs to know first. You can watch its thinking as it happens.",
    },
    {
      icon: "travel_explore",
      title: "It looks things up",
      text: "A fast model calls Velmora's own tools: search, episodes, ratings, schedules, cast, links and your history. 37 tools in all.",
    },
    {
      icon: "edit_note",
      title: "It writes the answer",
      text: "The main model writes the reply from what was found, in plain language, and adds cards made only from real results.",
    },
  ];

  const BLOCKS = [
    ["view_carousel", "Carousels", "Rows of posters with tabs, rank numbers and For you tags."],
    ["star", "Spotlight", "One big pick, with art, facts and buttons to play or save."],
    ["compare_arrows", "Compare", "Two or more titles side by side: length, ratings, tone."],
    ["format_list_numbered", "Watch order", "The order to watch a long franchise in, step by step."],
    ["video_library", "Episodes", "Seasons and episodes at a glance."],
    ["link", "Links", "Stream and download links with quality, size, audio and subtitles in clear words."],
    ["badge", "People", "Photo, age, what they're known for and what's coming next."],
    ["calendar_month", "Schedule", "What airs this week, in your own time zone."],
    ["donut_large", "Your taste", "The genres, kinds and decades you watch most."],
    ["movie", "Trailers and art", "Trailers to play and stills to browse."],
    ["checklist", "Choices", "Buttons for follow-up questions, so you can tap rather than type."],
    ["info", "Facts", "Runtime, studio, ratings and release dates at a glance."],
  ];

  const LEARNS = [
    ["search", "What you search for"],
    ["visibility", "Titles you open"],
    ["play_circle", "What you stream"],
    ["check_circle", "What you finish"],
    ["download", "What you download"],
    ["sticky_note_2", "Notes you ask it to keep, like \"I prefer dubs\""],
  ];

  const SETUP = [
    ["Get a free Gemini key", "Sign in to Google AI Studio and create an API key. The free tier is plenty for everyday use."],
    ["Turn Ani on", "In Velmora, open Settings, find Ani, switch it on and paste your key. Velmora checks the key before saving it."],
    ["Ask away", "Open Ani from the left menu and type a question, or tap one of the suggestions."],
  ];

  const FAQ = [
    [
      "Does Ani cost anything?",
      "Ani itself is free. It uses your own Google Gemini key, and Google's free tier covers normal use. If you set up billing on that key, Google charges you directly, not us.",
    ],
    [
      "What gets sent to Google?",
      "Your message, the last few turns of the chat, what Ani looked up for that answer, and a short summary of your taste. Nothing goes to a Velmora server, because there isn't one.",
    ],
    [
      "Can Ani play or download things by itself?",
      "No. Ani only answers. Its cards have Play, Save and Download buttons, but you press them.",
    ],
    [
      "Which models does it use?",
      "Gemini 3.5 Flash writes the answers and Gemini 3.1 Flash-Lite does the lookups. You can pick other models in Settings if your key has them.",
    ],
    [
      "How do I make it forget things?",
      "Settings, Ani, then Personalize shows everything Ani has learned. Clear it all or turn learning off. Deleting a chat removes it from your PC.",
    ],
  ];

  // Card rows that swipe sideways on phones.
  let asksTrack = $state<HTMLElement>();
  let stagesTrack = $state<HTMLElement>();
  let blocksTrack = $state<HTMLElement>();
  let setupTrack = $state<HTMLElement>();
</script>

<Backdrop />
<Nav />

<main id="top">
  <section class="hero">
    <Shapes
      spots={[
        { x: "5%", y: "180px", size: 70, color: "#ff5a5a", spin: 14, offset: 3 },
        { x: "52%", y: "130px", size: 46, color: "#ffb74d", spin: -10, offset: 6 },
        { x: "95%", y: "300px", size: 130, color: "#7b6cff", spin: -22 },
        { x: "48%", y: "88%", size: 88, color: "#c38bff", spin: 18, offset: 8, opacity: 0.6 },
      ]}
    />
    <div class="wrap grid">
      <div class="copy">
        <span class="on-shape mark" aria-hidden="true">
          <m3e-shape name="very-sunny"></m3e-shape><m3e-icon name="wand_stars" filled></m3e-icon>
        </span>
        <h1 class="display">Ask what to watch.<br /><em>Get answers you can play.</em></h1>
        <p class="sub">
          Ani is the assistant inside Velmora. It knows what you've watched, looks things up before it answers, and replies with cards you can play,
          save or download from.
        </p>
        <div class="ctas">
          <DownloadButton details={false} />
          <m3e-button variant="text" size={screen.big} href={KEY_URL} target="_blank" class="key">
            <m3e-icon slot="icon" name="key"></m3e-icon>Get a free Gemini key
          </m3e-button>
        </div>
        <span class="note">Optional and off by default. Runs on Google Gemini with your own key.</span>
      </div>

      <div class="demo" aria-hidden="true">
        <div class="msg me">Something like Low Tide, but shorter?</div>
        <div class="activity">
          <span class="dot"></span>
          <div>
            <strong>Looking things up</strong>
            <span class="trail">Low Tide · similar series · runtimes · your history</span>
          </div>
        </div>
        <div class="msg ani">
          Low Tide is slow and moody, so I went for short series with the same feel. <b>Echo Valley</b> is the closest: same writer, eight
          episodes, and you haven't started it.
        </div>
        <div class="spot">
          <div class="art"><span class="orb"></span><span class="ridge"></span></div>
          <div class="spot-text">
            <span class="kicker">For you</span>
            <strong class="display">Echo Valley</strong>
            <span class="facts">2025 · Mystery · 8 episodes · 4K</span>
            <div class="btns"><span class="play"><m3e-icon name="play_arrow" filled></m3e-icon>Play</span><span class="ghost">Save</span></div>
          </div>
        </div>
      </div>
    </div>
  </section>

  <section>
    <div class="wrap">
      <div use:reveal>
        <span class="eyebrow">Try asking</span>
        <h2 class="title">Ask it like <em>you'd ask a friend.</em></h2>
      </div>
      <div class="asks swipe" bind:this={asksTrack}>
        {#each ASK as [icon, q], i (q)}
          <div class="ask" use:reveal={(i % 4) * 0.06}>
            <m3e-icon name={icon}></m3e-icon>
            <span>{q}</span>
          </div>
      {#if screen.phone}<Dots track={asksTrack} labels={ASK.map((x) => x[1])} label="Example questions" />{/if}
        {/each}
      </div>
    </div>
  </section>

  <section>
    <div class="wrap">
      <div use:reveal>
        <span class="eyebrow">How it works</span>
        <h2 class="title">It checks <em>before it answers.</em></h2>
        <p class="lead">
          Ani doesn't guess from memory. Every title, link and date in its answer comes from a lookup it made for your question, so nothing in a card
          is made up.
        </p>
      </div>
      <ol class="stages swipe" bind:this={stagesTrack}>
        {#each STAGES as s, i (s.title)}
          <li use:reveal={i * 0.1}>
            <span class="num display">0{i + 1}</span>
            <span class="si"><m3e-icon name={s.icon}></m3e-icon></span>
            <strong>{s.title}</strong>
            <span class="text">{s.text}</span>
          </li>
        {/each}
      </ol>
      {#if screen.phone}<Dots track={stagesTrack} labels={STAGES.map((x) => x.title)} label="How it works" />{/if}
      <div class="live" use:reveal>
        <m3e-icon name="visibility"></m3e-icon>
        <span>While it works you see what it's doing: thinking, looking things up, planning, writing. Afterwards it says how long it thought and how many things it checked.</span>
      </div>
    </div>
  </section>

  <section>
    <div class="wrap">
      <div use:reveal>
        <span class="eyebrow">Answers you can use</span>
        <h2 class="title">Not just text. <em>Cards.</em></h2>
        <p class="lead">Ani picks from 23 kinds of cards to show its answer. These are some of them.</p>
      </div>
      <div class="blocks swipe" bind:this={blocksTrack}>
        {#each BLOCKS as [icon, name, text], i (name)}
          <article use:reveal={(i % 3) * 0.06}>
            <span class="bi"><m3e-icon name={icon}></m3e-icon></span>
            <strong>{name}</strong>
            <span>{text}</span>
          </article>
        {/each}
      </div>
      {#if screen.phone}<Dots track={blocksTrack} labels={BLOCKS.map((x) => x[1])} label="Card types" />{/if}
    </div>
  </section>

  <Ad kind="native" />

  <section>
    <div class="wrap split">
      <div use:reveal>
        <span class="eyebrow">Personalize</span>
        <h2 class="title">It learns your taste, <em>on your PC.</em></h2>
        <p class="lead">
          The more you use Velmora, the better Ani's picks get. It keeps a small record of what you watch, and it never leaves your PC except as a
          short summary sent with your question.
        </p>
        <p class="lead">
          You can see everything it has learned, clear it, or turn learning off in Settings. Tell it once that you prefer dubs or hate horror, and it
          remembers.
        </p>
      </div>
      <ul class="learns" use:reveal={0.1}>
        {#each LEARNS as [icon, text] (text)}
          <li><m3e-icon name={icon}></m3e-icon>{text}</li>
        {/each}
      </ul>
    </div>
  </section>

  <section>
    <div class="wrap">
      <div class="trust" use:reveal>
        <div>
          <m3e-icon name="key"></m3e-icon>
          <strong>Your key stays here</strong>
          <span>It's saved in Velmora's folder on your PC and is only ever sent to Google.</span>
        </div>
        <div>
          <m3e-icon name="back_hand"></m3e-icon>
          <strong>You press the buttons</strong>
          <span>Ani never plays, downloads or changes anything by itself.</span>
        </div>
        <div>
          <m3e-icon name="toggle_off"></m3e-icon>
          <strong>Off until you say so</strong>
          <span>Ani is opt-in. With it off, nothing is sent anywhere.</span>
        </div>
      </div>
    </div>
  </section>

  <section id="setup">
    <div class="wrap">
      <div use:reveal>
        <span class="eyebrow">Set up</span>
        <h2 class="title">Three steps, <em>two minutes.</em></h2>
      </div>
      <ol class="setup swipe" bind:this={setupTrack}>
        {#each SETUP as [title, text], i (title)}
          <li use:reveal={i * 0.08}>
            <span class="n display">{i + 1}</span>
            <strong>{title}</strong>
            <span class="text">{text}</span>
          </li>
        {/each}
      </ol>
      {#if screen.phone}<Dots track={setupTrack} labels={SETUP.map((x) => x[0])} label="Setup steps" />{/if}
    </div>
  </section>

  <section>
    <div class="wrap narrow">
      <div use:reveal>
        <span class="eyebrow">Questions</span>
        <h2 class="title">About <em>Ani.</em></h2>
      </div>
      <m3e-accordion class="faq" use:reveal={0.1}>
        {#each FAQ as [q, a] (q)}
          <m3e-expansion-panel>
            <span slot="header" class="q">{q}</span>
            <p>{a}</p>
          </m3e-expansion-panel>
        {/each}
      </m3e-accordion>
    </div>
  </section>

  <section>
    <div class="wrap">
      <div class="end" use:reveal>
        <Shapes
          spots={[
            { x: "10%", y: "70%", size: 120, color: "#7b6cff", spin: 20 },
            { x: "90%", y: "30%", size: 100, color: "#ff5a5a", spin: -16, offset: 5 },
          ]}
        />
        <span class="on-shape mark small" aria-hidden="true">
          <m3e-shape name="very-sunny"></m3e-shape><m3e-icon name="wand_stars" filled></m3e-icon>
        </span>
        <h2 class="title">Ani is waiting <em>in Velmora.</em></h2>
        <p class="lead">Free, and it comes with the app. Turn it on whenever you like.</p>
        <DownloadButton />
      </div>
    </div>
  </section>
</main>

<Footer />

<style>
  .hero {
    padding-top: 150px;
  }
  .grid {
    display: grid;
    grid-template-columns: minmax(0, 1.05fr) minmax(0, 1fr);
    gap: 64px;
    align-items: center;
  }
  .mark {
    --m3e-shape-size: 72px;
    --m3e-shape-container-color: var(--indigo);
    color: #fff;
    --m3e-icon-size: 34px;
    animation: in 0.9s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .mark m3e-shape {
    animation: spin 24s linear infinite;
  }
  .mark m3e-icon {
    z-index: 1;
  }
  .mark.small {
    --m3e-shape-size: 56px;
    --m3e-icon-size: 26px;
    margin-bottom: 8px;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  h1 {
    margin: 28px 0 0;
    font-size: clamp(40px, 10vw, 88px);
    line-height: 0.98;
    letter-spacing: -0.02em;
    animation: in 1s 0.1s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  h1 em {
    font-style: italic;
    background: linear-gradient(100deg, #ff8a7a, var(--coral) 30%, #c38bff 65%, var(--indigo));
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
    padding-right: 0.08em;
  }
  .sub {
    margin: 24px 0 0;
    max-width: 560px;
    font-size: 19px;
    line-height: 1.6;
    color: var(--muted);
    animation: in 1s 0.25s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .ctas {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    margin-top: 36px;
    animation: in 1s 0.4s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .key {
    --m3e-text-button-label-text-color: var(--ink);
    --m3e-text-button-icon-color: var(--ink);
  }
  .note {
    display: block;
    margin-top: 16px;
    font-size: 13px;
    color: var(--muted);
  }
  @keyframes in {
    from {
      opacity: 0;
      transform: translateY(24px);
    }
  }

  /* The chat demo on the right. */
  .demo {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 26px;
    border-radius: 32px;
    border: 1px solid var(--line);
    background:
      radial-gradient(90% 60% at 0% 0%, rgb(123 108 255 / 24%), transparent 60%),
      radial-gradient(70% 50% at 100% 100%, rgb(255 90 90 / 16%), transparent 60%),
      #15111f;
    box-shadow: 0 40px 100px -40px rgb(123 108 255 / 55%);
    font-size: 14px;
    animation: in 1.1s 0.5s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .msg {
    max-width: 88%;
    padding: 12px 16px;
    border-radius: 20px;
    line-height: 1.55;
  }
  .me {
    align-self: flex-end;
    background: var(--indigo);
    color: #fff;
    border-bottom-right-radius: 6px;
  }
  .ani {
    background: rgb(255 255 255 / 6%);
    border-bottom-left-radius: 6px;
  }
  .activity {
    display: flex;
    gap: 12px;
    align-items: center;
    padding: 10px 14px;
    border-radius: 16px;
    background: rgb(255 255 255 / 4%);
    font-size: 13px;
  }
  .activity div {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .activity strong {
    background: linear-gradient(90deg, var(--muted) 30%, #fff 50%, var(--muted) 70%);
    background-size: 200% 100%;
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
    animation: sweep 2.4s linear infinite;
  }
  @keyframes sweep {
    from {
      background-position: 100% 0;
    }
    to {
      background-position: -100% 0;
    }
  }
  .trail {
    color: var(--muted);
    font-size: 12px;
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--indigo);
    box-shadow: 0 0 0 0 rgb(123 108 255 / 60%);
    animation: pulse 1.6s infinite;
  }
  @keyframes pulse {
    to {
      box-shadow: 0 0 0 10px rgb(123 108 255 / 0%);
    }
  }
  .spot {
    position: relative;
    height: 200px;
    border-radius: 20px;
    overflow: hidden;
  }
  .art {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(100% 90% at 80% 10%, #a78bfa, transparent 50%),
      linear-gradient(180deg, #1f2a55, #0d0b1c);
  }
  .orb {
    position: absolute;
    right: 18%;
    top: 18%;
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: radial-gradient(circle at 40% 40%, #f3efff, #b9a6ff 60%, #7b6cff);
    box-shadow: 0 0 60px 14px rgb(167 139 250 / 45%);
  }
  .ridge {
    position: absolute;
    inset: auto 0 0;
    height: 55%;
    background: #10122a;
    clip-path: polygon(0 55%, 18% 30%, 34% 52%, 52% 18%, 68% 46%, 84% 26%, 100% 48%, 100% 100%, 0 100%);
  }
  .spot-text {
    position: absolute;
    inset: auto 0 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 18px;
    background: linear-gradient(0deg, rgb(10 8 20 / 85%), transparent);
  }
  .kicker {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: #ffd8e4;
  }
  .spot-text strong {
    font-size: 32px;
    line-height: 1;
  }
  .facts {
    font-size: 11px;
    color: var(--muted);
  }
  .btns {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }
  .play,
  .ghost {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 14px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    --m3e-icon-size: 18px;
  }
  .play {
    background: #fff;
    color: #140f22;
    padding-left: 10px;
  }
  .ghost {
    background: rgb(255 255 255 / 12%);
  }

  /* Example questions. */
  .asks {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 14px;
    margin-top: 48px;
  }
  .ask {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 22px;
    border-radius: 24px;
    border: 1px solid var(--line);
    background: var(--glass);
    font-size: 16px;
    line-height: 1.45;
    --m3e-icon-size: 22px;
    transition:
      border-color 0.3s,
      transform 0.4s cubic-bezier(0.2, 0.7, 0.2, 1),
      opacity 0.9s cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .ask:hover {
    border-color: rgb(123 108 255 / 50%);
    transform: translateY(-3px);
  }
  .ask m3e-icon {
    color: #c9c1ff;
  }

  /* The three stages. */
  .stages {
    list-style: none;
    margin: 56px 0 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 16px;
  }
  .stages li {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 30px 28px;
    border-radius: 28px;
    border: 1px solid var(--line);
    background: var(--glass);
    overflow: hidden;
  }
  .num {
    position: absolute;
    right: 20px;
    top: 6px;
    font-size: 88px;
    color: rgb(255 255 255 / 5%);
  }
  .si {
    display: grid;
    place-items: center;
    width: 52px;
    height: 52px;
    border-radius: 16px;
    background: linear-gradient(135deg, var(--coral), var(--indigo));
    color: #fff;
    --m3e-icon-size: 26px;
    margin-bottom: 8px;
  }
  .stages strong {
    font-family: var(--display);
    font-weight: 400;
    font-size: 28px;
  }
  .text {
    font-size: 15px;
    line-height: 1.6;
    color: var(--muted);
  }
  .live {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-top: 16px;
    padding: 18px 24px;
    border-radius: 20px;
    background: rgb(123 108 255 / 10%);
    border: 1px solid rgb(123 108 255 / 25%);
    font-size: 15px;
    line-height: 1.55;
    color: var(--muted);
    --m3e-icon-size: 22px;
  }
  .live m3e-icon {
    flex: none;
    color: #c9c1ff;
  }

  /* Card kinds. */
  .blocks {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 14px;
    margin-top: 48px;
  }
  .blocks article {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    grid-template-rows: auto auto;
    column-gap: 16px;
    row-gap: 4px;
    padding: 22px;
    border-radius: 24px;
    border: 1px solid var(--line);
    background: var(--glass);
  }
  .bi {
    grid-row: span 2;
    display: grid;
    place-items: center;
    width: 46px;
    height: 46px;
    border-radius: 14px;
    background: rgb(123 108 255 / 16%);
    color: #c9c1ff;
    --m3e-icon-size: 22px;
  }
  .blocks strong {
    font-size: 16px;
  }
  .blocks span:last-child {
    font-size: 14px;
    line-height: 1.55;
    color: var(--muted);
  }

  /* Personalize. */
  .split {
    display: grid;
    grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr);
    gap: 64px;
    align-items: center;
  }
  .learns {
    list-style: none;
    margin: 0;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    border-radius: 32px;
    border: 1px solid var(--line);
    background: linear-gradient(180deg, #17122a, #0f0c18);
  }
  .learns li {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px 18px;
    border-radius: 20px;
    font-size: 16px;
    --m3e-icon-size: 22px;
  }
  .learns li:nth-child(odd) {
    background: rgb(255 255 255 / 3%);
  }
  .learns m3e-icon {
    color: #c9c1ff;
  }

  /* Trust strip. */
  .trust {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 40px;
    padding: 48px;
    border-radius: 36px;
    border: 1px solid var(--line);
    background:
      radial-gradient(50% 80% at 10% 0%, rgb(123 108 255 / 18%), transparent 70%),
      radial-gradient(40% 80% at 95% 100%, rgb(255 90 90 / 12%), transparent 70%),
      #120e1d;
  }
  .trust div {
    display: flex;
    flex-direction: column;
    gap: 8px;
    --m3e-icon-size: 28px;
  }
  .trust m3e-icon {
    color: #c9c1ff;
    margin-bottom: 6px;
  }
  .trust strong {
    font-size: 17px;
  }
  .trust span {
    font-size: 14.5px;
    line-height: 1.55;
    color: var(--muted);
  }

  /* Setup steps. */
  .setup {
    list-style: none;
    margin: 48px 0 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 16px;
  }
  .setup li {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 28px 24px;
    border-radius: 28px;
    border: 1px solid var(--line);
    background: var(--glass);
  }
  .n {
    font-size: 64px;
    line-height: 0.9;
    color: transparent;
    -webkit-text-stroke: 1.5px color-mix(in srgb, var(--indigo) 80%, transparent);
    margin-bottom: 12px;
  }
  .setup strong {
    font-size: 17px;
  }

  .narrow {
    max-width: 860px;
  }
  .faq {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 48px;
    --m3e-expansion-panel-container-color: rgb(255 255 255 / 4%);
    --m3e-expansion-panel-shape: 22px;
    --m3e-expansion-header-collapsed-height: auto;
    --m3e-expansion-header-expanded-height: auto;
  }
  .q {
    display: block;
    padding: 16px 0;
    font-size: 17px;
    font-weight: 600;
    line-height: 1.35;
    /* m3e cuts headers off with an ellipsis; long questions should wrap. */
    white-space: normal;
  }
  .faq p {
    margin: 0;
    font-size: 15.5px;
    line-height: 1.65;
    color: var(--muted);
  }

  .end {
    position: relative;
    isolation: isolate;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 88px 32px;
    border-radius: 44px;
    border: 1px solid rgb(255 255 255 / 10%);
    background:
      radial-gradient(60% 80% at 20% 0%, rgb(255 90 90 / 26%), transparent 60%),
      radial-gradient(60% 90% at 90% 100%, rgb(123 108 255 / 40%), transparent 60%),
      #140f22;
  }
  .end .lead {
    margin-bottom: 36px;
  }
  .end :global(.dl) {
    align-items: center;
  }

  @media (max-width: 1000px) {
    .grid,
    .split {
      grid-template-columns: minmax(0, 1fr);
      gap: 48px;
    }
    .asks {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .stages,
    .blocks,
    .setup,
    .trust {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  @media (max-width: 600px) {
    .hero {
      padding-top: 104px;
    }
    .sub {
      font-size: 17px;
    }
    .demo {
      padding: 18px;
      border-radius: 26px;
    }
    .spot-text strong {
      font-size: 26px;
    }
    .stages li,
    .setup li {
      padding: 24px 22px;
    }
    .end {
      padding: 64px 20px;
      border-radius: 32px;
    }
  }
  @media (max-width: 560px) {
    .asks {
      grid-template-columns: minmax(0, 1fr);
    }
    .trust {
      padding: 32px 24px;
    }
  }
</style>
