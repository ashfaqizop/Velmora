// Small helpers for showing sports: regions, times, status text, grouping.
import type { SportsMatch } from "./types";

/** Countries the "where to watch" list can be asked for (ISO code, name). */
export const REGIONS: [string, string][] = [
  ["US", "United States"],
  ["GB", "United Kingdom"],
  ["CA", "Canada"],
  ["AU", "Australia"],
  ["NZ", "New Zealand"],
  ["IE", "Ireland"],
  ["IN", "India"],
  ["PK", "Pakistan"],
  ["BD", "Bangladesh"],
  ["LK", "Sri Lanka"],
  ["AE", "United Arab Emirates"],
  ["ZA", "South Africa"],
  ["NG", "Nigeria"],
  ["DE", "Germany"],
  ["FR", "France"],
  ["ES", "Spain"],
  ["IT", "Italy"],
  ["PT", "Portugal"],
  ["NL", "Netherlands"],
  ["BR", "Brazil"],
  ["AR", "Argentina"],
  ["MX", "Mexico"],
  ["JP", "Japan"],
  ["KR", "South Korea"],
];

export const regionName = (code: string) => REGIONS.find(([c]) => c === code)?.[1] ?? code;

/** The saved region, else the one in the system language, else the US. */
export function defaultRegion(): string {
  const saved = localStorage.getItem("velmora.region");
  if (saved) return saved;
  const guess = navigator.language?.split("-")[1]?.toUpperCase();
  if (guess && REGIONS.some(([c]) => c === guess)) return guess;
  return "US";
}

const sameDay = (a: Date, b: Date) => a.toDateString() === b.toDateString();

export const clockTime = (ts: number) => new Date(ts * 1000).toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });

/** "Today", "Tomorrow" or "Sat 8 Nov", in the viewer's time zone. */
export function dayLabel(ts: number): string {
  const d = new Date(ts * 1000);
  const now = new Date();
  if (sameDay(d, now)) return "Today";
  const tomorrow = new Date(now.getTime() + 86_400_000);
  if (sameDay(d, tomorrow)) return "Tomorrow";
  return d.toLocaleDateString([], { weekday: "short", day: "numeric", month: "short" });
}

/** What the corner of a match card says. */
export function statusText(m: SportsMatch): string {
  switch (m.status) {
    case "live":
      return m.phase && m.phase !== "Live" ? m.phase : "LIVE";
    case "finished":
      return "Full time";
    case "postponed":
      return "Postponed";
    case "cancelled":
      return "Cancelled";
    default:
      return m.kickoff ? clockTime(m.kickoff) : "TBD";
  }
}

/** "Today · 8:30 PM", for the line under the teams. */
export const whenText = (m: SportsMatch) => (m.kickoff ? `${dayLabel(m.kickoff)} · ${clockTime(m.kickoff)}` : "Time to be confirmed");

export const hasScore = (m: SportsMatch) => m.home_score != null && m.away_score != null && m.status !== "upcoming";

export interface LeagueGroup {
  id: string;
  name: string;
  badge?: string;
  matches: SportsMatch[];
}

/** Groups matches by league, in the order each league first appears. */
export function groupByLeague(matches: SportsMatch[]): LeagueGroup[] {
  const groups = new Map<string, LeagueGroup>();
  for (const m of matches) {
    const key = m.league.id || m.league.name || m.sport;
    let g = groups.get(key);
    if (!g) {
      g = { id: key, name: m.league.name || m.sport, badge: m.league.badge, matches: [] };
      groups.set(key, g);
    }
    g.matches.push(m);
  }
  return [...groups.values()];
}
