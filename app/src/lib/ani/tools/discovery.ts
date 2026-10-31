// Search and discovery tools: titles, details, people, franchises, schedules.
import * as api from "../../api";
import { feed, tonightPick } from "../../state.svelte";
import type { TitleSummary } from "../../types";
import type { ResearchTool } from "./types";
import { num, str, strs } from "./types";
import { ANILIST_MEDIA, anilistSummary, brief, briefs, clipText, detailOf, titleArg, tmdbPath } from "./shape";

const KIND_ENUM = ["movie", "tv", "anime"];
const q = (args: Record<string, unknown>) => str(args.query) ?? str(args.name) ?? "";

const ANILIST_GENRES = [
  "Action", "Adventure", "Comedy", "Drama", "Ecchi", "Fantasy", "Horror", "Mahou Shoujo", "Mecha", "Music",
  "Mystery", "Psychological", "Romance", "Sci-Fi", "Slice of Life", "Sports", "Supernatural", "Thriller",
];

// TMDB genre names → ids, fetched once per kind.
const genreCache = new Map<string, Promise<{ id: number; name: string }[]>>();
function tmdbGenres(kind: "movie" | "tv") {
  let p = genreCache.get(kind);
  if (!p) {
    p = api.tmdbRaw(`/genre/${kind}/list`).then((r) => ((r as { genres?: { id: number; name: string }[] }).genres ?? []));
    p.catch(() => genreCache.delete(kind));
    genreCache.set(kind, p);
  }
  return p;
}

async function genreIds(kind: "movie" | "tv", names: string[]) {
  if (!names.length) return { ids: [] as number[], unknown: [] as string[] };
  const all = await tmdbGenres(kind);
  const ids: number[] = [];
  const unknown: string[] = [];
  for (const n of names) {
    const g = all.find((x) => x.name.toLowerCase() === n.toLowerCase() || x.name.toLowerCase().includes(n.toLowerCase()));
    if (g) ids.push(g.id);
    else unknown.push(n);
  }
  return { ids, unknown };
}

async function keywordIds(text: string | undefined) {
  if (!text) return [];
  const words = text.split(",").map((w) => w.trim()).filter(Boolean).slice(0, 4);
  const found = await Promise.all(
    words.map((w) =>
      api
        .tmdbRaw("/search/keyword", { query: w })
        .then((r) => (r as { results?: { id: number }[] }).results?.[0]?.id)
        .catch(() => undefined),
    ),
  );
  return found.filter((x): x is number => !!x);
}

const universeSearch: ResearchTool = {
  decl: {
    name: "universe_search",
    description:
      "Search every source Velmora knows (TMDB, AniList and the streaming and download sites) at once, through UniverseAPI. Finds titles TMDB lacks. Returns titles with refs.",
    parameters: { type: "object", properties: { query: { type: "string", description: "A title or a few words." } }, required: ["query"] },
  },
  label: (a) => `Searching every source for "${q(a)}"`,
  async run(args, { refs }) {
    const found: TitleSummary[] = [];
    const answered: string[] = [];
    await api.searchStream(q(args), (b) => {
      if (b.items.length) answered.push(b.provider);
      found.push(...b.items);
    });
    return { titles: briefs(found, refs, 15), sources_with_results: answered.length };
  },
};

const tmdbSearch: ResearchTool = {
  decl: {
    name: "tmdb_search",
    description: "Search TMDB for movies and live-action series. Best for exact titles. Returns titles with refs.",
    parameters: {
      type: "object",
      properties: {
        query: { type: "string" },
        kind: { type: "string", enum: ["movie", "tv", "any"], description: "Defaults to any." },
        year: { type: "integer", description: "Release year, to tell remakes apart." },
      },
      required: ["query"],
    },
  },
  label: (a) => `Searching TMDB for "${q(a)}"`,
  async run(args, { refs }) {
    const kind = str(args.kind);
    const year = num(args.year);
    const query: Record<string, string> = { query: q(args), include_adult: "false" };
    if (kind === "movie" || kind === "tv") {
      if (year) query[kind === "movie" ? "primary_release_year" : "first_air_date_year"] = String(year);
      return { titles: briefs(await api.tmdbList(`/search/${kind}`, query, "results", kind), refs) };
    }
    if (year) query.year = String(year);
    return { titles: briefs(await api.tmdbList("/search/multi", query), refs) };
  },
};

