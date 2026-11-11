// A song's colors, taken from its artwork, for the player's tint.
//
// The image is drawn small onto a canvas and the most colorful pixels are
// averaged. Google's image hosts allow this (they send CORS headers); if one
// doesn't, the player keeps the app's own accent.

export interface ArtColors {
  /** The main color, e.g. "rgb(120 60 200)". */
  base: string;
  /** A darker shade for backgrounds. */
  deep: string;
  /** Text that reads on `base`. */
  on: string;
}

const cache = new Map<string, ArtColors | null>();

export async function artColors(url: string | null): Promise<ArtColors | null> {
  if (!url) return null;
  if (cache.has(url)) return cache.get(url) ?? null;
  const result = await new Promise<ArtColors | null>((resolve) => {
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.decoding = "async";
    img.onload = () => {
      try {
        const size = 24;
        const canvas = document.createElement("canvas");
        canvas.width = size;
        canvas.height = size;
        const ctx = canvas.getContext("2d", { willReadFrequently: true });
        if (!ctx) return resolve(null);
        ctx.drawImage(img, 0, 0, size, size);
        const { data } = ctx.getImageData(0, 0, size, size);
        let r = 0,
          g = 0,
          b = 0,
          weight = 0;
        for (let i = 0; i < data.length; i += 4) {
          const [pr, pg, pb] = [data[i], data[i + 1], data[i + 2]];
          const max = Math.max(pr, pg, pb);
          const min = Math.min(pr, pg, pb);
          // Favor saturated, mid-bright pixels over greys and near-black.
          const sat = max === 0 ? 0 : (max - min) / max;
          const w = sat * sat * (max > 40 ? 1 : 0.1) + 0.02;
          r += pr * w;
          g += pg * w;
          b += pb * w;
          weight += w;
        }
        r /= weight;
        g /= weight;
        b /= weight;
        // Lift dull colors so the tint shows on a dark UI.
        const top = Math.max(r, g, b, 1);
        const lift = Math.min(2.2, 200 / top);
        const [br, bg, bb] = [r, g, b].map((c) => Math.round(Math.min(255, c * lift)));
        const luma = 0.2126 * br + 0.7152 * bg + 0.0722 * bb;
        resolve({
          base: `rgb(${br} ${bg} ${bb})`,
          deep: `rgb(${Math.round(br * 0.22)} ${Math.round(bg * 0.22)} ${Math.round(bb * 0.22)})`,
          on: luma > 150 ? "rgb(20 16 28)" : "rgb(255 255 255)",
        });
      } catch {
        resolve(null);
      }
    };
    img.onerror = () => resolve(null);
    img.src = url;
  });
  cache.set(url, result);
  return result;
}
