<script lang="ts">
  import Logo from "./Logo.svelte";

  // "Android is coming" card with a small phone drawn in CSS.
  let {
    title = "Velmora for Android",
    text = "The same app on your phone and tablet: search, the player, downloads and Ani. Work on the Android build has started.",
  }: { title?: string; text?: string } = $props();

  const POSTERS = [
    ["#ff7a59", "#3b1d6e"],
    ["#2dd4bf", "#0f2747"],
    ["#a78bfa", "#14213d"],
    ["#f472b6", "#1e1b4b"],
  ];
</script>

<div class="android">
  <div class="text">
    <span class="chip"><m3e-icon name="android"></m3e-icon>Coming soon</span>
    <h3 class="display">{title}</h3>
    <p>{text}</p>
    <span class="hint"><m3e-icon name="schedule"></m3e-icon>Windows gets it first. Updates arrive on their own, so there's nothing to sign up for.</span>
  </div>

  <div class="phone" aria-hidden="true">
    <div class="screen">
      <div class="notch"></div>
      <div class="top"><Logo size={13} /><m3e-icon name="search"></m3e-icon></div>
      <div class="hero"><span class="orb"></span><strong class="display">Midnight Harbor</strong></div>
      <span class="row-title display">Continue watching</span>
      <div class="row">
        {#each POSTERS as [a, b] (a)}<span class="p" style:--a={a} style:--b={b}></span>{/each}
      </div>
      <div class="tabs">
        <m3e-icon name="home" filled></m3e-icon><m3e-icon name="search"></m3e-icon><m3e-icon name="wand_stars"></m3e-icon><m3e-icon
          name="download"
        ></m3e-icon>
      </div>
    </div>
  </div>
</div>

<style>
  .android {
    position: relative;
    overflow: hidden;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 220px;
    gap: 40px;
    align-items: center;
    padding: 40px 48px 0;
    border-radius: 36px;
    border: 1px solid rgb(61 220 132 / 22%);
    background:
      radial-gradient(70% 90% at 100% 100%, rgb(61 220 132 / 18%), transparent 60%),
      radial-gradient(60% 80% at 0% 0%, rgb(123 108 255 / 14%), transparent 60%),
      #110f1a;
    --md-sys-color-on-surface: #f4f1fb;
  }
  .text {
    padding-bottom: 40px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px 5px 9px;
    border-radius: 999px;
    font-size: 12.5px;
    font-weight: 700;
    background: rgb(61 220 132 / 16%);
    color: #8ff0b8;
    --m3e-icon-size: 18px;
  }
  h3 {
    margin: 16px 0 0;
    font-size: clamp(32px, 3.6vw, 46px);
    line-height: 1.02;
  }
  p {
    margin: 12px 0 0;
    max-width: 540px;
    font-size: 16px;
    line-height: 1.6;
    color: var(--muted);
  }
  .hint {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 18px;
    font-size: 13px;
    color: var(--muted);
    --m3e-icon-size: 16px;
  }

  /* The phone peeks up from the bottom edge. */
  .phone {
    align-self: end;
    width: 220px;
    height: 300px;
    padding: 10px 10px 0;
    border-radius: 34px 34px 0 0;
    background: #26222f;
    box-shadow:
      inset 0 0 0 1px rgb(255 255 255 / 12%),
      0 -20px 60px -20px rgb(61 220 132 / 35%);
    animation: peek 6s ease-in-out infinite alternate;
  }
  @keyframes peek {
    to {
      transform: translateY(-10px);
    }
  }
  .screen {
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 26px 10px 0;
    border-radius: 26px 26px 0 0;
    overflow: hidden;
    background: #100d18;
    font-size: 10px;
  }
  .notch {
    position: absolute;
    left: 50%;
    top: 8px;
    width: 10px;
    height: 10px;
    translate: -50% 0;
    border-radius: 50%;
    background: #000;
  }
  .top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    --m3e-icon-size: 14px;
    color: var(--muted);
  }
  .hero {
    position: relative;
    height: 96px;
    border-radius: 14px;
    overflow: hidden;
    display: flex;
    align-items: flex-end;
    padding: 8px;
    background:
      radial-gradient(100% 90% at 85% 10%, #ff8a65, transparent 50%),
      linear-gradient(180deg, #2a1846, #0d0b1c);
  }
  .orb {
    position: absolute;
    right: 16px;
    top: 14px;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: radial-gradient(circle at 40% 40%, #fff3d6, #ffb36b 55%, #ff6b5a);
  }
  .hero strong {
    position: relative;
    font-size: 17px;
  }
  .row-title {
    font-size: 13px;
  }
  .row {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 6px;
  }
  .p {
    aspect-ratio: 2 / 3;
    border-radius: 7px;
    background: linear-gradient(165deg, var(--a), var(--b) 78%);
  }
  .tabs {
    margin-top: auto;
    display: flex;
    justify-content: space-around;
    padding: 10px 0 12px;
    border-top: 1px solid var(--line);
    color: var(--muted);
    --m3e-icon-size: 16px;
  }
  .tabs m3e-icon:first-child {
    color: #c9c1ff;
  }

  @media (max-width: 760px) {
    .android {
      grid-template-columns: minmax(0, 1fr);
      padding: 32px 26px 0;
    }
    .phone {
      justify-self: center;
      width: 190px;
      height: 250px;
    }
    .android {
      border-radius: 28px;
    }
    .text {
      padding-bottom: 0;
    }
  }
</style>
