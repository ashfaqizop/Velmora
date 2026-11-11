<script lang="ts">
  import { reveal } from "../lib/reveal";

  const TIERS = [
    {
      icon: "verified",
      name: "Public",
      tag: "Free for everyone",
      points: [
        "The whole app: every source, the player, downloads, Ani, sports and Live TV",
        "A few quiet ads that help pay for development",
        "Never while a video plays, and none on your first launch",
      ],
    },
    {
      icon: "workspace_premium",
      name: "Max",
      tag: "Given, not sold",
      points: [
        "The same app with no ads at all",
        "Keys come from the developer, one per person",
        "Works on one PC at a time. Enter it in Settings, under License",
      ],
    },
  ];
</script>

<section id="tiers">
  <div class="wrap">
    <div use:reveal>
      <span class="eyebrow">Two ways to run it</span>
      <h2 class="title">Same app. <em>Your call on ads.</em></h2>
    </div>
    <div class="tiers">
      {#each TIERS as t, i (t.name)}
        <article class="tier" class:max={t.name === "Max"} use:reveal={0.1 + i * 0.08}>
          <header>
            <m3e-icon name={t.icon}></m3e-icon>
            <div>
              <strong>Velmora {t.name}</strong>
              <span>{t.tag}</span>
            </div>
          </header>
          <ul>
            {#each t.points as p (p)}<li>{p}</li>{/each}
          </ul>
        </article>
      {/each}
    </div>
  </div>
</section>

<style>
  .tiers {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 20px;
    margin-top: 40px;
  }
  .tier {
    padding: 32px;
    border-radius: 32px;
    border: 1px solid var(--line);
    background: var(--glass);
  }
  .tier.max {
    background: linear-gradient(160deg, rgb(123 108 255 / 22%), rgb(255 90 90 / 10%));
    border-color: rgb(123 108 255 / 40%);
  }
  header {
    display: flex;
    align-items: center;
    gap: 14px;
    --m3e-icon-size: 32px;
  }
  header m3e-icon {
    color: #c9c1ff;
  }
  header div {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  header strong {
    font-size: 20px;
  }
  header span {
    font-size: 13px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }
  ul {
    margin: 22px 0 0;
    padding-left: 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    font-size: 15px;
    line-height: 1.5;
    color: var(--muted);
  }
  @media (max-width: 760px) {
    .tiers {
      grid-template-columns: none;
    }
    .tier {
      padding: 24px 22px;
      border-radius: 26px;
    }
  }
</style>
