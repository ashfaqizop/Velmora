<script lang="ts">
  // Download queue UI lands with the player; this is the empty state.
</script>

<div class="page">
  <m3e-heading variant="headline" size="large" level={1}>Downloads</m3e-heading>
  <div class="empty">
    <m3e-shape name="soft-burst" class="art"><m3e-icon name="download"></m3e-icon></m3e-shape>
    <m3e-heading variant="title" size="large">Nothing downloaded yet</m3e-heading>
    <p>Saved movies and episodes appear here, ready to watch offline.</p>
  </div>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 64px 0;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  .empty p {
    margin: 0;
  }
  .art {
    --m3e-shape-size: 120px;
    --m3e-shape-container-color: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
    display: grid;
    place-items: center;
    --m3e-icon-size: 48px;
    margin-bottom: 8px;
  }
</style>
