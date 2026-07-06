# Operations: Packaging, Models, Performance, Diagnostics, Compliance

## Packaging, signing, updates

- **Installer:** NSIS via Tauri (5–12 MB) with the WebView2 Evergreen bootstrapper (preinstalled on Win11 and virtually all serviced Win10; bootstrap failures on old Win10 are a known long-tail — surface a clear error). MSIX not required for v1; direct download first, store distribution revisited post-launch.
- **Models are not bundled:** the ASR model (~70 MB int8) + Silero VAD download on first run with SHA-256 verification and resume, into `%LOCALAPPDATA%\Flowly\models`.
- **Updates:** `tauri-plugin-updater` — minisign-signed artifacts on a static bucket, server-driven channels, staged-rollout percentage flag, passive install. **Rollout gate:** crash-free sessions <99.5 % halts the stage.
- **Code signing:** **Azure Trusted Signing** through Tauri's `signCommand` hook (signs inner .exe and installer in one build) — the sane indie path since the 2023 HSM requirement.
- **CI (GitHub Actions, Windows runners):** every PR builds + runs hypothesis-replay suite + property tests + golden-file import tests; nightly full audio→ASR→aligner corpus run; release workflow signs, uploads, publishes the update manifest, and runs the overlay-probe capture-exclusion matrix (Zoom, Teams new + classic, Meet-in-Chrome, OBS display/window/WGC) as a **hard release gate**.

## ASR model licensing, attribution, redistribution

sherpa-onnx *code* is Apache-2.0, but model *weights* are separate artifacts, and several English icefall checkpoints are trained on GigaSpeech, whose dataset terms are non-commercial — redistribution to paying customers must be settled pre-GA:

1. **Model registry** (YAML in repo): per checkpoint — HF repo + commit hash, declared license, training corpora, LICENSE snapshot at mirror time. Default ship: a **LibriSpeech-only-trained streaming Zipformer** (CC BY 4.0, commercially usable with attribution). GigaSpeech-trained checkpoints only with written counsel sign-off. Upgrade path: NVIDIA Parakeet-TDT (CC-BY-4.0, explicitly commercial-cleared) exported to ONNX. Silero VAD is MIT — no issue.
2. **Attribution:** `THIRD_PARTY_NOTICES.txt` in the installer + in-app "About → Open-source licenses"; every downloaded model bundle embeds its own `LICENSE` + `ATTRIBUTION.md` so terms travel with the artifact.
3. **Hosting:** never hotlink Hugging Face from clients. Mirror pinned tarballs to Cloudflare R2 behind `models.flowly.app` (zero egress fees); manifests signed with the same key infrastructure as the updater; old versions retained 12 months.
4. **Cloud ASR (post-v1):** when the Deepgram build ships, execute their commercial agreement + DPA, disable model-training retention (`mip_opt_out`), gate to a paid plan, and badge the pill "cloud" while active.

## Performance & battery budget (real low-end hardware)

**Min-spec (enforced at install):** Win10 1903+, 4-core x64 **with AVX2** (Intel 8th-gen / Ryzen 2000), 8 GB RAM, iGPU, 300 MB disk. Below AVX2: refuse voice-following, offer timed-scroll mode.

**Hard ceilings, CI-gated on a throttled reference laptop (i5-8250U @ 15 W, 8 GB, Teams call + 20 Chrome tabs):**

- CPU ≤15 % of one core average, ≤35 % peak during continuous speech (VAD-gating does the heavy lifting: Silero always-on at <1 % core; Zipformer decodes speech frames only).
- RAM ≤350 MB total working set — Rust core ≤180 MB, WebView2 pill ≤150 MB (transform/opacity-only animations; no per-frame filters/blur).
- `VirtualLock` the ~80 MB of hot tensor memory so Windows can't page the model out during long pauses — first-word latency after 5 minutes of silence must equal steady state.
- Battery: ≤2 W incremental during active speech, ≤0.5 W idle-listening; measured per release via SRUM (`powercfg /srumutil`) on the reference laptop.

