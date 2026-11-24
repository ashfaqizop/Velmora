// Everything the social site reads and writes. All of it goes straight to
// Supabase with the signed-in person's own session; the rules in
// supabase/social.sql decide what each person may see and change.
import { SvelteMap, SvelteSet } from "svelte/reactivity";
import { supabase, restoreSession } from "../lib/session";
import { isPhotoUrl } from "../auth/avatars";
import type { ProfileLink } from "../lib/links";

export interface Favorite {
  k: string;
  name: string;
  year?: number | null;
  poster?: string | null;
}

export interface Profile {
  id: string;
  username: string;
  display_name: string;
  avatar: string;
  bio: string;
  links: ProfileLink[];
  favorites: Favorite[];
  verified: boolean;
  followers_count: number;
  following_count: number;
  username_changed_at: string | null;
  created_at: string;
}

/** The short form used in lists. */
export type Person = Pick<Profile, "id" | "username" | "display_name" | "avatar" | "bio" | "verified" | "followers_count">;

const PERSON = "id,username,display_name,avatar,bio,verified,followers_count";
const PAGE = 24;

// ── Who is signed in ──────────────────────────────────────────────────────

export const auth = $state<{
  phase: "loading" | "out" | "in" | "noprofile";
  me: Profile | null;
  /** The Google or Discord photo, offered as a profile picture. */
  photo: string | null;
  problem: string | null;
}>({ phase: "loading", me: null, photo: null, problem: null });

export const myId = () => auth.me?.id ?? null;

export async function boot() {
  try {
    const session = await restoreSession();
    if (!session) {
      auth.phase = "out";
      return;
    }
    const meta = (session.user.user_metadata ?? {}) as Record<string, unknown>;
    const pic = meta.avatar_url ?? meta.picture;
    auth.photo = isPhotoUrl(pic) ? pic : null;
    const { data, error } = await supabase.from("profiles").select("*").eq("id", session.user.id).maybeSingle();
    if (error) throw new Error(error.message);
    if (!data) {
      auth.phase = "noprofile";
      return;
    }
    auth.me = normalise(data);
    auth.phase = "in";
  } catch (e) {
    auth.problem = (e as Error).message;
    auth.phase = "out";
  }
}

function normalise(row: Record<string, unknown>): Profile {
  const p = row as unknown as Profile;
  return { ...p, bio: p.bio ?? "", links: Array.isArray(p.links) ? p.links : [], favorites: Array.isArray(p.favorites) ? p.favorites : [] };
}

// ── Readable errors ───────────────────────────────────────────────────────

export function friendly(e: { message?: string; code?: string } | null | undefined, fallback = "Something went wrong. Try again."): string {
  const m = e?.message ?? "";
  if (m.includes("follow_limit")) return "That's a lot of follows in one go. Take a break and try again in a while.";
  if (m.includes("follow_max")) return "You follow 5,000 people, the most there is room for.";
  if (m.includes("follow_unavailable")) return "That profile isn't available.";
  if (m.includes("report_limit")) return "You've sent a lot of reports today. Try again tomorrow.";
  if (m.includes("username_cooldown")) return "You changed your username in the last 30 days. Try again later.";
  if (m.includes("username_reserved")) return "That username isn't available.";
  if (m.includes("profiles_bio_len")) return "Your bio is too long.";
  if (m.includes("profiles_links_ok")) return "One of your links isn't right.";
  if (m.includes("profiles_favorites_ok")) return "One of your favourites couldn't be saved.";
  if (m.includes("profiles_avatar_ok")) return "That picture can't be used.";
  if (e?.code === "23505") return m.includes("username") ? "Someone already has that username." : "Already done.";
  if (/fetch|network/i.test(m)) return "You seem to be offline.";
  return fallback;
}

// ── Profiles ──────────────────────────────────────────────────────────────

export async function profileByName(username: string): Promise<Profile | null> {
  const { data, error } = await supabase.from("profiles").select("*").eq("username_key", username.toLowerCase()).maybeSingle();
  if (error) throw new Error(friendly(error, "Couldn't load that profile."));
  return data ? normalise(data) : null;
}

export async function visitsThisMonth(id: string): Promise<number> {
  const { data } = await supabase.rpc("visits_this_month", { target: id });
  return typeof data === "number" ? data : 0;
}

