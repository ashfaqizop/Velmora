<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import Logo from "./Logo.svelte";
  import PoweredBy from "./PoweredBy.svelte";
  import DownloadFolder from "./DownloadFolder.svelte";
  import AniKey from "./AniKey.svelte";
  import { LANGUAGES } from "../lib/options";
  import { ani, settings } from "../lib/state.svelte";

  let { ondone }: { ondone: () => void } = $props();

  // Local picks, saved together on finish.
  let quality = $state(1080);
  let audio = $state("en");
  let subs = $state("en");
  let torrents = $state(true);
  let french = $state(true);
  let adult = $state(false);
  let aniOn = $state(true);
  let saving = $state(false);

  onMount(async () => {
    const p = await settings.load();
    quality = p.preferred_resolution;
    audio = p.audio_languages[0] ?? "en";
    subs = p.subtitle_languages[0] ?? "en";
    torrents = p.include_torrents;
    french = p.include_french;
    adult = p.show_adult;
  });

  const checked = (e: Event) => (e.currentTarget as HTMLElement & { checked: boolean }).checked;
  const selectValue = (e: Event) => (e.currentTarget as HTMLElement & { value: string }).value;
  const first = (lang: string, list: string[]) => [lang, ...list.filter((l) => l !== lang)];

  async function finish() {
    saving = true;
    const p = await settings.load();
    await settings
      .update({
        preferred_resolution: quality,
        max_auto_resolution: quality === 2160 ? 2160 : p.max_auto_resolution,
        audio_languages: first(audio, p.audio_languages),
        subtitle_languages: first(subs, p.subtitle_languages),
        include_torrents: torrents,
        include_french: french,
        show_adult: adult,
      })
      .catch((e) => console.warn("saving preferences failed", e));
    // Turning torrents on here counts as reading the VPN note.
    if (torrents) localStorage.setItem("velmora.torrentNoticeAccepted", "1");
    ani.setEnabled(aniOn);
    localStorage.setItem("velmora.onboarded", "1");
    ondone();
  }
</script>

