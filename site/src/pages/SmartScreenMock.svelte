<script lang="ts">
  // The Windows SmartScreen warning, drawn in CSS and looped: the warning
  // shows, "More info" is pressed, "Run anyway" appears. A picture of what to click.
</script>

<div class="wrap" aria-label="What the SmartScreen warning looks like: click More info, then Run anyway" role="img">
  <div class="dlg">
    <div class="head"><strong class="a">Windows protected your PC</strong><strong class="b">Windows protected your PC</strong></div>
    <p class="txt">Microsoft Defender SmartScreen prevented an unrecognized app from starting. Running this app might put your PC at risk.</p>
    <p class="info"><span class="lbl">App:</span> Velmora_x64-setup.exe <span class="lbl">Publisher:</span> Unknown publisher</p>
    <div class="actions">
      <span class="btn run">Run anyway</span>
      <span class="btn dont">Don't run</span>
    </div>
    <span class="more">More info</span>
    <span class="cursor"><m3e-icon name="arrow_outward"></m3e-icon></span>
  </div>
</div>

<style>
  .wrap {
    display: flex;
    justify-content: center;
    margin: 22px 0 6px;
  }
  .dlg {
    --win: #1b5fb5;
    position: relative;
    width: min(460px, 100%);
    padding: 22px 22px 18px;
    border-radius: 6px;
    background: var(--win);
    color: #fff;
    font-family: "Segoe UI", system-ui, sans-serif;
    box-shadow: 0 30px 70px -24px rgb(0 0 0 / 70%);
  }
  .head {
    position: relative;
    height: 26px;
    margin-bottom: 8px;
  }
  .head strong {
    position: absolute;
    inset: 0;
    font-size: 19px;
    font-weight: 600;
  }
  .txt,
  .info {
    margin: 0 0 12px;
    font-size: 13.5px;
    line-height: 1.5;
  }
  .info {
    font-size: 12.5px;
    opacity: 0;
    max-height: 0;
    overflow: hidden;
    animation: reveal 7s ease-in-out infinite;
  }
  .lbl {
    font-weight: 700;
    margin-left: 4px;
  }
  .actions {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
    min-height: 34px;
  }
  .btn {
    padding: 7px 18px;
    border: 1px solid rgb(255 255 255 / 70%);
    font-size: 13px;
  }
  .run {
    opacity: 0;
    animation: runIn 7s ease-in-out infinite;
  }
  .dont {
    background: #fff;
    color: #17407a;
    font-weight: 600;
  }
  .more {
    position: absolute;
    left: 22px;
    bottom: 24px;
    font-size: 13px;
    text-decoration: underline;
    animation: moreOut 7s ease-in-out infinite;
  }
  .cursor {
    position: absolute;
    left: 40px;
    bottom: 16px;
    color: #fff;
    filter: drop-shadow(0 2px 4px rgb(0 0 0 / 60%));
    --m3e-icon-size: 26px;
    animation: cursor 7s ease-in-out infinite;
  }
  .a {
    animation: fadeOut 7s ease-in-out infinite;
  }
  .b {
    opacity: 0;
    animation: fadeIn 7s ease-in-out infinite;
  }
  @keyframes cursor {
    0%,
    10% {
      transform: translate(150px, -60px);
      opacity: 0;
    }
    22% {
      transform: translate(0, 0);
      opacity: 1;
    }
    30% {
      transform: translate(0, 0) scale(0.8);
    }
    36% {
      transform: translate(0, 0) scale(1);
    }
    52% {
      transform: translate(290px, 0);
    }
    58% {
      transform: translate(290px, 0) scale(0.8);
    }
    64%,
    100% {
      transform: translate(290px, 0) scale(1);
      opacity: 1;
    }
    96% {
      opacity: 0;
    }
  }
  @keyframes moreOut {
    0%,
    34% {
      opacity: 1;
    }
    40%,
    100% {
      opacity: 0;
    }
  }
  @keyframes reveal {
    0%,
    34% {
      opacity: 0;
      max-height: 0;
    }
    42%,
    94% {
      opacity: 1;
      max-height: 60px;
    }
    100% {
      opacity: 0;
      max-height: 0;
    }
  }
  @keyframes runIn {
    0%,
    38% {
      opacity: 0;
      transform: translateY(6px);
    }
    48%,
    94% {
      opacity: 1;
      transform: none;
    }
    100% {
      opacity: 0;
    }
  }
  @keyframes fadeOut {
    0%,
    100% {
      opacity: 1;
    }
  }
  @keyframes fadeIn {
    0%,
    100% {
      opacity: 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .info,
    .run,
    .more,
    .cursor,
    .a,
    .b {
      animation: none;
    }
    .info {
      opacity: 1;
      max-height: 60px;
    }
    .run {
      opacity: 1;
    }
    .more,
    .cursor,
    .b {
      display: none;
    }
  }
</style>
