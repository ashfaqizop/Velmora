// Pages on the social site, and moving between them without reloading.
//
//   /social/              home
//   /@name                a profile (also /@name/followers, /following, /visitors)
//   /search?q=            find people
//   /settings             edit your profile
//   /trending             what's popular (also /trending/shows, /music, /rated)
//   /leaderboards         most hours, reviews and followers
//   /title/movie/603      a title: Velmora ratings and reviews
//   /list/<id>            a list (/list/new to make one)
//
// On the real host those are real addresses (vercel.json rewrites them to the
// app). The dev server and the error fallback use /social/?at=/@name instead.

export type Tab = "activity" | "reviews" | "lists" | "stats" | "followers" | "following" | "visitors";
export type TrendTab = "movies" | "shows" | "music" | "rated";

export type Route =
  | { name: "home" }
  | { name: "profile"; user: string; tab: Tab }
  | { name: "search"; q: string }
  | { name: "settings" }
  | { name: "trending"; tab: TrendTab }
  | { name: "leaderboards" }
  | { name: "title"; k: string }
  | { name: "list"; id: string }
  | { name: "missing" };

const LOCAL = location.hostname === "localhost" || location.hostname === "127.0.0.1";
const TABS: Tab[] = ["reviews", "lists", "stats", "followers", "following", "visitors"];
const TRENDS: TrendTab[] = ["shows", "music", "rated"];

function current(): { path: string; query: URLSearchParams } {
  const at = new URLSearchParams(location.search).get("at");
  if (at && at.startsWith("/")) {
    const i = at.indexOf("?");
    return { path: i < 0 ? at : at.slice(0, i), query: new URLSearchParams(i < 0 ? "" : at.slice(i)) };
  }
  return { path: location.pathname, query: new URLSearchParams(location.search) };
}

function parse(): Route {
  const { path, query } = current();
  const clean = path.replace(/\/+$/, "") || "/";
  if (clean === "/" || clean === "/social" || clean === "/social/index.html") return { name: "home" };
  if (clean === "/search") return { name: "search", q: query.get("q") ?? "" };
  if (clean === "/settings") return { name: "settings" };
  if (clean === "/leaderboards") return { name: "leaderboards" };
  const tr = /^\/trending(?:\/([a-z]+))?$/.exec(clean);
  if (tr) return { name: "trending", tab: TRENDS.includes(tr[1] as TrendTab) ? (tr[1] as TrendTab) : "movies" };
  const ti = /^\/title\/((?:movie|tv)\/\d{1,9})$/.exec(clean);
  if (ti) return { name: "title", k: ti[1] };
  const li = /^\/list\/(new|[0-9a-f-]{36})$/.exec(clean);
  if (li) return { name: "list", id: li[1] };
  const m = /^\/@([A-Za-z0-9_]{3,20})(?:\/([a-z]+))?$/.exec(clean);
  if (m) {
    const tab = TABS.includes(m[2] as Tab) ? (m[2] as Tab) : "activity";
    return { name: "profile", user: m[1], tab };
  }
  return { name: "missing" };
}

export const nav = $state<{ route: Route }>({ route: parse() });

/** The address for a page, right for this host. */
export function href(path: string): string {
  if (path === "/" || path === "/social/") return "/social/";
  return LOCAL ? `/social/?at=${encodeURIComponent(path)}` : path;
}

export const profileHref = (username: string, tab?: Tab) => href(`/@${username}${tab && tab !== "activity" ? `/${tab}` : ""}`);
export const titleHref = (k: string) => href(`/title/${k}`);
export const listHref = (id: string) => href(`/list/${id}`);

/** The full public address of a profile, for sharing. */
export const profileUrl = (username: string) => `https://social.getvelmora.dpdns.org/@${username}`;

/** Go to a page. `replace` swaps the history entry instead of adding one. */
export function go(path: string, opts: { replace?: boolean; keepScroll?: boolean } = {}) {
  const url = href(path);
  if (opts.replace) history.replaceState(null, "", url);
  else history.pushState(null, "", url);
  nav.route = parse();
  if (!opts.keepScroll) window.scrollTo({ top: 0, behavior: "instant" as ScrollBehavior });
}

/** Follow clicks on our own links without a page load. */
export function startNav() {
  window.addEventListener("popstate", () => (nav.route = parse()));
  document.addEventListener("click", (e) => {
    if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    const a = e.composedPath().find((n): n is HTMLAnchorElement => n instanceof HTMLAnchorElement);
    if (!a || a.target === "_blank" || a.hasAttribute("download")) return;
    const url = new URL(a.href, location.href);
    if (url.origin !== location.origin) return;
    const inner = url.searchParams.get("at");
    const path = inner ?? `${url.pathname}${url.search}`;
    if (!/^\/(@|search|settings|trending|leaderboards|title\/|list\/|social\/?$|social\/\?)/.test(path) && path !== "/") return;
    e.preventDefault();
    go(path === "/social/" || path.startsWith("/social/?") ? "/social/" : path);
  });
}
