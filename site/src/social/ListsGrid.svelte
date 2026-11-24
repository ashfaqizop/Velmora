<script lang="ts">
  // Someone's lists as cards with a fan of posters.
  import { fly } from "svelte/transition";
  import { userLists, type TitleList } from "./activity.svelte";
  import { href, listHref } from "./nav.svelte";
  import { ago } from "./format";

  let { userId, mine, name }: { userId: string; mine: boolean; name: string } = $props();

  let lists = $state<TitleList[] | null>(null);

  $effect(() => {
    const id = userId;
    lists = null;
    userLists(id)
      .then((l) => (lists = l))
      .catch(() => (lists = []));
  });
</script>

{#if mine}
  <div class="new"><m3e-button variant="tonal" href={href("/list/new")}><m3e-icon slot="icon" name="add"></m3e-icon>New list</m3e-button></div>
{/if}
{#if lists === null}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else if !lists.length}
  <p class="empty">{mine ? "Make lists of titles to share: favourites of a year, a director, a mood." : `${name} hasn't made any lists yet.`}</p>
{:else}
  <div class="grid">
    {#each lists as l, i (l.id)}
      <a class="card" href={listHref(l.id)} in:fly={{ y: 10, duration: 260, delay: Math.min(i, 10) * 40 }}>
        <span class="fan">
          {#each l.items.slice(0, 3) as it, j (it.k)}
            <span class="p" style:--j={j}>{#if it.poster}<img src={it.poster} alt="" loading="lazy" />{/if}</span>
          {:else}
            <span class="p none"><m3e-icon name="list"></m3e-icon></span>
          {/each}
        </span>
        <strong>{l.name}</strong>
        <span class="meta">{l.items.length} {l.items.length === 1 ? "title" : "titles"} · {ago(l.updated_at)}{#if !l.public} · <m3e-icon name="lock"></m3e-icon>{/if}</span>
      </a>
    {/each}
  </div>
{/if}

<style>
  .new {
    display: flex;
    justify-content: flex-end;
    padding: 0 4px 12px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 14px;
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 16px;
    border-radius: 24px;
    background: var(--md-sys-color-surface-container-high);
    color: inherit;
    text-decoration: none;
    transition: transform 260ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .card:hover {
    transform: translateY(-3px);
  }
  .fan {
    position: relative;
    height: 120px;
    margin-bottom: 6px;
  }
  .p {
    position: absolute;
    top: 0;
    left: calc(var(--j) * 44px);
    width: 80px;
    height: 120px;
    border-radius: 12px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
    box-shadow: 6px 0 18px -6px rgb(0 0 0 / 70%);
    transform: rotate(calc((var(--j) - 1) * 4deg));
    z-index: calc(3 - var(--j));
    transition: transform 300ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .card:hover .p {
    transform: rotate(calc((var(--j) - 1) * 7deg)) translateX(calc(var(--j) * 6px));
  }
  .p img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .p.none {
    display: grid;
    place-items: center;
    --j: 0;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 14px;
  }
  .center {
    display: grid;
    place-items: center;
    padding: 32px 0;
  }
  .empty {
    margin: 0;
    padding: 28px 12px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
