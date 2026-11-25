// Svelte action: fade an element up when it scrolls into view. Unlike the
// landing page's `reveal`, it leaves anything already on screen alone, so a
// server-rendered page never flashes empty while it hydrates.
export function inview(node: HTMLElement, delay = 0) {
  if (typeof IntersectionObserver === "undefined") return {};
  if (node.getBoundingClientRect().top < innerHeight * 0.95) return {};
  if (matchMedia("(prefers-reduced-motion: reduce)").matches) return {};
  node.classList.add("iv");
  if (delay) node.style.setProperty("--delay", `${delay}s`);
  const io = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        node.classList.add("in");
        io.disconnect();
      }
    },
    { rootMargin: "0px 0px -8% 0px" },
  );
  io.observe(node);
  return { destroy: () => io.disconnect() };
}
