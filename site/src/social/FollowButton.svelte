<script lang="ts">
  // Follow, Follow back, or Following (click again to unfollow).
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import { beginSignIn } from "../lib/session";
  import { auth, following, followsMe, setFollow } from "./data.svelte";

  let { id, size = "small", wide = false }: { id: string; size?: "extra-small" | "small" | "medium"; wide?: boolean } = $props();

  let busy = $state(false);
  let hover = $state(false);
  const on = $derived(following.get(id) ?? false);
  const mine = $derived(auth.me?.id === id);

  async function toggle(e: Event) {
    e.preventDefault();
    e.stopPropagation();
    if (!auth.me) {
      await beginSignIn();
      return;
    }
    busy = true;
    try {
      await setFollow(id, !on);
    } catch (err) {
      M3eSnackbar.open((err as Error).message);
    } finally {
      busy = false;
    }
  }
</script>

{#if !mine}
  <m3e-button
    class="follow"
    class:wide
    class:on
    {size}
    variant={on ? "tonal" : "filled"}
    disabled-interactive={busy}
    onclick={toggle}
    onpointerenter={() => (hover = true)}
    onpointerleave={() => (hover = false)}
    aria-pressed={on}
  >
    {#if on}
      <m3e-icon slot="icon" name={hover ? "person_remove" : "how_to_reg"}></m3e-icon>{hover ? "Unfollow" : "Following"}
    {:else}
      <m3e-icon slot="icon" name="person_add"></m3e-icon>{followsMe.has(id) ? "Follow back" : "Follow"}
    {/if}
  </m3e-button>
{/if}

<style>
  .follow {
    flex: none;
    min-width: 112px;
  }
  .wide {
    min-width: 150px;
  }
  .on:hover {
    --m3e-tonal-button-container-color: color-mix(in srgb, var(--md-sys-color-error) 22%, var(--md-sys-color-secondary-container));
  }
</style>
