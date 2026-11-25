<script lang="ts">
  // The Windows SmartScreen warning as a short film, drawn in CSS so it also
  // plays before the page hydrates. One 14-second loop:
  //   1. a cursor double-clicks the installer in Downloads
  //   2. SmartScreen opens; the cursor clicks "More info"
  //   3. "Run anyway" appears; the cursor clicks it
  //   4. Velmora's installer runs and finishes
  // Everything is sized in em, and the stage's font size follows its width,
  // so the picture scales as one piece. Reduced motion shows the key frame.
  import mark from "../assets/velmora-mark-small.webp";

  let paused = $state(false);

  const STEPS = ["Run the installer", "Click More info", "Click Run anyway", "Velmora installs"];
</script>

<figure class="ss" class:paused>
  <m3e-icon-button class="pp" aria-label={paused ? "Play the animation" : "Pause the animation"} onclick={() => (paused = !paused)}>
    <m3e-icon name={paused ? "play_arrow" : "pause"}></m3e-icon>
  </m3e-icon-button>
  <div class="box" role="img" aria-label="An animation of the Windows SmartScreen warning: the installer is opened, More info is clicked, then Run anyway, and Velmora installs.">
    <div class="stage" aria-hidden="true">
      <!-- the desktop -->
      <div class="wall">
        <span class="blob b1"></span>
        <span class="blob b2"></span>
        <span class="peak p1"></span>
        <span class="peak p2"></span>
      </div>

      <div class="explorer">
        <div class="titlebar">
          <m3e-icon name="folder_open"></m3e-icon><span>Downloads</span>
          <span class="win"><i></i><i></i><i></i></span>
        </div>
        <div class="path"><m3e-icon name="arrow_back"></m3e-icon><span>This PC  ›  Downloads</span></div>
        <div class="files">
          <div class="file dummy"><span class="doc"></span><b>notes.txt</b></div>
          <div class="file target">
            <img class="ico" src={mark} alt="" />
            <b>Velmora_x64-setup.exe</b>
          </div>
          <div class="file dummy"><span class="doc img"></span><b>photo.jpg</b></div>
        </div>
      </div>

      <div class="taskbar">
        <i class="start"></i><i></i><i></i><i></i><i></i>
      </div>

      <div class="dim"></div>

      <!-- Windows Defender SmartScreen -->
      <div class="dlg">
        <span class="x">✕</span>
        <h4>Windows protected your PC</h4>
        <p class="msg">Microsoft Defender SmartScreen prevented an unrecognized app from starting. Running this app might put your PC at risk.</p>
        <p class="details"><span><b>App:</b> Velmora_x64-setup.exe</span><span><b>Publisher:</b> Unknown publisher</span></p>
        <span class="more">More info</span>
        <div class="buttons">
          <span class="btn run">Run anyway</span>
          <span class="btn dont">Don't run</span>
        </div>
        <span class="halo"></span>
      </div>
      <div class="tip"><m3e-icon name="touch_app"></m3e-icon>Click Run anyway</div>

      <!-- Velmora's own installer -->
      <div class="setup">
        <div class="stitle"><span class="dots"><i></i><i></i><i></i></span>Velmora Setup</div>
        <div class="sbody">
          <img class="slogo" src={mark} alt="" />
          <div class="sright">
            <b>Installing Velmora</b>
            <span class="status"><span class="s1">Copying files…</span><span class="s2">Installed. Open Velmora.</span></span>
            <span class="bar"><span class="fill"></span></span>
          </div>
        </div>
        <span class="check"><m3e-icon name="check" filled></m3e-icon></span>
        <span class="burst"><i></i><i></i><i></i><i></i><i></i><i></i><i></i><i></i></span>
      </div>

      <!-- clicks and the cursor -->
      <span class="ring r1"></span>
      <span class="ring r2"></span>
      <span class="ring r3"></span>
      <span class="cursor">
        <svg class="arrow" viewBox="0 0 13 20" width="100%" height="100%">
          <path d="M1 1v15.2l3.7-3.3L7.3 19l2.7-1.2L7.5 11.8H12.4z" fill="#fff" stroke="#111" stroke-width="1.1" stroke-linejoin="round" />
        </svg>
      </span>
    </div>
  </div>

  <figcaption>
    <ol class="steps">
      {#each STEPS as s, i (s)}
        <li class="st{i + 1}"><b>{i + 1}</b>{s}</li>
      {/each}
    </ol>
    <span class="timeline" aria-hidden="true"><span></span></span>
  </figcaption>

</figure>

<style>
  .ss {
    --T: 14s;
    position: relative;
    margin: 26px 0 8px;
  }
  .ss.paused :global(*),
  .ss.paused :global(*::before) {
    animation-play-state: paused !important;
  }
  .box {
    container-type: inline-size;
    position: relative;
    width: min(760px, 100%);
    aspect-ratio: 16 / 10;
    margin-inline: auto;
    border-radius: 26px;
    overflow: hidden;
    border: 1px solid var(--line);
    box-shadow:
      0 50px 110px -40px rgb(0 0 0 / 80%),
      0 0 0 1px rgb(255 255 255 / 4%) inset;
    background: #0b1020;
  }
  /* 60em wide, so 1em = 1/60 of the picture */
  .stage {
    position: relative;
    width: 60em;
    height: 37.5em;
    font-size: calc(100cqw / 60);
    font-family: "Segoe UI", "Segoe UI Variable Text", system-ui, sans-serif;
    color: #fff;
    overflow: hidden;
  }
  .stage * {
    box-sizing: border-box;
  }

  /* ── wallpaper ── */
  .wall {
    position: absolute;
    inset: 0;
    background: linear-gradient(165deg, #16233d 0%, #0b1224 55%, #1a1233 100%);
  }
  .blob {
    position: absolute;
    border-radius: 50%;
    filter: blur(3.4em);
    opacity: 0.7;
    animation: drift 18s ease-in-out infinite alternate;
  }
  .b1 {
    width: 34em;
    height: 34em;
    left: -8em;
    top: -12em;
    background: #2f5fd0;
  }
  .b2 {
    width: 28em;
    height: 28em;
    right: -6em;
    top: 4em;
    background: #6a3fb8;
    opacity: 0.45;
    animation-duration: 23s;
    animation-direction: alternate-reverse;
  }
  .peak {
    position: absolute;
    bottom: 3em;
    background: #0c1426;
  }
  .p1 {
    left: -6em;
    width: 44em;
    height: 14em;
    clip-path: polygon(0 100%, 20% 38%, 34% 62%, 52% 12%, 70% 58%, 82% 40%, 100% 100%);
    opacity: 0.9;
  }
  .p2 {
    right: -8em;
    width: 40em;
    height: 11em;
    background: #0f1a31;
    clip-path: polygon(0 100%, 24% 30%, 42% 66%, 60% 22%, 80% 62%, 100% 100%);
  }
  @keyframes drift {
    to {
      transform: translate(4em, 3em) scale(1.12);
    }
  }

  /* ── Downloads window ── */
  .explorer {
    position: absolute;
    left: 4em;
    top: 5em;
    width: 36em;
    height: 22em;
    border-radius: 0.9em;
    background: rgb(28 32 46 / 92%);
    border: 1px solid rgb(255 255 255 / 12%);
    box-shadow: 0 2em 4em -1.4em rgb(0 0 0 / 70%);
    backdrop-filter: blur(1em);
    overflow: hidden;
  }
  .titlebar {
    display: flex;
    align-items: center;
    gap: 0.7em;
    height: 2.6em;
    padding: 0 1em;
    font-size: 1em;
    color: #d7dae6;
    background: rgb(255 255 255 / 5%);
    --m3e-icon-size: 1.3em;
  }
  .win {
    margin-left: auto;
    display: flex;
    gap: 0.9em;
  }
  .win i {
    width: 0.8em;
    height: 0.8em;
    border-radius: 50%;
    background: rgb(255 255 255 / 22%);
  }
  .path {
    display: flex;
    align-items: center;
    gap: 0.7em;
    margin: 0.7em 1em 0;
    height: 2.2em;
    padding: 0 0.9em;
    border-radius: 0.5em;
    background: rgb(255 255 255 / 7%);
    font-size: 0.85em;
    color: #aeb4c6;
    --m3e-icon-size: 1.2em;
  }
  .files {
    display: flex;
    gap: 1em;
    padding: 1.4em 1em;
  }
  .file {
    width: 9em;
    height: 8.6em;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.7em;
    border: 1px solid transparent;
    border-radius: 0.6em;
    font-size: 0.85em;
    text-align: center;
  }
  .file b {
    max-width: 100%;
    padding: 0 0.4em;
    font-weight: 400;
    color: #cfd3e2;
    line-height: 1.25;
    word-break: break-all;
  }
  .doc {
    width: 3.2em;
    height: 4em;
    border-radius: 0.3em;
    background: linear-gradient(#f2f4fb, #cdd3e6);
    opacity: 0.8;
  }
  .doc.img {
    background: linear-gradient(135deg, #6ec6ff, #4a7cff 60%, #2b3a8c);
  }
  .dummy {
    opacity: 0.5;
  }
  .ico {
    width: 4.2em;
    height: 4.2em;
    border-radius: 0.9em;
  }
  .target {
    animation: pick var(--T) ease-in-out infinite;
  }
  @keyframes pick {
    0%,
    14.5% {
      background: transparent;
      border-color: transparent;
    }
    15.5%,
    100% {
      background: rgb(90 150 255 / 26%);
      border-color: rgb(120 175 255 / 70%);
    }
  }

  .taskbar {
    position: absolute;
    inset: auto 0 0;
    height: 3em;
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1.1em;
    background: rgb(14 18 30 / 78%);
    border-top: 1px solid rgb(255 255 255 / 8%);
    backdrop-filter: blur(1em);
  }
  .taskbar i {
    width: 1.7em;
    height: 1.7em;
    border-radius: 0.45em;
    background: rgb(255 255 255 / 16%);
  }
  .taskbar .start {
    background: linear-gradient(135deg, #4ea1ff, #2b6fe0);
  }

  /* ── dim, then the dialog ── */
  .dim {
    position: absolute;
    inset: 0;
    background: rgb(4 8 20 / 62%);
    opacity: 0;
    animation: dim var(--T) ease-in-out infinite;
  }
  @keyframes dim {
    0%,
    20% {
      opacity: 0;
    }
    24%,
    56% {
      opacity: 1;
    }
    60%,
    100% {
      opacity: 0;
    }
  }
  .dlg {
    position: absolute;
    left: 14em;
    top: 6.5em;
    width: 32em;
    height: 23.5em;
    padding: 2.4em 2.4em 0;
    background: #0a5aa6;
    box-shadow:
      0 3em 6em -1.5em rgb(0 0 0 / 75%),
      0 0 0 1px rgb(255 255 255 / 10%) inset;
    opacity: 0;
    transform-origin: 50% 60%;
    animation: dlg var(--T) infinite;
  }
  @keyframes dlg {
    0%,
    20% {
      opacity: 0;
      transform: scale(0.9) translateY(1.2em);
      animation-timing-function: cubic-bezier(0.2, 1.25, 0.35, 1);
    }
    24%,
    56% {
      opacity: 1;
      transform: none;
      animation-timing-function: ease-in;
    }
    59.5%,
    100% {
      opacity: 0;
      transform: scale(0.96);
    }
  }
  .x {
    position: absolute;
    right: 1.1em;
    top: 0.8em;
    font-size: 1em;
    opacity: 0.75;
  }
  h4 {
    margin: 0 0 1.1em;
    font-size: 2.1em;
    font-weight: 300;
    line-height: 1.1;
    letter-spacing: -0.01em;
  }
  .msg {
    margin: 0 0 1.4em;
    font-size: 1.12em;
    line-height: 1.45;
    opacity: 0.95;
  }
  .details {
    margin: 0;
    display: grid;
    gap: 0.4em;
    font-size: 1.05em;
    opacity: 0;
    animation: slideIn var(--T) ease-out infinite;
  }
  .details b {
    display: inline-block;
    width: 5.6em;
    font-weight: 400;
    opacity: 0.85;
  }
  @keyframes slideIn {
    0%,
    38% {
      opacity: 0;
      transform: translateY(-0.8em);
    }
    42%,
    100% {
      opacity: 1;
      transform: none;
    }
  }
  .more {
    position: absolute;
    left: 2.4em;
    bottom: 2.1em;
    font-size: 1.12em;
    text-decoration: underline;
    text-underline-offset: 0.25em;
    animation: more var(--T) ease-in-out infinite;
  }
  @keyframes more {
    0%,
    32% {
      opacity: 1;
      text-decoration-thickness: 1px;
    }
    34%,
    37.5% {
      opacity: 1;
      text-decoration-thickness: 0.14em;
    }
    39.5%,
    100% {
      opacity: 0;
    }
  }
  .buttons {
    position: absolute;
    right: 2em;
    bottom: 2em;
    display: flex;
    gap: 1em;
    opacity: 0;
    animation: btnIn var(--T) ease-out infinite;
  }
  @keyframes btnIn {
    0%,
    38.5% {
      opacity: 0;
      transform: translateY(1.2em);
    }
    43%,
    100% {
      opacity: 1;
      transform: none;
    }
  }
  .btn {
    display: grid;
    place-items: center;
    height: 2.9em;
    padding: 0 1.5em;
    border: 0.16em solid #fff;
    background: #0b6ac4;
    font-size: 1.05em;
    font-weight: 600;
  }
  .run {
    min-width: 8em;
    animation: runBtn var(--T) ease-in-out infinite;
  }
  @keyframes runBtn {
    0%,
    49% {
      background: #0b6ac4;
      transform: none;
    }
    50.5%,
    54.5% {
      background: #2a86dc;
      transform: none;
    }
    55.5% {
      background: #0a5aa6;
      transform: scale(0.95);
    }
    58%,
    100% {
      background: #0b6ac4;
      transform: none;
    }
  }
  .dont {
    min-width: 7em;
  }
  /* a ring that pulses round Run anyway until it's clicked */
  .halo {
    position: absolute;
    right: 9.6em;
    bottom: 1.4em;
    width: 10.2em;
    height: 4.1em;
    border-radius: 0.5em;
    border: 0.2em solid #f8e8c8;
    opacity: 0;
    animation: halo var(--T) ease-out infinite;
  }
  @keyframes halo {
    0%,
    42% {
      opacity: 0;
      transform: scale(1);
    }
    44% {
      opacity: 1;
      transform: scale(1);
    }
    49% {
      opacity: 0.2;
      transform: scale(1.12);
    }
    50% {
      opacity: 1;
      transform: scale(1);
    }
    54.5% {
      opacity: 0.2;
      transform: scale(1.12);
    }
    55.2%,
    100% {
      opacity: 0;
      transform: scale(1.12);
    }
  }
  .tip {
    position: absolute;
    left: 26.4em;
    top: 21.2em;
    display: inline-flex;
    align-items: center;
    gap: 0.5em;
    padding: 0.55em 1em;
    border-radius: 99em;
    background: #f8e8c8;
    color: #1b1530;
    font-size: 1em;
    font-weight: 700;
    white-space: nowrap;
    box-shadow: 0 0.8em 2em -0.6em rgb(0 0 0 / 60%);
    opacity: 0;
    animation: tip var(--T) ease-out infinite;
    --m3e-icon-size: 1.3em;
  }
  .tip::after {
    content: "";
    position: absolute;
    left: 50%;
    bottom: -0.4em;
    width: 0.9em;
    height: 0.9em;
    margin-left: -0.45em;
    background: #f8e8c8;
    rotate: 45deg;
    border-radius: 0.15em;
  }
  @keyframes tip {
    0%,
    43% {
      opacity: 0;
      transform: translateY(0.8em) scale(0.9);
    }
    46% {
      opacity: 1;
      transform: none;
    }
    50% {
      transform: translateY(-0.3em);
    }
    54% {
      opacity: 1;
      transform: none;
    }
    55.6%,
    100% {
      opacity: 0;
      transform: translateY(0.4em) scale(0.95);
    }
  }

  /* ── Velmora's installer ── */
  .setup {
    position: absolute;
    left: 16em;
    top: 9.5em;
    width: 28em;
    height: 15em;
    border-radius: 1em;
    background: linear-gradient(160deg, #1c2440, #11172b);
    border: 1px solid rgb(255 255 255 / 14%);
    box-shadow: 0 3em 6em -1.5em rgb(0 0 0 / 75%);
    opacity: 0;
    animation: setupWin var(--T) infinite;
  }
  @keyframes setupWin {
    0%,
    60% {
      opacity: 0;
      transform: scale(0.9) translateY(1em);
      animation-timing-function: cubic-bezier(0.2, 1.25, 0.35, 1);
    }
    64.5%,
    95.5% {
      opacity: 1;
      transform: none;
      animation-timing-function: ease-in;
    }
    98.5%,
    100% {
      opacity: 0;
      transform: scale(0.97);
    }
  }
  .stitle {
    display: flex;
    align-items: center;
    gap: 0.9em;
    padding: 0.9em 1.2em;
    font-size: 0.9em;
    color: #aeb4c6;
    border-bottom: 1px solid rgb(255 255 255 / 9%);
  }
  .dots {
    display: flex;
    gap: 0.5em;
  }
  .dots i {
    width: 0.8em;
    height: 0.8em;
    border-radius: 50%;
    background: rgb(255 255 255 / 20%);
  }
  .sbody {
    display: flex;
    gap: 1.6em;
    align-items: center;
    padding: 2.2em 2em;
  }
  .slogo {
    flex: none;
    width: 5.4em;
    height: 5.4em;
    border-radius: 1.2em;
    animation: logoPop var(--T) ease-in-out infinite;
  }
  @keyframes logoPop {
    0%,
    80% {
      transform: none;
    }
    83% {
      transform: scale(1.14) rotate(-5deg);
    }
    87%,
    100% {
      transform: none;
    }
  }
  .sright {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.8em;
  }
  .sright b {
    font-size: 1.3em;
  }
  .status {
    position: relative;
    height: 1.4em;
    font-size: 1em;
    color: #aeb4c6;
  }
  .status span {
    position: absolute;
    inset: 0;
  }
  .s1 {
    animation: s1 var(--T) infinite;
  }
  .s2 {
    color: #8be8b4;
    opacity: 0;
    animation: s2 var(--T) infinite;
  }
  @keyframes s1 {
    0%,
    80% {
      opacity: 1;
    }
    81.5%,
    100% {
      opacity: 0;
    }
  }
  @keyframes s2 {
    0%,
    80.5% {
      opacity: 0;
    }
    83%,
    95.5% {
      opacity: 1;
    }
    98%,
    100% {
      opacity: 0;
    }
  }
  .bar {
    height: 0.7em;
    border-radius: 99em;
    background: rgb(255 255 255 / 10%);
    overflow: hidden;
  }
  .fill {
    display: block;
    height: 100%;
    border-radius: 99em;
    background: linear-gradient(90deg, #3271fe, #f8e8c8);
    transform-origin: left;
    transform: scaleX(0);
    animation: fill var(--T) linear infinite;
  }
  @keyframes fill {
    0%,
    64.5% {
      transform: scaleX(0);
    }
    70% {
      transform: scaleX(0.34);
    }
    74% {
      transform: scaleX(0.52);
    }
    80% {
      transform: scaleX(1);
    }
    100% {
      transform: scaleX(1);
    }
  }
  .check {
    position: absolute;
    right: 1.4em;
    bottom: 1.3em;
    display: grid;
    place-items: center;
    width: 3em;
    height: 3em;
    border-radius: 50%;
    background: #1fa971;
    color: #fff;
    transform: scale(0);
    animation: tick var(--T) infinite;
    --m3e-icon-size: 1.9em;
  }
  @keyframes tick {
    0%,
    80.5% {
      transform: scale(0);
      animation-timing-function: cubic-bezier(0.34, 1.7, 0.5, 1);
    }
    84%,
    95.5% {
      transform: scale(1);
    }
    98%,
    100% {
      transform: scale(0);
    }
  }
  .burst {
    position: absolute;
    right: 2.9em;
    bottom: 2.8em;
    width: 0;
    height: 0;
  }
  .burst i {
    position: absolute;
    left: -0.35em;
    top: -0.35em;
    width: 0.7em;
    height: 0.7em;
    border-radius: 50%;
    background: var(--c, #f8e8c8);
    opacity: 0;
    animation: pop var(--T) ease-out infinite;
  }
  .burst i:nth-child(odd) {
    --c: #7aa8ff;
  }
  .burst i:nth-child(3n) {
    --c: #8be8b4;
  }
  .burst i:nth-child(1) {
    --a: 0deg;
  }
  .burst i:nth-child(2) {
    --a: 45deg;
  }
  .burst i:nth-child(3) {
    --a: 90deg;
  }
  .burst i:nth-child(4) {
    --a: 135deg;
  }
  .burst i:nth-child(5) {
    --a: 180deg;
  }
  .burst i:nth-child(6) {
    --a: 225deg;
  }
  .burst i:nth-child(7) {
    --a: 270deg;
  }
  .burst i:nth-child(8) {
    --a: 315deg;
  }
  @keyframes pop {
    0%,
    82% {
      opacity: 0;
      transform: rotate(var(--a)) translateX(0.8em) scale(0.4);
    }
    83% {
      opacity: 1;
    }
    89%,
    100% {
      opacity: 0;
      transform: rotate(var(--a)) translateX(3.6em) scale(1);
    }
  }

  /* ── clicks ── */
  .ring {
    position: absolute;
    width: 1em;
    height: 1em;
    margin: -0.5em 0 0 -0.5em;
    border-radius: 50%;
    border: 0.18em solid rgb(255 255 255 / 85%);
    background: rgb(255 255 255 / 25%);
    opacity: 0;
    pointer-events: none;
  }
  .r1 {
    left: 18.4em;
    top: 15.4em;
    animation: click1 var(--T) ease-out infinite;
  }
  .r2 {
    left: 18.7em;
    top: 27.3em;
    animation: click2 var(--T) ease-out infinite;
  }
  .r3 {
    left: 32em;
    top: 26.7em;
    animation: click3 var(--T) ease-out infinite;
  }
  @keyframes click1 {
    0%,
    16% {
      opacity: 0;
      transform: scale(0.4);
    }
    16.4% {
      opacity: 1;
    }
    18.2% {
      opacity: 0;
      transform: scale(4.2);
    }
    18.5% {
      opacity: 1;
      transform: scale(0.4);
    }
    20.5%,
    100% {
      opacity: 0;
      transform: scale(4.2);
    }
  }
  @keyframes click2 {
    0%,
    36.8% {
      opacity: 0;
      transform: scale(0.4);
    }
    37.1% {
      opacity: 1;
    }
    39.5%,
    100% {
      opacity: 0;
      transform: scale(4.2);
    }
  }
  @keyframes click3 {
    0%,
    54.8% {
      opacity: 0;
      transform: scale(0.4);
    }
    55.1% {
      opacity: 1;
    }
    57.5%,
    100% {
      opacity: 0;
      transform: scale(4.2);
    }
  }

  /* ── the cursor ── */
  .cursor {
    position: absolute;
    left: 0;
    top: 0;
    width: 2em;
    height: 3.1em;
    margin: -0.12em 0 0 -0.1em;
    opacity: 0;
    filter: drop-shadow(0 0.25em 0.45em rgb(0 0 0 / 55%));
    animation: cursor var(--T) infinite;
  }
  .arrow {
    display: block;
    transform-origin: 8% 6%;
    animation: press var(--T) ease-in-out infinite;
  }
  @keyframes cursor {
    0% {
      opacity: 0;
      transform: translate(52em, 36em);
    }
    3% {
      opacity: 1;
      transform: translate(52em, 36em);
      animation-timing-function: cubic-bezier(0.45, 0, 0.2, 1);
    }
    14% {
      transform: translate(18.4em, 15.4em);
    }
    21% {
      transform: translate(18.4em, 15.4em);
      animation-timing-function: cubic-bezier(0.45, 0, 0.2, 1);
    }
    33% {
      transform: translate(18.7em, 27.3em);
    }
    39% {
      transform: translate(18.7em, 27.3em);
      animation-timing-function: cubic-bezier(0.45, 0, 0.2, 1);
    }
    50% {
      transform: translate(32em, 26.7em);
    }
    56% {
      transform: translate(32em, 26.7em);
      animation-timing-function: ease-in;
    }
    62% {
      opacity: 1;
      transform: translate(46em, 31em);
    }
    65%,
    100% {
      opacity: 0;
      transform: translate(48em, 33em);
    }
  }
  @keyframes press {
    0%,
    16%,
    19.5%,
    36.8%,
    38.6%,
    54.8%,
    56.6%,
    100% {
      transform: scale(1);
    }
    16.7%,
    18.6% {
      transform: scale(0.82);
    }
    37.3% {
      transform: scale(0.82);
    }
    55.3% {
      transform: scale(0.82);
    }
  }

  /* ── captions ── */
  figcaption {
    width: min(760px, 100%);
    margin: 16px auto 0;
  }
  .steps {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px;
  }
  .steps li {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border-radius: 16px;
    border: 1px solid var(--line);
    background: var(--glass);
    color: var(--muted);
    font-size: 13.5px;
    font-weight: 600;
    line-height: 1.25;
    transition: none;
  }
  .steps b {
    flex: none;
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    font-size: 12px;
    background: rgb(255 255 255 / 10%);
  }
  .st1 {
    animation: on1 var(--T) infinite;
  }
  .st2 {
    animation: on2 var(--T) infinite;
  }
  .st3 {
    animation: on3 var(--T) infinite;
  }
  .st4 {
    animation: on4 var(--T) infinite;
  }
  @keyframes on1 {
    0%,
    19.5% {
      color: #fff;
      border-color: color-mix(in srgb, var(--blue) 70%, transparent);
      background: color-mix(in srgb, var(--blue) 22%, transparent);
    }
    21%,
    100% {
      color: var(--muted);
      border-color: var(--line);
      background: var(--glass);
    }
  }
  @keyframes on2 {
    0%,
    19.5% {
      color: var(--muted);
      border-color: var(--line);
      background: var(--glass);
    }
    21%,
    37.5% {
      color: #fff;
      border-color: color-mix(in srgb, var(--blue) 70%, transparent);
      background: color-mix(in srgb, var(--blue) 22%, transparent);
    }
    39%,
    100% {
      color: var(--muted);
      border-color: var(--line);
      background: var(--glass);
    }
  }
  @keyframes on3 {
    0%,
    37.5% {
      color: var(--muted);
      border-color: var(--line);
      background: var(--glass);
    }
    39%,
    55.5% {
      color: #fff;
      border-color: color-mix(in srgb, var(--blue) 70%, transparent);
      background: color-mix(in srgb, var(--blue) 22%, transparent);
    }
    57%,
    100% {
      color: var(--muted);
      border-color: var(--line);
      background: var(--glass);
    }
  }
  @keyframes on4 {
    0%,
    55.5% {
      color: var(--muted);
      border-color: var(--line);
      background: var(--glass);
    }
    57%,
    97% {
      color: #fff;
      border-color: color-mix(in srgb, #1fa971 70%, transparent);
      background: color-mix(in srgb, #1fa971 20%, transparent);
    }
    98.5%,
    100% {
      color: var(--muted);
      border-color: var(--line);
      background: var(--glass);
    }
  }
  .timeline {
    display: block;
    height: 3px;
    margin-top: 12px;
    border-radius: 3px;
    background: rgb(255 255 255 / 8%);
    overflow: hidden;
  }
  .timeline span {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, var(--blue), var(--cream));
    transform-origin: left;
    animation: line var(--T) linear infinite;
  }
  @keyframes line {
    from {
      transform: scaleX(0);
    }
    to {
      transform: scaleX(1);
    }
  }
  .pp {
    position: absolute;
    top: 10px;
    right: max(10px, calc(50% - 380px + 10px));
    --m3e-icon-button-container-color: rgb(10 13 22 / 70%);
    z-index: 2;
  }

  @media (max-width: 640px) {
    .steps {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  /* No motion: show the moment that matters, with the buttons in view. */
  @media (prefers-reduced-motion: reduce) {
    .ss *,
    .ss *::before {
      animation: none !important;
    }
    .dim {
      opacity: 1;
    }
    .dlg {
      opacity: 1;
    }
    .details,
    .buttons,
    .halo,
    .tip {
      opacity: 1;
    }
    .more,
    .cursor,
    .ring,
    .setup,
    .timeline,
    .pp {
      display: none;
    }
    .target {
      background: rgb(90 150 255 / 26%);
      border-color: rgb(120 175 255 / 70%);
    }
    .steps li {
      color: var(--ink);
    }
  }
</style>
