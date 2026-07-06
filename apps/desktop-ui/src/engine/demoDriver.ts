// Scripted fake engine feed: simulates a presenter reading the demo script,
// with the exact event semantics the Rust aligner produces — ASR-latency
// batched cursor updates, a thinking pause (PAUSED), an ad-lib excursion
// (LOST: cursor freezes, then re-anchors), and a retake (jump back to the
// sentence start). This is the plan's M2 device: the whole pill UI is built
// and demoable with zero live audio, and the Tauri bridge later swaps in
// behind the same EngineFeed interface.

import type { EngineEvent, EngineFeed, EngineListener, EngineState } from "./types";

export const DEMO_SCRIPT_TITLE = "Flowly launch demo";

export const DEMO_WORDS: string[] = (
  "Welcome back everyone, and thanks for joining the Flowly launch demo today. " +
  "We built a teleprompter that actually listens. " +
  "As you speak, the text moves with your voice through pauses, stumbles, and retakes. " +
  "Here are the numbers that matter. " +
  "Revenue reached three point five million dollars this quarter, growing forty seven percent year over year. " +
  "If you present from a Windows machine, Flowly keeps your eyes on the lens " +
  "and your script out of the screen share. Thanks for watching!"
).split(/\s+/);

interface Beat {
  afterMs: number;
  events: EngineEvent[];
}

/** Index of the first word of the sentence containing `i`. */
function sentenceStart(i: number): number {
  for (let j = i; j > 0; j--) {
    if (/[.!?]$/.test(DEMO_WORDS[j - 1])) return j;
  }
  return 0;
}

function buildTimeline(): Beat[] {
  const beats: Beat[] = [];
  const wordMs = 320; // ~190 WPM
  const wpm = 60000 / wordMs;
  let t = 0;
  let cursor = 0;

  const moveTo = (idx: number, confidence = 0.95) => {
    cursor = idx;
    beats.push({
      afterMs: t,
      events: [{ kind: "cursorMoved", tokenIndex: idx, confidence, wpm, atMs: t }],
    });
  };
  const state = (to: EngineState) => {
    beats.push({ afterMs: t, events: [{ kind: "stateChanged", to, atMs: t }] });
  };

  // Engine-style batching: a cursor fix every ~3 words (ASR chunk cadence).
  const read = (through: number) => {
    while (cursor < through) {
      const next = Math.min(cursor + 3, through);
      t += (next - cursor) * wordMs;
      moveTo(next);
    }
  };

  const pauseIdx = 12; // after "...demo today."
  const adlibIdx = 19; // after "...actually listens."
  const retakeFrom = 42; // mid numbers sentence
  const end = DEMO_WORDS.length - 1;

  read(pauseIdx);
  // Thinking pause: silence >2 s → PAUSED, cursor frozen.
  t += 2200;
  state("paused");
  t += 1400;
  state("tracking");

  read(adlibIdx);
  // Ad-lib: goes LOST, display holds still, amber pulse.
  t += 900;
  state("lost");
  t += 3800; // off-script chatter
  state("tracking");
  moveTo(adlibIdx + 2, 0.8); // re-anchor slightly ahead

  read(retakeFrom);
  // Retake: presenter restarts the numbers sentence.
  t += 700;
  const back = sentenceStart(retakeFrom - 2);
  beats.push({
    afterMs: t,
    events: [
      { kind: "jumpDetected", from: cursor, to: back, jump: "retake", atMs: t },
      { kind: "cursorMoved", tokenIndex: back, confidence: 0.9, wpm, atMs: t },
    ],
  });
  cursor = back;

  read(end);
  return beats;
}

export class DemoDriver implements EngineFeed {
  private listeners = new Set<EngineListener>();
  private timers: ReturnType<typeof setTimeout>[] = [];
  /** Engine-clock origin (performance.now at start). */
  startedAt = 0;

  subscribe(listener: EngineListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  start() {
    this.stop();
    this.startedAt = performance.now();
    for (const beat of buildTimeline()) {
      this.timers.push(
        setTimeout(() => {
          for (const e of beat.events) {
            for (const l of this.listeners) l(e);
          }
        }, beat.afterMs)
      );
    }
  }

  stop() {
    for (const t of this.timers) clearTimeout(t);
    this.timers = [];
  }

  /** Engine clock for the scroll controller's dead reckoning. */
  nowMs(): number {
    return performance.now() - this.startedAt;
  }
}
