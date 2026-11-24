<script lang="ts">
  // One person in a list: picture, name, handle, a line of bio, and Follow.
  import Avatar from "../auth/Avatar.svelte";
  import Badges from "./Badges.svelte";
  import FollowButton from "./FollowButton.svelte";
  import { profileHref } from "./nav.svelte";
  import type { Person } from "./data.svelte";

  let { person, note = "" }: { person: Person; note?: string } = $props();
</script>

<div class="row">
  <a class="who" href={profileHref(person.username)}>
    <Avatar value={person.avatar} size={48} />
    <span class="text">
      <span class="name"><span class="dn">{person.display_name}</span><Badges id={person.id} verified={person.verified} size={16} /></span>
      <span class="handle">@{person.username}{#if note}<span class="note"> · {note}</span>{/if}</span>
      {#if person.bio}<span class="bio">{person.bio}</span>{/if}
    </span>
  </a>
  <FollowButton id={person.id} />
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: 20px;
    transition: background-color 160ms ease;
  }
  .row:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 5%, transparent);
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 14px;
    color: inherit;
    text-decoration: none;
  }
  .text {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .name {
    display: flex;
    align-items: center;
    gap: 4px;
    font-weight: 700;
  }
  .dn,
  .bio,
  .handle {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .handle {
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .note {
    color: var(--md-sys-color-on-surface-variant);
    opacity: 0.8;
  }
  .bio {
    font-size: 14px;
    color: color-mix(in srgb, var(--md-sys-color-on-surface) 80%, transparent);
  }
</style>
