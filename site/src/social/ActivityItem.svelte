<script lang="ts">
  // One line of activity: who did what with which title, and when.
  import Avatar from "../auth/Avatar.svelte";
  import Badges from "./Badges.svelte";
  import Stars from "./Stars.svelte";
  import { ago } from "./format";
  import { titles } from "./meta.svelte";
  import { profileHref, titleHref } from "./nav.svelte";
  import type { Activity } from "./activity.svelte";

  let { a }: { a: Activity } = $props();

  let reveal = $state(false);
  const meta = $derived(titles.get(a.k));
  const ep = $derived(a.season != null && a.episode != null ? `S${a.season} E${a.episode}` : "");
  const verb = $derived(
    a.kind === "watching"
      ? "started watching"
      : a.kind === "finished"
        ? a.k.startsWith("tv/") && a.plays > 1
          ? `watched ${a.plays} episodes of`
          : "finished"
        : a.kind === "listed"
          ? "added to their list"
          : "rated",
  );
  const icon = $derived(a.kind === "watching" ? "play_circle" : a.kind === "finished" ? "check_circle" : a.kind === "listed" ? "bookmark" : "star");
</script>

<article class="item">
  {#if a.user}
    <a class="pic" href={profileHref(a.user.username)} aria-label={a.user.display_name}><Avatar value={a.user.avatar} size={44} /></a>
  {:else}
    <span class="kind-icon" data-k={a.kind}><m3e-icon name={icon} filled></m3e-icon></span>
  {/if}
  <div class="body">
    <p class="line">
      {#if a.user}<a class="who" href={profileHref(a.user.username)}>{a.user.display_name}</a><Badges id={a.user.id} verified={a.user.verified} size={15} />{/if}
      <span class="verb">{a.user ? verb : verb.charAt(0).toUpperCase() + verb.slice(1)}</span>
      <a class="ttl" href={titleHref(a.k)}>{meta?.name ?? "a title"}</a>{#if ep && a.kind !== "listed"}<span class="ep">{a.kind === "finished" && a.plays > 1 ? `, up to ${ep}` : ` · ${ep}`}</span>{/if}
    </p>
    {#if a.kind === "rated" && a.score}
      <div class="rated"><Stars value={a.score} size={16} /><span>{(a.score / 2).toFixed(1)}</span></div>
      {#if a.review}
        {#if a.spoiler && !reveal}
          <button type="button" class="spoiler" onclick={() => (reveal = true)}>Contains spoilers. Show the review</button>
        {:else}
          <p class="review">{a.review}</p>
        {/if}
      {/if}
    {/if}
    <span class="when">{ago(a.created_at)}{#if a.kind === "finished" && a.minutes >= 60} · {Math.round(a.minutes / 6) / 10} h{:else if a.kind === "finished" && a.minutes > 0} · {a.minutes} min{/if}</span>
  </div>
  <a class="thumb" href={titleHref(a.k)} aria-hidden="true" tabindex="-1">
    {#if meta?.poster}<img src={meta.poster} alt="" loading="lazy" />{/if}
  </a>
</article>

<style>
  .item {
    display: flex;
    gap: 14px;
    padding: 14px 12px;
    border-radius: 22px;
    transition: background-color 160ms ease;
  }
  .item:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, transparent);
  }
  .pic {
    flex: none;
    display: inline-grid;
    align-self: flex-start;
  }
  .kind-icon {
    flex: none;
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 14px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    --m3e-icon-size: 22px;
  }
  .kind-icon[data-k="rated"] {
    background: color-mix(in srgb, #f8d27a 22%, transparent);
    color: #f8d27a;
  }
  .kind-icon[data-k="finished"] {
    background: var(--md-sys-color-tertiary-container);
    color: var(--md-sys-color-on-tertiary-container);
  }
  .body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .line {
    margin: 0;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }
  .line > :global(*) {
    vertical-align: middle;
  }
  .who,
  .ttl {
    color: inherit;
    font: inherit;
    font-weight: 700;
    text-decoration: none;
  }
  .who:hover,
  .ttl:hover {
    text-decoration: underline;
  }
  .verb,
  .ep {
    color: var(--md-sys-color-on-surface-variant);
  }
  .verb {
    margin-right: 4px;
  }
  .who ~ .verb {
    margin-left: 4px;
  }
  .rated {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 700;
    color: #f8d27a;
  }
  .review {
    margin: 2px 0 0;
    padding: 10px 14px;
    border-radius: 4px 18px 18px 18px;
    background: var(--md-sys-color-surface-container-high);
    line-height: 1.55;
    white-space: pre-line;
    overflow-wrap: anywhere;
  }
  .spoiler {
    align-self: flex-start;
    padding: 8px 14px;
    border: 1px dashed var(--md-sys-color-outline);
    border-radius: 14px;
    background: none;
    color: var(--md-sys-color-on-surface-variant);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .when {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .thumb {
    flex: none;
    width: 48px;
    height: 72px;
    border-radius: 10px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
