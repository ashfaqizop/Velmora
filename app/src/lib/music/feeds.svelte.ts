// Music Home, kept between visits so going back to it is instant.
import * as api from "../api";
import type { MusicShelf } from "../types";

const STALE = 15 * 60 * 1000;

class HomeFeed {
  shelves = $state<MusicShelf[]>([]);
  loading = $state(false);
  loadingMore = $state(false);
  error = $state<string | null>(null);
  private next: string | null = null;
  private loadedAt = 0;

  async load(force = false) {
    if (this.loading) return;
    if (!force && this.shelves.length && Date.now() - this.loadedAt < STALE) return;
    this.loading = true;
    this.error = null;
    try {
      const page = await api.musicHome();
      this.shelves = page.shelves;
      this.next = page.next;
      this.loadedAt = Date.now();
    } catch (e) {
      this.error = typeof e === "string" ? e : "YouTube Music didn't answer.";
    } finally {
      this.loading = false;
    }
  }

  async more() {
    if (!this.next || this.loadingMore || this.loading) return;
    this.loadingMore = true;
    try {
      const page = await api.musicHome(this.next);
      const have = new Set(this.shelves.map((s) => s.id));
      this.shelves = [...this.shelves, ...page.shelves.filter((s) => !have.has(s.id))];
      this.next = page.next;
    } catch {
      this.next = null;
    } finally {
      this.loadingMore = false;
    }
  }

  reset() {
    this.shelves = [];
    this.next = null;
    this.loadedAt = 0;
  }
}

export const musicHome = new HomeFeed();
