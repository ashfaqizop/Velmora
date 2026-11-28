// Which platform a page should open on: the one you're browsing from, unless
// the address asks for one (?for=android). Shared by Download and the docs.

export type Platform = "windows" | "android";

export const PLATFORMS: Record<Platform, { name: string; icon: string }> = {
  windows: { name: "Windows", icon: "desktop_windows" },
  android: { name: "Android", icon: "android" },
};

/** The device's own platform, when we can tell. */
export function detectPlatform(): Platform | null {
  if (typeof navigator === "undefined") return null;
  const ua = navigator.userAgent;
  return /android/i.test(ua) ? "android" : /windows/i.test(ua) ? "windows" : null;
}

/** ?for= first, then the device, then Windows. */
export function initialPlatform(): Platform {
  if (typeof location !== "undefined") {
    const want = new URLSearchParams(location.search).get("for");
    if (want === "android" || want === "windows") return want;
  }
  return detectPlatform() ?? "windows";
}

/** Remember the choice in the address, so the link shares the same view. */
export function rememberPlatform(p: Platform) {
  const url = new URL(location.href);
  url.searchParams.set("for", p);
  history.replaceState(null, "", url);
}
