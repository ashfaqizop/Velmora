<script lang="ts">
  import { APPS, KINDS, type AltData } from "../../lib/compare";
  import { inview } from "../inview";
  import PageShell from "../PageShell.svelte";
  import Related from "../Related.svelte";
  import LogoTile from "./LogoTile.svelte";
  import Sources from "./Sources.svelte";

  let { d }: { d: AltData } = $props();

  const all = $derived(d.groups.flatMap((g) => g.apps.map((k) => APPS[k])));
</script>

<PageShell
  eyebrow="Compare"
  title={d.title}
  accent={d.accent}
  crumb={d.crumb}
  lead={d.lead}
  spots={[
    { x: "92%", y: "190px", size: 118, color: "#ffb74d", spin: 22, opacity: 0.5 },
    { x: "5%", y: "420px", size: 52, color: "#8fb4ff", spin: -14, offset: 4, opacity: 0.65 },
  ]}
>
  <section class="pg-section tight">
    <div class="pg-wide">
      <p class="pg-lead intro" use:inview>{d.intro}</p>

      <div class="glance" use:inview={0.05}>
        <table>
          <caption class="sr">The apps at a glance</caption>
          <thead>
            <tr><th scope="col">App</th><th scope="col">Price</th><th scope="col">On Windows</th><th scope="col">Source code</th></tr>
          </thead>
          <tbody>
            {#each all as a (a.name)}
              <tr>
                <th scope="row"><span class="who"><LogoTile src={a.logo} name={a.name} color={a.color} shape={a.shape} size={34} />{a.name}</span></th>
                <td data-h="Price">{a.price}</td>
                <td data-h="On Windows">{a.windows}</td>
                <td data-h="Source code">{a.source}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  </section>

  {#each d.groups as g (g.kind)}
    {@const k = KINDS[g.kind]}
    <section class="pg-section">
      <div class="pg-wide">
        <span class="eyebrow" use:inview><m3e-icon name={k.icon}></m3e-icon>{k.label}</span>
        <h2 class="pg-title" use:inview={0.04}>{k.label}</h2>
        <p class="pg-lead" use:inview={0.08}>{g.note}</p>
        <div class="cards">
          {#each g.apps as key, i (key)}
            {@const a = APPS[key]}
            <article class="pg-card app" class:mine={key === "velmora"} style="--c: {a.color}" use:inview={i * 0.07}>
              <header>
                <LogoTile src={a.logo} name={a.name} color={a.color} shape={a.shape} size={56} />
                <div>
                  <h3>{a.name}</h3>
                  <span class="meta">{a.price}</span>
                </div>
              </header>
              <dl>
                <div><dt>What it is</dt><dd>{a.what}</dd></div>
                <div><dt>Best for</dt><dd>{a.bestFor}</dd></div>
                <div><dt>Watch out for</dt><dd>{a.watchOut}</dd></div>
                <div><dt>On Windows</dt><dd>{a.windows}</dd></div>
              </dl>
              <a class="go" href={a.url} target={a.url.startsWith("http") ? "_blank" : undefined} rel={a.url.startsWith("http") ? "noopener" : undefined}>
                {key === "velmora" ? "Download Velmora" : `Visit ${a.name}`}<m3e-icon name={a.url.startsWith("http") ? "arrow_outward" : "arrow_forward"}></m3e-icon>
              </a>
            </article>
          {/each}
        </div>
      </div>
    </section>
  {/each}

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
    </div>
  </section>

  <Sources sources={d.sources} />
  <Related items={d.related} />
</PageShell>

<style>
  section.tight {
    padding-top: 40px;
  }
  .intro {
    margin: 0 0 28px;
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
  .glance {
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
    padding: 16px 20px;
    text-align: left;
    font-size: 12.5px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    background: rgb(255 255 255 / 4%);
    border-bottom: 1px solid var(--line);
  }
  tbody th,
  td {
    padding: 14px 20px;
    text-align: left;
    vertical-align: middle;
    border-bottom: 1px solid var(--line);
    font-size: 14.5px;
    line-height: 1.5;
  }
  tbody th {
    font-weight: 700;
    color: var(--ink);
  }
  td {
    color: var(--muted);
  }
  tbody tr:last-child th,
  tbody tr:last-child td {
    border-bottom: 0;
  }
  .who {
    display: inline-flex;
    align-items: center;
    gap: 12px;
    font-size: 16px;
  }
  .eyebrow {
    --m3e-icon-size: 18px;
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(310px, 1fr));
    gap: 16px;
    margin-top: 30px;
  }
  .app {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .app.mine {
    border-color: color-mix(in srgb, var(--blue) 50%, transparent);
    background: color-mix(in srgb, var(--blue) 9%, transparent);
  }
  header {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  header h3 {
    margin: 0 0 2px;
    font-size: 21px;
  }
  .meta {
    color: var(--muted);
    font-size: 13.5px;
    line-height: 1.4;
  }
  dl {
    display: grid;
    gap: 14px;
    margin: 0;
  }
  dt {
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--c);
    margin-bottom: 2px;
  }
  dd {
    margin: 0;
    color: var(--muted);
    font-size: 15px;
    line-height: 1.6;
  }
  .go {
    margin-top: auto;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-weight: 700;
    font-size: 14.5px;
    text-decoration: none;
    color: var(--c);
    --m3e-icon-size: 18px;
  }
  .go:hover m3e-icon {
    translate: 3px 0;
  }
  .go m3e-icon {
    transition: translate 0.25s;
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
  @media (max-width: 760px) {
    .narrow {
      width: calc(100% - 32px);
    }
    thead {
      display: none;
    }
    tbody tr {
      display: block;
      padding: 12px 4px;
      border-bottom: 1px solid var(--line);
    }
    tbody tr:last-child {
      border-bottom: 0;
    }
    tbody th,
    td {
      display: block;
      border: 0;
      padding: 5px 16px;
    }
    td::before {
      content: attr(data-h) ": ";
      font-weight: 700;
      color: var(--ink);
    }
  }
</style>
