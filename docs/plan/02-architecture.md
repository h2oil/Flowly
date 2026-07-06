# Architecture

## Design lens

The two existential risks are (1) **voice-following quality** — a cursor that jitters, jumps wrongly, or lags is worse than a manual scroller — and (2) **overlay window fidelity** — transparency, hit-testing, capture exclusion, and multi-monitor DPI on real Windows machines. Everything else (sync, editing, packaging) is well-trodden. The architecture isolates both risks behind narrow, mockable interfaces, builds each as an independently runnable testbed **before** any product glue exists, and schedules them as parallel week-1 spikes with explicit kill/pivot criteria (see [06-roadmap-and-risks.md](06-roadmap-and-risks.md)).

## Stack

**Tauri v2 + Rust core + React/TypeScript in WebView2.** The latency-critical path (WASAPI → VAD → ASR → aligner) must be native and in-process; Tauri hands us the raw HWND for wndproc-level work and `WDA_EXCLUDEFROMCAPTURE`, and a production Tauri overlay has been measured at ~14 MB RAM with <1 % CPU — an order of magnitude below Electron, which matters for an app idling next to a video call all day.

Framework decision details (from the shell research):

- **Window:** `decorations: false, transparent: true, alwaysOnTop: true, skipTaskbar: true`. The pill is a `border-radius: 9999px` div on a transparent page — **never** native DWM corner rounding or `SetWindowRgn` (known Tauri transparency issues on Win10; CSS shapes sidestep all of it and behave identically on Win10/11).
- **Morphing:** never animate window bounds — transparent-window resize shows artifacts (WebView2Feedback #2419) and Win32 resize isn't 60 fps-smooth. Fix the window at max-expanded size (~480×220 logical px) and morph the pill with GPU-composited CSS transforms.
- **Click-through:** ship the field-proven **60 Hz `GetCursorPos` poll** (toggle `set_ignore_cursor_events` based on whether the cursor is inside the pill's reported bounds; measured <1 % CPU) as the day-one default, and treat a `WM_NCHITTEST` wndproc subclass answering `HTTRANSPARENT` outside the pill as the polish upgrade — both live behind one flag so wndproc conflicts with WebView2 message handling can't block the spike.
- **Capture exclusion:** Tauri-native `contentProtected` (= `WDA_EXCLUDEFROMCAPTURE`, enforced at the DWM compositor). Per-session toggle — it also hides the bar from the user's *own* recordings. Pre-2004 Win10 falls back to a black rectangle: detect the build and warn.
- **Acrylic/Mica:** skipped. The real Dynamic Island is near-opaque black (`rgba(10,10,10,0.88)`); Acrylic lags on drag on Win10. CSS translucency over a solid-ish dark base is the look.
- **Electron** was the runner-up (better out-of-box click-through forwarding, JS-team familiarity) but loses on RAM (~10×), N-API/sidecar tax for the native pipeline, and recent content-protection regressions. **WinUI 3 / WPF / Avalonia / Flutter** lose on some mix of animation ergonomics, ecosystem, or native-pipeline friction.

Two corrections adopted from the winning proposal's review of the research:

1. **Bind sherpa-onnx's C API directly from Rust** (`sherpa-rs` if it exposes streaming + hotwords; otherwise a thin in-repo `flowly-sherpa-sys` bindgen crate, ~1 week, which we control). The C API is the stable contract; the C# NuGet path is dropped.
2. **Soft VAD gating:** hard-gating a streaming transducer clips word onsets and corrupts decoder state. Feed **300 ms pre-roll and 500 ms hang-over** around VAD speech segments, and never reset the ASR stream mid-utterance.

## Process & threading model

**One product process** (the Tauri app) plus OS-managed WebView2 renderer processes. No sidecar: the Rust core is in-process, so cursor updates reach the UI over Tauri's event channel in <1 ms.

| Thread | Role | Real-time discipline |
|---|---|---|
| **Audio capture** (cpal callback) | WASAPI shared mode, 10 ms buffers; downmix to mono; push into lock-free SPSC ring buffer | Allocation-free, no locks, no logging. Never blocks. |
| **Speech thread** | Ring buffer → resample 48k→16k → Silero VAD → sherpa-onnx streaming decode → `Hypothesis` events | Owns the ASR session; single-digit % of one core. Restartable without touching capture. |
| **Alignment thread** | Consumes `Hypothesis` events, runs the aligner state machine, emits `CursorUpdate`/`TrackingState`, coalesced ≤30 Hz toward UI | Pure CPU, <1 ms per update; isolated so slow alignment can never back-pressure decoding. |
| **Main/Tauri thread** | Window management, hit-testing, monitor events, commands | UI-rate work only. |
| **Tokio runtime** (2 workers) | Sync engine, model download, telemetry, updater | All I/O; fully async; can be offline forever. |

Device hot-swap: `IMMNotificationClient` fires → tear down and rebuild the capture thread; the ASR stream survives (sees a gap), the aligner sees silence — nothing is lost when a headset is plugged in mid-call.

## Module boundaries (Cargo workspace)

```
crates/
  flowly-script     # Prompter-Markdown parser + compiler → TokenMap {word, variants[], metaphone, charStart, charEnd, flags}
  flowly-align      # THE aligner. Pure. No I/O, no threads, no clocks (time injected). Depends only on flowly-script types.
  flowly-asr        # trait AsrStream + SherpaZipformer impl + ReplayAsr impl (reads recorded hypothesis logs); Deepgram slot reserved, not built
  flowly-audio      # cpal capture, resampler, Silero VAD, device notifications. trait AudioSource (mock = WAV file player)
  flowly-overlay    # HWND ownership: hit-testing (poll + wndproc), WDA_EXCLUDEFROMCAPTURE, monitor enum + per-monitor position persistence
  flowly-store      # rusqlite schema, outbox, migrations
  flowly-sync       # Supabase client, rev-checked push/pull, conflict fork, auth (PKCE + deep link)
  flowly-app        # Tauri glue: commands, event bridging, session orchestration (the only crate that knows about all others)
apps/
  desktop-ui        # React pill UI + settings window
  web               # Editor SPA (Supabase direct) — deliberately minimal for v1
  align-harness     # CLI corpus runner (see 03-voice-following.md). Depends on flowly-{script,align,asr,audio} only.
  overlay-probe     # Minimal Tauri app exercising ONLY flowly-overlay — the permanent compatibility canary
```

### The two load-bearing interfaces

**`AsrStream`** — `push_samples(&[i16])` in; stream of `Hypothesis { words: Vec<HypWord { text, t_start, t_end, stable: bool }>, is_final: bool }` out; `set_hotwords(&[String])` for script biasing. Three impls: sherpa, replay-from-log, cloud-later. The aligner never knows which is behind it.

**`Aligner`** — constructed from a `TokenMap`; `feed(&Hypothesis, now: Instant) -> Vec<AlignerEvent>` where events are `CursorMoved { token_index, confidence }`, `StateChanged { Tracking | Lost | Searching | Paused }`, `JumpDetected { from, to, kind: Retake | Skip }`. **Deterministic given the same input sequence** — this determinism is what makes the regression corpus meaningful.

These IPC/event contracts and the TokenMap source-map schema are **frozen at the end of the M0 spikes**, so the UI track (driven by a scripted fake cursor feed) and the engine track proceed fully in parallel.

### Where the scroll controller lives — a deliberate placement decision

**The aligner emits token positions; the UI owns motion.** The critically damped spring + WPM dead-reckoning runs in TypeScript at requestAnimationFrame rate, fed by `CursorMoved` events and a WPM estimate. Rationale: motion is a rendering concern — it must sync to the compositor frame clock, honor reduce-motion, and drive the karaoke fill — and UI-side motion lets the Rust event rate stay at a lazy ≤30 Hz while perceived motion is 60–120 fps. Dead-reckoning is **hard-capped at +8 words past the last engine-confirmed token** (an explicit invariant, so speculative motion can never overshoot and snap back). The aligner stays a pure, replayable position oracle — exactly what the test harness needs.

## IPC & data flow

- **Rust → UI (event channel):** `cursor` (coalesced ≤30 Hz: token index, confidence, WPM, state), `audio-level` (10 Hz RMS for the waveform), `hit-region-ack`, `sync-status`. All payloads <200 bytes JSON.
- **UI → Rust (commands):** `load_script(id)` — freezes a `script_version`, compiles the TokenMap (cached by `body_hash`), arms aligner + hotwords; `start/stop/pause`; `jump_to(token)`; `set_pill_geometry(rect)` — drives the hit-test region; `set_capture_hidden(bool)`; store CRUD.
- **Coordinate contract:** token index ↔ char offset ↔ display line, all derived from the single TokenMap source map. Take markers, cursor, and rendering share this one coordinate system; nothing else may invent positions.

## Web app

A separate minimal React SPA talking directly to Supabase: sign-in, script list, plain editor with debounced saves. Bit-identical tokenization across web and desktop (WASM-compiled `flowly-script`) is **not** needed until the web app renders prompter previews — deferred.
