<script lang="ts">
  // Where each claim came from, when we checked, and the trademark note.
  import { CHECKED, type Source } from "../../lib/compare";
  import { inview } from "../inview";

  let { sources }: { sources: Source[] } = $props();
</script>

<section class="pg-section src" id="sources">
  <div class="pg-wide narrow" use:inview>
    <span class="eyebrow">Sources</span>
    <h2 class="pg-title">Where this <em>comes from</em></h2>
    <p class="pg-lead">Checked on {CHECKED}. Apps change their plans, prices and platforms, so check the app's own site before you decide. We make Velmora, so we've kept to facts you can verify below.</p>
    <ul>
      {#each sources as s (s.url)}
        <li><a href={s.url} target="_blank" rel="noopener">{s.label}<m3e-icon name="arrow_outward"></m3e-icon></a></li>
      {/each}
    </ul>
    <p class="tm">Names and logos belong to their owners and appear here only to identify each app. Velmora isn't affiliated with, endorsed by or sponsored by any of them. Spot something out of date? <a href="/contact/">Tell us</a> and we'll fix it.</p>
  </div>
</section>

<style>
  .narrow {
    width: min(820px, 100% - 48px);
    margin-inline: auto;
  }
  ul {
    list-style: none;
    margin: 22px 0 0;
    padding: 0;
    display: grid;
    gap: 8px;
  }
  a {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: #c9c1ff;
    font-size: 15px;
    text-decoration: none;
    --m3e-icon-size: 16px;
  }
  a:hover {
    text-decoration: underline;
  }
  .tm {
    margin: 26px 0 0;
    padding: 16px 20px;
    border-radius: 18px;
    border: 1px solid var(--line);
    background: var(--glass);
    color: var(--muted);
    font-size: 13.5px;
    line-height: 1.6;
  }
  @media (max-width: 600px) {
    .narrow {
      width: calc(100% - 32px);
    }
  }
</style>
