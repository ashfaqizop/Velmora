// Admin pages live behind the hash (#/users/<id>), so one static page serves
// all of them and nothing needs a server rewrite.
export type Page = "overview" | "analytics" | "curation" | "theatre" | "users" | "reports" | "reviews" | "names" | "max" | "notices" | "ads" | "config" | "audit";

export const PAGES: { id: Page; label: string; icon: string }[] = [
  { id: "overview", label: "Overview", icon: "dashboard" },
  { id: "analytics", label: "Analytics", icon: "insights" },
  { id: "theatre", label: "Theatre", icon: "theaters" },
  { id: "users", label: "People", icon: "group" },
  { id: "reports", label: "Reports", icon: "flag" },
  { id: "reviews", label: "Reviews", icon: "rate_review" },
  { id: "names", label: "Names & words", icon: "spellcheck" },
  { id: "curation", label: "Curation", icon: "auto_awesome" },
  { id: "max", label: "Max licences", icon: "workspace_premium" },
  { id: "notices", label: "Announcements", icon: "campaign" },
  { id: "ads", label: "App ads", icon: "ads_click" },
  { id: "config", label: "Remote config", icon: "tune" },
  { id: "audit", label: "Audit log", icon: "history" },
];

export const route = $state({ page: "overview" as Page, id: "" });

function read() {
  const [p = "", id = ""] = location.hash.replace(/^#\/?/, "").split("/");
  route.page = PAGES.some((x) => x.id === p) ? (p as Page) : "overview";
  route.id = /^[0-9a-f-]{36}$/.test(id) ? id : "";
}

export function startRoute() {
  read();
  addEventListener("hashchange", read);
}

export function go(page: Page, id = "") {
  location.hash = id ? `#/${page}/${id}` : `#/${page}`;
}
