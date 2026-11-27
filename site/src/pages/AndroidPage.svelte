<script lang="ts">
  import { DEVICES, GROUPS, PHASES, STATUS_LABEL, UPDATED, currentPhase, progress, shortDate, type Phase, type PhaseStatus } from "../lib/android";
  import { REPO_URL } from "../lib/release";
  import { inview } from "./inview";
  import PageShell from "./PageShell.svelte";

  const cur = currentPhase();
  const pct = Math.round(progress() * 100);
  const done = PHASES.filter((p) => p.status === "done").length;
  const active = PHASES.filter((p) => p.status === "testing" || p.status === "building").length;
  const left = PHASES.length - done - active;

  const pad = (n: number) => String(n).padStart(2, "0");
  const inGroup = (from: number, to: number) => PHASES.filter((p) => p.n >= from && p.n <= to);
  const STATUS_ICON: Record<PhaseStatus, string> = { done: "task_alt", testing: "science", building: "construction", planned: "schedule" };
  const chip = (p: Phase) => (p.status === "done" && p.date ? `Done ${shortDate(p.date)}` : p === cur || p.status !== "planned" ? STATUS_LABEL[p.status] : p.n === cur.n + 1 ? "Up next" : "Planned");

  const SHIP = [
    { icon: "cloud_sync", title: "Built in the cloud", text: "Every change is built and signed by GitHub Actions. Nothing is built on a laptop." },
    { icon: "phone_android", title: "Tested on three devices", text: "The test build goes on an Android 9 phone, an Android 14 phone and an Android 16 tablet." },
    { icon: "bug_report", title: "Fixed", text: "Whatever breaks on any of the three gets fixed and tested again." },
    { icon: "arrow_forward", title: "Then the next phase", text: "A phase is done only when it works on all three. Then the next one starts." },
  ];

  const QA = [
    { q: "When does it come out?", a: "When phase 13 is done. There's no date yet, on purpose: it ships when it works well on every device, not on a deadline. This page shows how close it is." },
    { q: "Will it be free?", a: "Yes, like desktop Velmora. And the Android app has no ads at all." },
    { q: "Which phones and tablets?", a: "Android 8 and newer, phones and tablets, on both 64-bit and older 32-bit ARM chips." },
    { q: "Where will I get it?", a: "Here, on this website, as an APK. It updates itself from inside the app. It won't be on the Play Store." },
  ];
</script>

<PageShell
  eyebrow="Velmora for Android"
  title="Built in the open,"
  accent="one phase at a time."
  crumb="Android"
  lead="Velmora for Android is in production development. It's the whole app, not a lite version: movies, series, anime, live TV, sports, Theatre and music. Here's exactly where it stands."
  spots={[
    { x: "91%", y: "180px", size: 120, color: "#3ddc84", spin: 20, opacity: 0.45 },
    { x: "6%", y: "360px", size: 58, color: "#3271fe", spin: -16, offset: 4, opacity: 0.7 },
    { x: "78%", y: "430px", size: 40, color: "#f8e8c8", spin: 12, offset: 2, opacity: 0.75 },
  ]}
