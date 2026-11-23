// A highlight that slides to the selected item, instead of one item's
// background switching off and another's on.
//
// `use:indicator={{ selector }}` on a list: a pill is put behind the items
// and moved (with the default spatial spring) to whichever item matches
// `selector`. It follows clicks, items appearing or reordering, and resizes.
// The items' own selected background should be made transparent.
import { motion } from "./motion.svelte";

export interface IndicatorOptions {
  /** Matches the selected item, e.g. "[selected]" or ".on". */
  selector: string;
  /** CSS colour; defaults to the secondary container. */
  color?: string;
  radius?: string;
}

export function indicator(list: HTMLElement, opts: IndicatorOptions) {
  let options = opts;
  const pill = document.createElement("div");
  pill.setAttribute("aria-hidden", "true");
  pill.className = "slide-indicator";
  Object.assign(pill.style, {
    position: "absolute",
    left: "0",
    top: "0",
    width: "0",
    height: "0",
    zIndex: "-1",
    pointerEvents: "none",
    opacity: "0",
    borderRadius: options.radius ?? "9999px",
    background: options.color ?? "var(--md-sys-color-secondary-container)",
  } satisfies Partial<CSSStyleDeclaration>);
  if (getComputedStyle(list).position === "static") list.style.position = "relative";
  // Its own stacking context, so the pill sits above the list's background
  // and below the items.
  list.style.isolation = "isolate";
  list.prepend(pill);

  let placed = false;
  let frame = 0;

  function place(jump = false) {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      const target = [...list.querySelectorAll(options.selector)].find((el) => el !== pill) as HTMLElement | undefined;
      if (!target || !target.isConnected) {
        pill.style.opacity = "0";
        placed = false;
        return;
      }
      const box = list.getBoundingClientRect();
      const r = target.getBoundingClientRect();
      if (!r.width || !r.height) return;
      const x = r.left - box.left + list.scrollLeft;
      const y = r.top - box.top + list.scrollTop;
      // The first time (and with motion off) it just appears in place.
      const animate = placed && !jump && motion.level === "full";
      pill.style.transition = animate
        ? "transform var(--spring-default-spatial), width var(--spring-default-spatial), height var(--spring-default-spatial), opacity var(--spring-fast-effects)"
        : "opacity var(--spring-fast-effects)";
      pill.style.transform = `translate(${x}px, ${y}px)`;
      pill.style.width = `${r.width}px`;
      pill.style.height = `${r.height}px`;
      pill.style.opacity = "1";
      placed = true;
    });
  }

  // Selection lives in attributes (selected, class, aria-current).
  // Items that fly in are measured again once they've landed.
  let settle: ReturnType<typeof setTimeout> | undefined;
  const watch = new MutationObserver(() => {
    place();
    clearTimeout(settle);
    settle = setTimeout(place, 560);
  });
  watch.observe(list, { subtree: true, childList: true, attributes: true, attributeFilter: ["selected", "class", "aria-current"] });
  const sizes = new ResizeObserver(() => place(true));
  sizes.observe(list);
  // A list that scrolls carries the pill with it, without sliding.
  const onScroll = () => place(true);
  list.addEventListener("scroll", onScroll, { capture: true, passive: true });
  place();

  return {
    update(next: IndicatorOptions) {
      options = next;
      place();
    },
    destroy() {
      cancelAnimationFrame(frame);
      clearTimeout(settle);
      watch.disconnect();
      sizes.disconnect();
      list.removeEventListener("scroll", onScroll, { capture: true });
      pill.remove();
    },
  };
}
