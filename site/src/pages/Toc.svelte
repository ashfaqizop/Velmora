<script lang="ts">
  // "On this page": m3e's table of contents reads the headings inside the
  // element with id `for` and follows your scroll. Hidden on small screens.
  let { target = "doc" }: { target?: string } = $props();
</script>

<aside class="toc" aria-label="On this page">
  <m3e-toc for={target} max-depth="2">
    <span slot="overline">On this page</span>
  </m3e-toc>
</aside>

<style>
  .toc {
    position: sticky;
    top: 104px;
    align-self: start;
    --m3e-toc-item-color: var(--muted);
  }
  @media (max-width: 900px) {
    .toc {
      display: none;
    }
  }
</style>
