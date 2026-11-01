<script lang="ts">
  import { KIND_ICON, KIND_LABEL, longDate, NEXT, RELEASES, type ChangeKind } from "../lib/changes";
  import { reveal } from "../lib/reveal";
  import Shapes from "./Shapes.svelte";
  import AndroidSoon from "./AndroidSoon.svelte";
  import DownloadCount from "./DownloadCount.svelte";
  import { latest } from "../lib/latest.svelte";
  import { formatCount } from "../lib/release";
  import { screen } from "../lib/media.svelte";

  // The site's own changelog: a timeline of releases with highlights and
  // grouped changes, filterable by kind.
  type Filter = "all" | ChangeKind;
  let filter = $state<Filter>("all");
  const FILTERS: Filter[] = ["all", "new", "improved", "fixed"];

  // Phones fold each release's change list away until it's asked for.
  let opened = $state<string[]>([]);
  const toggle = (v: string) => (opened = opened.includes(v) ? opened.filter((x) => x !== v) : [...opened, v]);

  const released = RELEASES.filter((r) => !r.soon);
  const latestVersion = released[0]?.version;

  const count = (k: Filter) =>
    k === "all" ? RELEASES.reduce((n, r) => n + r.changes.length, 0) : RELEASES.reduce((n, r) => n + r.changes.filter((c) => c.kind === k).length, 0);

  const visible = $derived(
    RELEASES.map((r) => ({ ...r, shown: filter === "all" ? r.changes : r.changes.filter((c) => c.kind === filter) })).filter(
      (r) => filter === "all" || r.shown.length,
    ),
  );
</script>

