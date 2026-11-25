<script lang="ts">
  import { FAQ as ALL_FAQ } from "../lib/faq";
  import { reveal } from "../lib/reveal";

  const HOME_FAQ = ALL_FAQ.filter((x) => x.home);

  const FAQ = HOME_FAQ.map((x) => [x.q, x.a]);
</script>

<section id="faq">
  <div class="wrap narrow">
    <div use:reveal>
      <span class="eyebrow">FAQ</span>
      <h2 class="title">Good <em>questions.</em></h2>
    </div>
    <m3e-accordion class="list" use:reveal={0.1}>
      {#each FAQ as [q, a] (q)}
        <m3e-expansion-panel>
          <span slot="header" class="q">{q}</span>
          <p>{a}</p>
        </m3e-expansion-panel>
      {/each}
    </m3e-accordion>
  </div>
</section>

<style>
  .narrow {
    max-width: 860px;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 48px;
    --m3e-expansion-panel-container-color: rgb(255 255 255 / 4%);
    --m3e-expansion-panel-shape: 22px;
    --m3e-expansion-header-collapsed-height: auto;
    --m3e-expansion-header-expanded-height: auto;
  }
  .q {
    display: block;
    padding: 16px 0;
    font-size: 17px;
    font-weight: 600;
    line-height: 1.35;
    /* m3e cuts headers off with an ellipsis; long questions should wrap. */
    white-space: normal;
  }
  p {
    margin: 0;
    font-size: 15.5px;
    line-height: 1.65;
    color: var(--muted);
  }
</style>
