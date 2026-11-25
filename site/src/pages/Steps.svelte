<script lang="ts">
  // A vertical timeline: numbered steps joined by a line that fills in as you
  // scroll past each one.
  import type { Snippet } from "svelte";
  import { inview } from "./inview";

  let { steps, body }: { steps: { title: string; text: string }[]; body?: Snippet<[number]> } = $props();
</script>

<ol class="steps">
  {#each steps as s, i (s.title)}
    <li use:inview={0.04}>
      <span class="dot display">{i + 1}</span>
      <div class="card">
        <h3>{s.title}</h3>
        <p>{s.text}</p>
        {@render body?.(i)}
      </div>
    </li>
  {/each}
</ol>

<style>
  .steps {
    list-style: none;
    margin: 34px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  li {
    position: relative;
    display: grid;
    grid-template-columns: 56px minmax(0, 1fr);
    gap: 18px;
  }
  li:not(:last-child)::before {
    content: "";
    position: absolute;
    left: 27px;
    top: 58px;
    bottom: -20px;
    width: 2px;
    border-radius: 2px;
    background: linear-gradient(var(--blue), transparent);
    opacity: 0.55;
  }
  .dot {
    display: grid;
    place-items: center;
    width: 56px;
    height: 56px;
    border-radius: 18px;
    font-size: 28px;
    background: color-mix(in srgb, var(--blue) 22%, var(--bg));
    border: 1px solid color-mix(in srgb, var(--blue) 50%, transparent);
    color: var(--cream);
  }
  .card {
    padding: 22px 24px;
    border-radius: 24px;
    border: 1px solid var(--line);
    background: var(--glass);
  }
  h3 {
    margin: 0 0 6px;
    font-size: 18px;
  }
  p {
    margin: 0;
    color: var(--muted);
    font-size: 15.5px;
    line-height: 1.65;
  }
  @media (max-width: 600px) {
    li {
      grid-template-columns: 44px minmax(0, 1fr);
      gap: 12px;
    }
    .dot {
      width: 44px;
      height: 44px;
      font-size: 22px;
      border-radius: 14px;
    }
    li:not(:last-child)::before {
      left: 21px;
      top: 46px;
    }
  }
</style>
