// Gemini request and response shapes, and one streamed call through the Rust
// side (which holds the key). Streamed chunks are merged back into whole parts
// so a model turn can be replayed later with its thought signatures intact.
import * as api from "../api";

export interface FunctionCall {
  id?: string;
  name: string;
  args?: Record<string, unknown>;
}

export interface Part {
  text?: string;
  thought?: boolean;
  /** Gemini 3 needs these sent back with the turn that produced them. */
  thoughtSignature?: string;
  functionCall?: FunctionCall;
  functionResponse?: { id?: string; name: string; response: Record<string, unknown> };
}

export interface Content {
  role: "user" | "model";
  parts: Part[];
}

export interface Schema {
  type: "object" | "string" | "number" | "integer" | "boolean" | "array";
  description?: string;
  properties?: Record<string, Schema>;
  required?: string[];
  items?: Schema;
  enum?: string[];
  nullable?: boolean;
}

export interface FunctionDeclaration {
  name: string;
  description: string;
  parameters?: Schema;
}

export interface GenerateRequest {
  system: string;
  contents: Content[];
  tools?: FunctionDeclaration[];
  /** Gemini 3 thinking level; ignored for older models. */
  thinking?: "minimal" | "low" | "medium" | "high";
  temperature?: number;
}

export type Delta = { text: string } | { call: FunctionCall };

export interface GenerateResult {
  parts: Part[];
  finishReason?: string;
}

const BLOCKED: Record<string, string> = {
  SAFETY: "Gemini declined to answer that for safety reasons.",
  RECITATION: "Gemini stopped because the answer quoted too much from one source.",
  PROHIBITED_CONTENT: "Gemini declined to answer that.",
  BLOCKLIST: "Gemini declined to answer that.",
  SPII: "Gemini stopped because the answer touched on personal information.",
};

/**
 * One streamed `streamGenerateContent` call. `onDelta` sees text as it arrives
 * and each function call once it's complete, in the order Gemini sent them.
 */
export async function generate(
  model: string,
  req: GenerateRequest,
  onDelta?: (d: Delta) => void,
  signal?: AbortSignal,
): Promise<GenerateResult> {
  const id = crypto.randomUUID();
  const generationConfig: Record<string, unknown> = {};
  if (req.temperature !== undefined) generationConfig.temperature = req.temperature;
  if (req.thinking && model.startsWith("gemini-3")) generationConfig.thinkingConfig = { thinkingLevel: req.thinking };
  const body: Record<string, unknown> = {
    systemInstruction: { parts: [{ text: req.system }] },
    contents: req.contents,
    generationConfig,
  };
  if (req.tools?.length) {
    body.tools = [{ functionDeclarations: req.tools }];
    body.toolConfig = { functionCallingConfig: { mode: "AUTO" } };
  }

  const parts: Part[] = [];
  let finishReason: string | undefined;
  let blocked: string | undefined;
  let streamError: string | undefined;

  const onChunk = (raw: unknown) => {
    if (signal?.aborted) return;
    const chunk = raw as {
      candidates?: { content?: { parts?: Part[] }; finishReason?: string }[];
      promptFeedback?: { blockReason?: string };
      error?: { message?: string };
    };
    if (chunk.error) streamError = chunk.error.message ?? "Gemini sent an error.";
    if (chunk.promptFeedback?.blockReason) blocked = chunk.promptFeedback.blockReason;
    const cand = chunk.candidates?.[0];
    if (cand?.finishReason) finishReason = cand.finishReason;
    for (const p of cand?.content?.parts ?? []) merge(parts, p, onDelta);
  };

  const stop = () => api.aniStop(id).catch(() => {});
  signal?.addEventListener("abort", stop, { once: true });
  try {
    await api.aniGenerate(id, model, body, onChunk);
  } finally {
    signal?.removeEventListener("abort", stop);
  }
  if (signal?.aborted) return { parts, finishReason: "ABORTED" };
  if (streamError) throw new Error(streamError);
  if (blocked) throw new Error(BLOCKED[blocked] ?? "Gemini declined to answer that.");
  if (finishReason && BLOCKED[finishReason] && !parts.length) throw new Error(BLOCKED[finishReason]);
  return { parts, finishReason };
}

/** Fold one streamed part into the turn so far. */
function merge(parts: Part[], p: Part, onDelta?: (d: Delta) => void) {
  if (p.functionCall) {
    const call: FunctionCall = { ...p.functionCall, args: p.functionCall.args ?? {} };
    parts.push({ ...p, functionCall: call });
    onDelta?.({ call });
    return;
  }
  if (p.text === undefined) {
    // A bare signature (or something new) still has to go back to Gemini.
    if (p.thoughtSignature) {
      const last = parts.at(-1);
      if (last && !last.functionCall && !last.thoughtSignature) last.thoughtSignature = p.thoughtSignature;
      else parts.push({ ...p });
    }
    return;
  }
  const last = parts.at(-1);
  if (last && last.text !== undefined && !last.functionCall && !!last.thought === !!p.thought && !last.thoughtSignature) {
    last.text += p.text;
    if (p.thoughtSignature) last.thoughtSignature = p.thoughtSignature;
  } else {
    parts.push({ ...p });
  }
  if (!p.thought && p.text) onDelta?.({ text: p.text });
}

/** The visible text of a model turn. */
export const textOf = (parts: Part[]) =>
  parts
    .filter((p) => p.text && !p.thought)
    .map((p) => p.text)
    .join("");

export const callsOf = (parts: Part[]) => parts.flatMap((p) => (p.functionCall ? [p.functionCall] : []));
