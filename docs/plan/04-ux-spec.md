# UX Specification — The Island Bar

## 1. Bar states and behaviors

The bar is a single morphing surface with five states. All transitions are animated morphs of one pill (never a second window popping in) — that is what makes it read as "Dynamic Island" rather than "widget."

**S0 — Collapsed Idle (no script armed).** 36 px tall × 160 px wide (100 % DPI): Flowly glyph + script name, 12 px medium, 55 % opacity. Click opens the script picker; drag moves the bar.

**S1 — Armed / Countdown.** On Start (button or hotkey) the pill morphs to 44 × 520 px and runs a **3-2-1 countdown** (24 px centered numeral, scaling 1.15→1.0 per beat, soft mutable tick). Skippable; configurable 0–10 s. Gives the presenter time to look at the lens before words appear.

**S2 — Active Prompting (the core state).** A **3-line focus window, not classic scrolling** — continuous scroll fights voice-following (variable speaking rate = jittery scroll speed):

- Line 2 (center) is the **live line**: current word highlighted with a soft karaoke fill sweeping **word-by-word** (word granularity matches ASR token cadence and hides recognition jitter).
- Line 1 above: just-spoken text at 40 % opacity. Line 3 below: upcoming text at 75 % opacity.
- When the speaker crosses ~70 % of the live line, lines shift up one line-height with a **180 ms easeOutCubic translate + crossfade** — a discrete step the eye tolerates far better than continuous crawl at conversational distance.
- 1-line "minimal" and 5-line "context" variants are options; 3 is default.
- On loss of lock (ad-lib detected): karaoke fill stops, a subtle **amber dot pulses** at the left cap, text **holds still** — never scroll speculatively. On re-match (mid-paragraph, skipped-ahead, or jump-back for a retake) text snap-morphs (220 ms) to the new position. The alignment engine drives; the UI only ever animates to wherever the engine says.

**S3 — Expanded (hover ≥150 ms).** Spring-morphs to 76 × 640 px. Bottom row of controls appears while the top row keeps prompting: ⏯ play/pause · ⟲ restart paragraph · ⟲⟲ restart take (jump to take marker) · fallback-WPM stepper (80–220, used when voice-follow is off) · script picker · mic-level meter · ⋯ settings. Mouse-leave collapses after a 400 ms grace. All controls tooltip their hotkeys.

**S4 — Ducked / Boss-hidden.** Boss-key shrinks the bar to a 6 × 80 px sliver at the screen edge in 150 ms; voice-following continues silently so position isn't lost. Second press restores.

**Listening indicator:** in S2 the left end-cap hosts a 5-bar micro-waveform (14 px) driven by mic RMS — green while locked, amber while unlocked, gray when muted. Doubles as the mic-level and "the app is alive" signal during pauses.

## 2. Visual spec

- **Geometry (100 % DPI):** heights 36 / 44 / 76 px; widths 160 / 520 / 640 px (user-resizable 380–900 px via edge-drag). **Corner radius = height/2 always** — the pill identity survives every state. Scales with per-monitor-v2 DPI.
- **Material:** ~#0A0A0C at 82 % opacity with backdrop blur on Win11; solid #111114 at 92 % on Win10 (and a "solid background" toggle day one — blur over video can shimmer). 1 px inner border rgba(255,255,255,0.08). Shadow 0 8 32 rgba(0,0,0,0.45).
- **Typography:** **Inter** bundled (default), plus **Atkinson Hyperlegible** and **OpenDyslexic** options. Live line 17–28 px adjustable (default 21 px / 600 / 1.35 line-height / +0.01 em). Context lines same size — shrinking causes refocus cost. Sentence case, no justification, ~52 chars max per line.
- **Color:** past text rgba(255,255,255,0.38); live word #FFF via (a) karaoke fill #5AC8FA→#FFF or (b) a rounded 6 px pill behind the current word at rgba(90,200,250,0.22) — **(b) is the dyslexia-friendly default**. Upcoming rgba(255,255,255,0.75). Status: locked #34C759, searching #FFB340, error #FF6961.
- **Motion:** state morphs use a spring (mass 1, stiffness 380, damping 30 → ~330 ms settle, ~1.02 overshoot) matching the real Dynamic Island's springy 0.3–0.5 s feel; line advances 180 ms easeOutCubic; hover fades 120 ms. Honor Windows **reduce motion**: replace springs with 120 ms crossfades.

