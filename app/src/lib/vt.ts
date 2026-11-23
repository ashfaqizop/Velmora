// Page changes as view transitions.
//
// The browser photographs the page, the update runs, and the two pictures
// animate into each other (the CSS is in app.css, under "View transitions").
// The direction is set on <html data-vt="..."> so the CSS knows which way
// to slide. A "shared" element (a poster and the page it opens, the mini
// player's cover and the full player's) gets the same view-transition-name
// on both sides, so the browser moves and resizes it from one to the other.
//
// Names go on just before each picture and come off right after, so only one
// element ever holds a name at a time. When motion is off, or the WebView
// has no view transitions, the update just runs.
import { flushSync } from "svelte";
import { motion } from "./motion.svelte";

/** forward: deeper. back: the way you came. switch: between sidebar pages.
 *  search: into search results. open/close: the full music player. */
export type VtKind = "forward" | "back" | "switch" | "search" | "open" | "close";

export interface Shared {
  name: string;
  /** The element on the page now. */
  from?: () => Element | null | undefined;
  /** The element after the update. Waited for briefly (pages that load). */
  to?: () => Element | null | undefined;
}

type StartVT = (cb: () => Promise<void> | void) => {
  finished: Promise<void>;
  ready: Promise<void>;
  updateCallbackDone: Promise<void>;
  skipTransition(): void;
};

const named = new Set<HTMLElement>();
let running: ReturnType<StartVT> | null = null;

function setName(el: Element | null | undefined, name: string) {
  if (!(el instanceof HTMLElement)) return;
  el.style.viewTransitionName = name;
  named.add(el);
}

function clearNames() {
  for (const el of named) el.style.viewTransitionName = "";
  named.clear();
}

/** The element, as soon as it's there, for up to `ms`. */
function waitFor(find: () => Element | null | undefined, ms: number): Promise<Element | null> {
  const first = find();
  if (first) return Promise.resolve(first);
  return new Promise((resolve) => {
    const until = performance.now() + ms;
    const look = () => {
      const el = find();
      if (el || performance.now() > until) resolve(el ?? null);
      else requestAnimationFrame(look);
    };
    requestAnimationFrame(look);
  });
}

/** On screen at all (a shared element that's scrolled away shouldn't fly in from nowhere). */
function visible(el: Element | null | undefined): el is Element {
  if (!el) return false;
  const r = el.getBoundingClientRect();
  return r.width > 0 && r.height > 0 && r.bottom > 0 && r.right > 0 && r.top < innerHeight && r.left < innerWidth;
}

export function supported(): boolean {
  return typeof document !== "undefined" && "startViewTransition" in document;
}

/**
 * Run `update` as a `kind` transition. `after` runs once the new page is in
 * the DOM, before its picture is taken (restore a scroll position there).
 */
export function transition(kind: VtKind, update: () => void, opts: { shared?: Shared[]; after?: () => void } = {}) {
  const start = (document as Document & { startViewTransition?: StartVT }).startViewTransition?.bind(document);
  if (!start || motion.level === "off" || document.visibilityState !== "visible") {
    update();
    flushSync();
    opts.after?.();
    return;
  }
  running?.skipTransition();
  clearNames();
  const root = document.documentElement;
  for (const s of opts.shared ?? []) {
    const el = s.from?.();
    if (visible(el)) setName(el, s.name);
  }
  root.dataset.vt = kind;
  motion.vt = true;
  const t = start(async () => {
    clearNames();
    update();
    flushSync();
    opts.after?.();
    // Shared elements on the new page; a page that loads gets a moment.
    await Promise.all(
      (opts.shared ?? []).map(async (s) => {
        if (!s.to) return;
        const el = await waitFor(s.to, 320);
        if (visible(el)) setName(el, s.name);
      }),
    );
  });
  running = t;
  const done = () => {
    if (running !== t) return;
    running = null;
    motion.vt = false;
    delete root.dataset.vt;
    clearNames();
  };
  t.finished.then(done, done);
  t.ready.catch(() => {});
  t.updateCallbackDone.catch(() => {});
}
