// Small text helpers for the social pages.

const rtf = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });
const STEPS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 31536000],
  ["month", 2592000],
  ["week", 604800],
  ["day", 86400],
  ["hour", 3600],
  ["minute", 60],
];

/** "2 days ago", "yesterday", "just now". */
export function ago(iso: string): string {
  const s = (new Date(iso).getTime() - Date.now()) / 1000;
  for (const [unit, size] of STEPS) if (Math.abs(s) >= size) return rtf.format(Math.round(s / size), unit);
  return "just now";
}

const compact = new Intl.NumberFormat(undefined, { notation: "compact", maximumFractionDigits: 1 });

/** 1234 -> "1.2K". */
export const count = (n: number) => (n < 10000 ? n.toLocaleString() : compact.format(n));

export const plural = (n: number, one: string, many = `${one}s`) => (n === 1 ? one : many);

export const monthYear = (iso: string) => new Date(iso).toLocaleDateString(undefined, { month: "long", year: "numeric" });
