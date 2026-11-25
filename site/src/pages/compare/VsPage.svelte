<script lang="ts">
  import DownloadButton from "../../components/DownloadButton.svelte";
  import { onMount } from "svelte";
  import { latest } from "../../lib/latest.svelte";
  import { VELMORA_LOGO, type VsData } from "../../lib/compare";
  import { inview } from "../inview";
  import PageShell from "../PageShell.svelte";
  import Related from "../Related.svelte";
  import LogoTile from "./LogoTile.svelte";
  import Sources from "./Sources.svelte";

  let { d }: { d: VsData } = $props();

  onMount(() => {
    if (!latest.loaded) latest.load();
  });
</script>

<PageShell
  eyebrow="Compare"
  title="Velmora vs"
  accent={d.name}
  crumb="Velmora vs {d.name}"
  lead={d.lead}
  spots={[
    { x: "92%", y: "200px", size: 120, color: d.color, spin: 24, opacity: 0.5 },
    { x: "4%", y: "430px", size: 54, color: "#3271fe", spin: -15, offset: 5, opacity: 0.65 },
  ]}
>
  {#snippet hero()}
    <div class="vs">
      <LogoTile src={VELMORA_LOGO} name="Velmora" size={84} />
      <span class="badge">vs</span>
      <LogoTile src={d.logo} name={d.name} color={d.color} size={84} />
    </div>
  {/snippet}

  <section class="pg-section tight">
    <div class="pg-wide">
      <div class="wrap" use:inview>
        <table>
          <caption class="sr">Velmora and {d.name} compared</caption>
          <thead>
            <tr>
              <th scope="col"><span class="sr">Feature</span></th>
              <th scope="col"><span class="who"><LogoTile src={VELMORA_LOGO} name="Velmora" size={34} />Velmora</span></th>
              <th scope="col"><span class="who"><LogoTile src={d.logo} name={d.name} color={d.color} size={34} />{d.name}</span></th>
            </tr>
          </thead>
          <tbody>
            {#each d.rows as r (r.label)}
              <tr>
                <th scope="row">{r.label}</th>
                <td data-app="Velmora">{r.velmora}</td>
                <td data-app={d.name}>{r.other}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide">
      <span class="eyebrow" use:inview>What they share</span>
      <ul class="alike" use:inview={0.05}>
        {#each d.alike as a (a)}<li><m3e-icon name="check_circle" filled></m3e-icon>{a}</li>{/each}
      </ul>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide">
      <span class="eyebrow" use:inview>Which to choose</span>
      <h2 class="pg-title" use:inview={0.05}>Pick the one that <em>fits you</em></h2>
      <div class="picks">
        <div class="pick other" use:inview>
          <header><LogoTile src={d.logo} name={d.name} color={d.color} size={44} /><h3>Choose {d.name} if</h3></header>
          <ul>{#each d.pickOther as p (p)}<li>{p}</li>{/each}</ul>
        </div>
        <div class="pick vel" use:inview={0.08}>
          <header><LogoTile src={VELMORA_LOGO} name="Velmora" size={44} /><h3>Choose Velmora if</h3></header>
          <ul>{#each d.pickVelmora as p (p)}<li>{p}</li>{/each}</ul>
        </div>
      </div>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide narrow">
      <span class="eyebrow" use:inview>Straight talk</span>
      <h2 class="pg-title" use:inview={0.05}>Where Velmora <em>falls short</em></h2>
      <ul class="limits" use:inview={0.1}>
        {#each d.limits as l (l)}<li><m3e-icon name="info"></m3e-icon>{l}</li>{/each}
      </ul>
      <p class="pg-lead" use:inview>Velmora doesn't host or control any of the content it finds. You're responsible for what you watch and for following the law where you live. See the <a href="/disclaimer/">disclaimer</a>.</p>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide narrow">
      <span class="eyebrow" use:inview>Questions</span>
      <m3e-accordion class="faq" use:inview={0.05}>
        {#each d.faq as f (f.q)}
          <m3e-expansion-panel>
            <span slot="header" class="q">{f.q}</span>
            <p>{f.a}</p>
          </m3e-expansion-panel>
        {/each}
      </m3e-accordion>
      <div class="cta" use:inview><DownloadButton /></div>
    </div>
  </section>

  <Sources sources={d.sources} />
  <Related items={d.related} />
</PageShell>

<style>
  .vs {
    display: flex;
    align-items: center;
    gap: 18px;
    margin-top: 30px;
  }
  .badge {
    display: grid;
    place-items: center;
    width: 46px;
    height: 46px;
    border-radius: 50%;
    font-family: var(--display);
    font-style: italic;
    font-size: 24px;
    color: var(--cream);
    border: 1px solid var(--line);
    background: var(--glass);
    animation: pulse 3.2s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      transform: scale(1.12);
      box-shadow: 0 0 28px color-mix(in srgb, var(--blue) 60%, transparent);
    }
  }
  section.tight {
    padding-top: 40px;
  }
  .narrow {
    width: min(820px, 100% - 48px);
    margin-inline: auto;
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
  .wrap {
    border: 1px solid var(--line);
    border-radius: 28px;
    overflow: hidden;
    background: var(--glass);
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  thead th {
    padding: 18px 22px;
    text-align: left;
    background: rgb(255 255 255 / 4%);
    border-bottom: 1px solid var(--line);
  }
  .who {
    display: inline-flex;
    align-items: center;
    gap: 12px;
    font-size: 17px;
    font-weight: 700;
  }
  tbody th,
  td {
    padding: 18px 22px;
    text-align: left;
    vertical-align: top;
    border-bottom: 1px solid var(--line);
    font-size: 15.5px;
    line-height: 1.6;
  }
  tbody th {
    width: 18%;
    color: var(--ink);
    font-weight: 700;
  }
  td {
    width: 41%;
    color: var(--muted);
  }
  td:nth-child(2) {
    background: color-mix(in srgb, var(--blue) 6%, transparent);
  }
  tbody tr:last-child th,
  tbody tr:last-child td {
    border-bottom: 0;
  }
  .alike {
    list-style: none;
    margin: 18px 0 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 12px;
  }
  .alike li,
  .limits li {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    padding: 16px 18px;
    border-radius: 20px;
    border: 1px solid var(--line);
    background: var(--glass);
    color: var(--muted);
    font-size: 15.5px;
    line-height: 1.55;
    --m3e-icon-size: 22px;
  }
  .alike m3e-icon {
    color: #4fd18b;
    flex: none;
  }
  .limits {
    list-style: none;
    margin: 22px 0 18px;
    padding: 0;
    display: grid;
    gap: 10px;
  }
  .limits m3e-icon {
    color: var(--blue);
    flex: none;
  }
  .picks {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 16px;
    margin-top: 30px;
  }
  .pick {
    padding: 26px;
    border-radius: 28px;
    border: 1px solid var(--line);
    background: var(--glass);
  }
  .pick.vel {
    border-color: color-mix(in srgb, var(--blue) 45%, transparent);
    background: color-mix(in srgb, var(--blue) 9%, transparent);
  }
  .pick header {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 14px;
  }
  .pick h3 {
    margin: 0;
    font-size: 19px;
  }
  .pick ul {
    margin: 0;
    padding-left: 20px;
    display: grid;
    gap: 10px;
    color: var(--muted);
    font-size: 15.5px;
    line-height: 1.6;
  }
  .faq {
    display: grid;
    gap: 8px;
    margin-top: 18px;
    --m3e-expansion-panel-container-color: rgb(255 255 255 / 4%);
    --m3e-expansion-panel-shape: 22px;
    --m3e-expansion-header-collapsed-height: auto;
    --m3e-expansion-header-expanded-height: auto;
  }
  .q {
    display: block;
    padding: 4px 0;
    font-weight: 600;
    font-size: 16.5px;
  }
  .faq p {
    margin: 0;
    color: var(--muted);
    font-size: 15.5px;
    line-height: 1.7;
  }
  .cta {
    margin-top: 30px;
  }
  @media (max-width: 760px) {
    .narrow {
      width: calc(100% - 32px);
    }
    thead {
      display: none;
    }
    tbody tr {
      display: block;
      padding: 14px 4px;
      border-bottom: 1px solid var(--line);
    }
    tbody tr:last-child {
      border-bottom: 0;
    }
    tbody th,
    td {
      display: block;
      width: auto;
      border: 0;
      padding: 6px 18px;
    }
    td::before {
      content: attr(data-app);
      display: block;
      margin-bottom: 2px;
      font-size: 11.5px;
      font-weight: 700;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--ink);
    }
    td:nth-child(2) {
      background: none;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .badge {
      animation: none;
    }
  }
</style>
