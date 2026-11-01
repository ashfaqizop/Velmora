// One shared clock for the decorative shapes, so they all morph in step
// without each running its own timer.
export const SHAPES = [
  "very-sunny",
  "4-leaf-clover",
  "12-sided-cookie",
  "soft-burst",
  "puffy-diamond",
  "8-leaf-clover",
  "6-sided-cookie",
  "sunny",
  "soft-boom",
  "9-sided-cookie",
] as const;

class Clock {
  tick = $state(0);
  private timer: ReturnType<typeof setInterval> | undefined;
  private users = 0;

  start() {
    if (this.users++ === 0) this.timer = setInterval(() => this.tick++, 2600);
  }

  stop() {
    if (--this.users === 0) clearInterval(this.timer);
  }
}

export const clock = new Clock();

/** Where one decorative shape sits and how it moves. */
export interface ShapeSpot {
  /** Position inside the parent, as CSS (e.g. "8%", "calc(100% - 120px)"). */
  x: string;
  y: string;
  size: number;
  color: string;
  /** Seconds per full turn; negative spins the other way. */
  spin?: number;
  /** Where in the shape cycle this one starts. */
  offset?: number;
  opacity?: number;
  blur?: number;
}
