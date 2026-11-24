<script lang="ts">
  // Leaderboards: most hours watched and most reviews this month, and the most followed.
  import { fly } from "svelte/transition";
  import Avatar from "../auth/Avatar.svelte";
  import Badges from "./Badges.svelte";
  import FollowButton from "./FollowButton.svelte";
  import { leaderboard, type Board } from "./activity.svelte";
  import type { Person } from "./data.svelte";
  import { count } from "./format";
  import { profileHref } from "./nav.svelte";

  const BOARDS: { id: Board; label: string; icon: string; unit: (n: number) => string; note: string }[] = [
    { id: "hours", label: "Hours watched", icon: "schedule", unit: (n) => `${count(n)} h`, note: "Last 30 days, from people who share what they watch." },
    { id: "reviews", label: "Reviews", icon: "edit_note", unit: (n) => `${count(n)} ${n === 1 ? "review" : "reviews"}`, note: "Reviews written in the last 30 days." },
    { id: "followers", label: "Followers", icon: "group", unit: (n) => `${count(n)} ${n === 1 ? "follower" : "followers"}`, note: "All time." },
  ];

  let board = $state<Board>("hours");
  let list = $state<(Person & { value: number })[] | null>(null);
  let problem = $state("");
  const info = $derived(BOARDS.find((b) => b.id === board)!);

  $effect(() => {
    const b = board;
    document.title = "Leaderboards · Velmora Social";
    list = null;
    let alive = true;
    leaderboard(b)
      .then((l) => alive && (list = l))
      .catch((e) => {
        if (!alive) return;
        problem = (e as Error).message;
        list = [];
      });
    return () => (alive = false);
  });

  const MEDAL = ["#f8d27a", "#d9dee8", "#e0a271"];
</script>

<section class="head">
  <span class="eyebrow">Velmora Social</span>
  <h1 class="display">Leaderboards</h1>
</section>

<m3e-segmented-button class="seg">
  {#each BOARDS as b (b.id)}
    <m3e-button-segment checked={board === b.id} onclick={() => (board = b.id)}>
      <m3e-icon slot="icon" name={b.icon}></m3e-icon>{b.label}
    </m3e-button-segment>
  {/each}
</m3e-segmented-button>
<p class="note">{info.note}</p>

{#if list === null}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else if !list.length}
  <p class="empty">{problem || "No one on this board yet."}</p>
{:else}
  <ol class="board">
    {#each list as p, i (p.id)}
      <li class:podium={i < 3} style:--medal={MEDAL[i] ?? "transparent"} in:fly={{ y: 10, duration: 260, delay: Math.min(i, 12) * 30 }}>
        <span class="rank">{#if i < 3}<m3e-icon name="emoji_events" filled></m3e-icon>{:else}{i + 1}{/if}</span>
        <a class="who" href={profileHref(p.username)}>
          <Avatar value={p.avatar} size={i < 3 ? 56 : 44} ring={i < 3 ? 2 : 0} />
          <span class="text">
            <span class="name">{p.display_name}<Badges id={p.id} verified={p.verified} size={16} /></span>
            <span class="handle">@{p.username}</span>
          </span>
        </a>
        <span class="value">{info.unit(p.value)}</span>
        <FollowButton id={p.id} />
      </li>
    {/each}
  </ol>
{/if}

<style>
  .head {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .eyebrow {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
  }
  .display {
    margin: 0;
    font-family: var(--display);
    font-weight: 400;
    font-size: clamp(44px, 7vw, 72px);
    line-height: 1;
  }
  .seg {
    max-width: 640px;
  }
  .note {
    margin: -14px 0 0;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .board {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .board li {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px 14px;
    border-radius: 22px;
    background: var(--md-sys-color-surface-container-low);
  }
  .board li.podium {
    padding: 14px 16px;
    background:
      radial-gradient(120% 140% at 0% 50%, color-mix(in srgb, var(--medal) 16%, transparent), transparent 60%),
      var(--md-sys-color-surface-container);
    border: 1px solid color-mix(in srgb, var(--medal) 30%, transparent);
  }
  .rank {
    width: 32px;
    flex: none;
    text-align: center;
    font-family: var(--display);
    font-size: 24px;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 28px;
  }
  .podium .rank {
    color: var(--medal);
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 12px;
    color: inherit;
    text-decoration: none;
  }
  .text {
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .name {
    display: flex;
    align-items: center;
    gap: 4px;
    font-weight: 700;
    overflow: hidden;
    white-space: nowrap;
  }
  .handle {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .value {
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .center {
    display: grid;
    place-items: center;
    padding: 48px 0;
  }
  .empty {
    margin: 0;
    padding: 40px 12px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  @media (max-width: 560px) {
    .board li :global(.follow) {
      display: none;
    }
  }
</style>
