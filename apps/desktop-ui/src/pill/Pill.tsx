// The island. One morphing surface (docs/plan/04-ux-spec.md): countdown →
// active prompting → hover-expanded → done. The engine behind it is
// interchangeable — live voice (WASM aligner + speech recognition) or the
// scripted simulator — and the pill never invents positions: it only ever
// animates to wherever the engine says.

import { useCallback, useEffect, useRef, useState } from "react";
import { isTauri, nudgeWindow } from "../engine/tauriEngine";
import type { ScriptToken } from "../engine/liveEngine";
import { ScrollController } from "../engine/scrollController";
import type { EngineFeed, EngineState } from "../engine/types";
import { FocusWindow } from "./FocusWindow";

export interface PromptEngine extends EngineFeed {
  nowMs(): number;
  jumpTo?(tokenIndex: number): void;
}

type Phase = "countdown" | "active" | "done";

export function Pill({
  tokens,
  engine,
  live,
  onExit,
}: {
  tokens: ScriptToken[];
  engine: PromptEngine;
  live: boolean;
  onExit: () => void;
}) {
  const [phase, setPhase] = useState<Phase>("countdown");
  const [hovered, setHovered] = useState(false);
  const [count, setCount] = useState(3);
  const [engineState, setEngineState] = useState<EngineState>("tracking");
  const [position, setPosition] = useState(0);
  const [wpm, setWpm] = useState(0);
  const [hiddenFromCapture, setHiddenFromCapture] = useState(true);

  const controllerRef = useRef<ScrollController>();
  const rafRef = useRef<number>();
  const words = tokens.map((t) => t.display);

  useEffect(() => {
    let cancelled = false;
    setCount(3);
    let n = 3;
    const beat = setInterval(() => {
      n -= 1;
      if (cancelled) return;
      if (n > 0) {
        setCount(n);
        return;
      }
      clearInterval(beat);
      const controller = new ScrollController();
      controllerRef.current = controller;
      engine.subscribe((e) => {
        controller.onEvent(e);
        if (e.kind === "stateChanged") setEngineState(e.to);
        if (e.kind === "cursorMoved" && e.wpm > 0) setWpm(Math.round(e.wpm));
      });
      setPhase("active");
      engine.start();
      let last = performance.now();
      const frame = (nowRaw: number) => {
        if (cancelled) return;
        const dt = nowRaw - last;
        last = nowRaw;
        const pos = controller.tick(engine.nowMs(), dt);
        setPosition(pos);
        if (pos >= tokens.length - 1.05) {
          engine.stop();
          setPhase("done");
          setTimeout(() => !cancelled && onExit(), 3500);
          return;
        }
        rafRef.current = requestAnimationFrame(frame);
      };
      rafRef.current = requestAnimationFrame(frame);
    }, 1000);
    return () => {
      cancelled = true;
      clearInterval(beat);
      if (rafRef.current !== undefined) cancelAnimationFrame(rafRef.current);
      engine.stop();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [engine]);

  const restartSentence = useCallback(() => {
    const c = controllerRef.current;
    if (!c) return;
    let i = Math.min(Math.floor(c.position), tokens.length - 1);
    while (i > 0 && !tokens[i].sentenceStart) i--;
    if (engine.jumpTo) {
      engine.jumpTo(i);
    } else {
      c.onEvent({ kind: "cursorMoved", tokenIndex: i, confidence: 1, wpm: c.currentWpm, atMs: engine.nowMs() });
    }
  }, [engine, tokens]);

  const exit = useCallback(() => {
    engine.stop();
    onExit();
  }, [engine, onExit]);

  const expanded = hovered && phase === "active";
  const cls = ["pill", `pill--${phase}`, expanded ? "pill--expanded" : "", `pill--${engineState}`]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={cls} onMouseEnter={() => setHovered(true)} onMouseLeave={() => setHovered(false)}>
      {phase === "countdown" && (
        <div className="pill__countdown" key={count}>
          {count}
        </div>
      )}
      {(phase === "active" || phase === "done") && (
        <>
          <div className="pill__stage" data-tauri-drag-region>
            <Waveform state={engineState} done={phase === "done"} live={live} />
            <FocusWindow words={words} position={position} state={engineState} />
            {hiddenFromCapture && <span className="pill__ghost" title="Hidden from screen shares">⌀</span>}
          </div>
          {phase === "done" && <div className="pill__done">✓ End of script</div>}
          {expanded && (
            <div className="pill__controls">
              {isTauri() && (
                <>
                  <button onClick={() => nudgeWindow(-60)} title="Move bubble left">◀</button>
                  <button onClick={() => nudgeWindow(60)} title="Move bubble right">▶</button>
                </>
              )}
              <button onClick={restartSentence} title="Restart sentence (Ctrl+Alt+R)">⟲</button>
              <span className="pill__wpm">{wpm > 0 ? `${wpm} wpm` : "—"}</span>
              <span className={`pill__state pill__state--${engineState}`}>
                {live ? engineState : `${engineState} · demo`}
              </span>
              <button
                className={hiddenFromCapture ? "on" : ""}
                onClick={() => setHiddenFromCapture((v) => !v)}
                title="Hide from screen shares"
              >
                ⌀
              </button>
              <button onClick={exit} title="End take">✕</button>
            </div>
          )}
        </>
      )}
    </div>
  );
}

function Waveform({ state, done, live }: { state: EngineState; done: boolean; live: boolean }) {
  const mood = done ? "done" : state;
  return (
    <div className={`wave wave--${mood} ${live ? "wave--mic" : ""}`} aria-hidden>
      {[0, 1, 2, 3, 4].map((i) => (
        <span key={i} style={{ animationDelay: `${i * 0.13}s` }} />
      ))}
    </div>
  );
}
