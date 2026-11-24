<script lang="ts">
  // A name or picture in Theatre. Hovering (or clicking) shows a small profile
  // card: picture, name, badges, bio, followers, a link to Velmora Social and
  // a mute switch for this screening.
  import type { Snippet } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import Avatar from "../Avatar.svelte";
  import { account } from "../../lib/account.svelte";
  import * as api from "../../lib/api";
  import { room } from "../../lib/room.svelte";

  let { uid, username, children }: { uid: string; username: string; children: Snippet } = $props();

  interface Card {
    display_name: string;
    username: string;
    avatar: string;
    bio: string;
    verified: boolean;
    followers_count: number;
  }

  // Shared across every name on screen: one lookup per person.
  const cache = (globalThis as unknown as { __whoCache?: Map<string, Promise<Card | null>> });
  cache.__whoCache ??= new Map();

  let open = $state(false);
  let card = $state<Card | null>(null);
  let anchor = $state<HTMLElement>();
  let pos = $state({ x: 0, y: 0, below: true });
  let timer: ReturnType<typeof setTimeout> | undefined;

  function load(): Promise<Card | null> {
    const map = cache.__whoCache!;
    if (!map.has(uid)) {
      map.set(
        uid,
        Promise.resolve(
          account.client
          .from("profiles")
          .select("display_name,username,avatar,bio,verified,followers_count")
          .eq("id", uid)
          .maybeSingle()
          .then(({ data }) => (data as Card) ?? null),
        ),
      );
    }
    return map.get(uid)!;
  }

  function show() {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      if (!anchor) return;
      const r = anchor.getBoundingClientRect();
      const below = r.bottom + 230 < innerHeight;
      pos = { x: Math.min(Math.max(12, r.left), innerWidth - 312), y: below ? r.bottom + 8 : innerHeight - r.top + 8, below };
      open = true;
      card = await load();
    }, 350);
  }

  function hide() {
    clearTimeout(timer);
    timer = setTimeout(() => (open = false), 220);
  }

  const mine = $derived(account.profile?.id === uid);
  const muted = $derived(room.muted.has(uid));
  const social = () => api.openExternal(`https://social.getvelmora.dpdns.org/@${card?.username ?? username}`).catch(() => {});
</script>

<span
  class="who"
  bind:this={anchor}
  role="button"
  tabindex="0"
  onpointerenter={show}
  onpointerleave={hide}
  onfocus={show}
  onblur={hide}
  onclick={() => (open ? (open = false) : show())}
  onkeydown={(e) => e.key === "Enter" && show()}
>
  {@render children()}
</span>

{#if open}
  <div
    class="card"
    class:above={!pos.below}
    style:left="{pos.x}px"
    style:top={pos.below ? `${pos.y}px` : undefined}
    style:bottom={pos.below ? undefined : `${pos.y}px`}
    role="dialog"
    aria-label="Profile"
    tabindex="-1"
    onpointerenter={() => clearTimeout(timer)}
    onpointerleave={hide}
    in:scale={{ start: 0.94, duration: 200, easing: cubicOut }}
    out:fade={{ duration: 140 }}
  >
    {#if card}
      <div class="head">
        <Avatar value={card.avatar} size={52} ring={2} />
        <div class="names">
          <strong>{card.display_name}{#if card.verified}<m3e-icon name="verified" filled class="tick"></m3e-icon>{/if}</strong>
          <span>@{card.username} · {card.followers_count} {card.followers_count === 1 ? "follower" : "followers"}</span>
        </div>
      </div>
      {#if card.bio}<p class="bio">{card.bio}</p>{/if}
      <div class="row">
        <m3e-button variant="tonal" size="small" onclick={social}><m3e-icon slot="icon" name="open_in_new"></m3e-icon>View on Social</m3e-button>
        {#if !mine}
          <m3e-button variant="text" size="small" onclick={() => room.mute(uid, !muted)}>
            <m3e-icon slot="icon" name={muted ? "volume_up" : "volume_off"}></m3e-icon>{muted ? "Unmute" : "Mute"}
          </m3e-button>
        {/if}
      </div>
    {:else}
      <div class="loading"><m3e-loading-indicator></m3e-loading-indicator></div>
    {/if}
  </div>
{/if}

<style>
  .who {
    display: inline-flex;
    align-items: center;
    cursor: pointer;
    border-radius: 999px;
  }
  .who:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
    outline-offset: 2px;
  }
  .card {
    position: fixed;
    z-index: 90;
    width: 300px;
    padding: 14px;
    border-radius: 20px;
    background: color-mix(in srgb, #121a30 94%, transparent);
    border: 1px solid rgb(255 255 255 / 10%);
    box-shadow: 0 24px 60px -20px rgb(0 0 0 / 85%);
    backdrop-filter: blur(16px);
    color: #fff;
    display: flex;
    flex-direction: column;
    gap: 10px;
    transform-origin: top left;
  }
  .card.above {
    transform-origin: bottom left;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .names {
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .names strong {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 16px;
  }
  .names span {
    font-size: 13px;
    opacity: 0.7;
  }
  .tick {
    --m3e-icon-size: 16px;
    color: var(--md-sys-color-primary);
  }
  .bio {
    margin: 0;
    font-size: 13px;
    line-height: 1.5;
    opacity: 0.85;
    white-space: pre-line;
    overflow-wrap: anywhere;
  }
  .row {
    display: flex;
    gap: 6px;
  }
  .loading {
    display: grid;
    place-items: center;
    padding: 18px 0;
  }
</style>
