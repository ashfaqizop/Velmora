// Ani's tool registry: 37 research tools from the tool modules, and the
// answer tools (the Personalize notes, plus the generative UI blocks).
import { personalize } from "../personalize.svelte";
import type { AnswerTool, ResearchTool } from "./types";
import { str } from "./types";
import { discoveryTools } from "./discovery";
import { linkTools } from "./links";
import { profileTools } from "./profile";
import { renderTools } from "./render";

export const researchTools: ResearchTool[] = [...discoveryTools, ...linkTools, ...profileTools];

const profileNote: AnswerTool = {
  decl: {
    name: "profile_note",
    description:
      "Save a lasting fact about the user's taste or habits to Personalize, such as 'Prefers English dubs' or 'Doesn't like horror'. Only for things that will matter in later chats, never for one-off requests. The user sees a line saying you noted it.",
    parameters: {
      type: "object",
      properties: { text: { type: "string", description: "The note, one short sentence in the third person." } },
      required: ["text"],
    },
  },
  apply(args) {
    const text = str(args.text);
    if (!text) return { response: { error: "text is required" } };
    const note = personalize.addNote(text);
    return { block: { kind: "note", id: note.id, text: note.text }, response: { saved: true } };
  },
};

const profileForget: AnswerTool = {
  decl: {
    name: "profile_forget",
    description: "Remove a saved Personalize note when the user asks you to forget it. Match it by its text.",
    parameters: {
      type: "object",
      properties: { text: { type: "string", description: "The note to forget, or words from it." } },
      required: ["text"],
    },
  },
  apply(args) {
    const q = str(args.text)?.toLowerCase();
    const note = q ? personalize.notes.find((n) => n.text.toLowerCase().includes(q)) : undefined;
    if (!note) return { response: { error: "No saved note matches that." } };
    personalize.forgetNote(note.id);
    return { block: { kind: "note", id: note.id, text: note.text, removed: true }, response: { forgotten: note.text } };
  },
};

export const answerTools: AnswerTool[] = [profileNote, profileForget, ...renderTools];
