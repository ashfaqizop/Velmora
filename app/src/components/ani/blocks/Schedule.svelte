<script lang="ts">
  import Art from "./Art.svelte";
  import { openTitle } from "../../../lib/ani/actions";
  import type { RefBook } from "../../../lib/ani/refs";

  // render_schedule: episodes airing soon, grouped by day in local time.
  type Item = { ref: string; episode?: number; airs_at: string };
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const days = $derived.by(() => {
    const groups = new Map<string, { label: string; items: (Item & { title: NonNullable<ReturnType<RefBook["title"]>>; at: Date })[] }>();
    const today = new Date().toDateString();
    const tomorrow = new Date(Date.now() + 86_400_000).toDateString();
    const items = ((args.items as Item[]) ?? [])
      .map((i) => ({ ...i, at: new Date(i.airs_at), title: refs.title(i.ref)! }))
      .filter((i) => i.title)
      .sort((a, b) => a.at.getTime() - b.at.getTime());
    for (const i of items) {
      const k = i.at.toDateString();
      const label = k === today ? "Today" : k === tomorrow ? "Tomorrow" : i.at.toLocaleDateString(undefined, { weekday: "long", day: "numeric", month: "short" });
      if (!groups.has(k)) groups.set(k, { label, items: [] });
      groups.get(k)!.items.push(i);
    }
    return [...groups.values()];
  });

  const time = (d: Date) => d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
</script>

<m3e-card variant="filled" class="sched">
  <div slot="content" class="inner">
    <m3e-heading variant="title" size="medium" level={3}>{(args.heading as string) ?? "Airing soon"}</m3e-heading>
    {#each days as d (d.label)}
      <section>
        <span class="day">{d.label}</span>
        <ul>
          {#each d.items as i (i.ref + i.airs_at)}
            <li>
              <span class="time">{time(i.at)}</span>
              <div class="art"><Art title={i.title} onclick={() => openTitle(i.title)} /></div>
              <button class="name" onclick={() => openTitle(i.title)}>{i.title.name}</button>
              {#if i.episode}<span class="ep">Episode {i.episode}</span>{/if}
            </li>
          {/each}
        </ul>
      </section>
    {/each}
  </div>
</m3e-card>

<style>
  .sched {
    --m3e-card-shape: 24px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .day {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  li {
    display: grid;
    grid-template-columns: 58px 36px minmax(0, 1fr) auto;
    align-items: center;
    gap: 12px;
    padding: 6px 8px;
    border-radius: 14px;
  }
  li:hover {
    background: var(--md-sys-color-surface-container-high);
  }
  .time {
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    color: var(--md-sys-color-on-surface-variant);
  }
  .art {
    width: 36px;
  }
  .name {
    all: unset;
    cursor: pointer;
    font-size: 14px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .ep {
    font-size: 12px;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
</style>
