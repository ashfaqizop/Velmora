// Show the saved copy of a Music page at once, then the fresh one.
//
// Pages keep their last copy in the local database (musicdb.rs). `apply`
// runs with the saved copy if it arrives before the network does, then with
// the fresh copy, which is saved for next time. When the network fails but a
// saved copy is on screen, the page stays usable and the error is dropped.
import * as api from "../api";

export async function cached<T>(
  key: string,
  fetch: () => Promise<T>,
  apply: (value: T, fresh: boolean) => void,
): Promise<void> {
  let freshDone = false;
  let shown = false;
  const saved = api
    .musicCacheGet<T>(key)
    .then((v) => {
      if (v != null && !freshDone) {
        shown = true;
        apply(v, false);
      }
    })
    .catch(() => {});
  try {
    const v = await fetch();
    freshDone = true;
    apply(v, true);
    api.musicCachePut(key, v).catch(() => {});
  } catch (e) {
    await saved;
    if (!shown) throw e;
  }
}
