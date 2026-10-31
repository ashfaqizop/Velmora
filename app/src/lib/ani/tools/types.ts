// The two kinds of tool Ani has.
//
// Research tools run in the first stage, on the small model. They look things
// up and return compact JSON that names titles, links and people by ref.
//
// Answer tools run in the second stage, on the chat model. Render tools turn
// into generative UI blocks; the rest (profile_note) make a small, visible
// change and show a line saying so.
import type { FunctionDeclaration } from "../gemini";
import type { RefBook } from "../refs";

export interface ToolContext {
  refs: RefBook;
  signal: AbortSignal;
}

export interface ResearchTool {
  decl: FunctionDeclaration;
  /** Shown while it runs, e.g. `Searching TMDB for "Saiyaara"`. */
  label(args: Record<string, unknown>): string;
  run(args: Record<string, unknown>, ctx: ToolContext): Promise<unknown>;
}

export type Block =
  | { kind: "text"; text: string }
  | { kind: "ui"; name: string; args: Record<string, unknown> }
  | { kind: "note"; id: string; text: string; removed?: boolean };

export interface AnswerTool {
  decl: FunctionDeclaration;
  /**
   * Check the call and turn it into a block. Returns what Gemini hears back:
   * `{ shown: true }`, or an error that lets it correct itself next round.
   */
  apply(args: Record<string, unknown>, ctx: ToolContext): { block?: Block; response: Record<string, unknown> };
}

// Small argument readers shared by the tools.
export const str = (v: unknown): string | undefined => (typeof v === "string" && v.trim() ? v.trim() : undefined);
export const num = (v: unknown): number | undefined => {
  const n = typeof v === "string" ? Number(v) : v;
  return typeof n === "number" && Number.isFinite(n) ? n : undefined;
};
export const strs = (v: unknown): string[] => (Array.isArray(v) ? v.map(str).filter((s): s is string => !!s) : []);