<div class="onboarding" transition:fade={{ duration: 400 }}>
  <m3e-shape name="flower" class="blob a" aria-hidden="true"></m3e-shape>
  <m3e-shape name="soft-burst" class="blob b" aria-hidden="true"></m3e-shape>

  <m3e-card variant="elevated" class="panel">
    <div slot="content" class="content">
      <m3e-stepper linear>
        <m3e-step for="ob-welcome">Welcome</m3e-step>
        <m3e-step for="ob-watch">How you watch</m3e-step>
        <m3e-step for="ob-sources">Sources</m3e-step>
        <m3e-step for="ob-downloads">Downloads</m3e-step>
        <m3e-step for="ob-ani">Ani</m3e-step>

        <m3e-step-panel id="ob-welcome">
          <div class="step">
            <Logo size={56} />
            <h1 class="display">Movies, series and anime, all in one place.</h1>
            <p>
              Velmora searches many sources at once and plays the best link it finds. A few quick choices and you're in.
            </p>
            <PoweredBy height={12} />
          </div>
          <div slot="actions" class="actions">
            <m3e-button variant="filled"><m3e-stepper-next>Get started</m3e-stepper-next></m3e-button>
          </div>
        </m3e-step-panel>

        <m3e-step-panel id="ob-watch">
          <div class="step">
            <h2 class="display">How do you like to watch?</h2>
            <div class="field-row">
              <span>Quality</span>
              <m3e-segmented-button aria-label="Quality">
                {#each [720, 1080, 2160] as r (r)}
                  <m3e-button-segment checked={quality === r} onclick={() => (quality = r)}>
                    {r === 2160 ? "4K" : `${r}p`}
                  </m3e-button-segment>
                {/each}
              </m3e-segmented-button>
            </div>
            <div class="field-row">
              <span>Audio</span>
              <m3e-form-field class="field">
                <m3e-select aria-label="Audio language" value={audio} onchange={(e: Event) => (audio = selectValue(e))}>
                  {#each LANGUAGES as [code, name] (code)}
                    <m3e-option value={code} selected={audio === code}>{name}</m3e-option>
                  {/each}
                </m3e-select>
              </m3e-form-field>
            </div>
            <div class="field-row">
              <span>Subtitles</span>
              <m3e-form-field class="field">
                <m3e-select aria-label="Subtitle language" value={subs} onchange={(e: Event) => (subs = selectValue(e))}>
                  {#each LANGUAGES as [code, name] (code)}
                    <m3e-option value={code} selected={subs === code}>{name}</m3e-option>
                  {/each}
                </m3e-select>
              </m3e-form-field>
            </div>
            <div class="field-row">
              <span>Anime</span>
              <m3e-segmented-button aria-label="Anime audio">
                <m3e-button-segment checked={settings.audioMode === "sub"} onclick={() => settings.setAudioMode("sub")}>Sub</m3e-button-segment>
                <m3e-button-segment checked={settings.audioMode === "dub"} onclick={() => settings.setAudioMode("dub")}>Dub</m3e-button-segment>
              </m3e-segmented-button>
            </div>
          </div>
          <div slot="actions" class="actions">
            <m3e-button><m3e-stepper-previous>Back</m3e-stepper-previous></m3e-button>
            <m3e-button variant="filled"><m3e-stepper-next>Next</m3e-stepper-next></m3e-button>
          </div>
        </m3e-step-panel>

        <m3e-step-panel id="ob-sources">
          <div class="step">
            <h2 class="display">Where should Velmora look?</h2>
            <label class="switch-row">
              <div>
                <strong>Torrents</strong>
                <span>More choice and higher quality. Peers can see your IP, so use a VPN if you're unsure.</span>
              </div>
              <m3e-switch checked={torrents} onchange={(e: Event) => (torrents = checked(e))}></m3e-switch>
            </label>
            <label class="switch-row">
              <div>
                <strong>French sources</strong>
                <span>VF and VOSTFR releases, always labelled.</span>
              </div>
              <m3e-switch checked={french} onchange={(e: Event) => (french = checked(e))}></m3e-switch>
            </label>
            <label class="switch-row">
              <div>
                <strong>Show 18+ titles</strong>
                <span>Hidden unless you turn this on.</span>
              </div>
              <m3e-switch checked={adult} onchange={(e: Event) => (adult = checked(e))}></m3e-switch>
            </label>
          </div>
          <div slot="actions" class="actions">
            <m3e-button><m3e-stepper-previous>Back</m3e-stepper-previous></m3e-button>
            <m3e-button variant="filled"><m3e-stepper-next>Next</m3e-stepper-next></m3e-button>
          </div>
        </m3e-step-panel>

        <m3e-step-panel id="ob-downloads">
          <div class="step">
            <h2 class="display">Where should downloads go?</h2>
            <p>Downloaded movies and episodes play offline inside Velmora. Pick a drive with room to spare.</p>
            <DownloadFolder />
          </div>
          <div slot="actions" class="actions">
            <m3e-button><m3e-stepper-previous>Back</m3e-stepper-previous></m3e-button>
            <m3e-button variant="filled"><m3e-stepper-next>Next</m3e-stepper-next></m3e-button>
          </div>
        </m3e-step-panel>

        <m3e-step-panel id="ob-ani">
          <div class="step">
            <div class="ani-head">
              <span class="on-shape ani-mark" aria-hidden="true">
                <m3e-shape name="very-sunny"></m3e-shape>
                <m3e-icon name="wand_stars" filled></m3e-icon>
              </span>
              <h2 class="display">Meet Ani</h2>
            </div>
            <p>
              Ani is Velmora's assistant. Ask for "more like Saiyaara" or "a short anime for tonight" and it searches for you,
              learns what you like from what you watch, and answers with titles you can open.
            </p>
            <label class="switch-row">
              <div>
                <strong>Use Ani</strong>
                <span>Adds Ani to the menu. It runs on your own Gemini key, so nothing is sent anywhere without one.</span>
              </div>
              <m3e-switch checked={aniOn} onchange={(e: Event) => (aniOn = checked(e))}></m3e-switch>
            </label>
            {#if aniOn}
              <div transition:fade={{ duration: 200 }}><AniKey /></div>
            {/if}
            <p class="small">You can change all of this later in Settings.</p>
          </div>
          <div slot="actions" class="actions">
            <m3e-button><m3e-stepper-previous>Back</m3e-stepper-previous></m3e-button>
            <m3e-button variant="filled" disabled={saving} onclick={finish}>
              <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>Start watching
            </m3e-button>
          </div>
        </m3e-step-panel>
      </m3e-stepper>
    </div>
  </m3e-card>
</div>

<style>
  .onboarding {
    position: fixed;
    inset: 0;
    z-index: 90;
    display: grid;
    place-items: center;
    overflow: hidden;
    background:
      radial-gradient(90% 70% at 50% 110%, color-mix(in srgb, var(--md-sys-color-primary) 18%, transparent), transparent 60%),
      var(--md-sys-color-background);
  }
  .blob {
    position: absolute;
    filter: blur(80px);
    opacity: 0.35;
  }
  .blob.a {
    --m3e-shape-size: 460px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
    left: -120px;
    top: -100px;
  }
  .blob.b {
    --m3e-shape-size: 380px;
    --m3e-shape-container-color: var(--brand-coral);
    right: -100px;
    bottom: -120px;
    opacity: 0.2;
  }
  .panel {
    position: relative;
    width: min(720px, calc(100vw - 64px));
    --m3e-card-shape: var(--md-sys-shape-corner-extra-large, 28px);
  }
  .content {
    padding: 8px;
  }
  .step {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px 8px 8px;
    min-height: 300px;
  }
  h1,
  h2 {
    margin: 0;
    font-weight: 400;
    line-height: 1.05;
  }
  h1 {
    font-size: 44px;
  }
  h2 {
    font-size: 34px;
  }
  p {
    margin: 0;
    line-height: 1.55;
    color: var(--md-sys-color-on-surface-variant);
  }
  .small {
    font-size: 13px;
  }
  .field-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }
  .field {
    width: 240px;
  }
  .switch-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
  }
  .switch-row div {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .switch-row span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .ani-head {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .ani-mark {
    --m3e-shape-size: 52px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 26px;
    flex: none;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