const anilistSearch: ResearchTool = {
  decl: {
    name: "anilist_search",
    description: "Search AniList for anime, with score, format and year. Returns titles with refs.",
    parameters: {
      type: "object",
      properties: {
        query: { type: "string" },
        year: { type: "integer" },
        format: { type: "string", enum: ["TV", "MOVIE", "OVA", "ONA", "SPECIAL", "TV_SHORT"] },
      },
      required: ["query"],
    },
  },
  label: (a) => `Searching AniList for "${q(a)}"`,
  async run(args, { refs }) {
    const list = await api.anilistFind({ search: q(args), year: num(args.year), format: str(args.format) });
    return { titles: briefs(list, refs) };
  },
};

const titleDetail: ResearchTool = {
  decl: {
    name: "title_detail",
    description:
      "Full details for a title: plot, genres, runtime, status, cast and crew, studios, languages, keywords, seasons and recommendations.",
    parameters: { type: "object", properties: { ref: { type: "string", description: "A title ref such as t3." } }, required: ["ref"] },
  },
  label: (a) => `Reading about ${a.ref}`,
  async run(args, { refs }) {
    const t = titleArg(args, refs);
    const d = await detailOf(t);
    const x = d.extra;
    const ref = refs.addTitle({ ...t, ...d.summary });
    return {
      ref,
      name: d.summary.name,
      year: d.summary.year,
      kind: d.summary.kind,
      overview: clipText(d.summary.overview, 700),
      tagline: x.tagline,
      genres: d.genres,
      runtime_minutes: d.runtime_minutes,
      status: x.status,
      release_date: x.release_date,
      certification: x.certification,
      rating: d.summary.rating,
      rating_source: x.rating_source,
      votes: x.vote_count,
      original_language: x.original_language,
      languages: x.languages.slice(0, 5),
      countries: x.countries.slice(0, 4),
      studios: x.studios.slice(0, 5),
      format: x.format,
      source_material: x.source_material,
      seasons: d.seasons.length ? d.seasons.map((s) => ({ season: s.number, episodes: s.episodes.length })) : undefined,
      cast: x.cast.slice(0, 10).map((p) => ({ ref: refs.addPerson({ name: p.name, photo: p.photo, role: p.role }), name: p.name, role: p.role })),
      crew: x.crew.slice(0, 6).map((p) => ({ name: p.name, job: p.role })),
      keywords: x.keywords.slice(0, 12),
      has_trailer: !!d.trailer,
      recommendations: briefs(x.recommendations, refs, 10),
    };
  },
};

const titleRatings: ResearchTool = {
  decl: {
    name: "title_ratings",
    description: "Scores and age rating for a title: TMDB or AniList score, vote count, certification, IMDb id.",
    parameters: { type: "object", properties: { ref: { type: "string" } }, required: ["ref"] },
  },
  label: (a) => `Checking ratings for ${a.ref}`,
  async run(args, { refs }) {
    const t = titleArg(args, refs);
    const d = await detailOf(t);
    return {
      ref: str(args.ref),
      score: d.summary.rating,
      score_source: d.extra.rating_source,
      votes: d.extra.vote_count,
      certification: d.extra.certification,
      imdb: d.summary.ids.imdb,
      awards: d.extra.awards,
    };
  },
};

const similarTitles: ResearchTool = {
  decl: {
    name: "similar_titles",
    description: "Titles similar to one title: its recommendations, plus TMDB's similar list when it has one.",
    parameters: { type: "object", properties: { ref: { type: "string" } }, required: ["ref"] },
  },
  label: (a) => `Finding titles like ${a.ref}`,
  async run(args, { refs }) {
    const t = titleArg(args, refs);
    const [d, more] = await Promise.all([
      detailOf(t).catch(() => null),
      (async () => {
        const path = tmdbPath(t);
        if (!path) return [] as TitleSummary[];
        const kind = path.startsWith("movie") ? "movie" : "tv";
        const [rec, sim] = await Promise.all([
          api.tmdbList(`/${path}/recommendations`, {}, "results", kind).catch(() => []),
          api.tmdbList(`/${path}/similar`, {}, "results", kind).catch(() => []),
        ]);
        return [...rec, ...sim];
      })(),
    ]);
    return { titles: briefs([...(d?.extra.recommendations ?? []), ...more], refs, 16) };
  },
};