/** Count a visit. Visitors who aren't signed in count once a day per browser. */
export async function recordVisit(id: string) {
  if (id === myId()) return;
  if (!auth.me) {
    const key = `velmora.visit.${id}.${new Date().toISOString().slice(0, 10)}`;
    try {
      if (localStorage.getItem(key)) return;
      localStorage.setItem(key, "1");
    } catch {
      // Storage blocked: count it anyway.
    }
  }
  await supabase.rpc("record_visit", { target: id });
}

export async function saveProfile(patch: Partial<Pick<Profile, "display_name" | "username" | "avatar" | "bio" | "links" | "favorites">>) {
  const me = auth.me;
  if (!me) throw new Error("Sign in first.");
  const { data, error } = await supabase.from("profiles").update(patch).eq("id", me.id).select("*").single();
  if (error) throw new Error(friendly(error, `Couldn't save. (${error.code ?? "error"})`));
  auth.me = normalise(data);
}

export async function usernameFree(name: string): Promise<boolean | null> {
  const { data, error } = await supabase.rpc("username_available", { candidate: name });
  return error ? null : data === true;
}

// ── Lists of people ───────────────────────────────────────────────────────

type Rows = { p: Person | null; at: string }[];

async function people(
  query: PromiseLike<{ data: unknown; error: { message: string } | null }>,
): Promise<{ people: (Person & { at: string })[]; more: boolean }> {
  const { data, error } = await query;
  if (error) throw new Error(friendly(error, "Couldn't load the list."));
  const rows = (data ?? []) as Rows;
  const list = rows.filter((r) => r.p).map((r) => ({ ...(r.p as Person), at: r.at }));
  await Promise.all([checkFollowing(list.map((x) => x.id)), loadMax(list.map((x) => x.id))]);
  return { people: list, more: rows.length === PAGE };
}

export const followersOf = (id: string, from: number) =>
  people(
    supabase
      .from("follows")
      .select(`at:created_at, p:profiles!follows_follower_fkey(${PERSON})`)
      .eq("followee", id)
      .order("created_at", { ascending: false })
      .range(from, from + PAGE - 1),
  );

export const followingOf = (id: string, from: number) =>
  people(
    supabase
      .from("follows")
      .select(`at:created_at, p:profiles!follows_followee_fkey(${PERSON})`)
      .eq("follower", id)
      .order("created_at", { ascending: false })
      .range(from, from + PAGE - 1),
  );

/** Who visited your profile. Only you can read this (the database says so). */
export const visitorsOf = (id: string, from: number) =>
  people(
    supabase
      .from("profile_visitors")
      .select(`at:last_at, p:profiles!profile_visitors_visitor_id_fkey(${PERSON})`)
      .eq("profile_id", id)
      .order("last_at", { ascending: false })
      .range(from, from + PAGE - 1),
  );

export async function searchPeople(q: string): Promise<Person[]> {
  const { data, error } = await supabase.rpc("search_profiles", { q, lim: 30 });
  if (error) throw new Error(friendly(error, "Search isn't working right now."));
  const list = (data ?? []) as Person[];
  await Promise.all([checkFollowing(list.map((x) => x.id)), loadMax(list.map((x) => x.id))]);
  return list;
}

async function personList(order: "followers_count" | "created_at", n: number): Promise<Person[]> {
  const { data, error } = await supabase.from("profiles").select(PERSON).order(order, { ascending: false }).limit(n);
  if (error) throw new Error(friendly(error, "Couldn't load people."));
  const list = (data ?? []) as Person[];
  await Promise.all([checkFollowing(list.map((x) => x.id)), loadMax(list.map((x) => x.id))]);
  return list;
}

export const mostFollowed = () => personList("followers_count", 12);
export const newestPeople = () => personList("created_at", 12);

// ── Following ─────────────────────────────────────────────────────────────

/** Whether I follow each person I've seen. Missing means not checked yet. */
export const following = new SvelteMap<string, boolean>();
/** People who follow me, among those checked. */
export const followsMe = new SvelteSet<string>();

export async function checkFollowing(ids: string[]) {
  const me = myId();
  if (!me) return;
  const todo = [...new Set(ids)].filter((id) => id !== me && !following.has(id));
  if (!todo.length) return;
  const { data } = await supabase.from("follows").select("followee").eq("follower", me).in("followee", todo);
  const yes = new Set((data ?? []).map((r: { followee: string }) => r.followee));
  for (const id of todo) following.set(id, yes.has(id));
}

/** Does this person follow me? For the "Follows you" label. */
export async function checkFollowsMe(id: string) {
  const me = myId();
  if (!me || id === me) return;
  const { data } = await supabase.from("follows").select("follower").eq("follower", id).eq("followee", me).maybeSingle();
  if (data) followsMe.add(id);
  else followsMe.delete(id);
}

