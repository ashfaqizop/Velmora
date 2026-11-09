// Sports and live TV tools: scores, fixtures, results, where to watch, and the
// channel library. Matches and channels come back with refs (m1, c2), like
// titles do, so the cards Ani shows are made from real data.
//
// "Where to watch" is about who has the rights: official broadcasters. These
// tools never look up a stream for a paid channel, and a free channel only
// gets a play button when a hand-written rule names it.
import * as api from "../../api";
import { sports } from "../../live.svelte";
import { clockTime, dayLabel, regionName } from "../../sports";
import type { SportsMatch } from "../../types";
import type { RefBook } from "../refs";
import type { ResearchTool } from "./types";
import { num, str } from "./types";

const DELAYED = "Scores come from free sources and can run a few minutes behind.";
const SHOWN = 25;

function brief(m: SportsMatch, refs: RefBook) {
  const scored = m.home_score != null && m.away_score != null && m.status !== "upcoming";
  return {
    ref: refs.addMatch(m),
    title: m.title,
    sport: m.sport,
    league: m.league.name,
    status: m.status,
    ...(scored ? { score: `${m.home_score}-${m.away_score}` } : {}),
    ...(m.phase ? { phase: m.phase } : {}),
    ...(m.kickoff ? { kickoff: `${dayLabel(m.kickoff)} ${clockTime(m.kickoff)}` } : {}),
    ...(m.round ? { round: m.round } : {}),
  };
}

/** Keeps matches whose league, sport or title contains the text. */
function only(list: SportsMatch[], text?: string) {
  const t = text?.toLowerCase();
  return t ? list.filter((m) => [m.league.name, m.sport, m.title].some((s) => s.toLowerCase().includes(t))) : list;
}

const FILTER = { type: "string" as const, description: "Optional: a league, sport or team to narrow to, e.g. 'Premier League', 'cricket', 'Arsenal'." };
const DAYS = { type: "integer" as const, description: "How many days, 1 to 7. Defaults to 2." };
const days = (a: Record<string, unknown>) => Math.min(7, Math.max(1, Math.round(num(a.days) ?? 2)));
const matchList = (list: SportsMatch[], refs: RefBook) => ({
  note: DELAYED,
  total: list.length,
  matches: list.slice(0, SHOWN).map((m) => brief(m, refs)),
});

const sportsLive: ResearchTool = {
  decl: {
    name: "sports_live",
    description: "Matches in play right now, across football, basketball, cricket, baseball, hockey and more. Returns matches with refs.",
    parameters: { type: "object", properties: { filter: FILTER } },
  },
  label: () => "Checking live scores",
  async run(args, { refs }) {
    return matchList(only(await api.sportsLive(), str(args.filter)), refs);
  },
};

const sportsUpcoming: ResearchTool = {
  decl: {
    name: "sports_upcoming",
    description: "Matches coming up in the next few days, soonest first. Returns matches with refs and local kickoff times.",
    parameters: { type: "object", properties: { days: DAYS, filter: FILTER } },
  },
  label: () => "Checking upcoming matches",
  async run(args, { refs }) {
    return matchList(only(await api.sportsUpcoming(days(args)), str(args.filter)), refs);
  },
};

const sportsResults: ResearchTool = {
  decl: {
    name: "sports_results",
    description: "Finished matches from the last few days, newest first, with scores. Returns matches with refs.",
    parameters: { type: "object", properties: { days: DAYS, filter: FILTER } },
  },
  label: () => "Checking recent results",
  async run(args, { refs }) {
    return matchList(only(await api.sportsResults(days(args)), str(args.filter)), refs);
  },
};

const sportsDay: ResearchTool = {
  decl: {
    name: "sports_day",
    description: "Every match on one specific date. Use when the user names a day. The date is UTC.",
    parameters: {
      type: "object",
      properties: { date: { type: "string", description: "YYYY-MM-DD." }, filter: FILTER },
      required: ["date"],
    },
  },
  label: (a) => `Checking matches on ${str(a.date) ?? "that day"}`,
  async run(args, { refs }) {
    const date = str(args.date);
    if (!date || !/^\d{4}-\d{2}-\d{2}$/.test(date)) throw new Error("date must look like 2026-11-09");
    const day = await api.sportsDay(date);
    return { ...matchList(only(day.matches, str(args.filter)), refs), date: day.date };
  },
};

