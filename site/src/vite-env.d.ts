/// <reference types="svelte" />
/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Adsterra banner keys, one per size. Leave unset to show no ad there. */
  readonly VITE_ADSTERRA_BANNER_728?: string;
  readonly VITE_ADSTERRA_BANNER_320?: string;
  readonly VITE_ADSTERRA_BANNER_300?: string;
  /** Host Adsterra serves banner scripts from, as shown in your ad code. */
  readonly VITE_ADSTERRA_HOST?: string;
  /** Full `invoke.js` URL of an Adsterra Native Banner. */
  readonly VITE_ADSTERRA_NATIVE?: string;
}
