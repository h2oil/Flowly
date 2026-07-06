# Voice-Following Engine

## The key insight that shapes everything

Flowly does not need open-vocabulary transcription accuracy — it needs to decide *where in a known text* the speaker is. Matching against a known script is dramatically easier than transcription: even a 12–15 % WER stream re-anchors reliably with fuzzy matching, because the script itself is a powerful language-model prior. So we run a **small, cheap, on-device streaming model** and spend the engineering budget on the aligner. Perceived latency is also decoupled from ASR latency: the scroll controller dead-reckons at the speaker's estimated WPM between recognition updates, so 300–500 ms ASR corrections arrive as invisible micro-adjustments, not visible jumps.

## Streaming ASR selection (verified July 2026)

| Engine | Streaming latency | Size / CPU | License | Verdict |
|---|---|---|---|---|
| **sherpa-onnx streaming Zipformer transducer** | frame-synchronous; ~320 ms algorithmic, partials every ~100–300 ms; RTF ≈ 0.062 int8 | ~70 MB int8 total | Apache-2.0 | **Default** |
| Moonshine v2 (Feb 2026) | 50–258 ms response; WER 7.8–12 % | 34–123 M params; loadable via sherpa-onnx | permissive | Strong fallback; newer, less battle-tested |
| Vosk (Kaldi) | streaming, near-zero added | ~50 MB, ~300 MB RAM | Apache-2.0 | Solid but stagnant accuracy |
| whisper.cpp `--stream` | 0.5–2 s behind live; hallucinates on silence | tiny/base only | MIT | **Rejected** for the live path |
| NVIDIA streaming FastConformer/nemotron 0.6b | ~160 ms | 600 M params; sherpa-onnx support unfinished | CC-BY-ish | Watch-list |
| Windows.Media.SpeechRecognition | — | — | — | **Rejected**: deprecated Dec 2023 |
| Deepgram Nova-3 (cloud) | TTFT <300 ms | $0.0077/min | commercial | Reserved trait slot; **not in v1 binary** |
| Azure Speech real-time | 2–7 s reported in production | — | commercial | **Rejected** for live following |

**Decisive extras for sherpa-onnx:** built-in endpoint detection, built-in resampler, and — the killer feature for a teleprompter — **hotword/contextual biasing**. Continuously re-arming the bias list with the **next ~50 script tokens** collapses the effective error rate exactly where we need it. This is uniquely cheap because the target text is known; it is a **non-negotiable always-on mechanism**, not a tuning option. Re-bias on every final hypothesis.

Model licensing (which checkpoints we may redistribute commercially) is handled in [07-operations.md](07-operations.md).

## Microphone capture pipeline

- **WASAPI shared mode** via `IAudioClient3` (10 ms periods). Shared mode is mandatory — exclusive mode would steal the mic from Zoom/Teams. Capture at device mix format (typically 48 kHz float), downmix mono, resample to 16 kHz. Register `IMMNotificationClient` for default-device changes and rebuild the capture graph seamlessly. We wrap WASAPI directly with `windows-rs` where cpal falls short (see AEC below).
- **VAD: Silero VAD v6** (ONNX, ~2 MB, MIT, <1 ms per 30 ms chunk). Gates the ASR (CPU savings, no silence hallucinations) and drives PAUSED (speech-off >2 s). **Soft gating:** 300 ms pre-roll + 500 ms hang-over around speech segments; never reset the ASR stream mid-utterance — hard gating clips onsets and corrupts transducer state.
- Ring buffer: capture thread → lock-free SPSC → speech thread. Budget: 10 ms capture + 30 ms VAD framing + ~320 ms Zipformer chunk + ~20 ms decode ≈ **~380 ms worst-case ASR latency**, masked by predictive scrolling.

### Echo contamination (AEC) — the aligner must not chase the far end

Without echo cancellation the aligner will follow remote participants' words coming out of the laptop speakers:

- **Primary (Win11 22H2+):** open capture as `AudioCategory_Communications` and use `IAcousticEchoCancellationControl::SetEchoCancellationRenderEndpoint` for OS/driver AEC.
- **Fallback (Win10 / no APO):** software AEC3 (`webrtc-audio-processing`), reference signal from WASAPI loopback capture of the default render endpoint.
- **Aligner-side guard:** lightweight VAD on the loopback stream; when far-end speech is active and near-end energy delta is low, freeze cursor advancement. This alone kills the "remote person reads the shared doc aloud" failure even if AEC underperforms.

### Bluetooth HFP crater

