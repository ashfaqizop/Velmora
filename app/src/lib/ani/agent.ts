// One Ani turn, in two stages.
//
// 1. Research: the small model (ani.toolModel) calls Velmora's lookup tools
//    until it has what the answer needs. Its findings become notes.
// 2. Answer: the chat model (ani.chatModel) reads the notes and writes the
//    reply, streaming text and calling render tools for generative UI.
//
// Only the answer stage's turns go into the chat history; research notes stay
// attached to the last two user messages so follow-ups can still use them.
import { ani } from "../state.svelte";
import { callsOf, generate, type Content, type FunctionCall, type Part } from "./gemini";
import { answerPrompt, researchPrompt } from "./prompts";
import type { RefBook } from "./refs";
import { answerTools, researchTools } from "./tools";
import type { Block, ToolContext } from "./tools/types";

export type { Block } from "./tools/types";

export interface Step {
  label: string;
  state: "running" | "done" | "failed";
}

export interface TurnHandlers {
  /** Research progress, for the "Looking up…" list. */
  onSteps(steps: Step[]): void;
  /** The reply so far, as it streams. */
  onBlocks(blocks: Block[]): void;
}

export interface TurnInput {
  message: string;
  /** Answer-stage history from earlier turns. */
  history: Content[];
  /** Plain text of the last few exchanges, for the research stage. */
  recent: { role: "user" | "ani"; text: string }[];
  refs: RefBook;
}

export interface TurnResult {
  blocks: Block[];
  steps: Step[];
  /** History entries to append for this turn. */
  history: Content[];
}

const MAX_RESEARCH_ROUNDS = 5;
const MAX_ANSWER_ROUNDS = 3;
/** Per tool result, in characters, so one big answer can't crowd out the rest. */
const MAX_RESULT_CHARS = 6000;
const RESEARCH_TAG = "<research>";
/** User turns that keep their research notes in the history. */
const KEEP_RESEARCH_TURNS = 2;

export async function runTurn(input: TurnInput, h: TurnHandlers, signal: AbortSignal): Promise<TurnResult> {
  const ctx: ToolContext = { refs: input.refs, signal };
  const steps: Step[] = [];
  const notes = await research(input, ctx, steps, h);

  const user: Content = {
    role: "user",
    parts: [{ text: input.message }, ...(notes ? [{ text: `${RESEARCH_TAG}\n${notes}\n</research>` }] : [])],
  };
  const added: Content[] = [user];
  const contents = [...trimHistory(input.history), user];

  const blocks: Block[] = [];
  const responses = new Map<FunctionCall, Record<string, unknown>>();
  const tools = new Map(answerTools.map((t) => [t.decl.name, t]));
  const uiNames = answerTools.filter((t) => t.decl.name.startsWith("render_")).map((t) => t.decl.name);

  const onDelta = (d: { text: string } | { call: FunctionCall }) => {
    if ("text" in d) {
      const last = blocks.at(-1);
      if (last?.kind === "text") last.text += d.text;
      else blocks.push({ kind: "text", text: d.text });
    } else {
      const tool = tools.get(d.call.name);
      if (!tool) {
        responses.set(d.call, { error: `There is no function called ${d.call.name}.` });
        return;
      }
      const { block, response } = tool.apply(d.call.args ?? {}, ctx);
      if (block) blocks.push(block);
      responses.set(d.call, response);
    }
    h.onBlocks(blocks.map((b) => ({ ...b })));
  };

  for (let round = 0; round < MAX_ANSWER_ROUNDS; round++) {
    const res = await generate(
      ani.chatModel,
      { system: answerPrompt(uiNames), contents, tools: answerTools.map((t) => t.decl) },
      onDelta,
      signal,
    );
    if (res.parts.length) {
      const model: Content = { role: "model", parts: res.parts };
      contents.push(model);
      added.push(model);
    }
    const calls = callsOf(res.parts);
    if (!calls.length || signal.aborted) break;
    const reply: Content = {
      role: "user",
      parts: calls.map((c) => ({
        functionResponse: { id: c.id, name: c.name, response: responses.get(c) ?? { shown: true } },
      })),
    };
    contents.push(reply);
    added.push(reply);
  }

  if (!signal.aborted && !blocks.some((b) => (b.kind === "text" ? b.text.trim() : true))) {
    throw new Error("Ani didn't come up with an answer. Try asking another way.");
  }
  return { blocks, steps, history: added };
}

async function research(input: TurnInput, ctx: ToolContext, steps: Step[], h: TurnHandlers): Promise<string> {
  if (!researchTools.length) return "";
  const tools = new Map(researchTools.map((t) => [t.decl.name, t]));
  const contents: Content[] = [];
  for (const r of input.recent) {
    const role = r.role === "user" ? "user" : "model";
    if (!contents.length && role === "model") continue; // Gemini wants the user first
    contents.push({ role, parts: [{ text: r.text }] });
  }
  contents.push({ role: "user", parts: [{ text: input.message }] });

  const findings: string[] = [];
  for (let round = 0; round < MAX_RESEARCH_ROUNDS && !ctx.signal.aborted; round++) {
    const res = await generate(
      ani.toolModel,
      { system: researchPrompt(), contents, tools: researchTools.map((t) => t.decl), thinking: "low" },
      undefined,
      ctx.signal,
    );
    const calls = callsOf(res.parts);
    if (!calls.length) break;
    contents.push({ role: "model", parts: res.parts });

    const results = await Promise.all(
      calls.map(async (call): Promise<Part> => {
        const tool = tools.get(call.name);
        const args = call.args ?? {};
        if (!tool) return response(call, { error: `There is no tool called ${call.name}.` });
        const step: Step = { label: tool.label(args), state: "running" };
        steps.push(step);
        h.onSteps(steps.map((s) => ({ ...s })));
        try {
          const result = await tool.run(args, ctx);
          step.state = "done";
          const text = clip(JSON.stringify(result));
          findings.push(`${call.name} ${JSON.stringify(args)}\n${text}`);
          return response(call, { result: JSON.parse(text.endsWith("…") ? JSON.stringify(text) : text) });
        } catch (e) {
          step.state = "failed";
          return response(call, { error: e instanceof Error ? e.message : String(e) });
        } finally {
          h.onSteps(steps.map((s) => ({ ...s })));
        }
      }),
    );
    contents.push({ role: "user", parts: results });
  }
  return findings.join("\n\n");
}

const response = (call: FunctionCall, body: Record<string, unknown>): Part => ({
  functionResponse: { id: call.id, name: call.name, response: body },
});

const clip = (s: string) => (s.length > MAX_RESULT_CHARS ? `${s.slice(0, MAX_RESULT_CHARS)}…` : s);

/**
 * The answer history to send: research notes stay only on the most recent
 * user turns, and a leading model or function turn is dropped.
 */
function trimHistory(history: Content[]): Content[] {
  let kept = 0;
  const out: Content[] = [];
  for (let i = history.length - 1; i >= 0; i--) {
    const c = history[i];
    const isUserText = c.role === "user" && c.parts.some((p) => p.text !== undefined);
    if (isUserText) {
      kept++;
      if (kept > KEEP_RESEARCH_TURNS) {
        out.unshift({ ...c, parts: c.parts.filter((p) => !p.text?.startsWith(RESEARCH_TAG)) });
        continue;
      }
    }
    out.unshift(c);
  }
  while (out.length && !(out[0].role === "user" && out[0].parts.some((p) => p.text !== undefined))) out.shift();
  return out;
}