**Execution provider: CPU-only, deliberately.** On min-spec, the iGPU is already saturated compositing Teams video and encoding the camera; DirectML shares that queue and produces worse *tail* latency under call load. Pin ONNX Runtime to CPU EP, `intra_op_num_threads=2`, mark the decode thread MMCSS "Pro Audio," and opt out of EcoQoS power throttling so E-core parking can't starve it.

**Degradation ladder under contention** (watchdog on rolling 10 s decode RTF + end-to-end lag): (1) normal — 100 ms chunks, beam 4; (2) RTF >0.5 — 320 ms chunks, beam 2; (3) RTF >0.8 or lag >600 ms — greedy search, VAD-gated only; (4) sustained overload — pause ASR, switch to manual scroll with a subtle "low-power mode" pill indicator. Each step recovers after 30 s of headroom. **Never degrade by dropping audio** — the capture buffer holds 2 s. On battery-saver, start at step 2 and drop pill animations to discrete state changes.

## Crash reporting & field diagnostics

- **Three crash surfaces, one pipeline (Sentry via `tauri-plugin-sentry`):** Rust panics (custom hook flushes aligner scope tags first); native faults in cpal/WASAPI/sherpa/onnxruntime (out-of-process minidump handler that survives the crash); JS errors in the pill (Rust + browser breadcrumbs merged). WebView2 renderer crashes: subscribe `ProcessFailed`, report kind + exit code, **auto-recreate the webview so the pill self-heals**.
- **Symbolication in CI:** release profile `debug = "line-tables-only"`; `sentry-cli debug-files upload` for the exe and sherpa/onnxruntime PDBs in one invocation; releases bound to updater versions.
- **Alignment "black box" recorder:** always-on in-memory ring buffer (last 120 s, ~200 KB) of structured events — VAD segments, every ASR partial/final with timestamps, every aligner decision (cursor, match costs, jumps, confidence). No raw audio. A **"Report a glitch" hotkey (Ctrl+Shift+D)** snapshots ring buffer + script hash + device/config into a replay bundle that feeds the `ReplayAsr` harness — one-command deterministic repro of "the cursor jumped around during my take."
- **Privacy of diagnostics:** hypothesis logs are effectively a transcript — treat them as content. Per-incident, opt-in, with a preview dialog showing exactly what leaves the machine; default *redacted mode* replaces words with token indices + coarse phonetic classes (enough to replay alignment without revealing text). 30-day retention, user-initiated deletion.
- **Fleet metrics (anonymous counters only):** jumps/min, re-lock latency p95, RTF, dropped WASAPI callbacks, panic-free rate — surfaces alignment regressions within days of an update with zero content leaving the machine. A CI grep gate on logging calls enforces "no script text in logs."

## Privacy, legal, enterprise compliance

- **Core v1 claim: "Audio never leaves your device."** All ASR local; audio buffers RAM-only, never written to disk; rolling transcript discarded on session end. First-run consent screen before the mic opens; per-session tray indicator alongside Windows 11's own mic badge. Flowly captures only the presenter's own microphone — never remote call audio — which keeps it outside two-party-consent recording laws; the policy says so explicitly because exec buyers will ask.
- **UK/GDPR posture (company is UK-based):** register with the ICO (Tier 1); Supabase pinned to eu-west-2 (London) with their DPA signed before beta; account deletion cascades scripts/devices within 72 h + 30-day backup purge (Art. 17); JSON export endpoint (Art. 20); local cache wiped on sign-out.
- **Telemetry:** opt-in, default off; Aptabase (EU-hosted) for anonymous counts; Sentry with a `beforeSend` scrubber.
- **Enterprise readiness (unblocks the sales-team tier later):** Supabase SAML SSO for Okta/Entra at the Team tier; org policy flags (force local-only ASR, disable sync, domain-restricted sign-up); SOC 2 roadmap — instrument with Vanta from month 1, Type I at GA+6 months (~$30–40k budget incl. audit); trust page with pre-filled CAIQ-Lite/SIG-Lite and the sub-processor list (Supabase, Sentry, Aptabase, Azure Trusted Signing — Deepgram only if/when shipped).

**Exit criteria:** privacy policy + consent UX ship **in v1.0**, not after; Supabase DPA signed before beta; ICO registration before the first UK customer data; counsel-approved model-license matrix merged before GA.
