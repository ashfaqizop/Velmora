// Windows' "Animation effects" switch, as the webview reports it.
export const reducedMotion = () =>
  typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;
