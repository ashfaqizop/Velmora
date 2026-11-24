<script lang="ts">
  // A profile picture: one of the nine animals, or a Google or Discord photo.
  // A ring in the animal's colour can sit around it (ring > 0).
  import { animalOf } from "../lib/avatars";

  let { value, size = 40, ring = 0 }: { value: string; size?: number; ring?: number } = $props();

  const animal = $derived(animalOf(value));
</script>

<span
  class="avatar"
  style:width="{size}px"
  style:height="{size}px"
  style:--ring="{ring}px"
  style:--tint={animal?.color ?? "#3271fe"}
>
  <img src={animal ? animal.src : value} alt="" referrerpolicy="no-referrer" draggable="false" loading="lazy" decoding="async" />
</span>

<style>
  .avatar {
    display: inline-grid;
    flex: none;
    border-radius: 50%;
    overflow: hidden;
    background: color-mix(in srgb, var(--tint) 60%, #0a0d16);
    box-shadow: 0 0 0 var(--ring) color-mix(in srgb, var(--tint) 70%, transparent);
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
