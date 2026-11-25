<script lang="ts">
  // A decorative installer window: the bar fills, the shape morphs, a check
  // pops in, then it starts over. Pure CSS, so it also runs before hydration.
  import Logo from "../components/Logo.svelte";
</script>

<div class="win" aria-hidden="true">
  <div class="title"><span class="dots"><i></i><i></i><i></i></span><span>Velmora Setup</span></div>
  <div class="body">
    <div class="logo"><Logo size={56} word={false} /></div>
    <div class="right">
      <strong>Installing Velmora</strong>
      <span class="status"><span class="s1">Copying files…</span><span class="s2">Installed. Open Velmora.</span></span>
      <span class="bar"><span class="fill"></span></span>
    </div>
  </div>
  <span class="check"><m3e-icon name="check" filled></m3e-icon></span>
</div>

<style>
  .win {
    position: relative;
    width: min(380px, 100%);
    border-radius: 22px;
    border: 1px solid var(--line);
    background: linear-gradient(160deg, rgb(255 255 255 / 9%), rgb(255 255 255 / 3%));
    box-shadow:
      0 40px 90px -30px rgb(0 0 0 / 70%),
      0 0 0 1px rgb(255 255 255 / 4%) inset;
    overflow: hidden;
    animation: float 7s ease-in-out infinite alternate;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    font-size: 12.5px;
    color: var(--muted);
    border-bottom: 1px solid var(--line);
  }
  .dots {
    display: flex;
    gap: 6px;
  }
  .dots i {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: rgb(255 255 255 / 18%);
  }
  .body {
    display: flex;
    gap: 18px;
    align-items: center;
    padding: 26px 22px 30px;
  }
  .logo {
    flex: none;
    animation: pop 6s ease-in-out infinite;
  }
  .right {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  strong {
    font-size: 16px;
  }
  .status {
    position: relative;
    height: 18px;
    font-size: 13px;
    color: var(--muted);
  }
  .status span {
    position: absolute;
    inset: 0;
    animation: swap 6s ease-in-out infinite;
  }
  .s2 {
    animation-name: swap2 !important;
    color: #8be8b4;
  }
  .bar {
    height: 8px;
    border-radius: 99px;
    background: rgb(255 255 255 / 9%);
    overflow: hidden;
  }
  .fill {
    display: block;
    height: 100%;
    border-radius: 99px;
    background: linear-gradient(90deg, var(--blue), var(--cream));
    transform-origin: left;
    animation: fill 6s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  .check {
    position: absolute;
    right: 18px;
    bottom: 16px;
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: #1fa971;
    color: #fff;
    --m3e-icon-size: 22px;
    animation: tick 6s cubic-bezier(0.34, 1.6, 0.5, 1) infinite;
  }
  @keyframes fill {
    0% {
      transform: scaleX(0.02);
    }
    70%,
    100% {
      transform: scaleX(1);
    }
  }
  @keyframes swap {
    0%,
    68% {
      opacity: 1;
    }
    74%,
    100% {
      opacity: 0;
    }
  }
  @keyframes swap2 {
    0%,
    70% {
      opacity: 0;
    }
    78%,
    94% {
      opacity: 1;
    }
    100% {
      opacity: 0;
    }
  }
  @keyframes tick {
    0%,
    70% {
      transform: scale(0);
    }
    80%,
    94% {
      transform: scale(1);
    }
    100% {
      transform: scale(0);
    }
  }
  @keyframes pop {
    0%,
    68% {
      transform: scale(1);
    }
    78% {
      transform: scale(1.12) rotate(-4deg);
    }
    90%,
    100% {
      transform: scale(1);
    }
  }
  @keyframes float {
    from {
      transform: translateY(-6px) rotate(-0.6deg);
    }
    to {
      transform: translateY(8px) rotate(0.6deg);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .win,
    .logo,
    .fill,
    .status span,
    .check {
      animation: none;
    }
    .fill {
      transform: none;
    }
    .s1,
    .check {
      display: none;
    }
    .s2 {
      opacity: 1;
    }
  }
</style>
