// Screen size, for the few things CSS can't change (like an m3e button's size).
const query = matchMedia("(max-width: 600px)");

class Screen {
  phone = $state(query.matches);

  constructor() {
    query.addEventListener("change", (e) => (this.phone = e.matches));
  }

  /** "large" buttons drop to "medium" on phones. */
  get big() {
    return this.phone ? "medium" : "large";
  }
}

export const screen = new Screen();
