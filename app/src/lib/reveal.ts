// Small pieces of motion used across the pages (Phase 4 of the motion work).
//
// reveal:    a row or card rises in the first time it scrolls into view, a
//            few at a time one after another, instead of all at once.
// fadeImage: artwork fades and settles in when it arrives, instead of
//            popping onto its grey placeholder.
// pop:       a quick spring "pop" when something turns on (Like).
// springScrollTo: scroll a box with a spring instead of the browser's
//            fixed smooth scroll (lyrics following the song).
//
// Everything respects the Motion setting: Reduced keeps the fades and drops
// the movement, Off shows things at once.
import { motion, springOf, type SpringName } from "./motion.svelte";

// ── reveal ──────────────────────────────────────────────────────────

/** Items found visible in the same frame stagger by this much. */
const STAGGER_MS = 45;
const MAX_STAGGER = 8;

let batch = 0;
let batchFrame = 0;

const observer =
  typeof IntersectionObserver !== "undefined"
    ? new IntersectionObserver(
        (entries) => {
          for (const e of entries) {
            if (!e.isIntersecting) continue;
            const el = e.target as HTMLElement;
            observer!.unobserve(el);
            show(el);
          }
        },
        { rootMargin: "0px 0px -6% 0px", threshold: 0.01 },
      )
    : null;

function show(el: HTMLElement) {
  // Things that come into view together go one after another.
  if (!batchFrame) {
    batchFrame = requestAnimationFrame(() => {
      batchFrame = 0;
      batch = 0;
    });
  }
  const order = Math.min(batch++, MAX_STAGGER);
  el.style.setProperty("--reveal-delay", `${order * STAGGER_MS}ms`);
  el.classList.add("revealed");
}

export function reveal(el: HTMLElement) {
  // A page you come back to is simply there, as you left it.
  if (motion.level === "off" || !observer || document.documentElement.dataset.vt === "back") return;
  el.classList.add("reveal");
  observer.observe(el);
  return {
    destroy() {
      observer.unobserve(el);
    },
  };
}

// ── fadeImage ───────────────────────────────────────────────────────

export function fadeImage(img: HTMLImageElement) {
  if (motion.level === "off") return;
  // Already decoded (from the cache): no fade, it's simply there.
  if (img.complete && img.naturalWidth > 0) return;
  img.classList.add("img-fade");
  const done = () => img.classList.add("img-in");
  img.addEventListener("load", done, { once: true });
  img.addEventListener("error", done, { once: true });
  return {
    destroy() {
      img.removeEventListener("load", done);
      img.removeEventListener("error", done);
    },
  };
}

// ── pop ─────────────────────────────────────────────────────────────

/** `use:pop={on}`: pops when `on` turns true (not when it starts true). */
export function pop(el: HTMLElement, on: boolean) {
  let last = on;
  return {
    update(next: boolean) {
      if (next && !last && motion.level === "full") {
        const s = springOf("fast-spatial");
        el.animate(
          [{ transform: "scale(1)" }, { transform: "scale(1.32)", offset: 0.35 }, { transform: "scale(1)" }],
          { duration: Math.round(s.duration * 1000) + 120, easing: "cubic-bezier(0.3, 0, 0.2, 1)" },
        );
      }
      last = next;
    },
  };
}

// ── springScrollTo ──────────────────────────────────────────────────

const scrolling = new WeakMap<HTMLElement, number>();

/** Scroll `box` to `top` on a spring. A new call takes over from a running one. */
export function springScrollTo(box: HTMLElement, top: number, name: SpringName = "slow-spatial") {
  const target = Math.max(0, Math.min(top, box.scrollHeight - box.clientHeight));
  const running = scrolling.get(box);
  if (running) cancelAnimationFrame(running);
  if (motion.level !== "full") {
    box.scrollTop = target;
    return;
  }
  const s = springOf(name);
  const from = box.scrollTop;
  const span = target - from;
  if (Math.abs(span) < 1) return;
  const start = performance.now();
  const ms = s.duration * 1000;
  const step = (now: number) => {
    const p = Math.min(1, (now - start) / ms);
    box.scrollTop = from + span * s.ease(p);
    if (p < 1) scrolling.set(box, requestAnimationFrame(step));
    else scrolling.delete(box);
  };
  scrolling.set(box, requestAnimationFrame(step));
}

/** Stop a spring scroll (the user scrolled the box themselves). */
export function stopSpringScroll(box: HTMLElement) {
  const running = scrolling.get(box);
  if (running) cancelAnimationFrame(running);
  scrolling.delete(box);
}
