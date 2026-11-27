<script lang="ts">
  // One account: who they are, the counts we may see, and the things an admin
  // can do. No watch history and nothing synced is ever shown here.
  import { onMount } from "svelte";
  import Avatar from "../auth/Avatar.svelte";
  import { call, say, sayError } from "./api";
  import { ago, count, when } from "./format";

  let { id, onclose, onchange }: { id: string; onclose: () => void; onchange: () => void } = $props();

  interface Detail {
    id: string;
    email: string | null;
    created_at: string;
    last_sign_in_at: string | null;
    confirmed: boolean;
    providers: string[];
    profile: { username: string; display_name: string; avatar: string; bio: string; links: { url: string }[]; verified: boolean; hidden: boolean; banned: boolean; followers_count: number } | null;
    ratings: number;
    reviews: number;
    lists: number;
    following: number;
    followers: number;
    reports_against: number;
    reports_open: number;
    reports_filed: number;
    admin: boolean;
    max: boolean;
  }

  const FIELDS = [
    { id: "display_name", label: "Display name" },
    { id: "bio", label: "Bio" },
    { id: "links", label: "Links" },
    { id: "avatar", label: "Picture" },
    { id: "favorites", label: "Favourites" },
    { id: "username", label: "Username" },
  ];

  let dialog = $state<HTMLElement & { show(): void; hide(): void }>();
  let d = $state<Detail | null>(null);
  let busy = $state(false);
  let ask = $state<"ban" | "reset" | "delete" | null>(null);
  let picked = $state<string[]>([]);
  let typed = $state("");
  let note = $state("");

  async function load() {
    try {
      d = ((await call<{ user: Detail }>("user", { id })) as unknown as { user: Detail }).user;
    } catch (e) {
      sayError(e);
      onclose();
    }
  }

  onMount(() => {
    dialog?.show();
    void load();
  });

  async function act(op: string, extra: Record<string, unknown> = {}, done = "Done.") {
    busy = true;
    try {
      await call("userSet", { id, op, note: note.trim(), ...extra });
      say(done);
      ask = null;
      typed = "";
      note = "";
      picked = [];
      onchange();
      if (op === "delete") onclose();
      else await load();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  const p = $derived(d?.profile ?? null);
  const handle = $derived(p?.username ?? d?.id ?? "");
</script>

<m3e-dialog bind:this={dialog} dismissible onclosed={onclose}>
  <span slot="header">{p ? `@${p.username}` : "Account"}</span>

  {#if !d}
    <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
  {:else}
    <div class="body">
      <div class="who">
        <Avatar value={p?.avatar ?? "animal:panda"} size={64} />
        <div>
          <strong>{p?.display_name ?? "No profile yet"}</strong>
          <div class="muted">{d.email ?? "no email"}</div>
          <div class="chips">
            {#if d.admin}<span class="chip info">Admin</span>{/if}
            {#if d.max}<span class="chip ok">Max</span>{/if}
            {#if p?.verified}<span class="chip info">Verified</span>{/if}
            {#if p?.banned}<span class="chip bad">Banned</span>{:else if p?.hidden}<span class="chip warn">Hidden</span>{/if}
            {#if !d.confirmed}<span class="chip warn">Email not confirmed</span>{/if}
          </div>
        </div>
      </div>

      {#if p?.bio}<p class="bio">{p.bio}</p>{/if}
      {#if p?.links?.length}<p class="muted links">{p.links.map((l) => l.url).join(" · ")}</p>{/if}

      <dl class="facts">
        <div><dt>Joined</dt><dd>{when(d.created_at)}</dd></div>
        <div><dt>Last sign-in</dt><dd>{ago(d.last_sign_in_at)}</dd></div>
        <div><dt>Signs in with</dt><dd>{d.providers.length ? d.providers.join(", ") : "—"}</dd></div>
        <div><dt>Followers · following</dt><dd>{count(d.followers)} · {count(d.following)}</dd></div>
        <div><dt>Ratings · reviews · lists</dt><dd>{count(d.ratings)} · {count(d.reviews)} · {count(d.lists)}</dd></div>
        <div><dt>Reports against · filed</dt><dd>{count(d.reports_against)} ({count(d.reports_open)} open) · {count(d.reports_filed)}</dd></div>
      </dl>

      {#if d.admin}
        <p class="muted">This account is an admin, so nothing can be done to it from here.</p>
      {:else if ask === null}
        <div class="row-actions">
          {#if p}
            <m3e-button variant="tonal" disabled={busy} onclick={() => act(p.hidden ? "unhide" : "hide", {}, p.hidden ? "Profile is visible again." : "Profile hidden.")}>
              <m3e-icon slot="icon" name={p.hidden ? "visibility" : "visibility_off"}></m3e-icon>{p.hidden ? "Unhide" : "Hide"}
            </m3e-button>
            <m3e-button variant="tonal" disabled={busy} onclick={() => act(p.verified ? "unverify" : "verify", {}, p.verified ? "Verified badge removed." : "Verified.")}>
              <m3e-icon slot="icon" name="verified"></m3e-icon>{p.verified ? "Remove badge" : "Verify"}
            </m3e-button>
            <m3e-button variant="tonal" disabled={busy} onclick={() => (ask = "reset")}><m3e-icon slot="icon" name="restart_alt"></m3e-icon>Reset fields</m3e-button>
          {/if}
          {#if p?.banned}
            <m3e-button variant="filled" disabled={busy} onclick={() => act("unban", {}, "Unbanned. They can sign in again.")}><m3e-icon slot="icon" name="lock_open"></m3e-icon>Unban</m3e-button>
          {:else}
            <m3e-button variant="outlined" disabled={busy} onclick={() => (ask = "ban")}><m3e-icon slot="icon" name="block"></m3e-icon>Ban</m3e-button>
          {/if}
          <m3e-button variant="text" disabled={busy} onclick={() => (ask = "delete")}><m3e-icon slot="icon" name="delete_forever"></m3e-icon>Delete account</m3e-button>
        </div>
        {#if p}
          <a class="peek" href="https://social.getvelmora.dpdns.org/@{p.username}" target="_blank" rel="noopener">View their profile on Social<m3e-icon name="open_in_new"></m3e-icon></a>
        {/if}
      {:else}
        <div class="confirm">
          {#if ask === "ban"}
            <strong>Ban @{handle}?</strong>
            <p class="muted">Their profile disappears and they can't sign in again. You can undo this.</p>
          {:else if ask === "reset"}
            <strong>Reset what?</strong>
            <div class="picks">
              {#each FIELDS as f (f.id)}
                <label><input type="checkbox" value={f.id} bind:group={picked} />{f.label}</label>
              {/each}
            </div>
            <p class="muted">Display name goes back to the username, the picture to the panda, and the username to a plain placeholder.</p>
          {:else}
            <strong>Delete this account for good?</strong>
            <p class="muted">The account, profile, ratings, lists and everything synced go. A Max licence stays valid. Type <b>{handle}</b> to confirm.</p>
            <label class="field"><input bind:value={typed} placeholder={handle} aria-label="Type the username to confirm" autocomplete="off" spellcheck="false" /></label>
          {/if}
          <label class="field"><input bind:value={note} maxlength="200" placeholder="Note for the audit log (optional)" aria-label="Note" autocomplete="off" /></label>
          <div class="row-actions">
            <m3e-button variant="text" disabled={busy} onclick={() => (ask = null)}>Back</m3e-button>
            {#if ask === "ban"}
              <m3e-button variant="filled" disabled={busy} onclick={() => act("ban", {}, "Banned.")}>Ban</m3e-button>
            {:else if ask === "reset"}
              <m3e-button variant="filled" disabled={busy || !picked.length} onclick={() => act("reset", { fields: picked }, "Reset.")}>Reset</m3e-button>
            {:else}
              <m3e-button variant="filled" disabled={busy || typed.trim().toLowerCase() !== handle.toLowerCase()} onclick={() => act("delete", { confirm: typed.trim() }, "Account deleted.")}>Delete forever</m3e-button>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  {/if}

  <div slot="actions" class="actions"><m3e-button variant="text" onclick={() => dialog?.hide()}>Close</m3e-button></div>
</m3e-dialog>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 100%;
  }
  .who {
    display: flex;
    gap: 12px;
    align-items: center;
  }
  .who strong {
    font-size: 18px;
  }
  .who .muted {
    font-size: 13px;
  }
  .chips {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-top: 8px;
  }
  .bio {
    margin: 0;
    font-size: 14px;
  }
  .links {
    margin: 0;
    font-size: 13px;
    word-break: break-all;
  }
  .facts {
    margin: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px 20px;
  }
  .facts div {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .facts dt {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .facts dd {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }
  .confirm {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border-radius: 20px;
    background: var(--md-sys-color-surface-container-high);
  }
  .confirm p {
    margin: 0;
    font-size: 13.5px;
  }
  .picks {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px 16px;
    font-size: 14px;
  }
  .picks label {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .peek {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    text-decoration: none;
    color: var(--md-sys-color-primary);
    --m3e-icon-size: 16px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
  }
  @media (max-width: 560px) {
    .facts,
    .picks {
      grid-template-columns: 1fr;
    }
  }
</style>
