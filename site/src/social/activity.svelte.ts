// Phase 3 data: activity and the feed, ratings and reviews, lists, stats,
// suggestions, leaderboards and trending. The rules live in supabase/activity.sql.
import { supabase } from "../lib/session";
import { auth, checkFollowing, friendly, loadMax, type Favorite, type Person } from "./data.svelte";
import { loadSongs, loadTitles } from "./meta.svelte";

export interface Activity {
  id: number;
  kind: "watching" | "finished" | "listed" | "rated" | "booked" | "attended";
  k: string;
  season: number | null;
  episode: number | null;
  minutes: number;
  plays: number;
  score: number | null;
  created_at: string;
  review: string | null;
  spoiler: boolean;
  /** Who did it (feed only; a profile's own list leaves it out). */
  user?: Pick<Person, "id" | "username" | "display_name" | "avatar" | "verified">;
}

const PAGE = 30;

function asActivity(row: Record<string, unknown>, withUser: boolean): Activity {
  const a = row as unknown as Activity & { user_id: string; username: string; display_name: string; avatar: string; verified: boolean };
  const out: Activity = {
    id: a.id,
    kind: a.kind,
    k: a.k,
    season: a.season,
    episode: a.episode,
    minutes: a.minutes,
    plays: a.plays,
    score: a.score,
    created_at: a.created_at,
    review: a.review,
    spoiler: a.spoiler,
  };
  if (withUser) out.user = { id: a.user_id, username: a.username, display_name: a.display_name, avatar: a.avatar, verified: a.verified };
  return out;
}

async function activityPage(rpc: string, args: Record<string, unknown>, withUser: boolean): Promise<{ items: Activity[]; more: boolean }> {
  const { data, error } = await supabase.rpc(rpc, { ...args, lim: PAGE });
  if (error) throw new Error(friendly(error, "Couldn't load activity."));
  const items = ((data ?? []) as Record<string, unknown>[]).map((r) => asActivity(r, withUser));
  await Promise.all([loadTitles(items.map((i) => i.k)), withUser ? loadMax(items.map((i) => i.user!.id)) : null]);
  return { items, more: items.length === PAGE };
}

export const myFeed = (before: string | null) => activityPage("my_feed", { before }, true);
export const userActivity = (target: string, before: string | null) => activityPage("user_activity", { target, before }, false);

// ── Sharing ───────────────────────────────────────────────────────────────

export async function setSharing(on: boolean) {
  const me = auth.me;
  if (!me) throw new Error("Sign in first.");
  const { error } = await supabase.from("profiles").update({ share_activity: on }).eq("id", me.id);
  if (error) throw new Error(friendly(error, "Couldn't change that. Try again."));
  me.share_activity = on;
}

export async function clearActivity() {
  const me = auth.me;
  if (!me) return;
  const { error } = await supabase.from("activity").delete().eq("user_id", me.id);
  if (error) throw new Error(friendly(error, "Couldn't clear your activity."));
}

// ── Ratings and reviews ───────────────────────────────────────────────────

export interface Rating {
  user_id: string;
  k: string;
  score: number;
  review: string;
  spoiler: boolean;
  created_at: string;
  updated_at: string;
  p?: Pick<Person, "id" | "username" | "display_name" | "avatar" | "verified"> | null;
}

export interface Score {
  average: number | null;
  ratings: number;
  reviews: number;
}

export async function titleScore(k: string): Promise<Score> {
  const { data } = await supabase.rpc("title_score", { title: k });
  const row = (Array.isArray(data) ? data[0] : data) as Score | undefined;
  return { average: row?.average != null ? Number(row.average) : null, ratings: row?.ratings ?? 0, reviews: row?.reviews ?? 0 };
}

export async function myRating(k: string): Promise<Rating | null> {
  const me = auth.me;
  if (!me) return null;
  const { data } = await supabase.from("ratings").select("*").eq("user_id", me.id).eq("k", k).maybeSingle();
  return (data as Rating) ?? null;
}

export async function rate(k: string, score: number, review: string, spoiler: boolean) {
  const me = auth.me;
  if (!me) throw new Error("Sign in to rate.");
  const { error } = await supabase.from("ratings").upsert({ user_id: me.id, k, score, review: review.trim().slice(0, 500), spoiler }, { onConflict: "user_id,k" });
  if (error) throw new Error(error.message.includes("rating_limit") ? "That's a lot of ratings for one day. Try again tomorrow." : friendly(error, "Couldn't save your rating."));
}

export async function unrate(k: string) {
  const me = auth.me;
  if (!me) return;
  const { error } = await supabase.from("ratings").delete().eq("user_id", me.id).eq("k", k);
  if (error) throw new Error(friendly(error, "Couldn't remove your rating."));
}

const WHO = "id,username,display_name,avatar,verified";

export async function titleReviews(k: string, from: number): Promise<{ items: Rating[]; more: boolean }> {
  const { data, error } = await supabase
    .from("ratings")
    .select(`*, p:profiles!ratings_user_id_fkey(${WHO})`)
    .eq("k", k)
    .neq("review", "")
    .order("updated_at", { ascending: false })
    .range(from, from + 19);
  if (error) throw new Error(friendly(error, "Couldn't load reviews."));
  const items = ((data ?? []) as Rating[]).filter((r) => r.p);
  await loadMax(items.map((r) => r.user_id));
  return { items, more: (data ?? []).length === 20 };
}

export async function userRatings(userId: string, from: number): Promise<{ items: Rating[]; more: boolean }> {
  const { data, error } = await supabase.from("ratings").select("*").eq("user_id", userId).order("updated_at", { ascending: false }).range(from, from + 29);
  if (error) throw new Error(friendly(error, "Couldn't load ratings."));
  const items = (data ?? []) as Rating[];
  await loadTitles(items.map((r) => r.k));
  return { items, more: items.length === 30 };
}

