// What you watch, shared on Velmora Social: started, finished, added to My
// List. Only when you're signed in and sharing is on (Settings > Privacy, or
// Edit profile on the web). 18+ titles and anything without a TMDB number are
// never sent. Posts wait on this PC while offline and go out later; the server
// side is post_activity in supabase/activity.sql.
import { account } from "./account.svelte";
import { titleKey } from "./analytics.svelte";
import type { Target, TitleSummary } from "./types";

const QUEUE = "velmora.activity.queue";
const KEEP_MS = 3 * 86_400_000;

type What = "watching" | "finished" | "listed";
interface Post {
  what: What;
  k: string;
  season: number | null;
  episode: number | null;
  minutes: number;
  at: number;
  /** Whose it is, so a post never goes out under someone else's account. */
  uid: string;
}

function read(): Post[] {
  try {
    const v = JSON.parse(localStorage.getItem(QUEUE) ?? "[]");
    return Array.isArray(v) ? v : [];
  } catch {
    return [];
  }
}

function write(q: Post[]) {
  try {
    localStorage.setItem(QUEUE, JSON.stringify(q.slice(-100)));
  } catch {
    // Lost at worst.
  }
}

/** The TMDB key for a title we may share, or null. */
function shareable(t: TitleSummary): string | null {
  if (!account.signedIn || account.profile?.share_activity === false) return null;
  if (t.badges?.includes("18+")) return null;
  return titleKey(t);
}

class ActivityPoster {
  private flushing = false;

  start() {
    setTimeout(() => this.flush(), 15_000);
    setInterval(() => this.flush(), 5 * 60_000);
    window.addEventListener("online", () => this.flush());
  }

  private add(what: What, t: TitleSummary, target?: Target, minutes = 0) {
    const k = shareable(t);
    if (!k) return;
    const ep = target?.type === "episode" ? target : null;
    write([...read(), { what, k, season: ep?.season ?? null, episode: ep?.episode ?? null, minutes: Math.round(minutes), at: Date.now(), uid: account.profile!.id }]);
    this.flush();
  }

  /** A minute into something. */
  watching(t: TitleSummary, target: Target) {
    this.add("watching", t, target);
  }

  /** Watched to the end (95%). */
  finished(t: TitleSummary, target: Target, durationSeconds: number) {
    this.add("finished", t, target, Math.min(durationSeconds / 60, 600));
  }

  /** Added to My List. */
  listed(t: TitleSummary) {
    this.add("listed", t);
  }

  async flush() {
    if (this.flushing || !navigator.onLine || !account.signedIn || account.offline) return;
    const me = account.profile?.id;
    let queue = read().filter((p) => p.uid === me && Date.now() - p.at < KEEP_MS);
    if (!queue.length) return;
    this.flushing = true;
    try {
      while (queue.length) {
        const p = queue[0];
        const { error } = await account.client.rpc("post_activity", {
          what: p.what,
          title: p.k,
          season_no: p.season,
          episode_no: p.episode,
          watched_minutes: p.minutes,
        });
        // A network problem keeps the rest for later; anything else drops this one.
        if (error && /fetch|network|timeout/i.test(error.message)) break;
        queue = queue.slice(1);
        write(queue);
      }
    } catch {
      // Try again later.
    } finally {
      this.flushing = false;
    }
  }

  /** Signing out or turning sharing off forgets what hasn't gone out yet. */
  clear() {
    write([]);
  }
}

export const activity = new ActivityPoster();
