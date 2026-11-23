<script lang="ts">
  // Settings > Velmora account: sign in, your profile, and your data.
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import Avatar from "./Avatar.svelte";
  import NameStatus from "./NameStatus.svelte";
  import { account, USERNAME } from "../lib/account.svelte";
  import { isPhotoUrl, PRESETS, presetId } from "../lib/avatars";

  const p = $derived(account.profile);
  const since = $derived(p ? new Date(p.created_at).toLocaleDateString(undefined, { month: "long", year: "numeric" }) : "");

  // ── Editing ──
  let editing = $state(false);
  let name = $state("");
  let uname = $state("");
  let avatar = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let check = $state<"idle" | "same" | "checking" | "free" | "taken" | "bad">("same");
  let run = 0;

  /** A username can change once every 30 days (the first change is free). */
  const nextChange = $derived(
    p?.username_changed_at ? new Date(new Date(p.username_changed_at).getTime() + 30 * 86_400_000) : null,
  );
  const locked = $derived(!!nextChange && nextChange.getTime() > Date.now());
  const photo = $derived(isPhotoUrl(account.profile?.avatar) ? account.profile!.avatar : null);

  function edit() {
    if (!p) return;
    name = p.display_name;
    uname = p.username;
    avatar = p.avatar;
    error = null;
    check = "same";
    editing = true;
  }

  $effect(() => {
    const value = uname;
    if (!editing || !p) return;
    const mine = ++run;
    if (value.toLowerCase() === p.username.toLowerCase()) {
      check = "same";
      return;
    }
    if (!USERNAME.test(value)) {
      // Still typing the first letters is not a mistake.
      check = /^[A-Za-z0-9_]{0,2}$/.test(value) ? "idle" : "bad";
      return;
    }
    check = "checking";
    const t = setTimeout(async () => {
      try {
        const ok = await account.usernameAvailable(value);
        if (mine === run) check = ok ? "free" : "taken";
      } catch {
        if (mine === run) check = "idle";
      }
    }, 350);
    return () => clearTimeout(t);
  });

  const hint = $derived(
    locked
      ? `You can change this again on ${nextChange!.toLocaleDateString()}.`
      : check === "free"
        ? "Yours."
        : check === "taken"
          ? "That name isn't available."
          : check === "bad"
            ? "3 to 20 letters, numbers or underscores."
            : check === "checking"
              ? "Checking…"
              : "Capital letters don't make it a different name.",
  );

  const changed = $derived(!!p && (name.trim() !== p.display_name || avatar !== p.avatar || uname !== p.username));
  const canSave = $derived(!busy && changed && !!name.trim() && (check === "same" || check === "free"));

  async function save() {
    if (!canSave || !p) return;
    busy = true;
    error = null;
    try {
      const patch: Parameters<typeof account.update>[0] = {};
      if (name.trim() !== p.display_name) patch.display_name = name.trim();
      if (avatar !== p.avatar) patch.avatar = avatar;
      if (uname !== p.username) patch.username = uname;
      await account.update(patch);
      editing = false;
      M3eSnackbar.open("Profile saved.");
    } catch (e) {
      error = (e as Error).message;
    } finally {
      busy = false;
    }
  }

  // ── Data ──
  let confirmDelete = $state(false);
  let confirmTimer: ReturnType<typeof setTimeout> | undefined;

  async function copyData() {
    try {
      await navigator.clipboard.writeText(await account.exportData());
      M3eSnackbar.open("Copied. Paste it anywhere to see what Velmora holds about you.");
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    }
  }

  async function remove() {
    if (!confirmDelete) {
      confirmDelete = true;
      clearTimeout(confirmTimer);
      confirmTimer = setTimeout(() => (confirmDelete = false), 5000);
      return;
    }
    busy = true;
    try {
      await account.deleteAccount();
      M3eSnackbar.open("Your account was deleted.");
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    } finally {
      busy = false;
      confirmDelete = false;
    }
  }
</script>

