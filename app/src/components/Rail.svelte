<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    icon,
    onseeall,
    loading = false,
    children,
  }: { title: string; icon: string; onseeall?: () => void; loading?: boolean; children?: Snippet } = $props();
</script>

<section class="rail">
  <header>
    <m3e-icon name={icon} class="icon"></m3e-icon>
    <m3e-heading variant="title" size="large" level={2}>{title}</m3e-heading>
    {#if onseeall}
      <m3e-button variant="text" size="small" class="see-all" onclick={onseeall}>
        See all
        <m3e-icon slot="trailing-icon" name="chevron_right"></m3e-icon>
      </m3e-button>
    {/if}
  </header>

  {#if loading}
    <div class="row">
      {#each Array(7) as _, i (i)}
        <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
      {/each}
    </div>
  {:else}
    <m3e-slide-group threshold="48">
      <div class="row">{@render children?.()}</div>
    </m3e-slide-group>
  {/if}
</section>

<style>
  .rail {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }
  header {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .icon {
    color: var(--md-sys-color-primary);
  }
  .see-all {
    margin-left: auto;
  }
  m3e-slide-group {
    min-width: 0;
  }
  .row {
    display: flex;
    gap: 16px;
    padding-block: 2px 4px;
  }
  .ghost {
    width: 160px;
    height: 286px;
    flex: none;
  }
</style>