const sportsMatch: ResearchTool = {
  decl: {
    name: "sports_match",
    description: "Fresh details for one match you already found: score, status, venue, round, kickoff.",
    parameters: { type: "object", properties: { ref: { type: "string", description: "A match ref such as m2." } }, required: ["ref"] },
  },
  label: () => "Checking the match",
  async run(args, { refs }) {
    const known = refs.match(str(args.ref) ?? "");
    if (!known) throw new Error("No match with that ref. Use a ref from an earlier result.");
    const m = await api.sportsLookup(known.id).catch(() => known);
    return { ...brief(m, refs), venue: m.venue, home: m.home?.name, away: m.away?.name, note: DELAYED };
  },
};

const sportsWhere: ResearchTool = {
  decl: {
    name: "sports_where_to_watch",
    description:
      "Official broadcasters and streaming services for a match in a country. Only official ones are listed, from hand-checked rules and a TV listing. Free channels that carry it are marked and can be played in Velmora. Defaults to the user's country.",
    parameters: {
      type: "object",
      properties: {
        ref: { type: "string", description: "A match ref such as m2." },
        country: { type: "string", description: "ISO country code such as US, GB or IN. Optional." },
      },
      required: ["ref"],
    },
  },
  label: () => "Finding where to watch",
  async run(args, { refs }) {
    const m = refs.match(str(args.ref) ?? "");
    if (!m) throw new Error("No match with that ref. Use a ref from an earlier result.");
    const region = (str(args.country) ?? sports.region).toUpperCase();
    const options = await api.sportsWatch(m, region);
    const rows = await Promise.all(
      options.map(async (o) => {
        const channel = o.play_channel ? await api.tvChannel(o.play_channel).catch(() => null) : null;
        return { name: o.name, kind: o.kind, country: o.country, ...(channel ? { free_channel_ref: refs.addChannel(channel) } : {}) };
      }),
    );
    return {
      country: regionName(region),
      options: rows,
      note: rows.length
        ? "Official broadcasters only. Say if none is free; never suggest a stream from anywhere else."
        : "No official broadcaster is listed for this match in this country. Say so, and suggest checking the league's own site.",
    };
  },
};

const tvFind: ResearchTool = {
  decl: {
    name: "tv_find_channels",
    description:
      "Search Velmora's Live TV channel library, built from a public playlist. Use to answer whether a channel is available, or to find news, kids, music or sports channels in a country. It says nothing about what is airing. Returns channels with refs.",
    parameters: {
      type: "object",
      properties: {
        text: { type: "string", description: "Part of a channel name." },
        country: { type: "string", description: "ISO country code, e.g. IN." },
        category: { type: "string", description: "sports, news, movies, music, kids, documentary, entertainment, general..." },
        limit: { type: "integer", description: "Up to 15. Defaults to 10." },
      },
    },
  },
  label: (a) => (str(a.text) ? `Looking for channel "${str(a.text)}"` : "Browsing live channels"),
  async run(args, { refs }) {
    const page = await api.tvChannels({
      text: str(args.text),
      country: str(args.country)?.toUpperCase(),
      category: str(args.category)?.toLowerCase(),
      limit: Math.min(15, Math.max(1, Math.round(num(args.limit) ?? 10))),
    });
    return {
      total: page.total,
      channels: page.items.map((c) => ({
        ref: refs.addChannel(c),
        name: c.name,
        country: c.country,
        categories: c.categories,
        streams: c.streams.length,
      })),
      note: "From a public playlist. Some channels may not play.",
    };
  },
};

const tvCheck: ResearchTool = {
  decl: {
    name: "tv_check_channel",
    description: "Test whether a channel's streams are answering right now. Returns how many work.",
    parameters: { type: "object", properties: { ref: { type: "string", description: "A channel ref such as c1." } }, required: ["ref"] },
  },
  label: () => "Testing the channel",
  async run(args, { refs }) {
    const ch = refs.channel(str(args.ref) ?? "");
    if (!ch) throw new Error("No channel with that ref. Use a ref from tv_find_channels.");
    const health = await api.tvCheck(ch.id);
    const working = health.filter((h) => h.ok).length;
    return { channel: ch.name, working, tested: health.length, playable: working > 0 };
  },
};

export const sportsTools: ResearchTool[] = [sportsLive, sportsUpcoming, sportsResults, sportsDay, sportsMatch, sportsWhere, tvFind, tvCheck];
