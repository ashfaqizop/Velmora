// Generative UI: each render_* function is a card type Ani can put in a
// reply. A call is checked against the RefBook before it becomes a block, so
// unknown refs are dropped and Gemini hears about them.
import type { FunctionDeclaration, Schema } from "../gemini";
import type { RefBook } from "../refs";
import type { AnswerTool } from "./types";
import { num, str, strs } from "./types";

type Args = Record<string, unknown>;
type Check = (args: Args, refs: RefBook) => { args: Args } | { error: string };

function ui(decl: FunctionDeclaration, check: Check): AnswerTool {
  return {
    decl,
    apply(args, { refs }) {
      const r = check(args, refs);
      if ("error" in r) return { response: { error: r.error } };
      return { block: { kind: "ui", name: decl.name, args: r.args }, response: { shown: true } };
    },
  };
}

const isTitle = (refs: RefBook, r: unknown) => typeof r === "string" && !!refs.title(r) && refs.get(r)?.type === "title";
const isLink = (refs: RefBook, r: unknown) => typeof r === "string" && refs.get(r)?.type === "link";
const missing = (kind: string) => ({ error: `None of those refs are ${kind} from this chat. Use refs from the research notes.` });

const S = {
  str: (description?: string): Schema => ({ type: "string", description }),
  int: (description?: string): Schema => ({ type: "integer", description }),
  refs: (description: string): Schema => ({ type: "array", items: { type: "string" }, description }),
  strs: (description?: string): Schema => ({ type: "array", items: { type: "string" }, description }),
};

/** Titles from `refs`, keeping only real title refs. */
const titleRefs = (min: number, max: number): Check => (args, refs) => {
  const list = strs(args.refs).filter((r) => isTitle(refs, r));
  if (list.length < min) return list.length ? { error: `Show at least ${min} titles here.` } : missing("titles");
  return { args: { ...args, refs: list.slice(0, max) } };
};

const linkRefs: Check = (args, refs) => {
  const list = strs(args.refs).filter((r) => isLink(refs, r));
  if (!list.length) return missing("links");
  const pick = str(args.pick);
  const ok = pick && list.includes(pick);
  return { args: { ...args, refs: list.slice(0, 12), pick: ok ? pick : undefined, pick_reason: ok ? str(args.pick_reason) : undefined } };
};

const titleList = ui(
  {
    name: "render_title_list",
    description:
      "The main way to show titles. One row per title: poster, name and year, a short badge, a sentence on why it fits, and up to four bullet facts, with Play, My List and Download buttons. Use for 1 to 8 recommendations or answers about specific titles.",
    parameters: {
      type: "object",
      properties: {
        heading: S.str("Optional heading above the list."),
        items: {
          type: "array",
          items: {
            type: "object",
            properties: {
              ref: S.str("Title ref."),
              badge: S.str("Two to four words, e.g. 'Same director', 'Top pick', 'Hindi · 2023'."),
              why: S.str("One or two sentences on why it fits this user and request."),
              bullets: S.strs("Up to four short facts, e.g. runtime, rating, where it's available, the lead actor."),
            },
            required: ["ref"],
          },
        },
      },
      required: ["items"],
    },
  },
  (args, refs) => {
    const raw = Array.isArray(args.items) ? (args.items as Args[]) : [];
    const items = raw
      .filter((i) => isTitle(refs, i?.ref))
      .slice(0, 8)
      .map((i) => ({ ref: i.ref, badge: str(i.badge), why: str(i.why), bullets: strs(i.bullets).slice(0, 4) }));
    return items.length ? { args: { heading: str(args.heading), items } } : missing("titles");
  },
);

const carousel = ui(
  {
    name: "render_carousel",
    description:
      "A horizontal row of posters, like a Home rail, for 5 to 20 titles that need no explanation each. Use groups for tabs (Movies, Series, Anime), ranked for trending or top lists (big rank numbers), and for_you to mark titles that fit this user.",
    parameters: {
      type: "object",
      properties: {
        heading: S.str(),
        refs: S.refs("Title refs in display order. Leave out when using groups."),
        groups: {
          type: "array",
          description: "Tabs, each with its own row.",
          items: { type: "object", properties: { label: S.str("e.g. Movies"), refs: S.refs("Title refs.") }, required: ["label", "refs"] },
        },
        ranked: { type: "boolean", description: "Show rank numbers 1, 2, 3." },
        for_you: S.refs("Refs that match the user's taste; they get a 'For you' chip."),
      },
    },
  },
  (args, refs) => {
    const keep = (list: unknown) => strs(list).filter((r) => isTitle(refs, r)).slice(0, 20);
    const groups = (Array.isArray(args.groups) ? (args.groups as Args[]) : [])
      .map((g) => ({ label: str(g?.label) ?? "Titles", refs: keep(g?.refs) }))
      .filter((g) => g.refs.length)
      .slice(0, 4);
    const flat = keep(args.refs);
    if (!groups.length && flat.length < 2) return flat.length ? { error: "Show at least 2 titles here." } : missing("titles");
    return { args: { heading: str(args.heading), refs: flat, groups, ranked: args.ranked === true, for_you: keep(args.for_you) } };
  },
);

