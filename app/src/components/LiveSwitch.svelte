<script lang="ts">
  // Live TV and Sports share one "Live" item in the sidebar; this switch at
  // the top of both pages moves between them. The sidebar remembers which
  // one you were on last.
  import { nav } from "../lib/state.svelte";

  let { current }: { current: "live" | "sports" } = $props();

  function pick(to: "live" | "sports") {
    if (to === current) return;
    try {
      localStorage.setItem("velmora.live.last", to);
    } catch {
      // Not remembered; the sidebar opens Live TV.
    }
    nav.go({ name: to });
  }
</script>

<m3e-segmented-button class="live-switch" aria-label="Live">
  <m3e-button-segment checked={current === "live"} onclick={() => pick("live")}>
    <m3e-icon slot="icon" name="live_tv"></m3e-icon>TV channels
  </m3e-button-segment>
  <m3e-button-segment checked={current === "sports"} onclick={() => pick("sports")}>
    <m3e-icon slot="icon" name="sports_soccer"></m3e-icon>Sports
  </m3e-button-segment>
</m3e-segmented-button>

<style>
  .live-switch {
    flex: none;
  }
</style>
