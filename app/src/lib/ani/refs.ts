// Short references for things Ani found ("t3" for a title, "l2" for a link,
// "p1" for a person). Tools hand Gemini the ref instead of the full object,
// and generative UI blocks name refs, so every poster, link and button in a
// reply comes from real data rather than from the model's memory.
import type { Source, Target, TitleSummary } from "../types";

export interface PersonRef {
  name: string;
  photo?: string;
  role?: string;
  tmdbId?: number;
  anilistId?: number;
  /** Filled in by person_credits. */
  born?: string;
  died?: string;
  birthplace?: string;
  imdbId?: string;
}

export type Entity =
  | { type: "title"; title: TitleSummary }
  | { type: "link"; source: Source; title: TitleSummary; target: Target }
  | { type: "person"; person: PersonRef };

export interface RefData {
  next: { t: number; l: number; p: number };
  items: Record<string, Entity>;
}

const titleKey = (t: TitleSummary) => `${t.origin.provider}:${t.origin.id}`;

export class RefBook {
  private data: RefData;
  /** title key → ref, so the same title found twice keeps one ref. */
  private byTitle = new Map<string, string>();

  constructor(data?: RefData) {
    this.data = data ?? { next: { t: 0, l: 0, p: 0 }, items: {} };
    for (const [ref, e] of Object.entries(this.data.items)) {
      if (e.type === "title") this.byTitle.set(titleKey(e.title), ref);
    }
  }

  toJSON(): RefData {
    return this.data;
  }

  get(ref: string): Entity | undefined {
    return this.data.items[ref];
  }

  title(ref: string): TitleSummary | undefined {
    const e = this.get(ref);
    return e?.type === "title" || e?.type === "link" ? e.title : undefined;
  }

  addTitle(t: TitleSummary): string {
    const k = titleKey(t);
    const known = this.byTitle.get(k);
    if (known) {
      // Keep the richer copy (artwork, rating) when the same title turns up again.
      const old = this.data.items[known] as { type: "title"; title: TitleSummary };
      if ((!old.title.poster && t.poster) || (!old.title.overview && t.overview)) old.title = { ...old.title, ...t };
      return known;
    }
    const ref = `t${++this.data.next.t}`;
    this.data.items[ref] = { type: "title", title: t };
    this.byTitle.set(k, ref);
    return ref;
  }

  addLink(source: Source, title: TitleSummary, target: Target): string {
    const ref = `l${++this.data.next.l}`;
    this.data.items[ref] = { type: "link", source, title, target };
    return ref;
  }

  addPerson(person: PersonRef): string {
    const same = Object.entries(this.data.items).find(
      ([, e]) => e.type === "person" && e.person.name === person.name && e.person.tmdbId === person.tmdbId,
    );
    if (same) return same[0];
    const ref = `p${++this.data.next.p}`;
    this.data.items[ref] = { type: "person", person };
    return ref;
  }
}
