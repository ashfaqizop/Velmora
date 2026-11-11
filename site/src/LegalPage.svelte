<script lang="ts">
  import Backdrop from "./components/Backdrop.svelte";
  import Nav from "./components/Nav.svelte";
  import Footer from "./components/Footer.svelte";
  import { PRIVACY, TERMS, UPDATED } from "./lib/legal";

  let { page }: { page: "privacy" | "terms" } = $props();

  const doc = $derived(page === "privacy" ? PRIVACY : TERMS);
  const updated = new Date(`${UPDATED}T12:00:00`).toLocaleDateString(undefined, { day: "numeric", month: "long", year: "numeric" });
</script>

<Backdrop />
<Nav />
<main id="top">
  <article class="wrap narrow">
    <span class="eyebrow">Last updated {updated}</span>
    <h1 class="title">{doc.title}</h1>
    <p class="lead">{doc.intro}</p>
    {#each doc.sections as s (s.heading)}
      <section>
        <h2>{s.heading}</h2>
        {#each s.paragraphs as p (p)}<p>{p}</p>{/each}
      </section>
    {/each}
    <p class="other">
      {#if page === "privacy"}See also the <a href="/terms/">terms of use</a>.{:else}See also the <a href="/privacy/">privacy policy</a>.{/if}
    </p>
  </article>
</main>
<Footer />

<style>
  article {
    padding-top: 140px;
  }
  .narrow {
    width: min(760px, 100% - 48px);
  }
  h1 {
    margin-top: 12px;
  }
  section {
    padding: 36px 0 0;
  }
  h2 {
    margin: 0 0 12px;
    font-size: 22px;
  }
  p {
    margin: 0 0 14px;
    font-size: 16px;
    line-height: 1.7;
    color: var(--muted);
  }
  .lead {
    margin-top: 20px;
    color: inherit;
  }
  .other {
    margin-top: 40px;
  }
  .other a {
    color: #c9c1ff;
  }
  @media (max-width: 600px) {
    article {
      padding-top: 110px;
    }
    .narrow {
      width: calc(100% - 32px);
    }
  }
</style>
