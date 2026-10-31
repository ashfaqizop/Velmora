// Link tools: what UniverseAPI finds for a title, read-only. Ani shows these
// as cards; the user starts a stream or a download from the card.
import * as api from "../../api";
import { isTorrent } from "../../sources";
import { PROVIDER_GROUPS, providerLabel } from "../../options";
import { settings } from "../../state.svelte";
import type { AudioMode, Source, SourcesReport, Target, TitleSummary } from "../../types";
import type { ResearchTool, ToolContext } from "./types";
import { num, str, strs } from "./types";
import { link, targetArg, titleArg } from "./shape";

const LINK_PARAMS = {
  type: "object" as const,
  properties: {
    ref: { type: "string" as const, description: "A title ref." },
    season: { type: "integer" as const, description: "For series and anime. Defaults to 1." },
    episode: { type: "integer" as const, description: "For series and anime. Defaults to 1." },
    audio: { type: "string" as const, enum: ["sub", "dub"], description: "Anime only. Defaults to the user's setting." },
  },
  required: ["ref"],
};

// One sources lookup per title and episode per turn, shared by every link tool.
const lookups = new Map<string, Promise<SourcesReport>>();
function sourcesFor(t: TitleSummary, target: Target, mode: AudioMode) {
  const k = `${t.origin.provider}:${t.origin.id}|${JSON.stringify(target)}|${mode}`;
  let p = lookups.get(k);
  if (!p) {
    p = api.sources(t, target, mode);
    p.catch(() => lookups.delete(k));
    lookups.set(k, p);
    setTimeout(() => lookups.delete(k), 5 * 60_000);
  }
  return p;
}

async function find(args: Record<string, unknown>, { refs }: ToolContext, keep: (s: Source) => boolean, limit = 12) {
  const t = titleArg(args, refs);
  const target = targetArg(t, args);
  const mode = (str(args.audio) as AudioMode | undefined) ?? settings.audioMode;
  const report = await sourcesFor(t, target, mode);
  const kept = report.sources.filter(keep);
  return {
    title: t.name,
    episode: target.type === "episode" ? `S${target.season} E${target.episode}` : undefined,
    found: kept.length,
    links: kept.slice(0, limit).map((s) => link(s, t, target, refs)),
    failed_sources: report.errors.length ? report.errors.map((e) => providerLabel(e.provider)) : undefined,
    credit: "Found by UniverseAPI",
  };
}

const episodeLabel = (a: Record<string, unknown>) => (a.season || a.episode ? ` S${a.season ?? 1} E${a.episode ?? 1}` : "");

const findStreamLinks: ResearchTool = {
  decl: {
    name: "find_stream_links",
    description: "Ranked links Velmora can stream for a title or episode, best first, from UniverseAPI. Includes torrents when the user has them on.",
    parameters: LINK_PARAMS,
  },
  label: (a) => `Finding streams for ${a.ref}${episodeLabel(a)}`,
  run: (args, ctx) => find(args, ctx, () => true),
};

const findDownloadLinks: ResearchTool = {
  decl: {
    name: "find_download_links",
    description:
      "Direct downloads for a title or episode (no torrents), best first, with quality, size, codec, audio and subtitles. Velmora's downloader can save any of them.",
    parameters: LINK_PARAMS,
  },
  label: (a) => `Finding downloads for ${a.ref}${episodeLabel(a)}`,
  run: (args, ctx) => find(args, ctx, (s) => !isTorrent(s)),
};

const findTorrents: ResearchTool = {
  decl: {
    name: "find_torrents",
    description: "Torrent releases for a title or episode, most seeded first. Peers can see the user's IP, so mention a VPN.",
    parameters: LINK_PARAMS,
  },
  label: (a) => `Finding torrents for ${a.ref}${episodeLabel(a)}`,
  async run(args, ctx) {
    const r = await find(args, ctx, isTorrent, 30);
    r.links.sort((a, b) => (b.torrent?.seeders ?? 0) - (a.torrent?.seeders ?? 0));
    return { ...r, links: r.links.slice(0, 12), torrents_enabled: settings.prefs?.include_torrents ?? true };
  },
};

const findSeasonPacks: ResearchTool = {
  decl: {
    name: "find_season_packs",
    description: "Whole-season downloads (packs and archives) for one season of a series or anime.",
    parameters: {
      type: "object",
      properties: { ref: { type: "string" }, season: { type: "integer" }, audio: { type: "string", enum: ["sub", "dub"] } },
      required: ["ref", "season"],
    },
  },
  label: (a) => `Finding season ${a.season ?? 1} packs for ${a.ref}`,
  async run(args, { refs }) {
    const t = titleArg(args, refs);
    const target: Target = { type: "season", season: num(args.season) ?? 1 };
    const mode = (str(args.audio) as AudioMode | undefined) ?? settings.audioMode;
    const report = await sourcesFor(t, target, mode);
    const packs = report.sources.filter((s) => s.is_pack || s.tags.is_pack || s.format === "archive");
    return { title: t.name, season: target.season, found: packs.length, links: packs.slice(0, 12).map((s) => link(s, t, target, refs)) };
  },
};

const linkDetails: ResearchTool = {
  decl: {
    name: "link_details",
    description: "Everything known about one link: file name, release label, video, HDR, audio tracks, subtitles, size, seeders.",
    parameters: { type: "object", properties: { ref: { type: "string", description: "A link ref such as l2." } }, required: ["ref"] },
  },
  label: (a) => `Inspecting ${a.ref}`,
  async run(args, { refs }) {
    const e = refs.get(str(args.ref) ?? "");
    if (e?.type !== "link") throw new Error("Unknown link ref. Use a ref from a find_* result.");
    const s = e.source;
    return {
      ...link(s, e.title, e.target, refs),
      ref: str(args.ref),
      file_name: s.filename,
      release: s.label,
      source: s.tags.source,
      bit_depth: s.tags.bit_depth,
      audio_tracks: s.audio?.map((a) => a.label),
      subtitle_tracks: s.subtitles?.map((x) => x.label),
      expires: s.expires_at ? new Date(s.expires_at * 1000).toISOString() : undefined,
    };
  },
};

