// Mirrors the Rust engine's event surface (crates/flowly-align). In the Tauri
// shell these arrive over the event channel; in the browser demo they come
// from the DemoDriver. The UI never invents positions — it only animates to
// wherever the engine says (docs/plan/04-ux-spec.md).

export type EngineState = "tracking" | "lost" | "searching" | "paused";

export interface CursorMoved {
  kind: "cursorMoved";
  tokenIndex: number;
  confidence: number;
  wpm: number;
  atMs: number;
}

export interface StateChanged {
  kind: "stateChanged";
  to: EngineState;
  atMs: number;
}

export interface JumpDetected {
  kind: "jumpDetected";
  from: number;
  to: number;
  jump: "retake" | "skip";
  atMs: number;
}

export type EngineEvent = CursorMoved | StateChanged | JumpDetected;

export type EngineListener = (e: EngineEvent) => void;

/** Anything that produces engine events: the Tauri bridge or the demo driver. */
export interface EngineFeed {
  subscribe(listener: EngineListener): () => void;
  start(): void;
  stop(): void;
}