const spotlight = ui(
  {
    name: "render_spotlight",
    description: "One large backdrop card for a single strong pick, with why it fits and Play, My List and Download buttons.",
    parameters: {
      type: "object",
      properties: { ref: S.str("Title ref."), badge: S.str("Two to four words."), why: S.str("Two or three sentences.") },
      required: ["ref"],
    },
  },
  (args, refs) => (isTitle(refs, args.ref) ? { args: { ref: args.ref, badge: str(args.badge), why: str(args.why) } } : missing("titles")),
);

const titleCompare = ui(
  {
    name: "render_title_compare",
    description:
      "Two or three titles side by side. Year, rating and runtime are filled in automatically; add rows for anything else that matters to the question (tone, length, where to start), and a one-line verdict.",
    parameters: {
      type: "object",
      properties: {
        refs: S.refs("Two or three title refs."),
        rows: {
          type: "array",
          items: {
            type: "object",
            properties: { label: S.str(), values: S.strs("One value per title, in the same order as refs.") },
            required: ["label", "values"],
          },
        },
        verdict: S.str("One sentence: which to pick and why."),
      },
      required: ["refs"],
    },
  },
  (args, refs) => {
    const r = titleRefs(2, 3)(args, refs);
    if ("error" in r) return r;
    const rows = (Array.isArray(args.rows) ? (args.rows as Args[]) : [])
      .map((row) => ({ label: str(row?.label), values: strs(row?.values) }))
      .filter((row) => row.label && row.values.length)
      .slice(0, 6);
    return { args: { refs: r.args.refs, rows, verdict: str(args.verdict) } };
  },
);

const episodes = ui(
  {
    name: "render_episodes",
    description: "The episode list for one season of a series or anime, with stills, synopses and Play buttons. Highlight episodes you talk about.",
    parameters: {
      type: "object",
      properties: { ref: S.str("Title ref."), season: S.int("Season number."), highlight: { type: "array", items: { type: "integer" } } },
      required: ["ref", "season"],
    },
  },
  (args, refs) =>
    isTitle(refs, args.ref)
      ? { args: { ref: args.ref, season: num(args.season) ?? 1, highlight: (Array.isArray(args.highlight) ? args.highlight : []).map(num).filter((n) => n !== undefined) } }
      : missing("titles"),
);

const seasonOverview = ui(
  {
    name: "render_season_overview",
    description: "Every season of a show with its episode count, plus a short note per season when useful ('the best one', 'skippable filler').",
    parameters: {
      type: "object",
      properties: {
        ref: S.str("Title ref."),
        notes: { type: "array", items: { type: "object", properties: { season: S.int(), text: S.str() }, required: ["season", "text"] } },
      },
      required: ["ref"],
    },
  },
  (args, refs) => {
    if (!isTitle(refs, args.ref)) return missing("titles");
    const notes = (Array.isArray(args.notes) ? (args.notes as Args[]) : [])
      .map((n) => ({ season: num(n?.season), text: str(n?.text) }))
      .filter((n) => n.season !== undefined && n.text);
    return { args: { ref: args.ref, notes } };
  },
);

const watchOrder = ui(
  {
    name: "render_watch_order",
    description: "A numbered timeline for watching a franchise or anime: films, seasons, OVAs and specials in order, each with a short label and note. Mark optional entries.",
    parameters: {
      type: "object",
      properties: {
        heading: S.str(),
        order: { type: "string", enum: ["release", "story"] },
        items: {
          type: "array",
          items: {
            type: "object",
            properties: {
              ref: S.str("Title ref."),
              label: S.str("e.g. 'Season 1', 'Movie', 'OVA'."),
              note: S.str("Half a sentence."),
              optional: { type: "boolean" },
            },
            required: ["ref"],
          },
        },
      },
      required: ["items"],
    },
  },
  (args, refs) => {
    const items = (Array.isArray(args.items) ? (args.items as Args[]) : [])
      .filter((i) => isTitle(refs, i?.ref))
      .slice(0, 24)
      .map((i) => ({ ref: i.ref, label: str(i.label), note: str(i.note), optional: i.optional === true }));
    return items.length ? { args: { heading: str(args.heading), order: str(args.order) ?? "release", items } } : missing("titles");
  },
);