const discover: ResearchTool = {
  decl: {
    name: "discover",
    description:
      "Find titles by filters instead of by name: genres, original language, year range, minimum rating, keywords, sort. Use kind 'anime' for anime (AniList genres), 'movie' or 'tv' otherwise (TMDB genres). Check genre_list for valid names.",
    parameters: {
      type: "object",
      properties: {
        kind: { type: "string", enum: KIND_ENUM },
        genres: { type: "array", items: { type: "string" }, description: "Genre names, all of which must match." },
        language: { type: "string", description: "Original language as an ISO 639-1 code, e.g. hi, ko, ja, en." },
        year_from: { type: "integer" },
        year_to: { type: "integer" },
        min_rating: { type: "number", description: "0 to 10." },
        keywords: { type: "string", description: "Comma-separated themes, e.g. 'time travel, heist'. Movies and tv only." },
        sort: { type: "string", enum: ["popular", "rating", "newest"] },
      },
      required: ["kind"],
    },
  },
  label: (a) => `Browsing ${a.kind === "tv" ? "series" : a.kind ?? "titles"}${strs(a.genres).length ? ` in ${strs(a.genres).join(", ")}` : ""}`,
  async run(args, { refs }) {
    const kind = str(args.kind) ?? "movie";
    const sort = str(args.sort) ?? "popular";
    const genres = strs(args.genres);
    if (kind === "anime") {
      const list = await api.anilistFind({
        genres: genres.length ? genres : undefined,
        year: num(args.year_from) && num(args.year_from) === num(args.year_to) ? num(args.year_from) : undefined,
        sort: sort === "rating" ? "SCORE_DESC" : sort === "newest" ? "START_DATE_DESC" : "POPULARITY_DESC",
      });
      const from = num(args.year_from);
      const to = num(args.year_to);
      const min = num(args.min_rating);
      const kept = list.filter((t) => (!from || (t.year ?? 0) >= from) && (!to || (t.year ?? 9999) <= to) && (!min || (t.rating ?? 0) >= min));
      return { titles: briefs(kept, refs, 15) };
    }
    const k = kind === "tv" ? "tv" : "movie";
    const date = k === "movie" ? "primary_release_date" : "first_air_date";
    const [{ ids, unknown }, keywords] = await Promise.all([genreIds(k, genres), keywordIds(str(args.keywords))]);
    const query: Record<string, string> = {
      include_adult: "false",
      sort_by: sort === "rating" ? "vote_average.desc" : sort === "newest" ? `${date}.desc` : "popularity.desc",
    };
    if (ids.length) query.with_genres = ids.join(",");
    if (keywords.length) query.with_keywords = keywords.join("|");
    const lang = str(args.language);
    if (lang) query.with_original_language = lang;
    const from = num(args.year_from);
    const to = num(args.year_to);
    if (from) query[`${date}.gte`] = `${from}-01-01`;
    if (to) query[`${date}.lte`] = `${to}-12-31`;
    const min = num(args.min_rating);
    if (min) query["vote_average.gte"] = String(min);
    // Rating sorts need a vote floor, or obscure titles with one 10/10 vote win.
    if (sort === "rating" || min) query["vote_count.gte"] = "150";
    if (sort === "newest") query[`${date}.lte`] ??= new Date().toISOString().slice(0, 10);
    const list = await api.tmdbList(`/discover/${k}`, query, "results", k);
    return { titles: briefs(list, refs, 15), unknown_genres: unknown.length ? unknown : undefined };
  },
};

