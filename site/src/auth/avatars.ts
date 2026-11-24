// Profile pictures: nine animals, or the photo from Google or Discord. The
// database enforces the same rules (supabase/social.sql), so this list and
// that check have to agree. Mirrored in app/src/lib/avatars.ts.
import panda from "../assets/avatars/panda.webp";
import owl from "../assets/avatars/owl.webp";
import rabbit from "../assets/avatars/rabbit.webp";
import chameleon from "../assets/avatars/chameleon.webp";
import octopus from "../assets/avatars/octopus.webp";
import dragon from "../assets/avatars/dragon.webp";
import tiger from "../assets/avatars/tiger.webp";
import wolf from "../assets/avatars/wolf.webp";
import cat from "../assets/avatars/cat.webp";

export interface Animal {
  id: string;
  label: string;
  /** The flat colour behind the drawing, for rings and placeholders. */
  color: string;
  src: string;
}

export const ANIMALS: Animal[] = [
  { id: "animal:panda", label: "Panda", color: "#35caa3", src: panda },
  { id: "animal:owl", label: "Owl", color: "#7651f9", src: owl },
  { id: "animal:rabbit", label: "Rabbit", color: "#deb90e", src: rabbit },
  { id: "animal:chameleon", label: "Chameleon", color: "#04aba4", src: chameleon },
  { id: "animal:octopus", label: "Octopus", color: "#fc4185", src: octopus },
  { id: "animal:dragon", label: "Dragon", color: "#db4a2b", src: dragon },
  { id: "animal:tiger", label: "Tiger", color: "#fb7e20", src: tiger },
  { id: "animal:wolf", label: "Wolf", color: "#3f5c8d", src: wolf },
  { id: "animal:cat", label: "Cat", color: "#9bc933", src: cat },
];

/** The animal for an avatar value. Old fox presets ("preset:N") map onto the animals. */
export function animalOf(avatar: string | null | undefined): Animal | null {
  if (!avatar) return null;
  const hit = ANIMALS.find((a) => a.id === avatar);
  if (hit) return hit;
  const old = /^preset:(\d{1,2})$/.exec(avatar);
  return old ? ANIMALS[(Number(old[1]) - 1) % ANIMALS.length] : null;
}

/** A random animal, for a new profile. */
export const randomAnimal = () => ANIMALS[Math.floor(Math.random() * ANIMALS.length)].id;

/** Only photos from these hosts are accepted as an avatar. */
export const isPhotoUrl = (u: unknown): u is string =>
  typeof u === "string" && u.length <= 450 && /^https:\/\/(lh3\.googleusercontent\.com|cdn\.discordapp\.com)\/\S+$/.test(u);