Opening a BT headset's mic drops it from A2DP to HFP (8–16 kHz, telephone band), which craters Zipformer accuracy. Detect (endpoint name contains "Hands-Free AG Audio" or mix format ≤16 kHz) and warn: "Bluetooth headset in low-quality voice mode," with a one-click switch to the laptop's internal mic array for Flowly **while the call keeps the headset** — the mic is shared, so Flowly can listen on a different device than Zoom uses. Persist per-device preference.

### Permission and silence handling

Three-layer probe at capture start: (1) registry pre-check of the `CapabilityAccessManager\ConsentStore\microphone` consent keys (blocked desktop apps get *silence, not an error*), watched live via `RegNotifyChangeKeyValue`; (2) WASAPI HRESULT taxonomy → pill states (`E_ACCESSDENIED` → "Mic blocked by Windows privacy settings" + `ms-settings:privacy-microphone` button; `AUDCLNT_E_DEVICE_IN_USE` → name the culprit, offer switch); (3) silence watchdog — RMS < −55 dBFS for 5 s with no VAD speech → "We can't hear you" with live meter and device picker. **Voice-follow never dies silently:** any failure freezes the cursor and switches the pill to a distinct amber "manual mode" (arrow keys/scroll always work).

## The alignment algorithm

### Script preprocessing (once per script load; cached by `body_hash`)

- Tokenize display text; keep `token_index → char_offset/line` source map for rendering.
- Normalize each token into a *set* of spoken variants (1–8 per token, a tiny lattice): lowercase, strip punctuation, expand contractions both ways (`don't` ↔ `do not`), numbers/dates/currency to all plausible verbalizations (`$3.5M` → {"three point five million dollars", "three and a half million dollars", …}), abbreviations (`Dr.` → `doctor`). Full verbalization rules in [05-data-and-sync.md](05-data-and-sync.md).
- Precompute a **Double Metaphone code** per variant for homophone tolerance (`there/their`, `week/weak`).
- Build a **trigram anchor index**: hash of every normalized (and phonetic) trigram → script positions; content-word trigrams only.
- Filler stoplist {uh, um, er, like, you know, so, okay, right, well, …}: free insertions, never mismatches.

### Online matching — two cooperating mechanisms

1. **Rolling trigram anchors (fast path):** each new hypothesis word forms trigrams with its predecessors; look them up in the index restricted to the active window. An exact or phonetic trigram hit near the cursor is a high-confidence anchor.
2. **Banded local alignment (robust path):** every update, run a Smith-Waterman-style DP aligning the last K=10 recognized words against script tokens in a window around the predicted cursor (default −40…+120 tokens). Costs: exact 0; phonetic 0.2; fuzzy (normalized edit distance) 0.3–0.8; filler insertion ~0; other ASR insertions 0.6; **asymmetric gaps** — skipping script tokens forward costs 0.4/token (speakers drop words); backward motion is representable only via jump-back logic. Best-scoring endpoint = candidate cursor; score = matched fraction of the last 6 content words.

**Partials vs finals:** streaming transducers revise partials. Align only the *stable prefix* (words unchanged across two consecutive partials). Commit state on endpoint/final; partials may only advance the cursor a small bounded distance.

### Hysteresis — the cursor must never jitter

- Advance only when score ≥ θ_track (0.55). Max advance 6 tokens/update unless an exact ≥4-gram anchor justifies more.
- Never move backward in TRACKING. Jump-back (retake) requires an exact/phonetic trigram anchor *behind* the cursor, confirmed on two consecutive updates, with a prior boost at sentence/paragraph starts (people restart takes at sentence boundaries).
- Skipped paragraph: when LOST, the search window grows forward first (+400 tokens, then whole document with a decaying positional prior).

### State machine

```
states: TRACKING, LOST, SEARCHING, PAUSED

on_vad(silence > 2s):            any -> PAUSED (freeze cursor, stop WPM drift)
on_speech_resume:                PAUSED -> TRACKING if last score ok else SEARCHING

TRACKING:
  if score >= θ_track: cursor = clamp_advance(candidate); update WPM EMA
  elif score < θ_lost (0.30) for M=4 consecutive content words: -> LOST

LOST:   # ad-lib detour: freeze display at last confirmed position
  keep aligning against frozen window; widen window each update
  if re-anchor (>=3 consecutive exact/phonetic content-word matches): -> TRACKING (animated jump)
  after T=5s without anchor: -> SEARCHING

SEARCHING:  # skipped ahead, or jumped back for a retake
  window = whole script, prior peaked at cursor, boosted at sentence starts
  require >=4-word anchor, confirmed twice -> TRACKING
```

```
function on_asr_update(words, is_final):
    stable  = stable_prefix(words, prev_partial)
    tokens  = normalize(strip_fillers(stable))
    anchors = trigram_index.lookup(tokens, window)
    (pos, score) = banded_dp_align(last_k(tokens, 10), script[window], anchors)
    dispatch_to_state_machine(pos, score, is_final)
    if is_final: rebias_hotwords(script[pos : pos+50])
```