const genreList: ResearchTool = {
  decl: {
    name: "genre_list",
    description: "Valid genre names for discover, for movies, tv or anime.",
    parameters: { type: "object", properties: { kind: { type: "string", enum: KIND_ENUM } }, required: ["kind"] },
  },
  label: () => "Checking genre names",
  async run(args) {
    const kind = str(args.kind);
    if (kind === "anime") return { genres: ANILIST_GENRES };
    return { genres: (await tmdbGenres(kind === "tv" ? "tv" : "movie")).map((g) => g.name) };
  },
};

const trending: ResearchTool = {
  decl: {
    name: "trending",
    description: "What's trending right now.",
    parameters: {
      type: "object",
      properties: {
        kind: { type: "string", enum: ["movie", "tv", "anime", "all"] },
        window: { type: "string", enum: ["day", "week"], description: "Defaults to week." },
      },
      required: ["kind"],
    },
  },
  label: (a) => `Checking what's trending${a.kind && a.kind !== "all" ? ` in ${a.kind === "tv" ? "series" : a.kind}` : ""}`,
  async run(args, { refs }) {
    const kind = str(args.kind) ?? "all";
    if (kind === "anime") return { titles: briefs(await api.anilistFind({ sort: "TRENDING_DESC" }), refs, 15) };
    const window = str(args.window) === "day" ? "day" : "week";
    const k = kind === "movie" || kind === "tv" ? kind : "all";
    return { titles: briefs(await api.tmdbList(`/trending/${k}/${window}`, {}, "results", k === "tv" ? "tv" : "movie"), refs, 15) };
  },
};

