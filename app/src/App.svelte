<script lang="ts">
  import Splash from "./components/Splash.svelte";
  import Shell from "./components/Shell.svelte";
  import { feed } from "./lib/state.svelte";

  let splash = $state(true);

  // The shell renders under the splash so it is laid out when revealed;
  // the feed waits for the config refresh before calling the engine.
  feed.load();
</script>

<Shell revealed={!splash} />

{#if splash}
  <Splash
    ondone={() => {
      splash = false;
    }}
  />
{/if}

