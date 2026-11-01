// The newest Velmora release, straight from GitHub, so the download button
// never needs editing when a version ships. Cached for the visit.

export const REPO = "ashfaqizop/Velmora";
export const REPO_URL = `https://github.com/${REPO}`;
export const RELEASES_URL = `${REPO_URL}/releases`;

export interface Release {
  version: string;
  name: string;
  date: string;
  url: string;
  notes: string;
  installer?: { url: string; size: number; name: string; downloads: number };
}

const CACHE = "velmora.site.releases.v2";

/** App releases, newest first (the runtime release and drafts are skipped). */
export async function releases(): Promise<Release[]> {
  const cached = sessionStorage.getItem(CACHE);
  if (cached) return JSON.parse(cached) as Release[];
  const res = await fetch(`https://api.github.com/repos/${REPO}/releases?per_page=50`, {
    headers: { Accept: "application/vnd.github+json" },
  });
  if (!res.ok) throw new Error(`GitHub answered ${res.status}`);
  const raw = (await res.json()) as {
    tag_name: string;
    name: string | null;
    published_at: string;
    html_url: string;
    body: string | null;
    draft: boolean;
    prerelease: boolean;
    assets: { name: string; size: number; browser_download_url: string; download_count: number }[];
  }[];
  const list = raw
    .filter((r) => !r.draft && !r.prerelease && r.tag_name.startsWith("app-v"))
    .map((r): Release => {
      const exe = r.assets.find((a) => /x64-setup\.exe$/i.test(a.name));
      return {
        version: r.tag_name.replace(/^app-v/, ""),
        name: r.name || r.tag_name,
        date: r.published_at,
        url: r.html_url,
        notes: r.body ?? "",
        installer: exe ? { url: exe.browser_download_url, size: exe.size, name: exe.name, downloads: exe.download_count } : undefined,
      };
    });
  sessionStorage.setItem(CACHE, JSON.stringify(list));
  return list;
}

export const megabytes = (bytes: number) => `${(bytes / 1_048_576).toFixed(1)} MB`;

export const shortDate = (iso: string) =>
  new Date(iso).toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" });

/** "1,234" style, in the visitor's locale. */
export const formatCount = (n: number) => n.toLocaleString();
