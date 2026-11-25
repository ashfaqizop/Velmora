// Screen size, for the few things CSS can't change (like an m3e button's size).
const query = typeof matchMedia === "function" ? matchMedia("(max-width: 600px)") : null;

class Screen {
  phone = $state(query?.matches ?? false);

  constructor() {
    query?.addEventListener("change", (e) => (this.phone = e.matches));
  }

  /** "large" buttons drop to "medium" on phones. */
  get big() {
    return this.phone ? "medium" : "large";
  }
}

export const screen = new Screen();