>
  {#snippet hero()}
    <div class="board">
      <div class="now">
        <span class="live"><i></i>In production development</span>
        <div class="big">
          <span class="pct">{pct}<small>%</small></span>
          <span class="of">of the way<br />to release</span>
        </div>
        <p class="nowline">
          Now: <a href="#phase-{cur.n}">Phase {cur.n}, {cur.title}</a>
          <span class="state {cur.status}">{STATUS_LABEL[cur.status]}</span>
        </p>
      </div>

      <div class="track" aria-label="Progress by phase">
        <div class="segs">
          {#each PHASES as p (p.n)}
            <a class="seg {p.status}" class:cur={p === cur} href="#phase-{p.n}" title="Phase {p.n}: {p.title} ({STATUS_LABEL[p.status]})">
              <span>{p.n}</span>
            </a>
          {/each}
        </div>
        <div class="glabels">
          {#each GROUPS as g (g.name)}
            <span style="grid-column: span {g.to - g.from + 1}">{g.name}</span>
          {/each}
        </div>
      </div>

      <div class="facts">
        <span><m3e-icon name="task_alt"></m3e-icon>{done} done</span>
        <span><m3e-icon name="construction"></m3e-icon>{active} in progress</span>
        <span><m3e-icon name="schedule"></m3e-icon>{left} to go</span>
        <span class="upd"><m3e-icon name="history"></m3e-icon>Updated {shortDate(UPDATED)}</span>
      </div>
    </div>
  {/snippet}

  <section class="pg-section">
    <div class="pg-wide">
      <span class="eyebrow">The plan</span>
      <h2 class="pg-title">Thirteen phases, <em>three devices each.</em></h2>
      <p class="pg-lead">
        Nine phases build the app, three are spent only on how it looks and feels, and the last one ships it. Every phase is tested on real hardware
        before the next one starts.
      </p>
      <div class="devices">
        {#each DEVICES as d (d.label)}<span><m3e-icon name={d.icon}></m3e-icon>{d.label}</span>{/each}
      </div>

      {#each GROUPS as g, gi (g.name)}
        {@const list = inGroup(g.from, g.to)}
        {@const finished = list.filter((p) => p.status === "done").length}
        <div class="group" use:inview>
          <div class="ghead">
            <span class="gno">{gi + 1}</span>
            <div>
              <h3>{g.name}</h3>
              <p>{g.blurb}</p>
            </div>
            <span class="gcount">{finished} of {list.length} done</span>
          </div>

          <ol class="rail">
            {#each list as p, i (p.n)}
              <li class="ph {p.status}" class:cur={p === cur} id="phase-{p.n}" use:inview={i * 0.05}>
                <span class="node" aria-hidden="true">
                  {#if p.status === "done"}<m3e-icon name="check"></m3e-icon>{:else}{pad(p.n)}{/if}
                </span>
                <article class="card">
                  <header>
                    <span class="ico"><m3e-icon name={p.icon}></m3e-icon></span>
                    <div class="hd">
                      <span class="num">Phase {pad(p.n)}</span>
                      <h4>{p.title}</h4>
                    </div>
                    <span class="chip {p.status}"><m3e-icon name={STATUS_ICON[p.status]}></m3e-icon>{chip(p)}</span>
                  </header>
                  <p class="sum">{p.summary}</p>
                  <p class="res"><m3e-icon name="arrow_forward"></m3e-icon><span><b>You get:</b> {p.result}</span></p>
                  <ul class="pieces">
                    {#each p.pieces as x (x)}<li>{x}</li>{/each}
                  </ul>
                </article>
              </li>
            {/each}
          </ol>
        </div>
      {/each}
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide">
      <span class="eyebrow">How a phase ships</span>
      <h2 class="pg-title">No phase moves on <em>until it works.</em></h2>
      <ol class="ship">
        {#each SHIP as s, i (s.title)}
          <li use:inview={i * 0.07}>
            <span class="sico"><m3e-icon name={s.icon}></m3e-icon></span>
            <span class="step">Step {i + 1}</span>
            <h3>{s.title}</h3>
            <p>{s.text}</p>
          </li>
        {/each}
      </ol>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide">
      <span class="eyebrow">Questions</span>
      <h2 class="pg-title">What people <em>ask first.</em></h2>
      <div class="pg-grid qa">
        {#each QA as f, i (f.q)}
          <div class="pg-card" use:inview={i * 0.06}>
            <h3>{f.q}</h3>
            <p>{f.a}</p>
          </div>
        {/each}
      </div>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide">
      <div class="pg-grid end">
        <a class="pg-card credit" href="https://github.com/EchoMusicApp/Echo-Music" target="_blank" rel="noopener" use:inview>
          <span class="tag">Built on</span>
          <h3>Echo Music</h3>
          <p>
            Velmora for Android is built on Echo Music by iad1tya, the open-source YouTube Music app (GPL-3.0). Its player, lyrics and much of its design
            are the base we build on. Thank you.
          </p>
          <span class="go">See Echo Music<m3e-icon name="arrow_outward"></m3e-icon></span>
        </a>
        <div class="pg-card follow" use:inview={0.08}>
          <span class="tag">Follow along</span>
          <h3>See each phase land</h3>
          <p>Previews go up on Instagram as phases finish, and every change is public on GitHub.</p>
          <div class="links">
            <a href="https://instagram.com/stepbyte.studios" target="_blank" rel="noopener"><m3e-icon name="photo_camera"></m3e-icon>@stepbyte.studios</a>
            <a href="https://instagram.com/ashfaqizop" target="_blank" rel="noopener"><m3e-icon name="photo_camera"></m3e-icon>@ashfaqizop</a>
            <a href="{REPO_URL}/commits/main/android" target="_blank" rel="noopener"><m3e-icon name="history"></m3e-icon>Android commits</a>
            <a href={REPO_URL} target="_blank" rel="noopener"><m3e-icon name="star"></m3e-icon>Star on GitHub</a>
          </div>
        </div>
      </div>
    </div>
  </section>
</PageShell>

<style>
  /* ── Overview board ── */
  .board {
    --green: #3ddc84;
    margin-top: 40px;
    display: grid;
    grid-template-columns: minmax(0, 300px) minmax(0, 1fr);
    grid-template-areas: "now track" "facts facts";
    gap: 28px 40px;
    padding: 28px;
    border-radius: 32px;
    border: 1px solid var(--line);
    background:
      radial-gradient(120% 140% at 0% 0%, rgb(61 220 132 / 10%), transparent 55%),
      radial-gradient(90% 120% at 100% 100%, rgb(50 113 254 / 14%), transparent 60%),
      var(--glass);
    backdrop-filter: blur(14px);
  }
  .now {
    grid-area: now;
  }
  .live {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 12.5px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--green);
  }
  .live i {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--green);
    box-shadow: 0 0 0 0 rgb(61 220 132 / 60%);
    animation: ping 2s infinite;
  }
  @keyframes ping {
    70% {
      box-shadow: 0 0 0 10px rgb(61 220 132 / 0%);
    }
    100% {
      box-shadow: 0 0 0 0 rgb(61 220 132 / 0%);
    }
  }
  .big {
    display: flex;
    align-items: flex-end;
    gap: 14px;
    margin-top: 10px;
  }
  .pct {
    font-family: var(--display);
    font-size: clamp(72px, 9vw, 104px);
    line-height: 0.85;
    background: linear-gradient(120deg, var(--cream), var(--green) 55%, var(--blue));
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  .pct small {
    font-size: 0.5em;
  }
  .of {
    padding-bottom: 8px;
    font-size: 14px;
    line-height: 1.35;
    color: var(--muted);
  }
  .nowline {
    margin: 16px 0 0;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    color: var(--muted);
    font-size: 15px;
  }
  .nowline a {
    color: var(--ink);
    font-weight: 600;
    text-decoration: none;
  }
  .nowline a:hover {
    text-decoration: underline;
  }

  .track {
    grid-area: track;
    align-self: center;
  }
  .segs,
  .glabels {
    display: grid;
    grid-template-columns: repeat(13, minmax(0, 1fr));
    gap: 6px;
  }
  .seg {
    position: relative;
    height: 64px;
    border-radius: 14px;
    border: 1px solid var(--line);
    background: rgb(255 255 255 / 3%);
    display: grid;
    place-items: end center;
    padding-bottom: 8px;
    text-decoration: none;
    color: rgb(255 255 255 / 40%);
    font-size: 12px;
    font-weight: 700;
    overflow: hidden;
    transition:
      transform 0.25s cubic-bezier(0.2, 0.7, 0.2, 1),
      border-color 0.25s;
  }
  .seg:hover {
    transform: translateY(-3px);
    border-color: rgb(255 255 255 / 24%);
  }
  .seg span {
    position: relative;
  }
  .seg.done {
    background: linear-gradient(180deg, #5d8eff, var(--blue));
    border-color: transparent;
    color: #fff;
  }
  .seg.testing {
    background: linear-gradient(180deg, #ffcf85, var(--amber));
    border-color: transparent;
    color: #2a1a00;
  }
  .seg.building {
    border-color: rgb(61 220 132 / 50%);
    color: #eafff2;
    background: repeating-linear-gradient(-45deg, rgb(61 220 132 / 45%) 0 8px, rgb(61 220 132 / 22%) 8px 16px);
    background-size: 22.6px 22.6px;
    animation: stripes 1.2s linear infinite;
  }
  .seg.cur {
    box-shadow: 0 0 0 2px var(--bg), 0 0 0 3px var(--green), 0 10px 30px -8px rgb(61 220 132 / 55%);
  }
  @keyframes stripes {
    to {
      background-position: 22.6px 0;
    }
  }
  .glabels {
    margin-top: 10px;
  }
  .glabels span {
    padding-top: 8px;
    border-top: 2px solid rgb(255 255 255 / 12%);
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .facts {
    grid-area: facts;
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    padding-top: 20px;
    border-top: 1px solid var(--line);
  }
  .facts span,
  .devices span {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 7px 14px 7px 11px;
    border-radius: 999px;
    background: rgb(255 255 255 / 5%);
    border: 1px solid var(--line);
    font-size: 14px;
    color: var(--ink);
    --m3e-icon-size: 18px;
  }
  .facts .upd {
    margin-left: auto;
    color: var(--muted);
  }

  .state,
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 11px 4px 9px;
    border-radius: 999px;
    font-size: 12.5px;
    font-weight: 700;
    white-space: nowrap;
    --m3e-icon-size: 16px;
    background: rgb(255 255 255 / 6%);
    color: var(--muted);
  }
  .state {
    padding: 3px 10px;
  }
  .done.state,
  .chip.done {
    background: rgb(50 113 254 / 18%);
    color: #a9c3ff;
  }
  .testing.state,
  .chip.testing {
    background: rgb(255 183 77 / 16%);
    color: #ffd08a;
  }
  .building.state,
  .chip.building {
    background: rgb(61 220 132 / 15%);
    color: #8ff0b8;
  }

  /* ── Timeline ── */
  .devices {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-top: 24px;
  }
  .group {
    margin-top: 56px;
  }
  .ghead {
    display: flex;
    align-items: center;
    gap: 16px;
    padding-bottom: 18px;
    border-bottom: 1px solid var(--line);
  }
  .gno {
    flex: none;
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border-radius: 14px;
    font-family: var(--display);
    font-size: 26px;
    background: linear-gradient(135deg, rgb(248 232 200 / 18%), rgb(50 113 254 / 22%));
    color: var(--cream);
  }
  .ghead h3 {
    margin: 0;
    font-size: 21px;
  }
  .ghead p {
    margin: 2px 0 0;
    color: var(--muted);
    font-size: 15px;
  }
  .gcount {
    margin-left: auto;
    color: var(--muted);
    font-size: 13.5px;
    white-space: nowrap;
  }

  .rail {
    --gap: 16px;
    position: relative;
    list-style: none;
    margin: 24px 0 0;
    padding: 0;
    display: grid;
    gap: var(--gap);
  }
  .ph {
    position: relative;
    display: grid;
    grid-template-columns: 52px minmax(0, 1fr);
    gap: 18px;
  }
  /* The line between nodes, coloured once a phase is done. */
  .ph::before {
    content: "";
    position: absolute;
    left: 25px;
    top: 52px;
    bottom: calc(-1 * var(--gap));
    width: 2px;
    background: rgb(255 255 255 / 9%);
  }
  .ph:last-child::before {
    display: none;
  }
  .ph.done::before {
    background: linear-gradient(180deg, var(--blue), rgb(50 113 254 / 30%));
  }
  .node {
    position: relative;
    z-index: 1;
    display: grid;
    place-items: center;
    width: 52px;
    height: 52px;
    border-radius: 18px;
    border: 1px solid var(--line);
    background: #10142a;
    font-weight: 800;
    font-size: 15px;
    color: rgb(255 255 255 / 45%);
    --m3e-icon-size: 24px;
  }
  .done .node {
    background: linear-gradient(135deg, #5d8eff, var(--blue));
    border-color: transparent;
    color: #fff;
  }
  .building .node,
  .testing .node {
    color: #eafff2;
    border-color: rgb(61 220 132 / 55%);
    background: rgb(61 220 132 / 16%);
    box-shadow: 0 0 0 6px rgb(61 220 132 / 8%);
  }
  .testing .node {
    color: #2a1a00;
    border-color: transparent;
    background: var(--amber);
    box-shadow: 0 0 0 6px rgb(255 183 77 / 10%);
  }

  .card {
    padding: 22px 24px;
    border-radius: 24px;
    border: 1px solid var(--line);
    background: var(--glass);
    transition:
      border-color 0.3s,
      transform 0.35s cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .card:hover {
    transform: translateX(4px);
    border-color: rgb(255 255 255 / 15%);
  }
  .planned .card {
    background: rgb(255 255 255 / 2%);
  }
  .planned .sum,
  .planned .pieces li {
    opacity: 0.85;
  }
  /* The phase being built gets a slowly turning border. */
  .ph.cur .card {
    position: relative;
    border-color: transparent;
    background:
      linear-gradient(#0e1224, #0e1224) padding-box,
      conic-gradient(from var(--a, 0deg), rgb(61 220 132 / 80%), rgb(50 113 254 / 70%), rgb(248 232 200 / 50%), rgb(61 220 132 / 80%)) border-box;
    animation: turn 6s linear infinite;
    box-shadow: 0 24px 60px -30px rgb(61 220 132 / 45%);
  }
  @property --a {
    syntax: "<angle>";
    initial-value: 0deg;
    inherits: false;
  }
  @keyframes turn {
    to {
      --a: 360deg;
    }
  }
  .card header {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .ico {
    flex: none;
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    border-radius: 14px;
    background: rgb(255 255 255 / 6%);
    color: var(--cream);
    --m3e-icon-size: 22px;
  }
  .cur .ico {
    background: rgb(61 220 132 / 16%);
    color: #8ff0b8;
  }
  .done .ico {
    background: rgb(50 113 254 / 16%);
    color: #a9c3ff;
  }
  .hd {
    min-width: 0;
    flex: 1;
  }
  .num {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .card h4 {
    margin: 2px 0 0;
    font-size: 19px;
    line-height: 1.25;
  }
  .sum {
    margin: 14px 0 0;
    color: var(--muted);
    font-size: 15.5px;
    line-height: 1.65;
  }
  .res {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin: 12px 0 0;
    font-size: 15px;
    color: var(--ink);
    --m3e-icon-size: 18px;
  }
  .res m3e-icon {
    margin-top: 2px;
    color: var(--cream);
  }
  .res b {
    font-weight: 700;
  }
  .pieces {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    list-style: none;
    margin: 16px 0 0;
    padding: 0;
  }
  .pieces li {
    padding: 5px 12px;
    border-radius: 10px;
    background: rgb(255 255 255 / 5%);
    border: 1px solid var(--line);
    font-size: 13px;
    color: var(--ink);
  }

  /* ── How a phase ships ── */
  .ship {
    list-style: none;
    margin: 34px 0 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
    counter-reset: s;
  }
  .ship li {
    position: relative;
    padding: 24px;
    border-radius: 24px;
    border: 1px solid var(--line);
    background: var(--glass);
  }
  .ship li:not(:last-child)::after {
    content: "";
    position: absolute;
    top: 44px;
    right: -9px;
    width: 6px;
    height: 6px;
    border-top: 2px solid rgb(255 255 255 / 30%);
    border-right: 2px solid rgb(255 255 255 / 30%);
    transform: rotate(45deg);
  }
  .sico {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border-radius: 14px;
    background: linear-gradient(135deg, rgb(61 220 132 / 22%), rgb(50 113 254 / 22%));
    color: #c9ffe0;
  }
  .step {
    display: block;
    margin-top: 18px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .ship h3 {
    margin: 4px 0 6px;
    font-size: 18px;
  }
  .ship p {
    margin: 0;
    color: var(--muted);
    font-size: 15px;
    line-height: 1.6;
  }

  /* ── Credit and follow ── */
  .qa h3 {
    margin-top: 0;
  }
  .tag {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .credit {
    background:
      radial-gradient(120% 120% at 100% 0%, rgb(230 60 40 / 16%), transparent 60%),
      var(--glass);
  }
  .credit h3,
  .follow h3 {
    font-size: 24px;
  }
  .go {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-top: 18px;
    font-weight: 600;
    color: #ffb3a7;
    --m3e-icon-size: 18px;
  }
  .follow {
    background:
      radial-gradient(120% 120% at 100% 0%, rgb(61 220 132 / 12%), transparent 60%),
      var(--glass);
  }
  .links {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 18px;
  }
  .links a {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 8px 14px 8px 11px;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: rgb(255 255 255 / 5%);
    color: var(--ink);
    text-decoration: none;
    font-size: 14px;
    --m3e-icon-size: 18px;
    transition:
      border-color 0.25s,
      background 0.25s;
  }
  .links a:hover {
    border-color: rgb(61 220 132 / 55%);
    background: rgb(61 220 132 / 10%);
  }

  @media (max-width: 960px) {
    .board {
      grid-template-columns: minmax(0, 1fr);
      grid-template-areas: "now" "track" "facts";
    }
    .ship {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .ship li::after {
      display: none;
    }
  }
  @media (max-width: 600px) {
    .board {
      padding: 20px;
      border-radius: 26px;
    }
    .segs,
    .glabels {
      gap: 3px;
    }
    .seg {
      height: 46px;
      border-radius: 8px;
      font-size: 10.5px;
      padding-bottom: 5px;
    }
    .glabels span {
      font-size: 10.5px;
    }
    .facts .upd {
      margin-left: 0;
    }
    .ph {
      grid-template-columns: 40px minmax(0, 1fr);
      gap: 12px;
    }
    .node {
      width: 40px;
      height: 40px;
      border-radius: 14px;
      font-size: 13px;
    }
    .ph::before {
      left: 19px;
      top: 40px;
    }
    .card {
      padding: 18px;
    }
    .card header {
      flex-wrap: wrap;
    }
    .chip {
      order: 3;
    }
    .ship {
      grid-template-columns: minmax(0, 1fr);
    }
    .gcount {
      display: none;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .seg.building,
    .ph.cur .card,
    .live i {
      animation: none;
    }
  }
</style>
