<script lang="ts">
  // Rate and review a title on Velmora Social, from its page in the app.
  // Shows the Velmora score next to it; the reviews live on the social site.
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import Stars from "./Stars.svelte";
  import { account } from "../lib/account.svelte";
  import { titleKey } from "../lib/analytics.svelte";
  import * as api from "../lib/api";
  import type { TitleSummary } from "../lib/types";

  let { title }: { title: TitleSummary } = $props();

  const SOCIAL = "https://social.getvelmora.dpdns.org";
  const k = $derived(title.badges?.includes("18+") ? null : titleKey(title));

  let score = $state<{ average: number | null; ratings: number } | null>(null);
  let mine = $state<{ score: number; review: string; spoiler: boolean } | null>(null);
  let pick = $state(0);
  let text = $state("");
  let spoiler = $state(false);
  let busy = $state(false);
  let dialog = $state<HTMLElement & { show(): void; hide(): void }>();

  async function refresh(key: string) {
    const [s, r] = await Promise.all([
      account.client.rpc("title_score", { title: key }),
      account.client.from("ratings").select("score,review,spoiler").eq("user_id", account.profile!.id).eq("k", key).maybeSingle(),
    ]);
    const row = Array.isArray(s.data) ? s.data[0] : s.data;
    score = row ? { average: row.average != null ? Number(row.average) : null, ratings: row.ratings ?? 0 } : null;
    mine = (r.data as typeof mine) ?? null;
  }

  $effect(() => {
    const key = k;
    score = null;
    mine = null;
    if (!key || !account.signedIn || account.offline) return;
    refresh(key).catch(() => {});
  });

  function open() {
    pick = mine?.score ?? 0;
    text = mine?.review ?? "";
    spoiler = mine?.spoiler ?? false;
    dialog?.show();
  }

  async function save() {
    if (!k || !pick) return;
    busy = true;
    try {
      const { error } = await account.client
        .from("ratings")
        .upsert({ user_id: account.profile!.id, k, score: pick, review: text.trim().slice(0, 500), spoiler }, { onConflict: "user_id,k" });
      if (error) throw new Error(error.message.includes("rating_limit") ? "That's a lot of ratings for one day. Try again tomorrow." : "Couldn't save your rating. Try again.");
      dialog?.hide();
      M3eSnackbar.open("Rating saved on Velmora Social.");
      await refresh(k);
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    } finally {
      busy = false;
    }
  }

  async function remove() {
    if (!k) return;
    busy = true;
    try {
      await account.client.from("ratings").delete().eq("user_id", account.profile!.id).eq("k", k);
      dialog?.hide();
      await refresh(k);
    } finally {
      busy = false;
    }
  }
</script>

{#if k && account.signedIn && !account.offline}
  <m3e-button variant="tonal" size="medium" onclick={open}>
    <m3e-icon slot="icon" name="star" filled={!!mine}></m3e-icon>
    {mine ? `${(mine.score / 2).toFixed(1)} ★` : "Rate"}
  </m3e-button>
  {#if score?.ratings}
    <button type="button" class="score" title="Reviews on Velmora Social" onclick={() => api.openExternal(`${SOCIAL}/title/${k}`).catch(() => {})}>
      <Stars value={Math.round(score.average ?? 0)} size={14} />
      <span>{((score.average ?? 0) / 2).toFixed(1)} · {score.ratings} on Velmora</span>
    </button>
  {/if}

  <m3e-dialog bind:this={dialog} dismissible>
    <span slot="header">Rate {title.name}</span>
    <div class="body">
      <div class="pick">
        <Stars value={pick} size={36} onpick={(s) => (pick = s)} />
        <strong>{pick ? `${(pick / 2).toFixed(1)} / 5` : "Tap a star"}</strong>
      </div>
      <m3e-form-field variant="outlined" class="field">
        <label slot="label" for="rate-review">Review (optional)</label>
        <textarea id="rate-review" rows="4" maxlength="500" bind:value={text} disabled={busy}></textarea>
        <span slot="hint">{text.length}/500 · Public on your Velmora Social profile.</span>
      </m3e-form-field>
      <label class="spoil">
        <m3e-switch checked={spoiler} onchange={(e: Event) => (spoiler = (e.currentTarget as HTMLElement & { checked: boolean }).checked)}></m3e-switch>
        It has spoilers
      </label>
    </div>
    <div slot="actions" class="actions">
      {#if mine}<m3e-button variant="text" disabled={busy} onclick={remove}>Remove</m3e-button>{/if}
      <span class="grow"></span>
      <m3e-button variant="text" onclick={() => dialog?.hide()}>Cancel</m3e-button>
      <m3e-button variant="filled" disabled={!pick || busy} onclick={save}>{busy ? "Saving…" : mine ? "Update" : "Post"}</m3e-button>
    </div>
  </m3e-dialog>
{/if}

<style>
  .score {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border: 0;
    border-radius: 999px;
    background: color-mix(in srgb, #f8d27a 12%, transparent);
    color: #f8d27a;
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 16px;
    width: min(460px, 80vw);
  }
  .pick {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .pick strong {
    color: #f8d27a;
  }
  .field {
    width: 100%;
  }
  .spoil {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
  }
  .grow {
    flex: 1;
  }
</style>
