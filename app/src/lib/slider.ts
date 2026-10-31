// Reliable dragging for <m3e-slider>.
//
// m3e-slider caches its width and position, and draws the thumb from that
// cache. When the cache goes stale (the player controls fade in, the window
// maximizes or goes full screen, a page lays out after fonts load), the thumb
// is drawn in the wrong place and the first press snaps it to the right one:
// a 2 px drag that "flies" 100 px.
//
// This action takes pointer dragging over and leaves the look to m3e:
//  * the value comes from the slider's live geometry on every move, never a cache;
//  * grabbing the thumb keeps the grab offset, so it doesn't jump under the pointer;
//  * m3e is made to re-measure whenever the slider appears, resizes, moves,
//    is hovered, or its value changes, so the drawn thumb is always where it should be.
// Keyboard input is still handled by m3e itself.

type SliderEl = HTMLElement & { min: number; max: number; step: number; disabled: boolean; orientation?: string };
type ThumbEl = HTMLElement & { value: number | null };

export type SliderOptions = {
  /** Called when a pointer drag starts and ends (seek bars commit on end). */
  onstart?: () => void;
  onend?: () => void;
};

const thumbOf = (el: SliderEl) => el.querySelector("m3e-slider-thumb") as ThumbEl | null;

/**
 * Make m3e measure itself again. Setting `orientation` away and back inside one
 * task lands in a single Lit update that still sees the property as changed,
 * and that path re-measures with `force = true`. Nothing is drawn vertical.
 */
function remeasure(el: SliderEl) {
  if (!("orientation" in el)) return;
  const was = el.orientation ?? "horizontal";
  el.orientation = was === "vertical" ? "horizontal" : "vertical";
  el.orientation = was;
}

export function slider(el: HTMLElement, opts: SliderOptions = {}) {
  const s = el as SliderEl;
  let options = opts;
  let pointer: number | null = null;
  let grab = 0; // px between the pointer and the thumb centre when grabbed on the thumb
  let last: number | null = null;

  const range = () => ({ min: Number(s.min ?? 0), max: Number(s.max ?? 100), step: Number(s.step) || 1 });

  function valueAt(clientX: number) {
    const thumb = thumbOf(s);
    const r = s.getBoundingClientRect();
    const tw = thumb ? thumb.getBoundingClientRect().width : 0;
    const span = Math.max(1, r.width - tw);
    const ratio = Math.min(1, Math.max(0, (clientX - grab - r.left - tw / 2) / span));
    const { min, max, step } = range();
    const v = min + Math.round((ratio * (max - min)) / step) * step;
    // Round away float noise from steps like 0.1.
    return Math.min(max, Math.max(min, Number(v.toFixed(6))));
  }

  function set(v: number) {
    const thumb = thumbOf(s);
    if (!thumb || v === last) return;
    last = v;
    thumb.value = v;
    thumb.dispatchEvent(new Event("input", { bubbles: true }));
  }

  function down(e: PointerEvent) {
    if (s.disabled || (e.pointerType === "mouse" && e.button !== 0)) return;
    // Ours, not m3e's: stop the event before it reaches the slider's own handler.
    e.stopPropagation();
    e.preventDefault();
    const thumb = thumbOf(s);
    if (!thumb) return;
    remeasure(s);
    const onThumb = e.composedPath().includes(thumb);
    if (onThumb) {
      const t = thumb.getBoundingClientRect();
      grab = e.clientX - (t.left + t.width / 2);
    } else {
      grab = 0;
    }
    pointer = e.pointerId;
    last = thumb.value ?? null;
    s.setPointerCapture(e.pointerId);
    thumb.focus({ preventScroll: true });
    options.onstart?.();
    if (!onThumb) set(valueAt(e.clientX));
  }

  function move(e: PointerEvent) {
    if (e.pointerId !== pointer) return;
    e.stopPropagation();
    set(valueAt(e.clientX));
  }

  function up(e: PointerEvent) {
    if (e.pointerId !== pointer) return;
    e.stopPropagation();
    pointer = null;
    grab = 0;
    if (s.hasPointerCapture(e.pointerId)) s.releasePointerCapture(e.pointerId);
    thumbOf(s)?.dispatchEvent(new Event("change", { bubbles: true }));
    options.onend?.();
  }

  // Capture phase on the host runs before the listeners inside m3e's shadow root.
  s.addEventListener("pointerdown", down, { capture: true });
  s.addEventListener("pointermove", move, { capture: true });
  s.addEventListener("pointerup", up, { capture: true });
  s.addEventListener("pointercancel", up, { capture: true });
  s.addEventListener("lostpointercapture", up as EventListener);

  // Keep m3e's drawing in step with the real layout.
  const fresh = () => {
    if (pointer === null) remeasure(s);
  };
  s.addEventListener("pointerenter", fresh);
  const resize = new ResizeObserver(fresh);
  resize.observe(s);
  if (s.parentElement) resize.observe(s.parentElement);
  const visible = new IntersectionObserver((entries) => entries.some((x) => x.isIntersecting) && fresh());
  visible.observe(s);
  window.addEventListener("resize", fresh);
  document.addEventListener("fullscreenchange", fresh);
  document.fonts?.ready.then(fresh);
  // After the first paint and once more after enter transitions settle.
  requestAnimationFrame(() => requestAnimationFrame(fresh));
  const late = setTimeout(fresh, 450);

  return {
    update(next: SliderOptions = {}) {
      options = next;
    },
    destroy() {
      clearTimeout(late);
      resize.disconnect();
      visible.disconnect();
      window.removeEventListener("resize", fresh);
      document.removeEventListener("fullscreenchange", fresh);
      s.removeEventListener("pointerdown", down, { capture: true });
      s.removeEventListener("pointermove", move, { capture: true });
      s.removeEventListener("pointerup", up, { capture: true });
      s.removeEventListener("pointercancel", up, { capture: true });
      s.removeEventListener("lostpointercapture", up as EventListener);
      s.removeEventListener("pointerenter", fresh);
    },
  };
}