All thresholds (θ_track, θ_lost, M, T, window sizes, gap costs) are config, tuned against the corpus — never hard-coded folklore.

## Scroll controller (UI-side, TypeScript, rAF rate)

- The aligner emits `(token_index, confidence, timestamp)`; the layout map converts to a target such that the **current word sits on the fixed focal line**, with 1–2 words of lookahead highlight.
- Motion: **critically damped spring** (ζ=1, ω ≈ 2π·1.2 Hz), velocity-clamped — corrections glide, never snap. Re-anchor jumps after LOST/SEARCHING use a distinct 220–250 ms ease so the user perceives an intentional jump, not a glitch.
- **Dead reckoning:** between updates and when confidence dips, drift at the speaker's WPM (EMA over matched-word timestamps, counting VAD-speech time only), **hard-capped at +8 words past the last confirmed token**. This is what makes perceived latency <300 ms despite ~380 ms ASR latency.
- PAUSED freezes drift entirely; LOST freezes at the last confirmed word. The UI never scrolls speculatively while lost.

## Failure modes handled

Pauses (VAD→PAUSED, no drift) · ad-libs (LOST freezes, re-anchor resumes) · skipped sentences (forward search window) · retakes (guarded backward anchors, sentence-start prior) · fillers (zero-cost insertions) · homophones (Double Metaphone) · recognition errors (fuzzy DP + hotword biasing + confidence-gated hysteresis) · far-end speech (AEC + loopback guard) · mic failures (manual mode, never silent death). Escape hatch: manual nudge hotkeys always win instantly.

## Testing strategy — the regression corpus is the moat

CI-grade infrastructure from week 1.

**Two replay layers, one corpus:**

1. **Hypothesis-replay tests (every commit).** When the harness runs audio through real ASR once, it records the full `Hypothesis` event stream to JSONL. CI replays these logs through the aligner alone — milliseconds per case, bit-deterministic, isolating aligner changes from ASR nondeterminism.
2. **Full audio→ASR→aligner runs (nightly + on model/VAD changes).** The `align-harness` CLI feeds WAVs through the real sherpa pipeline (faster than real-time via injected clock) and regenerates hypothesis logs.

**Corpus:** each case = `{script.md, audio.wav, ground_truth.tsv}` (timeline `time_ms → true_token_index`). **Bootstrap ground truth with Montreal Forced Aligner** on the recordings, then hand-correct with a small labeling tool — far cheaper than pure hand labeling at the 60-case target. Scenario matrix (≥3 recordings each, multiple speakers/mics), organized in three tiers — *verbatim / ad-lib / hostile-acoustic*: verbatim read; filler-heavy; mid-script ad-lib (15–60 s off-script); skipped sentence/paragraph; retake jump-backs; numbers/currency/abbreviations; fast (>180 WPM) and slow (<100 WPM); background noise/echoing room; accented English. Target ~60 cases (~2 h audio) by end of M1. **Hypothesis JSONL logging is wired in during the very first ASR spike**, so every dogfood session from week 2 onward grows the corpus; beta users can donate a session's hypothesis log — never audio — with one click.

**CI-gated metrics:** median/95p token error while TRACKING; **wrong-motion rate** (cursor moves against ground truth — the cardinal sin, budget ≈ 0); lock-loss rate on verbatim material; re-lock time after ad-lib/skip/retake (target <1.5 s); end-to-end latency distribution (word-audible → CursorMoved). A one-page HTML report per run diffs metrics against `main`.

**Property tests:** fuzz hypothesis streams (random partial revisions, word drops, duplicated finals) asserting invariants — cursor monotone in TRACKING, never exceeds +6/update, backward moves only via double-confirmed anchors.

## Engine risks & pivots

- **Latency on low-end laptops under call load** must be validated in the M0 spike; fallback ladder: smaller chunk config → Moonshine v2 Tiny → degradation ladder in [07-operations.md](07-operations.md).
- **Aligner quality plateau** → the corpus gives an objective signal by end of M1. Pivot levers in order: Moonshine v2 small; cloud ASR opt-in build; loosen UX to paragraph-granularity following (the discrete-line UI already tolerates coarser cursors).
- **sherpa-rs API gaps** → budgeted 1-week bindgen fallback (`flowly-sherpa-sys`).
- **Token normalization long tail** (figures, product names — exactly what execs read) → multi-variant design from day one + numbers-heavy corpus cases.
- **Heavy accents / non-native English** degrade WER below what the aligner tolerates → this is the one case for the eventual opt-in cloud build; until then, honest messaging + manual mode.
