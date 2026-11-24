<script lang="ts">
  // After a screening: rate and review it right there (Velmora ratings).
  import { fade, fly } from "svelte/transition";
  import Stars from "../Stars.svelte";
  import { account } from "../../lib/account.svelte";

  let { k, name }: { k: string; name: string } = $props();

  let pick = $state(0);
  let text = $state("");
  let spoiler = $state(false);
  let busy = $state(false);
  let done = $state(false);
  let problem = $state("");

  $effect(() => {
    const key = k;
    if (!account.signedIn) return;
    account.client
      .from("ratings")
      .select("score,review,spoiler")
      .eq("user_id", account.profile!.id)
      .eq("k", key)
      .maybeSingle()
      .then(({ data }) => {
        if (!data) return;
        pick = data.score;
        text = data.review ?? "";
        spoiler = !!data.spoiler;
      });
  });

  async function post() {
    if (!pick || busy) return;
    busy = true;
    problem = "";
    const { error } = await account.client
      .from("ratings")
      .upsert({ user_id: account.profile!.id, k, score: pick, review: text.trim().slice(0, 500), spoiler }, { onConflict: "user_id,k" });
    busy = false;
    if (error) problem = error.message.includes("rating_limit") ? "That's a lot of ratings for one day. Try tomorrow." : "Couldn't save. Try again.";
    else done = true;
  }
</script>

<div class="rate">
  {#if done}
    <div class="thanks" in:fly={{ y: 8, duration: 300 }}>
      <m3e-icon name="check_circle" filled></m3e-icon>
      <span>Rated {(pick / 2).toFixed(1)}. It's on your Velmora Social profile.</span>
    </div>
  {:else}
    <strong>How was {name}?</strong>
    <div class="row">
      <Stars value={pick} size={32} onpick={(s) => (pick = s)} />
      {#if pick}<span class="num" in:fade>{(pick / 2).toFixed(1)}</span>{/if}
    </div>
    {#if pick}
      <div class="more" in:fly={{ y: 6, duration: 240 }}>
        <textarea rows="2" maxlength="500" placeholder="A few words (optional)" bind:value={text}></textarea>
        <label class="spoil"><input type="checkbox" bind:checked={spoiler} />It has spoilers</label>
        <m3e-button variant="filled" disabled={busy} onclick={post}><m3e-icon slot="icon" name="send"></m3e-icon>{busy ? "Posting…" : "Post"}</m3e-button>
        {#if problem}<span class="problem">{problem}</span>{/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .rate {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 18px;
    border-radius: 24px;
    background: rgb(255 255 255 / 6%);
    border: 1px solid rgb(255 255 255 / 8%);
    text-align: left;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .num {
    font-weight: 800;
    color: #f8d27a;
  }
  .more {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 10px 14px;
    border: 1px solid rgb(255 255 255 / 12%);
    border-radius: 16px;
    background: rgb(255 255 255 / 5%);
    color: #fff;
    font: inherit;
    font-size: 14px;
    resize: vertical;
    outline: none;
  }
  textarea:focus {
    border-color: var(--md-sys-color-primary);
  }
  .spoil {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    opacity: 0.85;
  }
  .problem {
    font-size: 13px;
    color: #ffb4ab;
  }
  .thanks {
    display: flex;
    align-items: center;
    gap: 10px;
    color: #8ff0c3;
    --m3e-icon-size: 22px;
  }
</style>
