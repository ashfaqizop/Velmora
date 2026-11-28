<script lang="ts">
  // The Windows 11 mark (four equal squares), for where Windows sits next to
  // Android. Material Symbols has the Android robot but no Windows logo.
  let { size = 20 }: { size?: number } = $props();
</script>

<svg class="win" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true" fill="currentColor">
  <rect x="2" y="2" width="9.5" height="9.5" rx="0.6" />
  <rect x="12.5" y="2" width="9.5" height="9.5" rx="0.6" />
  <rect x="2" y="12.5" width="9.5" height="9.5" rx="0.6" />
  <rect x="12.5" y="12.5" width="9.5" height="9.5" rx="0.6" />
</svg>

<style>
  .win {
    flex: none;
    display: inline-block;
    vertical-align: middle;
  }
</style>