const compareLinks: ResearchTool = {
  decl: {
    name: "compare_links",
    description:
      "Compare two to six links side by side and get Velmora's pick for a goal: best quality, smallest file, best audio, or a language.",
    parameters: {
      type: "object",
      properties: {
        refs: { type: "array", items: { type: "string" }, description: "Link refs." },
        goal: { type: "string", enum: ["quality", "smallest", "audio", "language"] },
        language: { type: "string", description: "For goal 'language': an ISO code or name, e.g. hi, Hindi." },
      },
      required: ["refs"],
    },
  },
  label: () => "Comparing links",
  async run(args, { refs }) {
    const items = strs(args.refs)
      .map((r) => ({ r, e: refs.get(r) }))
      .filter((x): x is { r: string; e: Extract<NonNullable<ReturnType<typeof refs.get>>, { type: "link" }> } => x.e?.type === "link");
    if (items.length < 2) throw new Error("Give at least two link refs.");
    const goal = str(args.goal) ?? "quality";
    const lang = str(args.language)?.toLowerCase();
    const size = (s: Source) => s.size_bytes ?? s.tags.size_bytes ?? Infinity;
    const score = (s: Source) => {
      if (goal === "smallest") return -size(s);
      if (goal === "audio") return (s.tags.multi_audio ? 2 : 0) + (s.audio?.length ?? s.tags.audio?.length ?? 0);
      if (goal === "language" && lang) {
        const langs = [...(s.tags.languages ?? []), ...(s.tags.audio ?? []), ...(s.audio ?? []).map((a) => a.lang ?? a.label)];
        return langs.some((l) => l?.toLowerCase().includes(lang)) ? 1000 + s.score : s.score;
      }
      return (s.tags.resolution ?? 0) * 10 + (s.tags.hdr && s.tags.hdr !== "sdr" ? 5 : 0) + s.score / 100;
    };
    const best = items.reduce((a, b) => (score(b.e.source) > score(a.e.source) ? b : a));
    return {
      goal,
      pick: best.r,
      links: items.map(({ e }) => link(e.source, e.title, e.target, refs)).map((l, i) => ({ ...l, ref: items[i].r })),
    };
  },
};

const sourceStatus: ResearchTool = {
  decl: {
    name: "source_status",
    description: "Which UniverseAPI sources are on or off for each kind of title, in the order Velmora tries them.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Checking sources",
  async run() {
    const p = await settings.load();
    return {
      torrents: p.include_torrents,
      french_sources: p.include_french,
      check_links_first: p.verify_sources,
      by_kind: PROVIDER_GROUPS.map((g) => ({
        kind: g.label,
        order: (p.provider_order[g.kind] ?? []).map((id) => ({ source: providerLabel(id), on: !p.disabled_providers.includes(id) })),
      })),
    };
  },
};

/** The file name base Velmora's downloader uses: "Dune (2021)" or "Frieren S01E03". */
function fileLabel(t: TitleSummary, target: Target) {
  if (target.type === "episode") {
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${t.name} S${pad(target.season)}E${pad(target.episode)}`;
  }
  return t.year ? `${t.name} (${t.year})` : t.name;
}

const alreadyDownloaded: ResearchTool = {
  decl: {
    name: "already_downloaded",
    description: "Whether a title or episode is already saved in the user's download folder.",
    parameters: {
      type: "object",
      properties: { ref: { type: "string" }, season: { type: "integer" }, episode: { type: "integer" } },
      required: ["ref"],
    },
  },
  label: (a) => `Checking downloads for ${a.ref}`,
  async run(args, { refs }) {
    const t = titleArg(args, refs);
    const target = targetArg(t, args);
    const exact = await api.downloadExisting(fileLabel(t, target));
    const library = await api.downloadsLibrary().catch(() => []);
    const name = t.name.toLowerCase();
    const others = library.filter((f) => f.title.toLowerCase().includes(name)).map((f) => f.title);
    return { downloaded: exact.length > 0, files: exact.map((p) => p.split(/[\\/]/).pop()), other_files_for_title: others.slice(0, 20) };
  },
};

const downloadsLibrary: ResearchTool = {
  decl: {
    name: "downloads_library",
    description: "What's on this PC: finished downloads in the download folder, plus anything downloading now.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Looking at your downloads",
  async run() {
    const [files, queue] = await Promise.all([api.downloadsLibrary().catch(() => []), api.downloadsList().catch(() => [])]);
    return {
      on_this_pc: files.slice(0, 60).map((f) => ({
        title: f.title,
        size_gb: Math.round((f.size / 1_073_741_824) * 10) / 10,
        saved: new Date(f.modified * 1000).toISOString().slice(0, 10),
      })),
      in_progress: queue
        .filter((d) => d.state.state !== "completed" && d.state.state !== "cancelled")
        .map((d) => ({ title: d.title, state: d.state.state, percent: d.total ? Math.round((d.downloaded / d.total) * 100) : undefined })),
    };
  },
};

export const linkTools: ResearchTool[] = [
  findStreamLinks,
  findDownloadLinks,
  findTorrents,
  findSeasonPacks,
  linkDetails,
  compareLinks,
  sourceStatus,
  alreadyDownloaded,
  downloadsLibrary,
];
