<script lang="ts">
  // A paged list of people (followers, following, visitors).
  import { fly } from "svelte/transition";
  import PersonRow from "./PersonRow.svelte";
  import type { Person } from "./data.svelte";

  type Loaded = { people: (Person & { at: string })[]; more: boolean };

  let {
    load,
    empty,
    note,
  }: {
    load: (from: number) => Promise<Loaded>;
    empty: string;
    note?: (p: Person & { at: string }) => string;
  } = $props();

  let list = $state<(Person & { at: string })[]>([]);
  let more = $state(false);
  let phase = $state<"loading" | "ready" | "error">("loading");
  let paging = $state(false);
  let problem = $state("");

  $effect(() => {
    const fn = load;
    let alive = true;
    phase = "loading";
    list = [];
    fn(0)
      .then((r) => {
        if (!alive) return;
        list = r.people;
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
      const r = await load(list.length);
      const seen = new Set(list.map((p) => p.id));
      list = [...list, ...r.people.filter((p) => !seen.has(p.id))];
      more = r.more;
    } catch (e) {
      problem = (e as Error).message;
    } finally {
      paging = false;
    }
  }
</script>

{#if phase === "loading"}
  <div class="skeletons" aria-busy="true">
    {#each Array(4) as _, i (i)}
      <div class="sk"><m3e-skeleton shape="circle" class="sk-pic"></m3e-skeleton><div class="sk-lines"><m3e-skeleton class="l1"></m3e-skeleton><m3e-skeleton class="l2"></m3e-skeleton></div></div>
    {/each}
  </div>
{:else if phase === "error"}
  <p class="empty" role="alert">{problem}</p>
{:else if !list.length}
  <p class="empty">{empty}</p>
{:else}
  <div class="list">
    {#each list as p, i (p.id)}
      <div in:fly={{ y: 10, duration: 260, delay: Math.min(i, 12) * 30 }}>
        <PersonRow person={p} note={note?.(p) ?? ""} />
      </div>
    {/each}
  </div>
  {#if more}
    <div class="more">
      <m3e-button variant="tonal" disabled={paging} onclick={next}>{paging ? "Loading…" : "Show more"}</m3e-button>
    </div>
  {/if}
{/if}

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .empty {
    margin: 0;
    padding: 28px 12px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  .more {
    display: flex;
    justify-content: center;
    padding-top: 12px;
  }
  .skeletons {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .sk {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px 12px;
  }
  .sk-pic {
    width: 48px;
    height: 48px;
    flex: none;
  }
  .sk-lines {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .l1 {
    width: 40%;
    height: 14px;
  }
  .l2 {
    width: 25%;
    height: 12px;
  }
</style>