const homeRails: ResearchTool = {
  decl: {
    name: "home_rails",
    description: "The rails on Velmora's Home page (trending, latest additions, airing today and so on) with their top titles.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Looking at the Home page",
  async run(_args, { refs }) {
    const rails = feed.rails.length ? feed.rails : await api.home();
    return { rails: rails.map((r) => ({ id: r.id, title: r.title, titles: briefs(r.items, refs, 8) })) };
  },
};

const tonight: ResearchTool = {
  decl: {
    name: "tonight_pick",
    description: "Today's pick from Velmora's Home page.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Checking tonight's pick",
  async run(_args, { refs }) {
    const pick = tonightPick();
    return pick ? { title: brief(pick, refs) } : { error: "Home hasn't loaded yet." };
  },
};

const listEpisodes: ResearchTool = {
  decl: {
    name: "list_episodes",
    description: "Seasons of a series or anime, and the episodes of one season with titles and air dates.",
    parameters: {
      type: "object",
      properties: { ref: { type: "string" }, season: { type: "integer", description: "Defaults to 1." } },
      required: ["ref"],
    },
  },
  label: (a) => `Listing episodes of ${a.ref}${a.season ? ` season ${a.season}` : ""}`,
  async run(args, { refs }) {
    const d = await detailOf(titleArg(args, refs));
    const n = num(args.season) ?? 1;
    const season = d.seasons.find((s) => s.number === n);
    return {
      seasons: d.seasons.map((s) => ({ season: s.number, name: s.name, episodes: s.episodes.length })),
      season: n,
      episodes: (season?.episodes ?? []).slice(0, 60).map((e) => ({
        episode: e.number,
        title: e.title,
        air_date: e.air_date,
        overview: clipText(e.overview, 140),
      })),
    };
  },
};

const trailer: ResearchTool = {
  decl: {
    name: "trailer",
    description: "Whether a title has a trailer Velmora can show.",
    parameters: { type: "object", properties: { ref: { type: "string" } }, required: ["ref"] },
  },
  label: (a) => `Finding the trailer for ${a.ref}`,
  async run(args, { refs }) {
    const d = await detailOf(titleArg(args, refs));
    return { ref: str(args.ref), has_trailer: !!d.trailer };
  },
};

const titleImages: ResearchTool = {
  decl: {
    name: "title_images",
    description: "How many stills and backdrops a title has, for a gallery.",
    parameters: { type: "object", properties: { ref: { type: "string" } }, required: ["ref"] },
  },
  label: (a) => `Collecting stills from ${a.ref}`,
  async run(args, { refs }) {
    const d = await detailOf(titleArg(args, refs));
    return { ref: str(args.ref), images: d.screenshots.length + (d.summary.backdrop ? 1 : 0) };
  },
};

const personSearch: ResearchTool = {
  decl: {
    name: "person_search",
    description: "Find an actor, director or other film person on TMDB. Returns people with refs and what they're known for.",
    parameters: { type: "object", properties: { name: { type: "string" } }, required: ["name"] },
  },
  label: (a) => `Looking up ${q(a)}`,
  async run(args, { refs }) {
    const r = (await api.tmdbRaw("/search/person", { query: q(args), include_adult: "false" })) as {
      results?: { id: number; name: string; profile_path?: string; known_for_department?: string; known_for?: { title?: string; name?: string }[] }[];
    };
    return {
      people: (r.results ?? []).slice(0, 5).map((p) => ({
        ref: refs.addPerson({
          name: p.name,
          tmdbId: p.id,
          role: p.known_for_department,
          photo: p.profile_path ? `https://image.tmdb.org/t/p/w185${p.profile_path}` : undefined,
        }),
        name: p.name,
        department: p.known_for_department,
        known_for: (p.known_for ?? []).map((k) => k.title ?? k.name).filter(Boolean),
      })),
    };
  },
};

const personCredits: ResearchTool = {
  decl: {
    name: "person_credits",
    description: "A person's short bio and filmography, newest first. Takes a person ref from person_search.",
    parameters: {
      type: "object",
      properties: { ref: { type: "string" }, role: { type: "string", enum: ["acting", "directing", "any"], description: "Defaults to acting." } },
      required: ["ref"],
    },
  },
  label: (a) => `Reading ${a.ref}'s filmography`,
  async run(args, { refs }) {
    const e = refs.get(str(args.ref) ?? "");
    if (e?.type !== "person") throw new Error("Unknown person ref. Use person_search first.");
    let id = e.person.tmdbId;
    if (!id) {
      const r = (await api.tmdbRaw("/search/person", { query: e.person.name })) as { results?: { id: number }[] };
      id = r.results?.[0]?.id;
      if (!id) throw new Error(`TMDB doesn't know ${e.person.name}.`);
      e.person.tmdbId = id;
    }
    const role = str(args.role) ?? "acting";
    const [bio, cast, crew] = await Promise.all([
      api.tmdbRaw(`/person/${id}`) as Promise<{ biography?: string; birthday?: string; place_of_birth?: string; deathday?: string }>,
      role === "directing" ? Promise.resolve([]) : api.tmdbList(`/person/${id}/combined_credits`, {}, "cast"),
      role === "acting" ? Promise.resolve([]) : api.tmdbList(`/person/${id}/combined_credits`, {}, "crew"),
    ]);
    const all = [...cast, ...crew].sort((a, b) => (b.year ?? 0) - (a.year ?? 0));
    return {
      name: e.person.name,
      born: bio.birthday,
      died: bio.deathday,
      birthplace: bio.place_of_birth,
      bio: clipText(bio.biography, 600),
      titles: briefs(all, refs, 25),
    };
  },
};

const collection: ResearchTool = {
  decl: {
    name: "collection",
    description:
      "The franchise a title belongs to. For movies, the TMDB collection in release order. For anime, its prequels, sequels, side stories and films from AniList.",
    parameters: { type: "object", properties: { ref: { type: "string" } }, required: ["ref"] },
  },
  label: (a) => `Mapping the franchise of ${a.ref}`,
  async run(args, { refs }) {
    const t = titleArg(args, refs);
    const anilistId = t.ids.anilist ?? (t.origin.provider === "anilist" ? Number(t.origin.id) : undefined);
    if (anilistId) {
      const r = (await api.anilistRaw(
        `query($id:Int){ Media(id:$id, type: ANIME){ relations { edges { relationType node { ${ANILIST_MEDIA} type } } } } }`,
        { id: anilistId },
      )) as { Media?: { relations?: { edges?: { relationType: string; node: Record<string, any> }[] } } };
      const edges = (r.Media?.relations?.edges ?? []).filter((e) => e.node.type === "ANIME");
      return {
        franchise: t.name,
        related: edges.map((e) => ({ relation: e.relationType.toLowerCase().replace(/_/g, " "), ...brief(anilistSummary(e.node), refs), format: e.node.format })),
      };
    }
    const path = tmdbPath(t);
    if (!path?.startsWith("movie")) return { error: "Only movies and anime have franchise data. For series, use list_episodes." };
    const m = (await api.tmdbRaw(`/${path}`)) as { belongs_to_collection?: { id: number; name: string } | null };
    if (!m.belongs_to_collection) return { franchise: null, note: "This movie isn't part of a TMDB collection." };
    const parts = await api.tmdbList(`/collection/${m.belongs_to_collection.id}`, {}, "parts", "movie");
    parts.sort((a, b) => (a.year ?? 9999) - (b.year ?? 9999));
    return { franchise: m.belongs_to_collection.name, titles: briefs(parts, refs, 20) };
  },
};

const upcoming: ResearchTool = {
  decl: {
    name: "upcoming",
    description:
      "Upcoming releases: movies coming soon, series on the air, or the anime airing schedule for the next days. With an anime ref, when its next episode airs.",
    parameters: {
      type: "object",
      properties: {
        kind: { type: "string", enum: KIND_ENUM },
        ref: { type: "string", description: "Optional anime ref for its next episode." },
        days: { type: "integer", description: "Anime schedule window, 1 to 7. Defaults to 7." },
      },
      required: ["kind"],
    },
  },
  label: (a) => (a.ref ? `Checking when ${a.ref} airs next` : `Checking upcoming ${a.kind === "tv" ? "series" : a.kind}`),
  async run(args, { refs }) {
    const kind = str(args.kind);
    if (kind === "anime") {
      const ref = str(args.ref);
      if (ref) {
        const t = titleArg(args, refs);
        const id = t.ids.anilist ?? (t.origin.provider === "anilist" ? Number(t.origin.id) : undefined);
        if (!id) return { error: "That title has no AniList id." };
        const r = (await api.anilistRaw(`query($id:Int){ Media(id:$id){ status episodes nextAiringEpisode { episode airingAt } } }`, { id })) as {
          Media?: { status?: string; episodes?: number; nextAiringEpisode?: { episode: number; airingAt: number } | null };
        };
        const next = r.Media?.nextAiringEpisode;
        return {
          ref,
          status: r.Media?.status,
          total_episodes: r.Media?.episodes,
          next_episode: next ? { episode: next.episode, airs_at: new Date(next.airingAt * 1000).toISOString() } : null,
        };
      }
      const now = Math.floor(Date.now() / 1000);
      const days = Math.min(7, Math.max(1, num(args.days) ?? 7));
      const r = (await api.anilistRaw(
        `query($from:Int,$to:Int){ Page(perPage: 50){ airingSchedules(airingAt_greater:$from, airingAt_lesser:$to, sort: TIME){ episode airingAt media { ${ANILIST_MEDIA} isAdult } } } }`,
        { from: now, to: now + days * 86_400 },
      )) as { Page?: { airingSchedules?: { episode: number; airingAt: number; media: Record<string, any> }[] } };
      return {
        schedule: (r.Page?.airingSchedules ?? [])
          .filter((s) => !s.media.isAdult)
          .slice(0, 40)
          .map((s) => ({ ...brief(anilistSummary(s.media), refs), episode: s.episode, airs_at: new Date(s.airingAt * 1000).toISOString() })),
      };
    }
    const path = kind === "tv" ? "/tv/on_the_air" : "/movie/upcoming";
    return { titles: briefs(await api.tmdbList(path, {}, "results", kind === "tv" ? "tv" : "movie"), refs, 15) };
  },
};

export const discoveryTools: ResearchTool[] = [
  universeSearch,
  tmdbSearch,
  anilistSearch,
  titleDetail,
  titleRatings,
  similarTitles,
  discover,
  genreList,
  trending,
  homeRails,
  tonight,
  listEpisodes,
  trailer,
  titleImages,
  personSearch,
  personCredits,
  collection,
  upcoming,
];
