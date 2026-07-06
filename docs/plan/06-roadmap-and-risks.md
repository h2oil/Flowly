# Roadmap & Risks

## Shape of the plan

The schedule attacks the two existential risks (alignment quality, overlay fidelity) as **parallel week-1 spikes with explicit exit and kill/pivot criteria** — we learn in two weeks whether the product is viable, not at week ten. UI and engine tracks then run in parallel against frozen contracts (the `AsrStream`/`Aligner` events and TokenMap schema are frozen at M0 exit; the UI is built against a scripted fake cursor feed).

Scope discipline applied from the judge panel: monetization plumbing, cloud ASR, automatic 3-way merge, and the WASM-shared web compiler are **out of the launch path**. Total: **~19 weeks to public beta** for a 1–2 person team (M1/M2 and M4/M5 partially parallelize with a second contributor).

## Milestones

| # | Milestone | Weeks | Scope | Exit criteria |
|---|---|---|---|---|
| **M0** | Twin de-risking spikes | 2 | **(a)** `align-harness` CLI skeleton; `AsrStream`/`Aligner` traits; sherpa-onnx streaming from Rust (bindgen fallback decision taken); hypothesis JSONL logging wired in; first ~10 corpus recordings, ground truth bootstrapped with Montreal Forced Aligner. **(b)** `overlay-probe` app: transparent CSS pill, 60 Hz cursor-poll hit-testing (wndproc subclass as polish follow-up), `WDA_EXCLUDEFROMCAPTURE`, per-monitor DPI persistence keyed by EDID identity. | Live partials <400 ms on laptop CPU during a Teams call; capture exclusion confirmed on Zoom + Teams + OBS on Win10 & Win11. **Kill/pivot decisions taken here.** IPC contracts frozen. |
| **M1** | Alignment engine to quality bar | 4 | `flowly-script` compiler (Prompter Markdown → TokenMap with variants/metaphone/source map); full aligner (trigram anchors, banded DP, hysteresis, four-state controller); soft VAD gating (pre-roll/hang-over); **always-on hotword re-biasing with next ~50 tokens**; corpus grown to ~60 cases across the verbatim/ad-lib/hostile-acoustic matrix; CI gates on wrong-motion rate, re-lock time, latency; property tests. | Verbatim tracking near-perfect; re-lock <1.5 s on retake/skip cases; wrong-motion ≈ 0. |
| **M2** | Pill UI + scroll controller | 3 *(parallel with M1)* | Full S0–S4 state machine per UX spec: 3-line focus window, word karaoke, discrete line steps, spring morphs, hover-expand controls, countdown, boss-key, waveform, drag + magnetic snap, notch mode; TS scroll controller (critically damped spring + WPM dead-reckoning, **+8-word cap**) driven by a scripted fake cursor feed; reduce-motion; accessibility typography (Atkinson Hyperlegible, OpenDyslexic, dyslexia-friendly highlight default). | UI complete and demo-able with zero live audio. |
| **M3** | Live integration | 2 | Wire audio→ASR→aligner→event channel→UI in `flowly-app`; device hot-swap, mic-failure taxonomy, silence watchdog, BT-HFP warning; AEC path (Win11 API + loopback guard); load-script freezes version + compiles TokenMap + arms hotwords; end-to-end latency measured against the 300 ms perceived budget. | **Dogfooding on real Zoom/Teams calls begins and never stops.** |
| **M4** | Local script management | 2 | rusqlite schema + migrations; script list + lightweight editor; paste/.docx/.txt/.md import funnel with golden-file tests; take markers pinned to frozen versions with jump-back-to-take; settings window; Prompter Markdown rendering (dim stage notes, section jump targets). | 25-script import corpus: 22+ import with zero manual fixes. |
| **M5** | Cloud sync + web app | 3 | Supabase schema, RLS, rev-bump triggers; auth (PKCE system browser + `flowly://` deep link; magic link + Google + Azure AD; custom SMTP); push/pull with rev-check + **conflicted-copy fork** (3-way merge deferred to v1.1); Realtime doorbell; sign-in-as-claim; **minimal web editor** (sign-in, list, plain editor, debounced save); offline/conflict test suite. | Two-device offline edit scenarios all lossless. |
| **M6** | Packaging, updates, beta hardening | 3 | NSIS installer; model download with checksum + resume from our CDN mirror; tauri-plugin-updater with staged rollout; Azure Trusted Signing in CI; Sentry crash reporting + black-box glitch recorder + opt-in telemetry (incl. hypothesis-log donation); overlay-probe as release-gate compatibility canary; onboarding flow; **private beta (25–50 users)**; perf/soak testing next to Zoom + OBS on the min-spec reference laptop. | Crash-free sessions ≥99.5 %; perf ceilings hold on min-spec (see [07-operations.md](07-operations.md)). |
| **M7** | Launch polish | 2 | Accessibility pass; Win10 pre-2004 warning path; docs + marketing site; name/trademark clearance resolved **before branding lock**; corpus expansion from beta logs. | Public beta ships. |

