// Browser script library (localStorage). The Windows build uses the SQLite
// store (crates/flowly-store) behind the same shape; this keeps the web demo
// fully client-side — scripts never leave the machine.

export interface LibScript {
  id: string;
  title: string;
  body: string;
  updatedAt: number;
}

const KEY = "flowly.scripts.v1";

export const SAMPLE_SCRIPT = `# Flowly launch demo

Welcome back everyone, and thanks for joining the Flowly launch demo today. //pause

We built a teleprompter that actually **listens**. As you speak, the text
moves with your voice — through pauses, stumbles, and retakes. (( smile ))

[SLOW] Here are the numbers that matter. Revenue reached $3.5M this quarter,
growing 47% year over year, with 1,200 teams onboarded since 2024.

# Closing

If you present from a Windows machine, Flowly keeps your eyes on the lens
and your script out of the screen share. Thanks for watching — goodbye!
`;

export function loadScripts(): LibScript[] {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) return JSON.parse(raw) as LibScript[];
  } catch {
    /* corrupted storage: fall through to seed */
  }
  const seed: LibScript[] = [
    { id: "sample", title: "Flowly launch demo", body: SAMPLE_SCRIPT, updatedAt: Date.now() },
  ];
  persist(seed);
  return seed;
}

export function upsertScript(script: Omit<LibScript, "updatedAt">): LibScript[] {
  const all = loadScripts().filter((s) => s.id !== script.id);
  all.unshift({ ...script, updatedAt: Date.now() });
  persist(all);
  return all;
}

export function removeScript(id: string): LibScript[] {
  const all = loadScripts().filter((s) => s.id !== id);
  persist(all);
  return all;
}

export function newId(): string {
  return `s-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

function persist(scripts: LibScript[]) {
  try {
    localStorage.setItem(KEY, JSON.stringify(scripts));
  } catch {
    /* storage full/blocked: library becomes session-only */
  }
}

export function wordCount(body: string): number {
  return body
    .replace(/\(\([\s\S]*?\)\)/g, " ")
    .replace(/\[[^\]]*\]/g, " ")
    .replace(/^#.*$/gm, " ")
    .split(/\s+/)
    .filter(Boolean).length;
}