{#if account.status === "loading"}
  <div class="state"><span class="muted">Checking your sign-in…</span></div>
{:else if account.signedIn && p}
  <div class="state">
    <Avatar value={p.avatar} size={64} />
    <div class="text">
      <strong>{p.display_name}</strong>
      <span>@{p.username}</span>
      <span class="small">{account.email ?? "Signed in"}{since ? ` · member since ${since}` : ""}</span>
      {#if account.offline}<span class="small">You're offline. Velmora will reconnect your account on its own.</span>{/if}
    </div>
    {#if !editing}
      <m3e-button variant="tonal" onclick={edit}><m3e-icon slot="icon" name="edit"></m3e-icon>Edit profile</m3e-button>
      <m3e-button variant="text" onclick={() => account.signOut()}>Sign out</m3e-button>
    {/if}
  </div>

  {#if editing}
    <div class="editor">
      <m3e-form-field class="field" variant="outlined">
        <label slot="label" for="acct-name">Display name</label>
        <input id="acct-name" maxlength="40" bind:value={name} disabled={busy} />
      </m3e-form-field>

      <m3e-form-field class="field" variant="outlined" error={check === "taken" || check === "bad"}>
        <label slot="label" for="acct-user">Username</label>
        <input
          id="acct-user"
          maxlength="20"
          autocomplete="off"
          spellcheck="false"
          bind:value={uname}
          disabled={busy || locked}
        />
        <span slot="suffix" class="tail"><NameStatus state={check} /></span>
        <span slot="error-icon"></span>
        <span slot="hint">{hint}</span>
        <span slot="error">{hint}</span>
      </m3e-form-field>

      <fieldset class="faces">
        <legend>Profile picture</legend>
        <div class="grid">
          {#if photo}
            <button type="button" class="face" class:on={avatar === photo} aria-label="Your photo" onclick={() => (avatar = photo!)}>
              <Avatar value={photo} size={44} />
            </button>
          {/if}
          {#each PRESETS as _, i (i)}
            <button type="button" class="face" class:on={avatar === presetId(i + 1)} aria-label="Fox {i + 1}" onclick={() => (avatar = presetId(i + 1))}>
              <Avatar value={presetId(i + 1)} size={44} />
            </button>
          {/each}
        </div>
      </fieldset>

      {#if error}<p class="error" role="alert">{error}</p>{/if}
      <div class="buttons">
        <m3e-button variant="text" disabled={busy} onclick={() => (editing = false)}>Cancel</m3e-button>
        <m3e-button variant="filled" disabled={!canSave} onclick={save}>{busy ? "Saving…" : "Save"}</m3e-button>
      </div>
    </div>
  {/if}

  <m3e-divider></m3e-divider>

  <div class="state" data-setting="account-data">
    <div class="text">
      <strong>Your data</strong>
      <span>Copy everything Velmora holds about you, or delete the account and all of it. Deleting can't be undone.</span>
    </div>
    <m3e-button variant="outlined" onclick={copyData}>Copy my data</m3e-button>
    <m3e-button variant="text" class="danger" disabled={busy} onclick={remove}>
      {confirmDelete ? "Click again to delete" : "Delete my account"}
    </m3e-button>
  </div>
{:else}
  <div class="state">
    <span class="badge" aria-hidden="true"><m3e-icon name="person"></m3e-icon></span>
    <div class="text">
      <strong>You're using Velmora as a guest</strong>
      <span>
        Sign in or sign up with Google, Discord or email to get a username and profile. Your list, progress and settings will follow your account in the
        next updates.
      </span>
    </div>
    <m3e-button variant="filled" disabled={account.status === "signing-in"} onclick={() => account.signIn()}>
      <m3e-icon slot="icon" name="login"></m3e-icon>Sign in or sign up
    </m3e-button>
  </div>
  {#if account.error}<p class="error" role="alert">{account.error}</p>{/if}
{/if}

<style>
  .state {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  .text strong {
    font-weight: 700;
  }
  .text span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .small {
    opacity: 0.85;
  }
  .muted {
    color: var(--md-sys-color-on-surface-variant);
  }
  .badge {
    display: grid;
    place-items: center;
    width: 48px;
    height: 48px;
    flex: none;
    border-radius: 50%;
    background: var(--md-sys-color-surface-container-highest);
    color: var(--md-sys-color-on-surface-variant);
  }
  .editor {
    display: flex;
    flex-direction: column;
    gap: 14px;
    margin-top: 16px;
  }
  .field {
    min-width: 0;
  }
  .faces {
    border: 0;
    margin: 0;
    padding: 0;
  }
  .faces legend {
    padding: 0;
    margin-bottom: 8px;
    font-size: 13px;
    font-weight: 600;
    color: var(--md-sys-color-on-surface-variant);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, 48px);
    gap: 8px;
  }
  .face {
    all: unset;
    cursor: pointer;
    display: grid;
    place-items: center;
    width: 48px;
    height: 48px;
    border-radius: 50%;
    transition: transform var(--spring-fast-spatial);
  }
  .face:hover {
    transform: scale(1.08);
  }
  .face:focus-visible,
  .face.on {
    box-shadow:
      0 0 0 3px var(--md-sys-color-surface-container),
      0 0 0 5px var(--md-sys-color-primary);
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .error {
    margin: 0;
    font-size: 14px;
    color: var(--md-sys-color-error);
  }
  .danger {
    --m3e-button-label-text-color: var(--md-sys-color-error);
  }
  m3e-divider {
    margin: 16px 0;
  }
  .tail {
    display: grid;
    padding-right: 12px;
    --status-busy: var(--md-sys-color-primary);
    --status-no: var(--md-sys-color-error);
  }
</style>