// ── Lists ─────────────────────────────────────────────────────────────────

export interface TitleList {
  id: string;
  user_id: string;
  name: string;
  description: string;
  items: Favorite[];
  public: boolean;
  created_at: string;
  updated_at: string;
  p?: Pick<Person, "id" | "username" | "display_name" | "avatar" | "verified"> | null;
}

export async function userLists(userId: string): Promise<TitleList[]> {
  const { data, error } = await supabase.from("lists").select("*").eq("user_id", userId).order("updated_at", { ascending: false });
  if (error) throw new Error(friendly(error, "Couldn't load lists."));
  return (data ?? []) as TitleList[];
}

export async function getList(id: string): Promise<TitleList | null> {
  if (!/^[0-9a-f-]{36}$/.test(id)) return null;
  const { data, error } = await supabase.from("lists").select(`*, p:profiles!lists_user_id_fkey(${WHO})`).eq("id", id).maybeSingle();
  if (error) throw new Error(friendly(error, "Couldn't load that list."));
  return (data as TitleList) ?? null;
}

export async function saveList(list: Pick<TitleList, "name" | "description" | "items" | "public"> & { id?: string }): Promise<string> {
  const me = auth.me;
  if (!me) throw new Error("Sign in first.");
  const row = { name: list.name.trim(), description: list.description.trim(), items: list.items, public: list.public };
  const q = list.id
    ? supabase.from("lists").update(row).eq("id", list.id).select("id").single()
    : supabase.from("lists").insert({ ...row, user_id: me.id }).select("id").single();
  const { data, error } = await q;
  if (error) throw new Error(error.message.includes("list_limit") ? "50 lists is the most. Delete one first." : friendly(error, "Couldn't save the list."));
  return (data as { id: string }).id;
}

export async function deleteList(id: string) {
  const { error } = await supabase.from("lists").delete().eq("id", id);
  if (error) throw new Error(friendly(error, "Couldn't delete the list."));
}

/** Add one title to one of my lists (from a title page). */
export async function addToList(list: TitleList, item: Favorite) {
  if (list.items.some((i) => i.k === item.k)) return;
  if (list.items.length >= 100) throw new Error("A list holds 100 titles at most.");
  await saveList({ ...list, items: [...list.items, item] });
  list.items = [...list.items, item];
}

// ── Stats ─────────────────────────────────────────────────────────────────

export interface Stats {
  minutes: number;
  minutes_month: number;
  movies: number;
  shows: number;
  episodes: number;
  ratings: number;
  reviews: number;
  average: number | null;
  top: string[];
}

export async function profileStats(id: string): Promise<Stats | null> {
  const { data, error } = await supabase.rpc("profile_stats", { target: id });
  if (error) throw new Error(friendly(error, "Couldn't load stats."));
  if (!data) return null;
  const s = data as Stats;
  await loadTitles(s.top ?? []);
  return { ...s, average: s.average != null ? Number(s.average) : null, top: s.top ?? [] };
}

// ── People ────────────────────────────────────────────────────────────────

export async function suggestedPeople(): Promise<(Person & { mutual?: number })[]> {
  if (!auth.me) return [];
  const { data } = await supabase.rpc("suggested_people", { lim: 8 });
  const list = (data ?? []) as (Person & { mutual: number })[];
  await Promise.all([checkFollowing(list.map((p) => p.id)), loadMax(list.map((p) => p.id))]);
  return list;
}

export type Board = "hours" | "reviews" | "followers";

export async function leaderboard(board: Board): Promise<(Person & { value: number })[]> {
  const { data, error } = await supabase.rpc("leaderboard", { board, lim: 30 });
  if (error) throw new Error(friendly(error, "Couldn't load the leaderboard."));
  const list = ((data ?? []) as (Person & { value: number })[]).map((p) => ({ ...p, bio: p.bio ?? "" }));
  await Promise.all([checkFollowing(list.map((p) => p.id)), loadMax(list.map((p) => p.id))]);
  return list;
}

// ── Trending ──────────────────────────────────────────────────────────────

export interface Trend {
  k: string;
  people: number;
}

const trendCache = new Map<string, Promise<{ people: number; titles: Trend[] }>>();

function trending(kind: "titles" | "music") {
  if (!trendCache.has(kind)) {
    trendCache.set(
      kind,
      fetch(`/api/stats/trending?days=7&limit=50${kind === "music" ? "&kind=music" : ""}`)
        .then((r) => r.json())
        .then((j) => ({ people: j.people ?? 0, titles: (j.titles ?? []) as Trend[] }))
        .catch(() => {
          trendCache.delete(kind);
          return { people: 0, titles: [] };
        }),
    );
  }
  return trendCache.get(kind)!;
}

export async function trendingTitles(kind: "movie" | "tv"): Promise<Trend[]> {
  const all = (await trending("titles")).titles.filter((t) => t.k.startsWith(`${kind}/`));
  await loadTitles(all.map((t) => t.k));
  return all;
}

export async function trendingSongs(): Promise<Trend[]> {
  const all = (await trending("music")).titles;
  await loadSongs(all.map((t) => t.k));
  return all;
}

export async function topRated(): Promise<{ k: string; average: number; ratings: number }[]> {
  const { data, error } = await supabase.rpc("top_rated", { lim: 30, least_ratings: 3 });
  if (error) throw new Error(friendly(error, "Couldn't load top rated."));
  const list = ((data ?? []) as { k: string; average: number; ratings: number }[]).map((r) => ({ ...r, average: Number(r.average) }));
  await loadTitles(list.map((r) => r.k));
  return list;
}
