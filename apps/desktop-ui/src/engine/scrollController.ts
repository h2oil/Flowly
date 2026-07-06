// The scroll controller: a critically damped spring toward the engine cursor,
// with WPM dead-reckoning between engine updates (docs/plan/03, §scroll
// controller). Motion lives UI-side at requestAnimationFrame rate so the
// engine can emit lazily (≤30 Hz) while perceived motion is 60–120 fps.
//
// Invariants:
// - Dead-reckoning is HARD-CAPPED at +8 words past the last confirmed token —
//   speculative motion may never run ahead of truth.
// - LOST/SEARCHING/PAUSED freeze the drift entirely; the display holds still.
// - Re-anchor jumps (retake/skip) snap the spring target; the spring itself
//   keeps the perceived motion continuous.

import type { EngineEvent, EngineState } from "./types";

const DEAD_RECKON_CAP_WORDS = 8;
// Critically damped: zeta = 1, omega ≈ 2π·1.2 Hz.
const OMEGA = 2 * Math.PI * 1.2;

export class ScrollController {
  /** Continuous position in token units — what the renderer draws. */
  position = 0;
  private velocity = 0;
  private confirmed = 0;
  private confirmedAtMs = 0;
  private wpm = 0;
  private state: EngineState = "tracking";
  private started = false;

  /** While not TRACKING the display freezes exactly where it is — retracting
   * speculative drift back to the confirmed token reads as a glitch. */
  private hold: number | null = null;

  onEvent(e: EngineEvent) {
    switch (e.kind) {
      case "cursorMoved":
        if (!this.started) {
          // First fix: place the text instantly, no glide from token 0.
          this.position = e.tokenIndex;
          this.started = true;
        }
        this.confirmed = e.tokenIndex;
        this.confirmedAtMs = e.atMs;
        if (e.wpm > 0) this.wpm = e.wpm;
        // A fresh engine fix releases any hold — including the re-anchor
        // after LOST/SEARCHING, which the spring animates as one glide.
        this.hold = null;
        break;
      case "stateChanged":
        this.state = e.to;
        this.hold = e.to === "tracking" ? this.hold : this.position;
        break;
      case "jumpDetected":
        // The spring animates the snap; nothing else to do.
        break;
    }
  }

  /** Advance the simulation. `nowMs` is the engine clock, `dtMs` frame time. */
  tick(nowMs: number, dtMs: number): number {
    const dt = Math.min(dtMs, 100) / 1000;

    let target: number;
    if (this.hold !== null) {
      target = this.hold;
    } else {
      target = this.confirmed;
      if (this.state === "tracking" && this.started && this.wpm > 0) {
        const driftWords = ((nowMs - this.confirmedAtMs) / 60000) * this.wpm;
        target = this.confirmed + Math.min(driftWords, DEAD_RECKON_CAP_WORDS);
      }
    }

    // Critically damped spring (no overshoot, glides through corrections).
    const displacement = this.position - target;
    const accel = -OMEGA * OMEGA * displacement - 2 * OMEGA * this.velocity;
    this.velocity += accel * dt;
    this.position += this.velocity * dt;
    if (Math.abs(this.position - target) < 0.01 && Math.abs(this.velocity) < 0.01) {
      this.position = target;
      this.velocity = 0;
    }
    return this.position;
  }

  get engineState(): EngineState {
    return this.state;
  }

  get currentWpm(): number {
    return this.wpm;
  }
}
