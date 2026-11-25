<script lang="ts">
  // A highlighted aside: a tip, a warning, or "what to expect".
  import type { Snippet } from "svelte";

  let {
    label,
    icon = "info",
    variant = "primary",
    children,
  }: { label: string; icon?: string; variant?: "info" | "primary" | "secondary" | "tertiary" | "error"; children: Snippet } = $props();
</script>

<m3e-callout {variant} shape="rounded" class="note">
  <m3e-icon slot="icon" name={icon}></m3e-icon>
  <span slot="label">{label}</span>
  {@render children()}
</m3e-callout>

<style>
  .note {
    display: block;
    margin: 22px 0;
    --m3e-callout-content-font-size: 15px;
    --m3e-callout-content-line-height: 1.6;
  }
</style>
