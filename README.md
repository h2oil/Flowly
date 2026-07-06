# Flowly

**A voice-following teleprompter for Windows that lives in a Dynamic-Island-style pill just below your webcam.**

You load a script. Flowly listens to your microphone, aligns what you actually say to the script in real time, and moves the text to match — through pauses, ad-libs, skipped sentences, and retakes. The pill sits directly under your camera lens so you keep eye contact on Zoom/Teams/OBS, and it can hide itself from screen shares entirely. Scripts sync online (edit in the browser, present from the desktop), but the app is offline-first and **audio never leaves your device**.

## Status

🔩 **Rev1 — the engine and the pill.** The two components the plan says the product lives or dies on are built and tested:

- **Alignment engine** (`crates/flowly-align`) — pure, deterministic Rust: banded fuzzy/phonetic local alignment with the four-state TRACKING/LOST/SEARCHING/PAUSED controller. 16 scenario tests cover verbatim reading, fillers, misrecognitions, ad-lib freezes, forward skips, retake jump-backs, silence pauses, spoken numbers, and a fuzz invariant (*the cursor never moves backward without a Retake event*).
- **Script compiler** (`crates/flowly-script`) — Prompter Markdown → TokenMap with byte-offset source spans and spoken-variant lattices (`$3.5M` → "three point five million dollars", `2026` → "twenty twenty six", `3rd` → "third", `SQL` → "s q l").
- **ASR contract** (`crates/flowly-asr`) — `AsrStream` trait + hypothesis-log replay. The Windows sherpa-onnx implementation plugs in behind it; no cloud engine exists in the binary by design.
- **Local store** (`crates/flowly-store`) — SQLite (WAL) with the plan's schema: immutable frozen `script_version`s on arm, take markers pinned to frozen versions, outbox for future sync.
- **Corpus harness** (`apps/align-harness`) — `synth` generates hypothesis logs + ground truth from any script; `run --gate` enforces wrong-motion = 0 and p95 token error ≤ 8 in CI. A demo corpus case lives in `corpus/demo/`.
- **The pill UI** (`apps/desktop-ui`) — React island with S0–S3 states, 3-line focus window, word karaoke, amber hold-still on lock loss, hover controls, and the critically damped spring + dead-reckoning scroll controller (hard-capped at +8 words). Runs in a browser off a scripted engine feed, exactly as the plan's M2 prescribes.

**Not yet built** (needs a Windows machine — next revs): the Tauri shell (transparent always-on-top window, `WDA_EXCLUDEFROMCAPTURE`, hit-testing), WASAPI capture + Silero VAD + sherpa-onnx bindings, and cloud sync.

## Running Rev1

```bash
# Engine: all unit + scenario tests
cargo test --workspace

# Corpus harness: simulate a speaker (8% WER) and gate the aligner on it
cargo build -p align-harness
./target/debug/align-harness synth --script corpus/demo/script.md \
    --out-hyp /tmp/h.jsonl --out-truth /tmp/t.tsv --seed 7 --wer 0.10
./target/debug/align-harness run --script corpus/demo/script.md \
    --hyp /tmp/h.jsonl --truth /tmp/t.tsv --gate

# Pill UI demo (browser stand-in for the transparent Tauri window)
cd apps/desktop-ui && npm install && npm run dev
```

Click the pill to watch the demo take: countdown → tracking with word karaoke → thinking pause (PAUSED) → off-script ad-lib (LOST, amber, text holds still) → re-anchor → retake jump-back → end-of-script.

## The product in five features

1. **Voice following that survives real speech** — on-device streaming ASR fuzzily aligned to the script; tolerates fillers and ad-libs (holds position), skipped sentences (jumps forward), and retakes (jumps back). Perceived latency under 300 ms.
2. **The island** — an always-on-top translucent pill, top-center under the webcam by default, draggable with magnetic snapping, animated morphs between states, per-monitor position memory.
3. **Screen-share invisibility** — a toggle backed by `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`, verified against Zoom, Teams, Meet-in-Chrome, and OBS on every release.
4. **Offline-first script library with online sync** — SQLite locally, Supabase in the cloud, a thin web editor; account-free by default, sign-in "claims" your local data.
5. **Frictionless mic coexistence** — WASAPI shared mode so Zoom/Teams/OBS keep the microphone; device picker with live level meter; rehearsal mode with pacing stats.

## Chosen stack (summary)

| Layer | Choice |
|---|---|
| App shell | Tauri v2 — frameless transparent always-on-top window; pill drawn/animated in CSS |
| UI | React 18 + TypeScript + Vite in WebView2 |
| Native core | Rust: cpal (WASAPI shared mode) → Silero VAD v6 → sherpa-onnx streaming Zipformer (int8) → custom alignment engine |
| Local store | SQLite (rusqlite, WAL), UUIDv7 ids, outbox pattern |
| Sync backend | Supabase (Postgres + Auth + Realtime + RLS), whole-doc rev-checked sync |
| Packaging | NSIS + tauri-plugin-updater, Azure Trusted Signing |

## Plan documents

| Doc | Contents |
|---|---|
| [docs/plan/00-overview.md](docs/plan/00-overview.md) | Executive summary, decisions at a glance, MVP definition |
| [docs/plan/01-product-and-market.md](docs/plan/01-product-and-market.md) | Competitive landscape, positioning, pricing, naming |
| [docs/plan/02-architecture.md](docs/plan/02-architecture.md) | Process/threading model, module layout, IPC contracts |
| [docs/plan/03-voice-following.md](docs/plan/03-voice-following.md) | ASR engine, alignment algorithm, scroll controller, test corpus |
| [docs/plan/04-ux-spec.md](docs/plan/04-ux-spec.md) | The island bar states, visual spec, placement, hotkeys, onboarding |
| [docs/plan/05-data-and-sync.md](docs/plan/05-data-and-sync.md) | Script format, schema, sync protocol, conflicts, import, privacy |
| [docs/plan/06-roadmap-and-risks.md](docs/plan/06-roadmap-and-risks.md) | Milestones M0–M7, risk register, open questions |
| [docs/plan/07-operations.md](docs/plan/07-operations.md) | Packaging, signing, updates, model licensing, perf budget, compliance |
