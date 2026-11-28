<script lang="ts">
  // The Windows mark (four panes), for where Windows sits next to Android.
  // Material Symbols has the Android robot but no Windows logo.
  let { size = 20 }: { size?: number } = $props();
</script>

<svg class="win" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true" fill="currentColor">
  <path d="M3 5.1 10.4 4v7.3H3zM11.4 3.9 21 2.5v8.8h-9.6zM3 12.3h7.4v7.3L3 18.5zM11.4 12.3H21v8.8l-9.6-1.4z" />
</svg>

<style>
  .win {
    flex: none;
    display: inline-block;
    vertical-align: middle;
  }
</style>
