// Release data shared by the download buttons and the changelog.
import { releases, type Release } from "./release";

class Latest {
  list = $state<Release[]>([]);
  failed = $state(false);
  loaded = $state(false);

  get current(): Release | undefined {
    return this.list[0];
  }

  /** Installer downloads across every release, from GitHub's own counters. */
  get downloads(): number | null {
    return this.list.length ? this.list.reduce((n, r) => n + (r.installer?.downloads ?? 0), 0) : null;
  }

  downloadsOf(version: string): number | null {
    return this.list.find((r) => r.version === version)?.installer?.downloads ?? null;
  }

  load() {
    releases()
      .then((l) => (this.list = l))
      .catch((e) => {
        console.warn("releases", e);
        this.failed = true;
      })
      .finally(() => (this.loaded = true));
  }
}

export const latest = new Latest();