const collection = ui(
  {
    name: "render_collection",
    description: "A franchise or collection as a poster grid. Titles the user already finished get a tick.",
    parameters: { type: "object", properties: { heading: S.str(), refs: S.refs("Title refs in order.") }, required: ["refs"] },
  },
  titleRefs(2, 24),
);

const LINK_PARAMS: Schema = {
  type: "object",
  properties: {
    refs: S.refs("Link refs, best first."),
    pick: S.str("The link ref you recommend; it gets highlighted."),
    pick_reason: S.str("Why that one, in a few words, e.g. 'Best quality for the size'."),
    note: S.str("Optional one-line note shown above the links."),
  },
  required: ["refs"],
};

const downloadLinks = ui(
  {
    name: "render_download_links",
    description: "Download links found by UniverseAPI, one row each with quality, size, codec, audio and subtitles, and a Download button the user can press.",
    parameters: LINK_PARAMS,
  },
  linkRefs,
);

const torrents = ui(
  {
    name: "render_torrents",
    description: "Torrent links with seeders, size and release group, each with a Download button. The card carries the VPN reminder itself.",
    parameters: LINK_PARAMS,
  },
  linkRefs,
);

const streamLinks = ui(
  {
    name: "render_stream_links",
    description: "Streaming links found by UniverseAPI with quality and audio. The Play button starts the title in Velmora's player.",
    parameters: LINK_PARAMS,
  },
  linkRefs,
);

const linkCompare = ui(
  {
    name: "render_link_compare",
    description: "Two to four links compared in columns (quality, size, codec, HDR, audio, subtitles, seeders) with your pick highlighted and the reasons for it.",
    parameters: {
      type: "object",
      properties: { refs: S.refs("Link refs."), pick: S.str("The recommended link ref."), reasons: S.strs("Up to three short reasons for the pick.") },
      required: ["refs", "pick"],
    },
  },
  (args, refs) => {
    const r = linkRefs(args, refs);
    if ("error" in r) return r;
    const list = r.args.refs as string[];
    if (list.length < 2) return { error: "Compare at least two links." };
    return { args: { refs: list.slice(0, 4), pick: r.args.pick ?? list[0], reasons: strs(args.reasons).slice(0, 3) } };
  },
);


const isPerson = (refs: RefBook, r: unknown) => typeof r === "string" && refs.get(r)?.type === "person";
const oneTitle: Check = (args, refs) => (isTitle(refs, args.ref) ? { args: { ...args, ref: args.ref } } : missing("titles"));

const cast = ui(
  {
    name: "render_cast",
    description: "The cast and key crew of a title as a strip of photos with character names.",
    parameters: { type: "object", properties: { ref: S.str("Title ref."), heading: S.str() }, required: ["ref"] },
  },
  oneTitle,
);

const person = ui(
  {
    name: "render_person",
    description: "A card for one actor, director or other person: photo, role, a short bio in your words, and posters of what they're known for.",
    parameters: {
      type: "object",
      properties: {
        ref: S.str("Person ref from person_search or title_detail."),
        bio: S.str("Two or three sentences."),
        known_for: S.refs("Up to eight title refs."),
      },
      required: ["ref"],
    },
  },
  (args, refs) =>
    isPerson(refs, args.ref)
      ? { args: { ref: args.ref, bio: str(args.bio), known_for: strs(args.known_for).filter((r) => isTitle(refs, r)).slice(0, 8) } }
      : missing("people"),
);

const ratings = ui(
  {
    name: "render_ratings",
    description: "Score tiles for a title: TMDB or AniList score with vote count, age rating, and a link to IMDb.",
    parameters: { type: "object", properties: { ref: S.str("Title ref.") }, required: ["ref"] },
  },
  oneTitle,
);

