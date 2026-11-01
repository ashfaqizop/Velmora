// Svelte action: fade an element up the first time it scrolls into view.
export function reveal(node: HTMLElement, delay = 0) {
  node.classList.add("reveal");
  if (delay) node.style.setProperty("--delay", `${delay}s`);
  const io = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        node.classList.add("shown");
        io.disconnect();
      }
    },
    { rootMargin: "0px 0px -8% 0px" },
  );
  io.observe(node);
  return { destroy: () => io.disconnect() };
}
