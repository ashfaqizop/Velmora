<script lang="ts">
  // "On this page": a list built from the page's section headings, with a
  // marker that follows your scroll. (m3e's own table of contents trims the
  // space between a heading and its italic word, so this reads the real text.)
  import { onMount } from "svelte";

  let { target = "doc" }: { target?: string } = $props();

  let items = $state<{ id: string; label: string }[]>([]);
  let active = $state("");

  onMount(() => {
    const root = document.getElementById(target);
    if (!root) return;
    const sections = [...root.querySelectorAll<HTMLElement>("section[id]")].filter((s) => s.querySelector("h2"));
    items = sections.map((s) => ({ id: s.id, label: (s.querySelector("h2")?.textContent ?? "").replace(/\s+/g, " ").trim() }));
    active = items[0]?.id ?? "";

    // The section nearest the top of the screen is the one you're reading.
    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) if (e.isIntersecting) active = e.target.id;
      },
      { rootMargin: "-96px 0px -65% 0px" },
    );
    for (const s of sections) io.observe(s);
    return () => io.disconnect();
  });

  function go(e: MouseEvent, id: string) {
    e.preventDefault();
    document.getElementById(id)?.scrollIntoView({ behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth", block: "start" });
    history.replaceState(null, "", `#${id}`);
    active = id;
  }
</script>

<aside class="toc" aria-label="On this page">
  <span class="over">On this page</span>
  <ul>
    {#each items as it (it.id)}
      <li>
        <a href="#{it.id}" class:on={active === it.id} aria-current={active === it.id ? "location" : undefined} onclick={(e) => go(e, it.id)}>{it.label}</a>
      </li>
    {/each}
  </ul>
</aside>

<style>
  .toc {
    position: sticky;
    top: 104px;
    align-self: start;
    min-height: 40px;
  }
  .over {
    display: block;
    margin: 0 0 12px 14px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    border-left: 1px solid var(--line);
  }
  a {
    display: block;
    margin-left: -1px;
    padding: 8px 14px;
    border-left: 2px solid transparent;
    border-radius: 0 14px 14px 0;
    color: var(--muted);
    font-size: 14px;
    line-height: 1.35;
    text-decoration: none;
    transition:
      color 0.2s,
      background 0.2s,
      border-color 0.25s;
  }
  a:hover {
    color: var(--ink);
    background: rgb(255 255 255 / 4%);
  }
  a.on {
    color: var(--ink);
    font-weight: 600;
    border-left-color: var(--blue);
    background: color-mix(in srgb, var(--blue) 14%, transparent);
  }
  @media (max-width: 900px) {
    .toc {
      display: none;
    }
  }
</style>
