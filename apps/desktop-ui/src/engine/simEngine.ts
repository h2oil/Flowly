// Scripted fake engine feed for any compiled script: simulates a presenter
// reading it with the exact event semantics the Rust aligner produces —
// batched cursor fixes, a thinking pause (PAUSED), an ad-lib excursion (LOST:
// cursor freezes then re-anchors), and a retake (jump back to a sentence
// start). Used for the no-microphone demo mode and for UI work, per the
// plan's "UI needs no live audio" track.

import type { ScriptToken } from "./liveEngine";
import type { EngineEvent, EngineFeed, EngineListener, EngineState } from "./types";

interface Beat {
  afterMs: number;
  events: EngineEvent[];
}

function sentenceStartBefore(tokens: ScriptToken[], i: number): number {
  for (let j = Math.min(i, tokens.length - 1); j > 0; j--) {
    if (tokens[j].sentenceStart) return j;
  }
  return 0;
}

function buildTimeline(tokens: ScriptToken[]): Beat[] {
  const beats: Beat[] = [];
  const wordMs = 320; // ~190 WPM
  const wpm = 60000 / wordMs;
  const n = tokens.length;
  let t = 0;
  let cursor = 0;

  const moveTo = (idx: number, confidence = 0.95) => {
    cursor = idx;
    beats.push({ afterMs: t, events: [{ kind: "cursorMoved", tokenIndex: idx, confidence, wpm, atMs: t }] });
  };
  const state = (to: EngineState) => {
    beats.push({ afterMs: t, events: [{ kind: "stateChanged", to, atMs: t }] });
  };
  const read = (through: number) => {
    while (cursor < through) {
      const next = Math.min(cursor + 3, through);
      t += (next - cursor) * wordMs;
      moveTo(next);
    }
  };

  const pauseAt = Math.floor(n * 0.22);
  const adlibAt = Math.floor(n * 0.45);
  const retakeAt = Math.floor(n * 0.75);
  const end = n - 1;

  read(pauseAt);
  t += 2200;
  state("paused");
  t += 1400;
  state("tracking");

  read(adlibAt);
  t += 900;
  state("lost");
  t += 3800;
  state("tracking");
  moveTo(Math.min(adlibAt + 2, end), 0.8);

  read(retakeAt);
  if (retakeAt < end - 4) {
    t += 700;
    const back = sentenceStartBefore(tokens, retakeAt - 2);
    beats.push({
      afterMs: t,
      events: [
        { kind: "jumpDetected", from: cursor, to: back, jump: "retake", atMs: t },
        { kind: "cursorMoved", tokenIndex: back, confidence: 0.9, wpm, atMs: t },
      ],
    });
    cursor = back;
  }
  read(end);
  return beats;
}

export class SimEngine implements EngineFeed {
  private listeners = new Set<EngineListener>();
  private timers: ReturnType<typeof setTimeout>[] = [];
  private startedAt = 0;

  constructor(private tokens: ScriptToken[]) {}

  subscribe(listener: EngineListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  start() {
    this.stop();
    this.startedAt = performance.now();
    for (const beat of buildTimeline(this.tokens)) {
      this.timers.push(
        setTimeout(() => {
          for (const e of beat.events) for (const l of this.listeners) l(e);
        }, beat.afterMs)
      );
    }
  }

  stop() {
    for (const t of this.timers) clearTimeout(t);
    this.timers = [];
  }

  jumpTo(tokenIndex: number) {
    const atMs = this.nowMs();
    for (const l of this.listeners) {
      l({ kind: "cursorMoved", tokenIndex, confidence: 1, wpm: 0, atMs });
    }
  }

  nowMs(): number {
    return performance.now() - this.startedAt;
  }
}
