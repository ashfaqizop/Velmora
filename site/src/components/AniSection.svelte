<script lang="ts">
  import { reveal } from "../lib/reveal";

  const POINTS = [
    ["person_heart", "Learns your taste", "From what you search, watch and download. It stays on your PC, and you can see or clear it any time."],
    ["view_carousel", "Answers you can act on", "Recommendations come back as cards you can play, save or download from."],
    ["key", "Your key, your choice", "Ani runs on Google Gemini with your own free API key. It's off until you turn it on."],
  ];
  const CARDS = [
    ["Echo Valley", "#a78bfa", "#14213d"],
    ["Paper Moons", "#ff7a59", "#3b1d6e"],
    ["Glass Orchard", "#f472b6", "#1e1b4b"],
  ];
</script>

<section id="ani">
  <div class="wrap split">
    <div use:reveal>
      <span class="eyebrow">Meet Ani</span>
      <h2 class="title">An assistant that knows <em>what you watch.</em></h2>
      <p class="lead">
        Ask for something like the show you just finished, the right watch order for a long franchise, or the smallest 1080p download. Ani looks it up
        with Velmora's own tools before it answers.
      </p>
      <ul>
        {#each POINTS as [icon, title, text] (title)}
          <li>
            <span class="pi"><m3e-icon name={icon}></m3e-icon></span>
            <div><strong>{title}</strong><span>{text}</span></div>
          </li>
        {/each}
      </ul>
      <a class="more" href="/ani/">Everything Ani can do<m3e-icon name="arrow_forward"></m3e-icon></a>
    </div>

    <div class="chat" use:reveal={0.15} aria-hidden="true">
      <div class="msg me">Something like Low Tide, but shorter?</div>
      <div class="steps"><m3e-icon name="check_circle" filled></m3e-icon>Looked up 4 things · Thought for 3s</div>
      <div class="msg ani">
        Low Tide is slow and moody, so here are three short series with the same feel. <b>Echo Valley</b> is the closest: same writer, eight
        episodes.
      </div>
      <div class="cards">
        {#each CARDS as [name, a, b], i (name)}
          <div class="c" style:--a={a} style:--b={b}>
            {#if i === 0}<span class="for">For you</span>{/if}
            <span class="display">{name}</span>
          </div>
        {/each}
      </div>
      <div class="input"><span>Ask Ani anything</span><m3e-icon name="arrow_upward"></m3e-icon></div>
    </div>
  </div>
</section>

<style>
  .split {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 72px;
    align-items: center;
  }
  ul {
    list-style: none;
    margin: 36px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 22px;
  }
  li {
    display: flex;
    gap: 16px;
  }
  .pi {
    flex: none;
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border-radius: 14px;
    background: rgb(123 108 255 / 16%);
    color: #c9c1ff;
    --m3e-icon-size: 22px;
  }
  li div {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  li strong {
    font-size: 16px;
  }
  li span {
    color: var(--muted);
    line-height: 1.55;
    font-size: 15px;
  }
  .more {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-top: 32px;
    font-weight: 600;
    text-decoration: none;
    color: #c9c1ff;
    --m3e-icon-size: 18px;
    transition: gap 0.25s;
  }
  .more:hover {
    gap: 10px;
  }
  .chat {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 26px;
    border-radius: 32px;
    border: 1px solid var(--line);
    background:
      radial-gradient(90% 60% at 0% 0%, rgb(123 108 255 / 22%), transparent 60%),
      radial-gradient(70% 50% at 100% 100%, rgb(255 90 90 / 16%), transparent 60%),
      #15111f;
    box-shadow: 0 40px 100px -40px rgb(123 108 255 / 50%);
    font-size: 14px;
  }
  .msg {
    max-width: 85%;
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
  .steps {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--muted);
    --m3e-icon-size: 16px;
  }
  .steps m3e-icon {
    color: #b8f5c9;
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
  }
  .c {
    position: relative;
    aspect-ratio: 2 / 3;
    border-radius: 14px;
    background: linear-gradient(165deg, var(--a), var(--b) 78%);
    display: flex;
    align-items: flex-end;
    padding: 12px;
    font-size: 18px;
    line-height: 1.05;
  }
  .for {
    position: absolute;
    left: 10px;
    top: 10px;
    padding: 3px 9px;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 700;
    background: #ffd8e4;
    color: #31111d;
  }
  .input {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 4px;
    height: 48px;
    padding: 0 8px 0 18px;
    border-radius: 999px;
    background: rgb(255 255 255 / 6%);
    color: var(--muted);
  }
  .input m3e-icon {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    background: #fff;
    color: #140f22;
    --m3e-icon-size: 18px;
  }
  @media (max-width: 960px) {
    .split {
      grid-template-columns: minmax(0, 1fr);
      gap: 48px;
    }
  }
  @media (max-width: 600px) {
    .chat {
      padding: 18px;
      border-radius: 26px;
    }
    .c {
      font-size: 14px;
      padding: 8px;
    }
  }
</style>