const facts = ui(
  {
    name: "render_facts",
    description: "A fact sheet for a title: runtime, release date, status, studio, language, country, seasons. Add extra rows for anything else worth knowing.",
    parameters: {
      type: "object",
      properties: {
        ref: S.str("Title ref."),
        extra: { type: "array", items: { type: "object", properties: { label: S.str(), value: S.str() }, required: ["label", "value"] } },
      },
      required: ["ref"],
    },
  },
  (args, refs) => {
    if (!isTitle(refs, args.ref)) return missing("titles");
    const extra = (Array.isArray(args.extra) ? (args.extra as Args[]) : [])
      .map((e) => ({ label: str(e?.label), value: str(e?.value) }))
      .filter((e) => e.label && e.value)
      .slice(0, 6);
    return { args: { ref: args.ref, extra } };
  },
);

const trailer = ui(
  {
    name: "render_trailer",
    description: "The title's trailer, playable inside the chat. Only when the trailer tool said it has one.",
    parameters: { type: "object", properties: { ref: S.str("Title ref.") }, required: ["ref"] },
  },
  oneTitle,
);

const gallery = ui(
  {
    name: "render_gallery",
    description: "Stills and backdrops from a title.",
    parameters: { type: "object", properties: { ref: S.str("Title ref.") }, required: ["ref"] },
  },
  oneTitle,
);

const schedule = ui(
  {
    name: "render_schedule",
    description: "An airing calendar grouped by day, from the upcoming tool's schedule: each entry is a title ref, an episode number and its airs_at time.",
    parameters: {
      type: "object",
      properties: {
        heading: S.str(),
        items: {
          type: "array",
          items: {
            type: "object",
            properties: { ref: S.str(), episode: S.int(), airs_at: S.str("ISO time from the upcoming tool.") },
            required: ["ref", "airs_at"],
          },
        },
      },
      required: ["items"],
    },
  },
  (args, refs) => {
    const items = (Array.isArray(args.items) ? (args.items as Args[]) : [])
      .filter((i) => isTitle(refs, i?.ref) && !Number.isNaN(Date.parse(String(i?.airs_at))))
      .slice(0, 40)
      .map((i) => ({ ref: i.ref, episode: num(i.episode), airs_at: String(i.airs_at) }));
    return items.length ? { args: { heading: str(args.heading), items } } : missing("titles with air times");
  },
);

const progress = ui(
  {
    name: "render_progress",
    description: "Continue Watching cards with progress bars and Resume buttons. Pass title refs to show some, or none for the user's whole list.",
    parameters: { type: "object", properties: { refs: S.refs("Optional title refs.") } },
  },
  (args, refs) => ({ args: { refs: strs(args.refs).filter((r) => isTitle(refs, r)) } }),
);

const taste = ui(
  {
    name: "render_taste_profile",
    description: "The user's taste from Personalize: genre bars, what they watch most, decades, and recent titles. Use when they ask what you know about them or what their taste is.",
    parameters: { type: "object", properties: { summary: S.str("One or two sentences describing their taste in your words.") } },
  },
  (args) => ({ args: { summary: str(args.summary) } }),
);

const choices = ui(
  {
    name: "render_choices",
    description: "Two to five tappable replies. Use to ask a clarifying question ('Movies or series?', 'Sub or dub?') or to offer follow-ups. Tapping one sends it as the user's next message.",
    parameters: {
      type: "object",
      properties: { question: S.str("Optional short question above the options."), options: S.strs("Short replies in the user's voice.") },
      required: ["options"],
    },
  },
  (args) => {
    const options = strs(args.options).slice(0, 5);
    return options.length >= 2 ? { args: { question: str(args.question), options } } : { error: "Give at least two options." };
  },
);

const about = ui(
  {
    name: "render_about",
    description: "A card about Velmora, UniverseAPI or Ani itself, with live details (version, number of sources, what Personalize holds) and links. Use when the user asks what one of them is or can do.",
    parameters: {
      type: "object",
      properties: { topic: { type: "string", enum: ["velmora", "universeapi", "ani"] } },
      required: ["topic"],
    },
  },
  (args) => {
    const topic = str(args.topic);
    return topic && ["velmora", "universeapi", "ani"].includes(topic) ? { args: { topic } } : { error: "topic must be velmora, universeapi or ani." };
  },
);

export const renderTools: AnswerTool[] = [
  titleList,
  carousel,
  spotlight,
  titleCompare,
  episodes,
  seasonOverview,
  watchOrder,
  collection,
  downloadLinks,
  torrents,
  streamLinks,
  linkCompare,
  cast,
  person,
  ratings,
  facts,
  trailer,
  gallery,
  schedule,
  progress,
  taste,
  choices,
  about,
];
