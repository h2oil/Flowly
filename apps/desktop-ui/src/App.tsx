// Flowly web app: script library + importer + the prompter pill.
// Two views. The library manages scripts (imported files, pastes, edits —
// stored locally, nothing uploaded). Launching a take compiles the script in
// the WASM engine and drives the pill from live voice (Chrome/Edge speech
// recognition) or the scripted demo take.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { compileTokens, LiveEngine, speechRecognitionAvailable, type ScriptToken } from "./engine/liveEngine";
import { SimEngine } from "./engine/simEngine";
import { importFile, importPaste } from "./importer/import";
import { loadScripts, newId, removeScript, upsertScript, wordCount, type LibScript } from "./library";
import { Pill, type PromptEngine } from "./pill/Pill";

type View = { kind: "library" } | { kind: "prompt"; script: LibScript; mode: "voice" | "demo" };

export default function App() {
  const [scripts, setScripts] = useState<LibScript[]>(() => loadScripts());
  const [view, setView] = useState<View>({ kind: "library" });
  const [notice, setNotice] = useState<string | null>(null);

  const flash = useCallback((msg: string) => {
    setNotice(msg);
    setTimeout(() => setNotice(null), 5000);
  }, []);

  return (
    <div className="desktop">
      <div className="camera-hint">webcam</div>
      {view.kind === "prompt" ? (
        <Prompter
          script={view.script}
          mode={view.mode}
          onExit={() => setView({ kind: "library" })}
          onError={(m) => {
            flash(m);
            setView({ kind: "library" });
          }}
        />
      ) : (
        <div className="pill pill--idle" onClick={() => {}} style={{ pointerEvents: "none", opacity: 0.4 }}>
          <div className="pill__idle">
            <span className="pill__glyph" />
            <span className="pill__title">Flowly</span>
          </div>
        </div>
      )}
      {notice && <div className="notice">{notice}</div>}
      {view.kind === "library" && (
        <Library
          scripts={scripts}
          setScripts={setScripts}
          onLaunch={(script, mode) => setView({ kind: "prompt", script, mode })}
          flash={flash}
        />
      )}
    </div>
  );
}

// ---- prompter -------------------------------------------------------------

function Prompter({
  script,
  mode,
  onExit,
  onError,
}: {
  script: LibScript;
  mode: "voice" | "demo";
  onExit: () => void;
  onError: (message: string) => void;
}) {
  const [ready, setReady] = useState<{ tokens: ScriptToken[]; engine: PromptEngine } | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const tokens = await compileTokens(script.body);
        if (tokens.length === 0) {
          onError("This script has no speakable text.");
          return;
        }
        let engine: PromptEngine;
        if (mode === "voice") {
          const live = new LiveEngine(script.body);
          live.onError = onError;
          await live.prepare();
          engine = live;
        } else {
          engine = new SimEngine(tokens);
        }
        if (!cancelled) setReady({ tokens, engine });
      } catch (e) {
        onError(`Couldn't start the take: ${e instanceof Error ? e.message : String(e)}`);
      }
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [script.id, mode]);

  if (!ready) return <div className="pill pill--countdown">…</div>;
  return <Pill tokens={ready.tokens} engine={ready.engine} live={mode === "voice"} onExit={onExit} />;
}

// ---- library ---------------------------------------------------------------

