<script lang="ts">
  // A profile's Theatre side: screenings they're going to, and a stamp for
  // every one they attended. Others only see this while that person shares
  // what they watch (the database decides).
  import { fly, scale } from "svelte/transition";
  import { backOut } from "svelte/easing";
  import { supabase } from "../lib/session";
  import { href, titleHref } from "./nav.svelte";
  import { clock, dayLabel } from "./theatre.svelte";

  let { userId, mine, name }: { userId: string; mine: boolean; name: string } = $props();

  interface Row {
    status: "booked" | "attended";
    s: { id: string; k: string; name: string; poster: string | null; starts_at: string; ends_at: string } | null;
  }

  let rows = $state<Row[] | null>(null);

  $effect(() => {
    const id = userId;
    rows = null;
    supabase
      .from("theatre_tickets")
      .select("status, s:theatre_screenings(id,k,name,poster,starts_at,ends_at)")
      .eq("user_id", id)
      .in("status", ["booked", "attended"])
      .limit(300)
      .then(({ data }) => (rows = ((data ?? []) as unknown as Row[]).filter((r) => r.s)));
  });

  const now = Date.now();
  const going = $derived((rows ?? []).filter((r) => r.status === "booked" && Date.parse(r.s!.ends_at) > now).sort((a, b) => a.s!.starts_at.localeCompare(b.s!.starts_at)));
  const stamps = $derived((rows ?? []).filter((r) => r.status === "attended").sort((a, b) => b.s!.starts_at.localeCompare(a.s!.starts_at)));
  const HUES = ["#6c9bff", "#ff8a65", "#5ad17a", "#c792ea", "#ffb547", "#3fd3c6"];
</script>

{#if rows === null}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else}
  {#if going.length}
    <section class="going">
      <h3><m3e-icon name="confirmation_number" filled></m3e-icon>Going to</h3>
      <div class="chips">
        {#each going as r (r.s!.id)}
          <a class="chip" href={href("/theatre")}>
            <strong>{r.s!.name}</strong><span>{dayLabel(r.s!.starts_at, now)}, {clock(r.s!.starts_at)}</span>
          </a>
        {/each}
      </div>
    </section>
  {/if}

  <section>
    <h3><m3e-icon name="local_activity" filled></m3e-icon>{stamps.length} {stamps.length === 1 ? "screening" : "screenings"} attended</h3>
    {#if !stamps.length}
      <p class="empty">{mine ? "Watch a screening in Theatre and its stamp lands here." : `${name} hasn't been to a Theatre screening yet, or keeps it private.`}</p>
    {:else}
      <div class="stamps">
        {#each stamps as r, i (r.s!.id)}
          <a class="stamp" href={titleHref(r.s!.k)} title="{r.s!.name}, {new Date(r.s!.starts_at).toLocaleDateString()}"
            style:--hue={HUES[i % HUES.length]} style:--tilt="{((i * 37) % 11) - 5}deg"
            in:scale={{ start: 0.6, duration: 420, delay: Math.min(i, 12) * 40, easing: backOut }}>
            <span class="disc">{#if r.s!.poster}<img src={r.s!.poster} alt="" loading="lazy" />{/if}</span>
            <span class="date">{new Date(r.s!.starts_at).toLocaleDateString([], { day: "numeric", month: "short", year: "2-digit" })}</span>
            <span class="nm">{r.s!.name}</span>
          </a>
        {/each}
      </div>
    {/if}
  </section>
{/if}

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 6px 4px 14px;
  }
  h3 {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: 16px;
    --m3e-icon-size: 20px;
  }
  h3 m3e-icon {
    color: var(--md-sys-color-primary);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .chip {
    display: flex;
    flex-direction: column;
    padding: 10px 14px;
    border-radius: 16px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    text-decoration: none;
    font-size: 13px;
  }
  .chip span {
    opacity: 0.8;
  }
  .stamps {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(116px, 1fr));
    gap: 18px 12px;
  }
  .stamp {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    color: inherit;
    text-decoration: none;
    text-align: center;
  }
  .disc {
    position: relative;
    width: 96px;
    height: 96px;
    border-radius: 50%;
    overflow: hidden;
    rotate: var(--tilt);
    box-shadow:
      0 0 0 3px var(--md-sys-color-surface-container-low),
      0 0 0 6px var(--hue),
      0 14px 28px -14px rgb(0 0 0 / 80%);
    background: var(--md-sys-color-surface-container-highest);
    transition: rotate 300ms cubic-bezier(0.2, 0.8, 0.2, 1.4), transform 300ms ease;
  }
  .stamp:hover .disc {
    rotate: 0deg;
    transform: scale(1.06);
  }
  .disc img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .date {
    margin-top: 4px;
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.04em;
    background: var(--hue);
    color: #0a0d16;
  }
  .nm {
    font-size: 12px;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .empty {
    margin: 0;
    padding: 12px 0;
    color: var(--md-sys-color-on-surface-variant);
  }
  .center {
    display: grid;
    place-items: center;
    padding: 32px 0;
  }
</style>
