<script lang="ts">
  import fox from "../assets/velmora-mark-small.webp";
  import { presetOf } from "./avatars";

  let { value, size = 56 }: { value: string; size?: number } = $props();

  const preset = $derived(presetOf(value));
</script>

<span
  class="avatar"
  style:width="{size}px"
  style:height="{size}px"
  style:background={preset ? `linear-gradient(145deg, ${preset[0]}, ${preset[1]})` : "#1e293b"}
>
  {#if preset}
    <img class="fox" src={fox} alt="" draggable="false" />
  {:else}
    <img class="photo" src={value} alt="" referrerpolicy="no-referrer" draggable="false" />
  {/if}
</span>

<style>
  .avatar {
    display: inline-grid;
    place-items: center;
    flex: none;
    border-radius: 50%;
    overflow: hidden;
  }
  .fox {
    width: 82%;
    height: 82%;
    object-fit: contain;
  }
  .photo {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