<section id="changelog">
  <Shapes
    spots={[
      { x: "86%", y: "230px", size: 130, color: "#7b6cff", spin: 22 },
      { x: "70%", y: "150px", size: 50, color: "#ff5a5a", spin: -12, offset: 5 },
      { x: "95%", y: "380px", size: 64, color: "#ffb74d", spin: 16, offset: 8 },
    ]}
  />
  <div class="wrap">
    <div use:reveal>
      <span class="eyebrow">What's new</span>
      <h1 class="title">Shipped <em>recently.</em></h1>
      <p class="lead">Every Velmora release and what changed in it. The app updates itself, so you only need this page if you're curious.</p>
      <div class="total"><DownloadCount /></div>
    </div>

    <div class="next" use:reveal={0.1}><AndroidSoon title={NEXT.title} text={NEXT.text} /></div>

    <div class="controls" use:reveal={0.15}>
      <div class="filters" role="group" aria-label="Show changes">
        {#each FILTERS as f (f)}
          <button class="filter" class:on={filter === f} aria-pressed={filter === f} onclick={() => (filter = f)}>
            {#if f !== "all"}<m3e-icon name={KIND_ICON[f]}></m3e-icon>{/if}
            {f === "all" ? "Everything" : KIND_LABEL[f]}
            <span class="count">{count(f)}</span>
          </button>
        {/each}
      </div>
      <nav class="jump" aria-label="Releases">
        {#each RELEASES as r (r.version)}<a href="#v{r.version}" style:--c={r.color}>{r.version}</a>{/each}
      </nav>
    </div>

    <ol class="timeline">
      {#each visible as r (r.version)}
        <li id="v{r.version}" style:--c={r.color} use:reveal>
          <aside class="meta">
            <span class="on-shape node" aria-hidden="true">
              <m3e-shape name={r.shape}></m3e-shape>
            </span>
            <span class="version">v{r.version}</span>
            <span class="name display">{r.name}</span>
            <span class="date">{r.soon ? "Coming soon" : longDate(r.date)}</span>
            {#if !r.soon && latest.downloadsOf(r.version) !== null}
              <span class="dl"><m3e-icon name="download"></m3e-icon>{formatCount(latest.downloadsOf(r.version) ?? 0)} downloads</span>
            {/if}
            {#if r.soon}
              <span class="badge soon">In testing</span>
            {:else if r.version === latestVersion}
              <span class="badge">Latest</span>
            {/if}
          </aside>

          <article class="card">
            <p class="summary">{r.summary}</p>

            {#if filter === "all" && r.highlights?.length}
              <div class="highlights swipe">
                {#each r.highlights as h (h.title)}
                  <div class="hl">
                    <span class="hi"><m3e-icon name={h.icon}></m3e-icon></span>
                    <strong>{h.title}</strong>
                    <span>{h.text}</span>
                  </div>
                {/each}
              </div>
            {/if}

            {#if !screen.phone || filter !== "all" || opened.includes(r.version)}
              <ul class="changes">
                {#each r.shown as c (c.title)}
                  <li>
                    <span class="kind k-{c.kind}"><m3e-icon name={KIND_ICON[c.kind]}></m3e-icon>{KIND_LABEL[c.kind]}</span>
                    <div><strong>{c.title}</strong><span>{c.text}</span></div>
                  </li>
                {/each}
              </ul>
            {/if}
            {#if screen.phone && filter === "all"}
              <button class="more" aria-expanded={opened.includes(r.version)} onclick={() => toggle(r.version)}>
                {opened.includes(r.version) ? "Hide changes" : `Show ${r.changes.length} ${r.changes.length === 1 ? "change" : "changes"}`}
                <m3e-icon name={opened.includes(r.version) ? "keyboard_arrow_up" : "keyboard_arrow_down"}></m3e-icon>
              </button>
            {/if}
          </article>
        </li>
      {/each}
    </ol>
  </div>
</section>

<style>
  section {
    padding-top: 150px;
  }
  .next {
    margin-top: 48px;
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-top: 64px;
  }
  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .filter {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 40px;
    padding: 0 14px;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: rgb(255 255 255 / 3%);
    color: var(--muted);
    font: 600 14px var(--font);
    cursor: pointer;
    --m3e-icon-size: 18px;
    transition:
      background 0.25s,
      color 0.25s,
      border-color 0.25s;
  }
  .filter:hover {
    color: #fff;
  }
  .filter.on {
    background: #fff;
    color: #140f22;
    border-color: #fff;
  }
  .count {
    min-width: 22px;
    padding: 1px 7px;
    border-radius: 999px;
    font-size: 12px;
    background: rgb(255 255 255 / 8%);
  }
  .filter.on .count {
    background: rgb(20 15 34 / 10%);
  }
  .jump {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .jump a {
    padding: 6px 12px;
    border-radius: 10px;
    font-size: 13px;
    font-weight: 600;
    text-decoration: none;
    color: var(--muted);
    border: 1px solid var(--line);
    transition:
      color 0.2s,
      border-color 0.2s;
  }
  .jump a:hover {
    color: #fff;
    border-color: var(--c);
  }

  /* Timeline: a line down the left with a shape for each release. */
  .timeline {
    position: relative;
    list-style: none;
    margin: 40px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }
  .timeline::before {
    content: "";
    position: absolute;
    left: 27px;
    top: 28px;
    bottom: 28px;
    width: 2px;
    background: linear-gradient(180deg, rgb(255 255 255 / 18%), rgb(255 255 255 / 4%));
  }
  .timeline > li {
    display: grid;
    grid-template-columns: 260px minmax(0, 1fr);
    gap: 32px;
    scroll-margin-top: 100px;
  }
  .meta {
    position: sticky;
    top: 100px;
    align-self: start;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    padding-left: 76px;
    min-height: 56px;
  }
  .node {
    position: absolute;
    left: 0;
    top: 0;
    --m3e-shape-size: 56px;
    --m3e-shape-container-color: var(--c);
    filter: drop-shadow(0 0 18px color-mix(in srgb, var(--c) 60%, transparent));
    animation: spin 18s linear infinite;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  .version {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--c);
  }
  .name {
    font-size: 34px;
    line-height: 1.02;
  }
  .date {
    font-size: 13px;
    color: var(--muted);
  }
  .dl {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12.5px;
    color: var(--muted);
    --m3e-icon-size: 15px;
  }
  .total {
    margin-top: 20px;
  }
  .badge {
    margin-top: 6px;
    padding: 3px 10px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 700;
    background: var(--c);
    color: #140f22;
  }
  .badge.soon {
    background: transparent;
    color: var(--c);
    border: 1px dashed var(--c);
  }

  .card {
    padding: 30px;
    border-radius: 30px;
    border: 1px solid var(--line);
    background:
      radial-gradient(90% 70% at 100% 0%, color-mix(in srgb, var(--c) 14%, transparent), transparent 60%),
      rgb(255 255 255 / 3%);
  }
  .summary {
    margin: 0;
    font-size: 19px;
    line-height: 1.55;
  }
  .highlights {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
    margin-top: 24px;
  }
  .hl {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    column-gap: 14px;
    row-gap: 3px;
    padding: 18px;
    border-radius: 20px;
    background: rgb(255 255 255 / 4%);
  }
  .hi {
    grid-row: span 2;
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    border-radius: 13px;
    background: color-mix(in srgb, var(--c) 22%, transparent);
    color: color-mix(in srgb, var(--c) 60%, #fff);
    --m3e-icon-size: 22px;
  }
  .hl strong {
    font-size: 15.5px;
  }
  .hl span:last-child {
    font-size: 14px;
    line-height: 1.5;
    color: var(--muted);
  }
  .changes {
    list-style: none;
    margin: 24px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
  }
  .changes li {
    display: grid;
    grid-template-columns: 116px minmax(0, 1fr);
    gap: 16px;
    padding: 14px 0;
    border-top: 1px solid var(--line);
  }
  .changes li div {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .changes strong {
    font-size: 15.5px;
  }
  .changes li div span {
    font-size: 14.5px;
    line-height: 1.55;
    color: var(--muted);
  }
  .kind {
    align-self: start;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    width: fit-content;
    padding: 4px 10px 4px 8px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    --m3e-icon-size: 16px;
  }
  .k-new {
    background: rgb(123 108 255 / 18%);
    color: #c9c1ff;
  }
  .k-improved {
    background: rgb(45 212 191 / 15%);
    color: #8ff0de;
  }
  .k-fixed {
    background: rgb(255 183 77 / 15%);
    color: #ffd59a;
  }

  @media (max-width: 860px) {
    .timeline > li {
      grid-template-columns: minmax(0, 1fr);
      gap: 16px;
    }
    .meta {
      position: relative;
      top: 0;
    }
    .highlights {
      grid-template-columns: minmax(0, 1fr);
    }
    .changes li {
      grid-template-columns: minmax(0, 1fr);
      gap: 8px;
    }
    .card {
      padding: 22px;
    }
  }
  .more {
    display: none;
  }
  /* Phones: no timeline line, a compact header above each card, swipeable
     highlights and folded change lists. */
  @media (max-width: 600px) {
    section {
      padding-top: 110px;
    }
    .controls {
      margin-top: 40px;
    }
    .filter {
      height: 36px;
      padding: 0 12px;
      font-size: 13px;
    }
    .jump {
      display: none;
    }
    .timeline {
      gap: 36px;
      margin-top: 28px;
    }
    .timeline::before {
      display: none;
    }
    .timeline > li {
      gap: 14px;
    }
    .meta {
      display: grid;
      grid-template-columns: 44px minmax(0, 1fr) auto;
      grid-template-areas:
        "node version badge"
        "node name name"
        "node date date"
        "node dl dl";
      column-gap: 14px;
      row-gap: 2px;
      align-items: center;
      padding-left: 0;
      min-height: 0;
    }
    .node {
      position: static;
      grid-area: node;
      align-self: start;
      --m3e-shape-size: 44px;
    }
    .version {
      grid-area: version;
    }
    .badge {
      grid-area: badge;
      margin-top: 0;
      justify-self: end;
    }
    .name {
      grid-area: name;
      font-size: 26px;
    }
    .date {
      grid-area: date;
    }
    .dl {
      grid-area: dl;
      margin-top: 2px;
    }
    .card {
      padding: 20px;
      border-radius: 24px;
    }
    .summary {
      font-size: 16.5px;
    }
    .highlights {
      margin-top: 18px;
      grid-auto-columns: 78%;
    }
    .hl {
      padding: 16px;
    }
    .changes {
      margin-top: 8px;
    }
    .changes li {
      gap: 6px;
      padding: 12px 0;
    }
    .more {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 4px;
      width: 100%;
      height: 42px;
      margin-top: 14px;
      border: 1px solid var(--line);
      border-radius: 999px;
      background: rgb(255 255 255 / 4%);
      color: var(--ink);
      font: 600 14px var(--font);
      cursor: pointer;
      --m3e-icon-size: 20px;
    }
  }
</style>
