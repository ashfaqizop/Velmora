<script lang="ts">
  import type { RefBook } from "../../lib/ani/refs";

  // One generative UI block. Each render_* function maps to a component here;
  // they arrive in steps 7 and 8 of the Ani plan.
  let { name }: { name: string; args: Record<string, unknown>; refs: RefBook | null } = $props();
</script>

<div class="pending" data-block={name}></div>

<style>
  .pending {
    display: none;
  }
</style>
