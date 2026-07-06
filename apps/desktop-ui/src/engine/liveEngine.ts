// The live engine: the real Rust aligner (compiled to WASM) fed by the
// browser's SpeechRecognition. This is a demo rig for the shipping Windows
// pipeline — same aligner, same event contract; only the recognizer differs
// (Chrome's speech API instead of on-device sherpa-onnx, so audio goes to the
// browser vendor's recognizer here; the product path stays on-device).

import initWasm, { compile_tokens, WasmAligner } from "../wasm/flowly_wasm";
import wasmUrl from "../wasm/flowly_wasm_bg.wasm?inline";
import type { EngineEvent, EngineFeed, EngineListener, EngineState } from "./types";

let wasmReady: Promise<unknown> | null = null;

export function ensureWasm(): Promise<unknown> {
  if (!wasmReady) wasmReady = initWasm({ module_or_path: wasmUrl });
  return wasmReady;
}

export interface ScriptToken {
  display: string;
  line: number;
  sentenceStart: boolean;
  paragraphStart: boolean;
}

export async function compileTokens(source: string): Promise<ScriptToken[]> {
  await ensureWasm();
  return JSON.parse(compile_tokens(source)) as ScriptToken[];
}

// Rust's serde enum shape: {"CursorMoved":{"token_index":...}} etc.
type RustEvent =
  | { CursorMoved: { token_index: number; confidence: number; wpm: number } }
  | { StateChanged: { from: string; to: string } }
  | { JumpDetected: { from: number; to: number; kind: string } };

function mapEvents(json: string, atMs: number): EngineEvent[] {
  const out: EngineEvent[] = [];
  for (const e of JSON.parse(json) as RustEvent[]) {
    if ("CursorMoved" in e) {
      out.push({
        kind: "cursorMoved",
        tokenIndex: e.CursorMoved.token_index,
        confidence: e.CursorMoved.confidence,
        wpm: e.CursorMoved.wpm,
        atMs,
      });
    } else if ("StateChanged" in e) {
      out.push({ kind: "stateChanged", to: e.StateChanged.to.toLowerCase() as EngineState, atMs });
    } else if ("JumpDetected" in e) {
      out.push({
        kind: "jumpDetected",
        from: e.JumpDetected.from,
        to: e.JumpDetected.to,
        jump: e.JumpDetected.kind.toLowerCase() as "retake" | "skip",
        atMs,
      });
    }
  }
  return out;
}

type AnySpeechRecognition = {
  new (): SpeechRecognitionLike;
};

interface SpeechRecognitionLike {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  onresult: ((event: SpeechResultEventLike) => void) | null;
  onerror: ((event: { error?: string }) => void) | null;
  onend: (() => void) | null;
  start(): void;
  stop(): void;
}

interface SpeechResultEventLike {
  results: ArrayLike<{ isFinal: boolean; 0: { transcript: string } }>;
}

export function speechRecognitionAvailable(): boolean {
  const w = window as unknown as Record<string, unknown>;
  return Boolean(w.SpeechRecognition ?? w.webkitSpeechRecognition);
}

/** Live voice feed: SpeechRecognition → WASM aligner → engine events. */
export class LiveEngine implements EngineFeed {
  private listeners = new Set<EngineListener>();
  private aligner: WasmAligner | null = null;
  private recognition: SpeechRecognitionLike | null = null;
  private ticker: ReturnType<typeof setInterval> | null = null;
  private startedAt = 0;
  private utteranceStart = 0;
  private active = false;
  onError: ((message: string) => void) | null = null;

  constructor(private source: string) {}

  subscribe(listener: EngineListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  nowMs(): number {
    return performance.now() - this.startedAt;
  }

  async prepare(): Promise<void> {
    await ensureWasm();
    this.aligner = new WasmAligner(this.source);
  }

  start(): void {
    const w = window as unknown as Record<string, unknown>;
    const Ctor = (w.SpeechRecognition ?? w.webkitSpeechRecognition) as AnySpeechRecognition | undefined;
    if (!Ctor || !this.aligner) {
      this.onError?.("Speech recognition is not available in this browser. Use Chrome or Edge.");
      return;
    }
    this.active = true;
    this.startedAt = performance.now();
    this.utteranceStart = 0;

    const rec = new Ctor();
    this.recognition = rec;
    rec.continuous = true;
    rec.interimResults = true;
    rec.lang = "en-US";

    rec.onresult = (event) => {
      // Concatenate the session's results into one utterance view; the
      // aligner's stable-prefix logic handles interim revisions.
      const words: string[] = [];
      let sawFinal = false;
      for (let i = 0; i < event.results.length; i++) {
        const r = event.results[i];
        for (const t of r[0].transcript.trim().split(/\s+/)) {
          if (t) words.push(t);
        }
        if (r.isFinal) sawFinal = true;
      }
      if (words.length === 0) return;
      const now = this.nowMs();
      if (this.utteranceStart === 0) this.utteranceStart = now;
      // Approximate per-word times by spreading the utterance evenly — the
      // engine only uses them for WPM estimation.
      const span = Math.max(now - this.utteranceStart, words.length * 120);
      const step = span / words.length;
      const hyp = {
        words: words.map((text, i) => ({
          text,
          t_start: Math.round(this.utteranceStart + i * step),
          t_end: Math.round(this.utteranceStart + (i + 1) * step),
          stable: false,
        })),
        is_final: sawFinal,
        t_emitted: Math.round(now),
      };
      this.emit(this.aligner!.feed(JSON.stringify(hyp), now), now);
      if (sawFinal) this.utteranceStart = 0;
    };
    rec.onerror = (e) => {
      const err = e.error ?? "unknown";
      if (err === "not-allowed" || err === "service-not-allowed") {
        this.active = false;
        this.onError?.("Microphone access was blocked. Allow the mic and try again.");
      }
      // "no-speech" and "aborted" are routine; onend restarts us.
    };
    rec.onend = () => {
      // Chrome stops on long silence; keep listening while a take is active.
      if (this.active) {
        try {
          rec.start();
        } catch {
          /* already started */
        }
      }
    };
    try {
      rec.start();
    } catch {
      this.onError?.("Could not start the microphone.");
      return;
    }

    this.ticker = setInterval(() => {
      if (!this.aligner) return;
      const now = this.nowMs();
      this.emit(this.aligner.tick(now), now);
    }, 400);
  }

  stop(): void {
    this.active = false;
    this.recognition?.stop();
    this.recognition = null;
    if (this.ticker) clearInterval(this.ticker);
    this.ticker = null;
  }

  /** Manual override (restart sentence, nudge): always wins. */
  jumpTo(tokenIndex: number): void {
    if (!this.aligner) return;
    const now = this.nowMs();
    this.emit(this.aligner.jump_to(tokenIndex, now), now);
  }

  private emit(eventsJson: string, atMs: number): void {
    for (const e of mapEvents(eventsJson, atMs)) {
      for (const l of this.listeners) l(e);
    }
  }
}
