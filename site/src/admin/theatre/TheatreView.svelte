<script lang="ts">
  // Theatre, run from here: the lineup, what's live, tickets, pins and blocks,
  // and the switches.
  import Controls from "./Controls.svelte";
  import Lineup from "./Lineup.svelte";
  import Live from "./Live.svelte";
  import Overrides from "./Overrides.svelte";
  import Tickets from "./Tickets.svelte";

  const TABS = [
    { id: "lineup", label: "Lineup" },
    { id: "live", label: "Live now" },
    { id: "tickets", label: "Tickets" },
    { id: "overrides", label: "Pins & blocks" },
    { id: "controls", label: "Controls" },
  ] as const;

  let tab = $state<(typeof TABS)[number]["id"]>("lineup");
</script>

<div class="adm-page">
  <div class="adm-head">
    <div class="grow">
      <h1>Theatre</h1>
      <p>The daily lineup, screenings you add by hand, who is in the room, and the switches that apply to everything.</p>
    </div>
  </div>

  <m3e-filter-chip-set aria-label="Section">
    {#each TABS as t (t.id)}
      <m3e-filter-chip selected={tab === t.id} onclick={() => (tab = t.id)}>{t.label}</m3e-filter-chip>
    {/each}
  </m3e-filter-chip-set>

  {#key tab}
    {#if tab === "lineup"}
      <Lineup />
    {:else if tab === "live"}
      <Live />
    {:else if tab === "tickets"}
      <Tickets />
    {:else if tab === "overrides"}
      <Overrides />
    {:else}
      <Controls />
    {/if}
  {/key}
</div>
