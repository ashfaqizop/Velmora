// Site-wide Adsterra formats: the Social Bar and the Popunder.
//
// Both load a few seconds after the page, so the first paint and the
// Download button are never held up. The Popunder runs once per visit: its
// script loads only on the first page of a session. It opens behind the
// current tab and never stops a click, so a download still starts.
// Imported by every page except the legal ones.
import { POPUNDER, SOCIAL_BAR } from "./ads";

const POP_FLAG = "velmora.pop";

function load(src: string) {
  const s = document.createElement("script");
  s.src = src;
  s.async = true;
  s.dataset.cfasync = "false";
  document.body.appendChild(s);
}

function seen(): boolean {
  try {
    if (sessionStorage.getItem(POP_FLAG)) return true;
    sessionStorage.setItem(POP_FLAG, "1");
  } catch {
    // Storage blocked: treat as a fresh visit.
  }
  return false;
}

// Local dev and preview builds stay ad free.
if (location.hostname !== "localhost" && location.hostname !== "127.0.0.1") {
  setTimeout(() => {
    if (!seen()) load(POPUNDER);
  }, 3000);
  setTimeout(() => load(SOCIAL_BAR), 8000);
}
