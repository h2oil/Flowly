// Tauri bridge: when the app runs inside the Windows shell, voice takes are
// driven by the NATIVE pipeline (WASAPI shared-mode capture → on-device Vosk
// → the Rust aligner) and arrive here as engine events. Same event contract
// as the WASM/browser path; only the transport differs.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { mapEvents } from "./liveEngine";
import type { EngineFeed, EngineListener } from "./types";

export function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export function listMics(): Promise<string[]> {
  return invoke<string[]>("list_mics");
}

export function preloadModel(): Promise<void> {
  return invoke("preload_model");
}

export function nudgeWindow(dx: number): void {
  void invoke("nudge", { dx });
}

export function resizeForTake(take: boolean): void {
  void invoke("resize_for_take", { take });
}

export class TauriEngine implements EngineFeed {
  private listeners = new Set<EngineListener>();
  private unlisten: UnlistenFn[] = [];
  private startedAt = 0;
  onError: ((message: string) => void) | null = null;

  constructor(
    private source: string,
    private mic: string | null
  ) {}

  subscribe(listener: EngineListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  nowMs(): number {
    return performance.now() - this.startedAt;
  }

  async prepare(): Promise<void> {
    await preloadModel();
    this.unlisten.push(
      await listen<unknown>("engine-event", (e) => {
        const atMs = this.nowMs();
        for (const ev of mapEvents(JSON.stringify(e.payload), atMs)) {
          for (const l of this.listeners) l(ev);
        }
      })
    );
    this.unlisten.push(
      await listen<string>("engine-error", (e) => {
        this.onError?.(e.payload);
      })
    );
  }

  start(): void {
    this.startedAt = performance.now();
    invoke("start_session", { script: this.source, mic: this.mic }).catch((e) =>
      this.onError?.(String(e))
    );
  }

  stop(): void {
    void invoke("stop_session");
    for (const u of this.unlisten) u();
    this.unlisten = [];
  }

  jumpTo(tokenIndex: number): void {
    void invoke("jump_to", { tokenIndex });
  }
}
