<script lang="ts">
  // A paged list of activity (the feed, or one person's).
  import { fly } from "svelte/transition";
  import ActivityItem from "./ActivityItem.svelte";
  import type { Activity } from "./activity.svelte";

  let { load, empty }: { load: (before: string | null) => Promise<{ items: Activity[]; more: boolean }>; empty: import("svelte").Snippet } = $props();

  let items = $state<Activity[]>([]);
  let more = $state(false);
  let phase = $state<"loading" | "ready" | "error">("loading");
  let paging = $state(false);
  let problem = $state("");

  $effect(() => {
    const fn = load;
    let alive = true;
    phase = "loading";
    fn(null)
      .then((r) => {
        if (!alive) return;
        items = r.items;
        more = r.more;
        phase = "ready";
      })
      .catch((e) => {
        if (!alive) return;
        problem = (e as Error).message;
        phase = "error";
      });
    return () => (alive = false);
  });

  async function next() {
    paging = true;
    try {
      const r = await load(items[items.length - 1]?.created_at ?? null);
      const seen = new Set(items.map((i) => i.id));
      items = [...items, ...r.items.filter((i) => !seen.has(i.id))];
      more = r.more;
    } catch (e) {
      problem = (e as Error).message;
    } finally {
      paging = false;
    }
  }
</script>

{#if phase === "loading"}
  <div class="sk-list" aria-busy="true">
    {#each Array(4) as _, i (i)}
      <div class="sk"><m3e-skeleton shape="circle" class="sk-pic"></m3e-skeleton><div class="sk-lines"><m3e-skeleton class="l1"></m3e-skeleton><m3e-skeleton class="l2"></m3e-skeleton></div></div>
    {/each}
  </div>
{:else if phase === "error"}
  <p class="msg" role="alert">{problem}</p>
{:else if !items.length}
  {@render empty()}
{:else}
  <div class="list">
    {#each items as a, i (a.id)}
      <div in:fly={{ y: 10, duration: 260, delay: Math.min(i, 10) * 30 }}><ActivityItem {a} /></div>
    {/each}
  </div>
  {#if more}
    <div class="more"><m3e-button variant="tonal" disabled={paging} onclick={next}>{paging ? "Loading…" : "Show more"}</m3e-button></div>
  {/if}
{/if}

<style>
  .list {
    display: flex;
    flex-direction: column;
  }
  .msg {
    margin: 0;
    padding: 24px 12px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  .more {
    display: flex;
    justify-content: center;
    padding-top: 10px;
  }
  .sk-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .sk {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px;
  }
  .sk-pic {
    width: 44px;
    height: 44px;
    flex: none;
  }
  .sk-lines {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .l1 {
    width: 60%;
    height: 14px;
  }
  .l2 {
    width: 20%;
    height: 12px;
  }
</style>
