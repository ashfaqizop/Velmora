<script lang="ts">
  import { FAQ, FAQ_GROUPS, type FaqGroup } from "../lib/faq";
  import { inview } from "./inview";
  import PageShell from "./PageShell.svelte";
  import Related from "./Related.svelte";

  let query = $state("");
  let group = $state<FaqGroup | "all">("all");

  const needle = $derived(query.trim().toLowerCase());
  const visible = $derived(
    FAQ.filter((f) => (group === "all" || f.group === group) && (!needle || `${f.q} ${f.a}`.toLowerCase().includes(needle))),
  );
</script>

<PageShell
  eyebrow="FAQ"
  title="Good"
  accent="questions."
  crumb="FAQ"
  lead="Is it free, is it safe, what does it keep, and what happens when Windows complains. Search or browse."
  spots={[
    { x: "90%", y: "190px", size: 110, color: "#ffb74d", spin: 20, opacity: 0.55 },
    { x: "6%", y: "360px", size: 56, color: "#3271fe", spin: -15, offset: 4, opacity: 0.7 },
  ]}
>
  <section class="pg-section tight">
    <div class="pg-wide narrow">
      <label class="search">
        <m3e-icon name="search"></m3e-icon>
        <input type="search" bind:value={query} placeholder="Search the questions" aria-label="Search the questions" autocomplete="off" spellcheck="false" />
        {#if query}
          <m3e-icon-button aria-label="Clear search" onclick={() => (query = "")}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
        {/if}
      </label>

      <m3e-filter-chip-set class="chips" aria-label="Topic">
        <m3e-filter-chip selected={group === "all"} onclick={() => (group = "all")}>All</m3e-filter-chip>
        {#each FAQ_GROUPS as g (g.id)}
          <m3e-filter-chip selected={group === g.id} onclick={() => (group = g.id)}>{g.label}</m3e-filter-chip>
        {/each}
      </m3e-filter-chip-set>

      <p class="count" aria-live="polite">{visible.length} {visible.length === 1 ? "answer" : "answers"}</p>

      {#each FAQ_GROUPS as g (g.id)}
        {@const items = visible.filter((f) => f.group === g.id)}
        {#if items.length}
          <div class="group" id={g.id}>
            <h2><m3e-icon name={g.icon}></m3e-icon>{g.label}</h2>
            <m3e-accordion class="list">
              {#each items as f (f.q)}
                <m3e-expansion-panel>
                  <span slot="header" class="q">{f.q}</span>
                  <p>{f.a}</p>
                </m3e-expansion-panel>
              {/each}
            </m3e-accordion>
          </div>
        {/if}
      {/each}

      {#if !visible.length}
        <div class="none" use:inview>
          <m3e-icon name="search_off"></m3e-icon>
          <strong>Nothing matches "{query}"</strong>
          <span>Try fewer words, or <a href="/contact/">ask us directly</a>.</span>
        </div>
      {/if}
    </div>
  </section>

  <Related
    items={[
      { href: "/contact/", icon: "forum", title: "Still stuck?", text: "Report a bug, ask a question or suggest something." },
      { href: "/docs/install/", icon: "menu_book", title: "Install guide", text: "Step by step, with the SmartScreen warning explained." },
      { href: "/disclaimer/", icon: "gavel", title: "Content and the law", text: "What Velmora does and doesn't do with content, and how to send a notice." },
    ]}
  />
</PageShell>

<style>
  section.tight {
    padding-top: 40px;
  }
  .narrow {
    width: min(820px, 100% - 48px);
    margin-inline: auto;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 58px;
    padding: 0 18px;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: var(--glass);
    color: var(--muted);
    transition:
      border-color 0.2s,
      box-shadow 0.2s,
      background 0.2s;
  }
  .search:focus-within {
    border-color: color-mix(in srgb, var(--blue) 70%, transparent);
    background: rgb(255 255 255 / 7%);
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--blue) 22%, transparent);
  }
  .search input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: none;
    color: var(--ink);
    font: inherit;
    font-size: 17px;
  }
  .search input::-webkit-search-cancel-button {
    display: none;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 18px;
  }
  .count {
    color: var(--muted);
    margin: 22px 0 0;
    font-size: 13px;
  }
  .group {
    margin-top: 34px;
    animation: pop 0.45s cubic-bezier(0.2, 0.7, 0.2, 1) both;
  }
  h2 {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0 0 14px;
    font-size: 19px;
    --m3e-icon-size: 24px;
  }
  h2 m3e-icon {
    color: var(--blue);
  }
  .list {
    display: grid;
    gap: 8px;
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
  p {
    margin: 0;
    color: var(--muted);
    font-size: 15.5px;
    line-height: 1.7;
  }
  .none {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    margin-top: 40px;
    padding: 48px 20px;
    text-align: center;
    border-radius: 28px;
    border: 1px dashed var(--line);
    color: var(--muted);
    --m3e-icon-size: 40px;
  }
  .none strong {
    color: var(--ink);
  }
  .none a {
    color: #c9c1ff;
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
  }
  @media (max-width: 600px) {
    .narrow {
      width: calc(100% - 32px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .group {
      animation: none;
    }
  }
</style>
