// Release data shared by the download buttons and the changelog.
import { releases, type Release } from "./release";

const total = (r: Release) => (r.installer?.downloads ?? 0) + (r.android?.arm64?.downloads ?? 0) + (r.android?.universal?.downloads ?? 0);

class Latest {
  list = $state<Release[]>([]);
  failed = $state(false);
  loaded = $state(false);

  get current(): Release | undefined {
    return this.list[0];
  }

  /** The newest release that has Android APKs. */
  get android(): Release | undefined {
    return this.list.find((r) => r.android);
  }

  /** Windows and Android downloads across every release, from GitHub's own counters. */
  get downloads(): number | null {
    return this.list.length ? this.list.reduce((n, r) => n + total(r), 0) : null;
  }

  downloadsOf(version: string): number | null {
    const r = this.list.find((x) => x.version === version);
    return r ? total(r) : null;
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
