// The island. One morphing surface, five states (docs/plan/04-ux-spec.md):
// S0 collapsed idle · S1 armed/countdown · S2 active prompting · S3 hover-
// expanded · (S4 boss-duck arrives with the Tauri shell's global hotkeys).
// All transitions morph this single pill — never a second window popping in.

import { useCallback, useEffect, useRef, useState } from "react";
import { DemoDriver, DEMO_SCRIPT_TITLE, DEMO_WORDS } from "../engine/demoDriver";
import { ScrollController } from "../engine/scrollController";
import type { EngineState } from "../engine/types";
import { FocusWindow, WORDS_PER_LINE } from "./FocusWindow";

type Phase = "idle" | "countdown" | "active" | "done";

export function Pill() {
  const [phase, setPhase] = useState<Phase>("idle");
  const [hovered, setHovered] = useState(false);
  const [count, setCount] = useState(3);
  const [engineState, setEngineState] = useState<EngineState>("tracking");
  const [position, setPosition] = useState(0);
  const [wpm, setWpm] = useState(0);
  const [hiddenFromCapture, setHiddenFromCapture] = useState(true);

  const driverRef = useRef<DemoDriver>();
  const controllerRef = useRef<ScrollController>();
  const rafRef = useRef<number>();

  const stopEverything = useCallback(() => {
    driverRef.current?.stop();
    if (rafRef.current !== undefined) cancelAnimationFrame(rafRef.current);
  }, []);

  useEffect(() => stopEverything, [stopEverything]);

  const begin = useCallback(() => {
    setPhase("countdown");
    setCount(3);
    let n = 3;
    const beat = setInterval(() => {
      n -= 1;
      if (n > 0) {
        setCount(n);
      } else {
        clearInterval(beat);
        startPrompting();
      }
    }, 1000);
  }, []);

  const startPrompting = useCallback(() => {
    const driver = new DemoDriver();
    const controller = new ScrollController();
    driverRef.current = driver;
    controllerRef.current = controller;
    driver.subscribe((e) => {
      controller.onEvent(e);
      if (e.kind === "stateChanged") setEngineState(e.to);
      if (e.kind === "cursorMoved" && e.wpm > 0) setWpm(Math.round(e.wpm));
    });
    setEngineState("tracking");
    setPhase("active");
    driver.start();

    let last = performance.now();
    const frame = (nowRaw: number) => {
      const dt = nowRaw - last;
      last = nowRaw;
      const pos = controller.tick(driver.nowMs(), dt);
      setPosition(pos);
      if (pos >= DEMO_WORDS.length - 1.05) {
        setPhase("done");
        setTimeout(() => setPhase("idle"), 4000);
        return;
      }
      rafRef.current = requestAnimationFrame(frame);
    };
    rafRef.current = requestAnimationFrame(frame);
  }, []);

  const restartSentence = useCallback(() => {
    // Manual override always wins — mirrors Aligner::jump_to.
    const c = controllerRef.current;
    if (!c) return;
    let i = Math.floor(c.position);
    while (i > 0 && !/[.!?]$/.test(DEMO_WORDS[i - 1])) i--;
    c.onEvent({ kind: "cursorMoved", tokenIndex: i, confidence: 1, wpm: c.currentWpm, atMs: driverRef.current?.nowMs() ?? 0 });
  }, []);

  const expanded = hovered && phase === "active";
  const cls = ["pill", `pill--${phase}`, expanded ? "pill--expanded" : "", `pill--${engineState}`]
    .filter(Boolean)
    .join(" ");

  return (
    <div
      className={cls}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={phase === "idle" ? begin : undefined}
      role={phase === "idle" ? "button" : undefined}
      title={phase === "idle" ? "Start the demo take" : undefined}
    >
      {phase === "idle" && (
        <div className="pill__idle">
          <span className="pill__glyph" />
          <span className="pill__title">{DEMO_SCRIPT_TITLE}</span>
        </div>
      )}

      {phase === "countdown" && (
        <div className="pill__countdown" key={count}>
          {count}
        </div>
      )}

      {(phase === "active" || phase === "done") && (
        <>
          <div className="pill__stage">
            <Waveform state={engineState} done={phase === "done"} />
            <FocusWindow words={DEMO_WORDS} position={position} state={engineState} />
            {hiddenFromCapture && <span className="pill__ghost" title="Hidden from screen shares">⌀</span>}
          </div>
          {phase === "done" && <div className="pill__done">✓ End of script</div>}
          {expanded && (
            <div className="pill__controls">
              <button onClick={restartSentence} title="Restart sentence (Ctrl+Alt+R)">⟲</button>
              <span className="pill__wpm">{wpm > 0 ? `${wpm} wpm` : "—"}</span>
              <span className={`pill__state pill__state--${engineState}`}>{engineState}</span>
              <button
                className={hiddenFromCapture ? "on" : ""}
                onClick={() => setHiddenFromCapture((v) => !v)}
                title="Hide from screen shares"
              >
                ⌀
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}

function Waveform({ state, done }: { state: EngineState; done: boolean }) {
  const mood = done ? "done" : state;
  return (
    <div className={`wave wave--${mood}`} aria-hidden>
      {[0, 1, 2, 3, 4].map((i) => (
        <span key={i} style={{ animationDelay: `${i * 0.13}s` }} />
      ))}
    </div>
  );
}

export { WORDS_PER_LINE };
