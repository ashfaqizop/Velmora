<script lang="ts">
  import Poster from "../../Poster.svelte";
  import * as api from "../../../lib/api";
  import type { RefBook } from "../../../lib/ani/refs";
  import type { TitleSummary } from "../../../lib/types";

  // render_person: photo, role, birth details, Ani's short bio, what they're
  // known for and what's coming next, with links out to IMDb and TMDB.
  let { args, refs }: { args: Record<string, unknown>; refs: RefBook } = $props();

  const e = $derived(refs.get(args.ref as string));
  const p = $derived(e?.type === "person" ? e.person : null);

  const ROLE: Record<string, string> = {
    Acting: "Actor",
    Directing: "Director",
    Writing: "Writer",
    Production: "Producer",
    Sound: "Composer",
    Camera: "Cinematographer",
    Editing: "Editor",
  };
  const role = $derived(p?.role ? (ROLE[p.role] ?? p.role) : undefined);

  const key = (t: TitleSummary) => t.origin.provider + t.origin.id;
  const picked = $derived(((args.known_for as string[]) ?? []).flatMap((r) => refs.title(r) ?? []).filter((t) => t.poster));

  // Their filmography from TMDB, for "Known for" and "Coming next".
  let credits = $state<TitleSummary[]>([]);
  $effect(() => {
    const id = p?.tmdbId;
    if (!id) return;
    const field = p?.role && p.role !== "Acting" ? "crew" : "cast";
    api
      .tmdbList(`/person/${id}/combined_credits`, {}, field)
      .then((list) => (credits = list))
      .catch(() => {});
  });

  const year = new Date().getFullYear();
  const upcoming = $derived(dedupe(credits.filter((t) => (t.year ?? 0) > year || ((t.year ?? 0) === year && !t.rating))).slice(0, 8));
  const known = $derived.by(() => {
    const seen = new Set(upcoming.map(key));
    const best = credits.filter((t) => t.poster && !seen.has(key(t)) && (t.year ?? 0) <= year).sort((a, b) => (b.rating ?? 0) - (a.rating ?? 0));
    return dedupe([...picked, ...best]).slice(0, 10);
  });

  function dedupe(list: TitleSummary[]) {
    const seen = new Set<string>();
    return list.filter((t) => !seen.has(key(t)) && seen.add(key(t)));
  }

  const born = $derived.by(() => {
    if (!p?.born) return undefined;
    const d = new Date(p.born);
    if (Number.isNaN(d.getTime())) return p.born;
    const end = p.died ? new Date(p.died) : new Date();
    const age = end.getFullYear() - d.getFullYear() - (end < new Date(end.getFullYear(), d.getMonth(), d.getDate()) ? 1 : 0);
    const date = d.toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" });
    return p.died ? `${date} (died at ${age})` : `${date} · ${age}`;
  });
</script>

{#if p}
  <m3e-card variant="filled" class="person">
    <div slot="content" class="inner">
      <div class="top">
        <div class="photo">
          {#if p.photo}<img src={p.photo.replace("/w185/", "/h632/")} alt="" />{:else}<m3e-icon name="person"></m3e-icon>{/if}
        </div>
        <div class="text">
          <strong class="display">{p.name}</strong>
          {#if role}<span class="role">{role}</span>{/if}
          {#if born || p.birthplace}
            <div class="facts">
              {#if born}<span><m3e-icon name="cake"></m3e-icon>{born}</span>{/if}
              {#if p.birthplace}<span><m3e-icon name="location_on"></m3e-icon>{p.birthplace}</span>{/if}
              {#if credits.length}<span><m3e-icon name="movie"></m3e-icon>{credits.length} credits</span>{/if}
            </div>
          {/if}
          {#if args.bio}<p>{args.bio}</p>{/if}
          <div class="links">
            {#if p.imdbId}
              <m3e-button variant="tonal" size="small" onclick={() => api.openExternal(`https://www.imdb.com/name/${p.imdbId}/`)}>
                <m3e-icon slot="icon" name="open_in_new"></m3e-icon>IMDb
              </m3e-button>
            {/if}
            {#if p.tmdbId}
              <m3e-button variant="text" size="small" onclick={() => api.openExternal(`https://www.themoviedb.org/person/${p.tmdbId}`)}>
                <m3e-icon slot="icon" name="open_in_new"></m3e-icon>TMDB
              </m3e-button>
            {/if}
          </div>
        </div>
      </div>
      {#if upcoming.length}
        <span class="label">Coming next</span>
        <div class="row">
          {#each upcoming as t (key(t))}<Poster title={t} />{/each}
        </div>
      {/if}
      {#if known.length}
        <span class="label">Known for</span>
        <div class="row">
          {#each known as t (key(t))}<Poster title={t} />{/each}
        </div>
      {/if}
    </div>
  </m3e-card>
{/if}

<style>
  .person {
    --m3e-card-shape: 24px;
  }
  .inner {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 20px;
  }
  .top {
    display: grid;
    grid-template-columns: 136px minmax(0, 1fr);
    gap: 22px;
    align-items: start;
  }
  .photo {
    aspect-ratio: 2 / 3;
    border-radius: 18px;
    overflow: hidden;
    display: grid;
    place-items: center;
    background: var(--md-sys-color-secondary-container);
    --m3e-icon-size: 40px;
  }
  .photo img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .text {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  .text strong {
    font-size: 34px;
    font-weight: 400;
    line-height: 1.05;
  }
  .role {
    font-size: 13px;
    color: var(--md-sys-color-primary);
    font-weight: 700;
  }
  .facts {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .facts span {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    --m3e-icon-size: 16px;
  }
  p {
    margin: 2px 0 0;
    font-size: 14.5px;
    line-height: 1.55;
  }
  .links {
    display: flex;
    gap: 6px;
    margin-top: 2px;
  }
  .label {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
  }
  .row {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 128px;
    gap: 12px;
    overflow-x: auto;
    scrollbar-width: none;
    mask-image: linear-gradient(90deg, #000 88%, transparent);
  }
</style>
