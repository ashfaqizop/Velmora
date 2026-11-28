<script lang="ts">
  // A decorative phone installing Velmora: Android's install sheet comes up,
  // the bar fills, then "App installed" with Open. Pure CSS, loops.
  import Logo from "../components/Logo.svelte";
</script>

<div class="phone" aria-hidden="true">
  <div class="screen">
    <div class="notch"></div>
    <div class="status"><span>9:41</span><span class="sys"><i></i><i></i><i></i></span></div>
    <div class="files">
      <span class="ftitle">Downloads</span>
      <div class="file">
        <span class="apk">APK</span>
        <span class="fname">Velmora-android-arm64.apk</span>
      </div>
      <div class="file dim"><span class="apk pdf">PDF</span><span class="fname">ticket.pdf</span></div>
    </div>
    <div class="shade"></div>
    <div class="sheet">
      <div class="app">
        <span class="mark"><Logo size={30} word={false} /></span>
        <strong>Velmora</strong>
      </div>
      <div class="msgs">
        <span class="m1">Do you want to install this app?</span>
        <span class="m2">Installing…</span>
        <span class="m3">App installed.</span>
      </div>
      <span class="bar"><span class="fill"></span></span>
      <div class="btns">
        <span class="b ghost">Cancel</span>
        <span class="b go"><span class="t1">Install</span><span class="t2">Open</span></span>
      </div>
    </div>
  </div>
</div>

<style>
  .phone {
    width: 250px;
    height: 430px;
    padding: 10px;
    border-radius: 40px;
    background: #26222f;
    box-shadow:
      inset 0 0 0 1px rgb(255 255 255 / 12%),
      0 40px 90px -30px rgb(0 0 0 / 75%),
      0 0 80px -30px rgb(61 220 132 / 45%);
    animation: float 7s ease-in-out infinite alternate;
  }
  .screen {
    position: relative;
    height: 100%;
    border-radius: 31px;
    overflow: hidden;
    background: #0d1017;
    font-size: 11px;
  }
  .notch {
    position: absolute;
    left: 50%;
    top: 9px;
    width: 10px;
    height: 10px;
    translate: -50% 0;
    border-radius: 50%;
    background: #000;
    z-index: 3;
  }
  .status {
    display: flex;
    justify-content: space-between;
    padding: 8px 18px 0;
    color: var(--muted);
    font-size: 10px;
  }
  .sys {
    display: flex;
    gap: 3px;
    align-items: center;
  }
  .sys i {
    width: 9px;
    height: 6px;
    border-radius: 2px;
    background: rgb(255 255 255 / 40%);
  }
  .files {
    padding: 22px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .ftitle {
    font-size: 18px;
    font-weight: 600;
    margin-bottom: 6px;
  }
  .file {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px;
    border-radius: 14px;
    background: rgb(255 255 255 / 6%);
  }
  .file.dim {
    opacity: 0.45;
  }
  .apk {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 10px;
    background: rgb(61 220 132 / 20%);
    color: #8ff0b8;
    font-size: 9px;
    font-weight: 800;
  }
  .apk.pdf {
    background: rgb(255 120 100 / 18%);
    color: #ffb3a7;
  }
  .fname {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .shade {
    position: absolute;
    inset: 0;
    background: rgb(0 0 0 / 55%);
    animation: shade 7s ease-in-out infinite;
  }
  .sheet {
    position: absolute;
    left: 8px;
    right: 8px;
    bottom: 8px;
    padding: 18px 16px 14px;
    border-radius: 26px;
    background: #1b1f2a;
    box-shadow: 0 -10px 40px rgb(0 0 0 / 50%);
    display: flex;
    flex-direction: column;
    gap: 12px;
    animation: rise 7s cubic-bezier(0.2, 0.8, 0.2, 1) infinite;
  }
  .app {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .mark {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 14px;
    background: #0a0d16;
    animation: pop 7s ease-in-out infinite;
  }
  .app strong {
    font-size: 15px;
  }
  .msgs {
    position: relative;
    height: 16px;
    color: var(--muted);
  }
  .msgs span {
    position: absolute;
    inset: 0;
    opacity: 0;
  }
  .m1 {
    animation: m1 7s infinite;
  }
  .m2 {
    animation: m2 7s infinite;
  }
  .m3 {
    color: #8ff0b8;
    animation: m3 7s infinite;
  }
  .bar {
    height: 5px;
    border-radius: 99px;
    background: rgb(255 255 255 / 8%);
    overflow: hidden;
  }
  .fill {
    display: block;
    height: 100%;
    border-radius: 99px;
    background: linear-gradient(90deg, #3ddc84, #8fb4ff);
    transform-origin: left;
    animation: fill 7s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  .btns {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .b {
    position: relative;
    padding: 7px 14px;
    border-radius: 999px;
    font-weight: 600;
  }
  .ghost {
    color: #8ff0b8;
  }
  .go {
    min-width: 62px;
    text-align: center;
    background: #3ddc84;
    color: #06210f;
    animation: press 7s infinite;
  }
  .go span {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
  }
  .t1 {
    animation: m1 7s infinite;
  }
  .t2 {
    opacity: 0;
    animation: m3 7s infinite;
  }
  .go::after {
    content: "Install";
    visibility: hidden;
  }

  @keyframes shade {
    0%,
    8% {
      opacity: 0;
    }
    16%,
    92% {
      opacity: 1;
    }
    100% {
      opacity: 0;
    }
  }
  @keyframes rise {
    0%,
    8% {
      transform: translateY(120%);
    }
    18%,
    92% {
      transform: translateY(0);
    }
    100% {
      transform: translateY(120%);
    }
  }
  @keyframes m1 {
    0%,
    30% {
      opacity: 1;
    }
    34%,
    100% {
      opacity: 0;
    }
  }
  @keyframes m2 {
    0%,
    32% {
      opacity: 0;
    }
    36%,
    66% {
      opacity: 1;
    }
    70%,
    100% {
      opacity: 0;
    }
  }
  @keyframes m3 {
    0%,
    68% {
      opacity: 0;
    }
    72%,
    92% {
      opacity: 1;
    }
    100% {
      opacity: 0;
    }
  }
  @keyframes fill {
    0%,
    33% {
      transform: scaleX(0);
    }
    66%,
    100% {
      transform: scaleX(1);
    }
  }
  @keyframes press {
    0%,
    28% {
      transform: scale(1);
    }
    31% {
      transform: scale(0.9);
    }
    35%,
    100% {
      transform: scale(1);
    }
  }
  @keyframes pop {
    0%,
    68% {
      transform: scale(1);
    }
    74% {
      transform: scale(1.14) rotate(-5deg);
    }
    82%,
    100% {
      transform: scale(1);
    }
  }
  @keyframes float {
    from {
      transform: translateY(-6px) rotate(0.8deg);
    }
    to {
      transform: translateY(8px) rotate(-0.8deg);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .phone,
    .sheet,
    .shade,
    .mark,
    .fill,
    .go,
    .msgs span,
    .t1,
    .t2 {
      animation: none;
    }
    .m1,
    .t2 {
      opacity: 0;
    }
    .m3,
    .t2 {
      opacity: 1;
    }
    .t1 {
      opacity: 0;
    }
  }
</style>