**Post-beta (v1.1+):** monetization (Stripe + Supabase: free tier limits, Pro, lifetime founder license), automatic 3-way merge, Deepgram opt-in build with consent UX, E2E vault scripts, team tier, take-marker sync, localization.

## Risk register

| Risk | Signal | Mitigation / pivot |
|---|---|---|
| **Aligner quality plateaus** below "magic" | Corpus metrics at end of M1 | Pivot levers in order: Moonshine v2 small model → cloud ASR opt-in build → paragraph-granularity following (the discrete-line UI already tolerates coarser cursors) |
| **ASR latency on low-end laptops** under call load | M0 spike benchmark; degradation-ladder telemetry | Smaller chunk config; Moonshine Tiny; ladder steps down to manual mode (never drops audio) |
| **WebView2 transparency/hit-testing edge cases** (drivers, runtime auto-updates) | overlay-probe in week 2; canary before each release | Cursor-poll fallback shipped behind a flag; solid-color pill mode; pinned regression tests on transparency + capture exclusion against new WebView2 runtimes |
| **sherpa-rs API gaps** (streaming/hotwords) | M0 spike | Budgeted 1-week in-repo bindgen crate (`flowly-sherpa-sys`) |
| **Capture-exclusion inconsistency** across capture APIs/OBS backends | Release-gate matrix | Verify Zoom, Teams **new and classic**, Meet-in-Chrome, OBS display/window/WGC capture separately, every release, Win10 + Win11 |
| **Echo/far-end contamination** in echoing rooms and speaker setups | Hostile-acoustic corpus tier; dogfood | AEC (OS API on Win11, webrtc AEC3 fallback) + loopback-VAD cursor freeze |
| **Trademark: "Flowly"** collides (USPTO filing, flowly.run, flowly-app.com; confusable with Speakflow/FlowPrompter) | Counsel clearance in M7, before branding spend | Backup names held: Islet, Cueline, Eyeline, Underlens, Sayso |
| **PromptSmart VoiceTrack patent** | Freedom-to-operate check pre-US-launch | Our approach (trigram anchoring + banded DP + explicit state machine) differs materially, but get counsel's view |
| **Fast followers** (ShareSpeak, FlowPrompter) or platform players (Elgato free Voice Sync, Teams native prompter) | Market watch | Win on polish, island UX, alignment quality, sync; stay call-platform-agnostic and privacy-first |
| **Supabase dependency** (Auth/Realtime proprietary; free tier pauses) | — | Pro plan from launch; plain-Postgres schema; auth/realtime behind thin adapters; documented exit path (self-hosted Supabase/PocketBase) |
| **Retakes that restart mid-sentence** leave the engine SEARCHING | Corpus retake cases | Sentence-start prior covers most; always-visible manual nudge is the guaranteed escape hatch |
| **Rust talent/velocity** if the team is JS-heavy | — | The Electron fallback exists but costs ~10× RAM + native-module tax; decision point is M0, not later |

## Open questions (need answers, not code)

1. **Name:** commit to Flowly pending clearance, or rebrand early while it's cheap?
2. **Team size/skills:** is a Rust-capable contributor available for the M0 engine spike? (This is the single biggest schedule variable.)
3. **Beta channel:** where do the first 25–50 users come from (creator communities vs. exec/sales network)? This decides whether rehearsal-mode polish or Teams-coexistence polish leads M6.
4. **Elgato/Stream Deck integration** (deferred): worth a v1.1 look — their audience already buys prompter hardware.
