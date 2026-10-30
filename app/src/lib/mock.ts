// Browser-only stand-ins for the engine, used when the UI runs outside Tauri.
import type { AppInfo, ConfigStatus, MediaKind, ProviderRef, Rail, SearchBatch, TitleDetail, TitleSummary } from "./types";

const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

function t(id: string, name: string, kind: MediaKind, year: number, rating: number, overview: string): TitleSummary {
  return { origin: { provider: "cinemeta", id }, name, kind, year, rating, overview, ids: {} };
}

const movies = [
  t("m1", "Northern Lights", "movie", 2025, 7.9, "A night-shift pilot chases an aurora storm across the Arctic to bring her brother home."),
  t("m2", "Glass Harbor", "movie", 2024, 7.2, "A port town hides a smuggling ring inside its annual lantern festival."),
  t("m3", "The Long Quiet", "movie", 2023, 8.1, "Two radio operators on opposite sides of a war keep talking after the line goes dead."),
  t("m4", "Paper Kings", "movie", 2025, 6.8, "A forger and an art critic team up for one last impossible auction."),
  t("m5", "Saltwater", "movie", 2022, 7.5, "A free diver returns to the reef that nearly killed her."),
  t("m6", "Tin Moon", "movie", 2024, 7.0, "A robot built for the moon gets stranded in a small desert town."),
];
const series = [
  t("s1", "Lowland", "series", 2025, 8.4, "A flooded city elects a mayor nobody expected."),
  t("s2", "Night Market", "series", 2024, 7.8, "Food stalls, debts and a missing cook in Taipei."),
  t("s3", "Signal & Noise", "series", 2023, 8.0, "A podcast about an unsolved case starts solving it."),
  t("s4", "Ember Hall", "series", 2025, 7.6, "Students at a fire academy learn the job the hard way."),
];
const anime = [
  t("a1", "Skyward Courier", "anime", 2025, 8.6, "A girl delivers letters between floating islands."),
  t("a2", "Iron Bloom", "anime", 2024, 8.2, "A mech pilot who would rather be a gardener."),
  t("a3", "Hollow Stars", "anime", 2023, 7.9, "Ghost hunters in a city that never sleeps."),
];

export async function appInfo(): Promise<AppInfo> {
  return { version: "0.1.0-dev", platform: "browser", tmdb: false };
}

export async function refreshConfig(): Promise<ConfigStatus> {
  await wait(900);
  return { updated: true };
}

export async function home(): Promise<Rail[]> {
  await wait(700);
  return [
    { id: "trending_movies", title: "Trending movies", items: movies },
    { id: "trending_series", title: "Trending series", items: series },
    { id: "trending_anime", title: "Trending anime", items: anime },
    { id: "airing_today", title: "Airing today", items: [...series].reverse() },
    { id: "hdhub_latest", title: "Latest on HDHub", items: [...movies].reverse() },
  ];
}

export async function detail(origin: ProviderRef): Promise<TitleDetail> {
  await wait(400);
  const summary = [...movies, ...series, ...anime].find((x) => x.origin.id === origin.id) ?? movies[0];
  const episodic = summary.kind !== "movie";
  return {
    summary,
    genres: ["Drama", "Adventure"],
    cast: ["Ada Lune", "Kei Morrow", "Sam Ortega"],
    directors: ["R. Vale"],
    runtime_minutes: episodic ? 45 : 118,
    screenshots: [],
    seasons: episodic
      ? [1, 2].map((n) => ({
          number: n,
          episodes: Array.from({ length: 8 }, (_, i) => ({ season: n, number: i + 1, title: `Episode ${i + 1}` })),
        }))
      : [],
  };
}

export async function searchStream(query: string, onBatch: (b: SearchBatch) => void): Promise<void> {
  const q = query.toLowerCase();
  const all = [...movies, ...series, ...anime];
  for (const provider of ["cinemeta", "anilist", "hdhub", "moviebox"]) {
    await wait(250 + Math.random() * 500);
    onBatch({ provider, items: all.filter((x) => x.name.toLowerCase().includes(q) && Math.random() > 0.3) });
  }
}
