<script lang="ts">
  // The frame every content page shares: the aurora backdrop, the nav, a
  // hero with morphing Material shapes, a breadcrumb, and the footer.
  import type { Snippet } from "svelte";
  import Backdrop from "../components/Backdrop.svelte";
  import Footer from "../components/Footer.svelte";
  import Nav from "../components/Nav.svelte";
  import Shapes from "../components/Shapes.svelte";
  import type { ShapeSpot } from "../lib/morph.svelte";

  let {
    eyebrow,
    title,
    accent = "",
    lead = "",
    crumb,
    spots,
    hero,
    children,
  }: {
    eyebrow: string;
    title: string;
    accent?: string;
    lead?: string;
    /** The last step of the breadcrumb (the page's own name). */
    crumb: string;
    spots?: ShapeSpot[];
    hero?: Snippet;
    children: Snippet;
  } = $props();

  const DEFAULT: ShapeSpot[] = [
    { x: "92%", y: "170px", size: 130, color: "#3271fe", spin: 24, opacity: 0.6 },
    { x: "5%", y: "330px", size: 60, color: "#ffb74d", spin: -16, offset: 5, opacity: 0.7 },
    { x: "80%", y: "400px", size: 44, color: "#f8e8c8", spin: 14, offset: 2, opacity: 0.8 },
  ];
</script>

<Backdrop />
<Nav />
<main class="pg" id="top">
  <header class="hero">
    <Shapes spots={spots ?? DEFAULT} />
    <div class="pg-wide">
      <m3e-breadcrumb class="crumbs pg-rise" aria-label="Breadcrumb">
        <m3e-breadcrumb-item href="/">Home</m3e-breadcrumb-item>
        <m3e-breadcrumb-item current="page">{crumb}</m3e-breadcrumb-item>
      </m3e-breadcrumb>
      <span class="eyebrow pg-rise" style="--d: 0.05s">{eyebrow}</span>
      <h1 class="title pg-rise" style="--d: 0.12s">{title}{#if accent}{" "}<em>{accent}</em>{/if}</h1>
      {#if lead}<p class="lead pg-rise" style="--d: 0.2s">{lead}</p>{/if}
      {#if hero}<div class="pg-rise" style="--d: 0.3s">{@render hero()}</div>{/if}
    </div>
  </header>
  {@render children()}
</main>
<Footer />

<style>
  .hero {
    position: relative;
    padding: 132px 0 8px;
  }
  .crumbs {
    display: flex;
    margin: 0 0 22px -8px;
    --m3e-breadcrumb-item-container-height: 32px;
  }
  .eyebrow {
    display: inline-flex;
  }
  @media (max-width: 600px) {
    .hero {
      padding-top: 104px;
    }
  }
</style>