function Library({
  scripts,
  setScripts,
  onLaunch,
  flash,
}: {
  scripts: LibScript[];
  setScripts: (s: LibScript[]) => void;
  onLaunch: (script: LibScript, mode: "voice" | "demo") => void;
  flash: (msg: string) => void;
}) {
  const [editing, setEditing] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const fileInput = useRef<HTMLInputElement>(null);
  const voiceOk = useMemo(() => speechRecognitionAvailable(), []);

  const addImported = useCallback(
    (title: string, body: string, warning?: string) => {
      const next = upsertScript({ id: newId(), title, body });
      setScripts(next);
      flash(warning ? `Imported "${title}" — ${warning}` : `Imported "${title}".`);
    },
    [setScripts, flash]
  );

  const handleFiles = useCallback(
    async (files: FileList | File[]) => {
      for (const file of Array.from(files)) {
        try {
          const r = await importFile(file);
          addImported(r.title, r.body, r.warning);
        } catch (e) {
          flash(`${file.name}: ${e instanceof Error ? e.message : String(e)}`);
        }
      }
    },
    [addImported, flash]
  );

  return (
    <div className="library">
      <header className="library__head">
        <h1>Scripts</h1>
        <p className="library__sub">
          Import from Word (.docx), PDF, text, Markdown, or RTF — or paste straight from Google Docs
          and Notion. Everything stays in this browser.
        </p>
      </header>

      <div
        className={`dropzone${dragOver ? " dropzone--over" : ""}`}
        onDragOver={(e) => {
          e.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onDrop={(e) => {
          e.preventDefault();
          setDragOver(false);
          void handleFiles(e.dataTransfer.files);
        }}
      >
        <button className="btn" onClick={() => fileInput.current?.click()}>
          Import a file
        </button>
        <span>or drop .docx / .pdf / .txt / .md / .rtf / .html here</span>
        <input
          ref={fileInput}
          type="file"
          accept=".txt,.md,.markdown,.docx,.pdf,.rtf,.html,.htm"
          multiple
          hidden
          data-testid="file-input"
          onChange={(e) => {
            if (e.target.files) void handleFiles(e.target.files);
            e.target.value = "";
          }}
        />
      </div>

      <textarea
        className="pastezone"
        rows={2}
        placeholder="…or paste your script here (headings and bold survive from Docs/Notion/Word)"
        data-testid="paste-zone"
        onPaste={(e) => {
          e.preventDefault();
          const r = importPaste(
            e.clipboardData.getData("text/plain"),
            e.clipboardData.getData("text/html") || undefined
          );
          if (r.body.trim()) addImported(r.title, r.body);
          else flash("Nothing readable in that paste.");
          (e.target as HTMLTextAreaElement).value = "";
        }}
      />

      <ul className="scripts" data-testid="script-list">
        {scripts.map((s) => (
          <li key={s.id} className="script">
            {editing === s.id ? (
              <ScriptEditor
                script={s}
                onSave={(title, body) => {
                  setScripts(upsertScript({ id: s.id, title, body }));
                  setEditing(null);
                }}
                onCancel={() => setEditing(null)}
              />
            ) : (
              <>
                <div className="script__meta">
                  <span className="script__title">{s.title}</span>
                  <span className="script__count">{wordCount(s.body)} words</span>
                </div>
                <div className="script__actions">
                  <button
                    className="btn btn--primary"
                    disabled={!voiceOk}
                    title={voiceOk ? "Follow your voice (needs mic)" : "Needs Chrome/Edge with mic access"}
                    onClick={() => onLaunch(s, "voice")}
                  >
                    ● Start with voice
                  </button>
                  <button className="btn" onClick={() => onLaunch(s, "demo")}>
                    ▶ Demo take
                  </button>
                  <button className="btn btn--quiet" onClick={() => setEditing(s.id)}>
                    Edit
                  </button>
                  <button
                    className="btn btn--quiet"
                    onClick={() => setScripts(removeScript(s.id))}
                    title="Delete script"
                  >
                    Delete
                  </button>
                </div>
              </>
            )}
          </li>
        ))}
      </ul>
      {!voiceOk && (
        <p className="library__foot">
          Live voice following needs Chrome or Edge (browser speech recognition). The demo take works
          everywhere. On Windows the shipping app listens on-device — no browser, no cloud.
        </p>
      )}
    </div>
  );
}

function ScriptEditor({
  script,
  onSave,
  onCancel,
}: {
  script: LibScript;
  onSave: (title: string, body: string) => void;
  onCancel: () => void;
}) {
  const [title, setTitle] = useState(script.title);
  const [body, setBody] = useState(script.body);
  return (
    <div className="editor">
      <input className="editor__title" value={title} onChange={(e) => setTitle(e.target.value)} />
      <textarea
        className="editor__body"
        rows={10}
        value={body}
        onChange={(e) => setBody(e.target.value)}
        spellCheck={false}
      />
      <div className="script__actions">
        <button className="btn btn--primary" onClick={() => onSave(title.trim() || "Untitled", body)}>
          Save
        </button>
        <button className="btn btn--quiet" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}
