// The 3-line focus window (docs/plan/04-ux-spec.md §S2): line 2 is the live
// line with a word-level highlight pill (the dyslexia-friendly default), line
// 1 is just-spoken text at 40% opacity, line 3 upcoming at 75%. Lines advance
// as discrete 180ms steps — the eye tolerates steps far better than a
// continuous crawl — and on loss of lock the text simply holds still.

import { useMemo } from "react";
import type { EngineState } from "../engine/types";

export const WORDS_PER_LINE = 7;
const LINE_HEIGHT = 30; // px

export function FocusWindow({
  words,
  position,
  state,
}: {
  words: string[];
  position: number;
  state: EngineState;
}) {
  const lines = useMemo(() => {
    const out: { text: string; start: number }[] = [];
    for (let i = 0; i < words.length; i += WORDS_PER_LINE) {
      out.push({ text: "", start: i });
    }
    return out.map((l) => ({ ...l, text: words.slice(l.start, l.start + WORDS_PER_LINE).join(" ") }));
  }, [words]);

  const current = Math.max(0, Math.min(Math.floor(position), words.length - 1));
  const currentLine = Math.floor(current / WORDS_PER_LINE);
  // Keep the live line centered: show lines currentLine-1 .. currentLine+1.
  const offset = -(currentLine - 1) * LINE_HEIGHT;

  return (
    <div className="focus" style={{ height: LINE_HEIGHT * 3 }}>
      <div className="focus__scroller" style={{ transform: `translateY(${offset}px)` }}>
        {lines.map((line, li) => (
          <div
            key={line.start}
            className={
              "focus__line" +
              (li < currentLine ? " focus__line--past" : li > currentLine ? " focus__line--next" : " focus__line--live")
            }
            style={{ height: LINE_HEIGHT }}
          >
            {words.slice(line.start, line.start + WORDS_PER_LINE).map((w, wi) => {
              const idx = line.start + wi;
              const cls =
                idx < current
                  ? "w w--past"
                  : idx === current
                    ? `w w--live${state !== "tracking" ? " w--held" : ""}`
                    : "w w--next";
              return (
                <span key={idx} className={cls}>
                  {w}{" "}
                </span>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}
