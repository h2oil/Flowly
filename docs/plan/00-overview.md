# Flowly — Plan Overview

## Executive summary

Flowly is a Windows 10/11 desktop teleprompter rendered as a slim, pill-shaped, always-on-top bar styled like Apple's Dynamic Island, docked just below the webcam. Its core feature is **voice following**: on-device streaming speech recognition aligned against the loaded script moves the text to wherever the speaker actually is — through pauses, ad-libs, skipped sentences, and retakes — with perceived latency under 300 ms. Scripts live in an offline-first local library that syncs to a cloud account with a companion web editor. The bar can exclude itself from screen captures so it never leaks into a Zoom/Teams share or an OBS recording.

The market gap is verified and specific: every high-quality voice-following prompter is iPhone/Mac-bound (PromptSmart, CueNotch, Notchie, Beast) or requires Elgato hardware plus an RTX GPU; every existing Windows overlay prompter (VODIUM, Virtual Teleprompter) scrolls on a timer and cannot listen. The Mac "notch prompter" wave of 2025–26 proved the design pattern; Windows — a 3–4× larger desktop base — has no equivalent. Flowly is that product.

## How this plan was produced

Five parallel research tracks (app shell, voice engine, UX, sync, market) fed three independently authored architecture proposals (optimized respectively for fastest-MVP, risk-first, and product-polish). A three-judge panel scored them; the **risk-first architecture won decisively** (75.5 vs 69 and 60.5 points), and the best ideas from the other two were grafted in. Notably, all three proposals independently converged on the same core stack, which gives high confidence in it. A completeness critique then produced six additional operational sections (model licensing, mic/AEC edge cases, perf budget, compliance, script import, field diagnostics), folded into these docs.

## Decisions at a glance

| Decision | Choice | Why (one line) |
|---|---|---|
| App framework | **Tauri v2** (Rust core + React/TS in WebView2) | Latency-critical audio→ASR→aligner path must be native and in-process; raw HWND access for capture exclusion and hit-testing; ~10× less RAM than Electron next to a video call |
| Pill rendering | Fixed max-size transparent window; pill drawn and morphed in **CSS**, never by resizing the window | Win32 resize can't hit 60 fps and transparent-window resize artifacts are a known WebView2 issue; GPU-composited CSS morphs are free |
| ASR engine | **sherpa-onnx streaming Zipformer transducer, int8, on-device CPU** | True frame-synchronous streaming (~320 ms algorithmic latency, RTF ≈ 0.06); Apache-2.0; hotword biasing toward the next ~50 script words is uniquely valuable when the target text is known |
| Alignment | Custom pure-Rust engine: trigram anchor index + banded DP with phonetic/fuzzy costs, four-state controller (TRACKING/LOST/SEARCHING/PAUSED) | The script is a powerful prior — robust alignment on a cheap model beats expensive ASR; hysteresis rules make "never jump wildly" enforceable and testable |
| Scroll motion | Critically damped spring + WPM dead-reckoning, in TypeScript at rAF rate, capped at +8 words past last confirmed token | Motion must sync to the compositor frame clock; prediction masks ASR latency so perceived latency < 300 ms |
| Text presentation | 3-line focus window with word-level karaoke highlight and discrete 180 ms line steps — not continuous scroll | Voice-following produces variable rates; discrete steps + word highlight absorb jitter and keep the gaze inside the eye-contact cone |
| Screen-share invisibility | `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` toggle with visible "hidden" badge | Compositor-level exclusion since Win10 2004; verified per-release against Zoom/Teams/Meet/OBS |
| Local store | SQLite (rusqlite, WAL), UUIDv7, immutable `script_version` history, outbox | Offline-first with zero-migration sign-in; frozen versions make take markers exact forever |
| Sync backend | **Supabase** (Postgres + Auth + Realtime + RLS) | Magic link + Google + Azure AD out of the box (the exec/M365 audience), one-line multi-tenancy via RLS, portable plain Postgres underneath |
| Conflicts (v1) | Rev-checked LWW; on conflict, fork a "(conflicted copy — Device, date)" — automatic 3-way merge deferred to v1.1 | Lossless, rare for single-author scripts, deletes the hairiest code from the launch path |
| Privacy stance | **Audio never leaves the device** — no upload code path exists; cloud ASR kept out of the binary until a deliberate opt-in build | Headline claim for the exec/sales/legal audience; also the moat web competitors can't copy |
| Packaging | NSIS (5–12 MB) + model download on first run; tauri-plugin-updater; Azure Trusted Signing | Small installer, staged rollouts, post-2023 HSM signing reality |

## MVP definition

The five features that beat "a Google Doc next to the call" (full argument in [01-product-and-market.md](01-product-and-market.md)):

1. Voice following that survives real speech, with an instant manual override that always wins.
2. The island: always-on-top pill under the webcam with per-monitor memory and smooth morphs.
3. Screen-share invisibility toggle, verified and badged.
4. Offline-first script library + web editor sync; paste from Docs/Word/Notion.
5. Frictionless mic coexistence + rehearsal mode with pacing stats.

Explicitly deferred past first release: cloud ASR fallback (Deepgram — trait slot only), automatic 3-way merge, E2E "vault" scripts, team sharing, monetization plumbing (lands post-beta), mobile, localization.

## Success criteria for the plan's riskiest bets

- **M0 exit (week 2):** live ASR partials < 400 ms on a mid-range laptop CPU while a Teams call runs; capture exclusion confirmed against Zoom, Teams, and OBS on Win10 + Win11. Kill/pivot decisions are taken here, not at week 10.
- **M1 exit (week 6):** near-perfect tracking on verbatim reading; re-lock < 1.5 s after ad-libs/skips/retakes; wrong-motion rate ≈ 0 on a ~60-case recorded regression corpus, gated in CI.