## 3. Placement UX

- **First run:** bar spawns top-center with a one-time coach mark — "Drag me so I sit just below your camera" — plus a ghost camera glyph. (No OS API reveals physical webcam position; placement is a UX heuristic, and onboarding sets that expectation.)
- **Magnetic snapping:** targets at top-center of each monitor (±24 px zone) and horizontal center; 8 px gap below the top edge; 80 ms settle + faint tick.
- **Notch mode (opt-in):** bar hugs y=0 with top corners squared — the strongest "cut from the bezel" read. Opt-in because some apps do put UI at top-center.
- **Per-monitor memory:** keyed by **EDID/device-path monitor identity** (display indices are unstable across dock/undock), storing x-offset from monitor center + y. On topology change re-resolve; if the monitor is gone, fall back to primary top-center. User placement is sacred — the app never auto-moves the bar.
- **Screen-share invisibility:** "Hide from screen shares/recordings" toggle (`WDA_EXCLUDEFROMCAPTURE`), with a small crossed-eye badge in the expanded state while active so users trust it. Pre-2004 Win10: warn that the bar appears as a black box in shares.

## 4. Reading science baked into defaults

Chen (CHI 2002, Stanford) found perceived eye contact survives gaze deviations up to ~7°, and viewers are **an order of magnitude more tolerant of downward gaze than lateral or upward**. Consequences: (1) the bar belongs **directly below** the lens — hence top-center default and center-snap; (2) keep the text block vertically tight — 3 lines at 21 px ≈ 88 px ≈ 3–4° at 55–70 cm, inside the tolerance cone; (3) the live line rides as high in the pill as possible. Onboarding one-liner: *"Keep text within a coffee-mug's width of the lens."*

**Mirror mode** (horizontal flip of the render surface, controls excluded) for beam-splitter glass. **Dyslexia-friendly preset:** Atkinson/OpenDyslexic, word-pill highlight, +0.05 em spacing, 1.5 line-height, optional warm tint. **End of script:** last line fades, checkmark pulses, elapsed time + average WPM show for 4 s, bar auto-collapses to S0.

*(Validation note: the 7° numbers come from 2002 viewing conditions; the 3-line default gets webcam-position user testing during beta.)*

## 5. Hotkeys, tray, onboarding, empty states

**Global hotkeys** (all rebindable, conflict-detected; defaults chosen to dodge Zoom/Teams/OBS): Ctrl+Alt+Space pause/resume follow · Ctrl+Alt+R restart paragraph · Ctrl+Alt+T restart take (drops a take marker first) · Ctrl+Alt+↑/↓ nudge a sentence back/forward (also the manual fallback — **manual input always beats the engine instantly**) · Ctrl+Alt+H boss-key · Ctrl+Alt+M mirror.

**Tray:** state-colored dot (gray idle / green listening / amber paused); left-click toggles bar visibility; right-click: recent 5 scripts, Start/Pause, Hide-from-capture ✓, Settings, Quit. Closing the bar minimizes to tray, never exits.

**Onboarding (≤90 s):** sign-in / continue-offline → mic pick with live meter → **5-second "read this sentence" test that demos the highlight following the user's own voice** (the magic moment, before any placement fiddling) → drag-under-camera coach mark → hotkey cheat card.

**Empty/error states:** no scripts → pill reads "No script yet — press to add," main window shows a starter script ("Read me to see how Flowly follows your voice"). Offline → quiet "You're offline; scripts will sync later." Mic missing/denied → red mic glyph with click-through to Windows privacy settings (full mic-failure taxonomy in [03-voice-following.md](03-voice-following.md)).

## 6. Settings surface & editor split

**Settings** open in a separate compact window — the pill stays glanceable. Tabs: General (launch at login, countdown, language) · Appearance (lines 1/3/5, font, size, highlight style, notch mode, solid background) · Voice (mic device, sensitivity, fallback WPM, filler tolerance) · Hotkeys · Account/Sync.

**Script editing split:** in-app editor is deliberately lightweight — large-type text pane, paragraph markers, per-script settings, insert-pause — for last-minute tweaks. Full authoring (folders, import, history, later team sharing) lives on the **web app**; the desktop links "Edit full script on web," edits sync back in seconds. This keeps the desktop cold-start-fast and the pill the hero.