/** Follow or unfollow. Shows the change at once and undoes it if the server says no. */
export async function setFollow(target: string, on: boolean): Promise<void> {
  const me = auth.me;
  if (!me) throw new Error("Sign in to follow people.");
  const before = following.get(target) ?? false;
  if (before === on) return;
  following.set(target, on);
  me.following_count = Math.max(0, me.following_count + (on ? 1 : -1));
  const { error } = on
    ? await supabase.from("follows").insert({ follower: me.id, followee: target })
    : await supabase.from("follows").delete().eq("follower", me.id).eq("followee", target);
  if (error && !(on && error.code === "23505")) {
    following.set(target, before);
    me.following_count = Math.max(0, me.following_count + (on ? -1 : 1));
    throw new Error(friendly(error, on ? "Couldn't follow. Try again." : "Couldn't unfollow. Try again."));
  }
  followDelta.set(target, (followDelta.get(target) ?? 0) + (on ? 1 : -1));
}

/** Follower changes made on this page, so counts on screen stay right without a reload. */
export const followDelta = new SvelteMap<string, number>();

// ── Badges ────────────────────────────────────────────────────────────────

/** Accounts with Velmora Max. */
export const maxIds = new SvelteSet<string>();
const maxAsked = new Set<string>();

export async function loadMax(ids: string[]) {
  const todo = [...new Set(ids)].filter((id) => !maxAsked.has(id));
  if (!todo.length) return;
  todo.forEach((id) => maxAsked.add(id));
  try {
    const r = await fetch(`/api/social/max?ids=${todo.join(",")}`);
    const j = (await r.json()) as { max?: string[] };
    for (const id of j.max ?? []) maxIds.add(id);
  } catch {
    // No badge is fine.
  }
}

// ── Reports ───────────────────────────────────────────────────────────────

export type ReportReason = "username" | "display_name" | "bio" | "links" | "avatar" | "spam" | "other";

export async function report(target: string, reason: ReportReason, note: string) {
  const me = auth.me;
  if (!me) throw new Error("Sign in to report a profile.");
  const { error } = await supabase.from("reports").insert({ reporter: me.id, target, reason, note: note.trim().slice(0, 500) });
  if (error && error.code !== "23505") throw new Error(friendly(error, "Couldn't send the report. Try again."));
}

// ── Favourites: picked from your My List ──────────────────────────────────

const POSTER = /^https:\/\/image\.tmdb\.org\/t\/p\/[a-z0-9]{2,10}\/[A-Za-z0-9_.-]{1,80}$/;

interface ListTitle {
  origin?: { provider?: string; id?: string };
  name?: string;
  kind?: string;
  year?: number;
  poster?: string;
  ids?: { tmdb?: number | string };
}

function favoriteOf(t: ListTitle | null): Favorite | null {
  if (!t?.name) return null;
  let k: string | null = null;
  if (t.origin?.provider === "tmdb" && /^(movie|tv)\/\d{1,9}$/.test(t.origin.id ?? "")) k = t.origin.id!;
  else if (t.ids?.tmdb && /^\d{1,9}$/.test(String(t.ids.tmdb))) k = `${t.kind === "movie" ? "movie" : "tv"}/${t.ids.tmdb}`;
  if (!k) return null;
  // Smaller posters load faster on the profile.
  const poster = t.poster ? t.poster.replace(/\/t\/p\/[a-z0-9]+\//, "/t/p/w342/") : null;
  return { k, name: t.name.slice(0, 200), year: typeof t.year === "number" ? t.year : null, poster: poster && POSTER.test(poster) ? poster : null };
}

/** Titles from My List (synced from the app) that can be shown as favourites. */
export async function myListFavorites(): Promise<Favorite[]> {
  const me = myId();
  if (!me) return [];
  const { data, error } = await supabase
    .from("list_items")
    .select("title")
    .eq("user_id", me)
    .eq("deleted", false)
    .order("updated_at", { ascending: false })
    .limit(200);
  if (error) throw new Error(friendly(error, "Couldn't load My List."));
  const seen = new Set<string>();
  const out: Favorite[] = [];
  for (const row of data ?? []) {
    const f = favoriteOf((row as { title: ListTitle | null }).title);
    if (f && !seen.has(f.k)) {
      seen.add(f.k);
      out.push(f);
    }
  }
  return out;
}
