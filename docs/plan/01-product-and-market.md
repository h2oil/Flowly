# Product & Market

*Competitive research verified July 2026. Vendor-site claims should be re-validated hands-on before finalizing the pricing page.*

## The market splits into four camps — none owns "Windows + video call + voice-following + near-camera overlay"

### A. Voice-following prompter apps (mobile/web-first)

- **PromptSmart Pro** — the patented VoiceTrack pioneer; iOS/Android + web control room; on-device and offline. $29.99 one-time + ~$34.99/yr for sync. **No Windows overlay app**, and App Store reviews report VoiceTrack "fails most of the time," freezes, and is confused by room echo; English-only detection. "Mostly works" is demonstrably not enough — reliability is the bar Flowly must clear.
- **Speakflow** — browser-based voice scrolling; Free / ~$15–19/mo / $99/mo. Has an overlay mode invisible to screen share, but it's a browser tab that can't control apps beneath it, and Speakflow is phasing it out for a desktop app still in beta. Browser ASR latency is a documented weakness.
- **Teleprompter.com / BIGVU / Descript** — recording-studio workflows (BIGVU $19–49/mo; Descript free–$50/mo with post-hoc AI eye contact). They optimize the *recording* pipeline; a live Zoom call has no post-production. Descript's AI Eye Contact proves demand for "look at the lens" — and fixes it in the wrong place.
- **CuePrompter / QPrompt** — free scrollers, no voice tracking, no overlay. This is the "reading a Google Doc" baseline.

### B. Windows video-call overlay prompters (the direct arena)

- **VODIUM** — "the only desktop teleprompter positioned directly underneath your computer's camera" (their words), aimed at execs/sales, $15/mo. **Manual/timed scroll only — no voice following.** VODIUM validated Flowly's exact placement thesis and left the core feature on the table.
- **Virtual Teleprompter** — Windows/Mac translucent overlay, $7.99 Pro, no voice tracking.
- **FlowPrompter** — freemium, voice tracking in 14 languages, capture-hidden, cross-device sync. Closest feature-for-feature competitor, but web-centric, no island aesthetic, unclear offline story.
- **ShareSpeak** — indie Mac+Windows+iOS; capture-invisible; voice scroll with selectable engines including fully-offline local models; $12.50–17.90 lifetime beta pricing. Its notch bar mode is **Mac-only**.

### C. Mac "notch" prompters — the pattern Flowly ports to Windows

A 2025–26 wave fused notch aesthetic + voice following + capture invisibility: **CueNotch** ($29.99 lifetime, macOS 14+), **Notchie**, **VoicePrompter**, **Beast Teleprompter**, **Textream**. Every one is Apple-only. Flowly is essentially *CueNotch for the other 70 % of desktops*.

### D. Dynamic-Island clones for Windows — aesthetic proven, no prompter

DynamicWin, dynamic-island-for-windows, Windhawk's Dynamic Island mod, DockBar ($5 on Steam) show Windows users pay for this look — media controls, widgets, timers — but none does teleprompting. The pill-under-webcam prompter slot on Windows is empty.

## Where incumbents fail on Windows video calls

1. **The best voice tracking isn't on Windows.** PromptSmart is iOS-first; the entire notch wave is Apple-only; Elgato's Voice Sync requires an **NVIDIA RTX 2060+ on PC** — excluding most corporate laptops — plus $279–599 hardware.
2. **The Windows overlays don't listen.** VODIUM and Virtual Teleprompter scroll on a timer; the speaker chases the text (the #1 cause of "teleprompter voice").
3. **Web prompters add latency and tab-juggling**, and Speakflow's own docs concede the browser overlay's limits.
4. **Studio tools abandon you live.**
5. **Screen-share leakage.** A plain always-on-top window is captured by Zoom/Teams/OBS. `WDA_EXCLUDEFROMCAPTURE` has existed since Win10 2004, yet only two tiny products ship it on Windows today.

## Feature comparison (verified 2026)

| Product | Voice tracking | Near-camera overlay | Hidden from share | Offline | Sync | Windows native | Price |
|---|---|---|---|---|---|---|---|
| PromptSmart Pro | Yes (flaky per reviews, EN-only) | No | No | Yes | Yes | No | $29.99 + $34.99/yr |
| Speakflow | Yes (web) | Browser overlay | Yes | No | Yes | Beta | Free / $15–19/mo |
| Teleprompter.com | Paid | No | No | Partial | Yes | PWA only | ~$7.50–10/mo |
| Elgato Prompter | Yes (needs RTX/Apple Si) | Hardware | N/A | Yes | No | Camera Hub | $279–599 hw |
| VODIUM | **No** | **Yes (under webcam)** | Limited | Yes | No | **Yes** | $15/mo |
| FlowPrompter | Yes (14 langs) | Floating | Yes | Unclear | Yes | Yes | Freemium |
| ShareSpeak | Yes (incl. local) | Notch (Mac only) | Yes | Yes | Partial | Basic | $12.50–17.90 lifetime |
| CueNotch (Mac) | Yes (on-device) | Yes (notch) | Yes | Yes | Local | No | $29.99 lifetime |
| **Flowly (target)** | **On-device, ad-lib/skip/retake-robust** | **Island pill under webcam** | **Toggleable** | **Offline-first** | **Web editor + sync** | **Yes** | below |

## Positioning

> For creators, executives, and sales teams who present from a Windows PC, **Flowly** is a Dynamic-Island-style teleprompter that floats just under your webcam and follows your voice in real time — on-device, invisible to screen shares — so you keep perfect eye contact and never sound like you're reading. Unlike PromptSmart (phone-bound), Elgato (a $279 rig needing an RTX GPU), and web prompters (laggy tabs that leak into your share), Flowly lives where the call happens.

Tagline candidates: *"Say it, don't read it."* · *"The teleprompter that listens."* · *"Eye contact, on autopilot."*

**Differentiators to lean on:** (1) on-device privacy — audio never leaves the machine (enterprise/legal-safe; web rivals stream audio to cloud ASR); (2) screen-share invisibility — table stakes on Mac, near-absent on Windows; (3) the island aesthetic — screenshots are the growth loop; (4) voice-tracking *quality* — win the head-to-head videos reviewers already film against PromptSmart/Elgato.

**Framing caution:** market invisibility as presenter-assist ("your notes, off the share"), never as deception — the Cluely-style backlash pattern is real.

## Pricing (post-beta; nothing monetized at first launch)

Anchor between VODIUM ($15/mo, no voice) and Teleprompter.com (~$7.50–10/mo, no overlay); undercut Speakflow Plus:

- **Free:** 3 synced scripts, unlimited manual scroll, voice-following capped (~15 min/day). On-device ASR has near-zero marginal cost — be generous; the free tier is the demo that spreads in meetings.
- **Pro: $8/mo or $72/yr.** Unlimited voice-following, capture invisibility, unlimited sync.
- **Lifetime founder license: $99–129** (sync-light tier) — defuses indie "lifetime vs subscription" attack marketing (ShareSpeak $17.90, CueNotch $29.99).
- **Team: $15/seat/mo** — shared libraries, roles; the sales-team ICP VODIUM validated.

## Naming

"Flowly" is crowded: a USPTO-filed trademark exists (Tamade Inc., relaxation app, software classes), plus flowly.run and flowly-app.com, and the name is confusable with competitors Speakflow and FlowPrompter. **Run a class 9/42 clearance search before any branding spend.** Backup names: Islet, Cueline, Eyeline, Underlens, Sayso.

## Market risks

- **Fast followers:** ShareSpeak and FlowPrompter already ship voice tracking + invisibility on Windows; Flowly must win on polish, island UX, alignment quality, and sync.
- **Patent:** PromptSmart holds a VoiceTrack patent — a freedom-to-operate check on the alignment approach is prudent before US launch.
- **Platform risk:** Windows updates or new Zoom/Teams capture modes could change `WDA_EXCLUDEFROMCAPTURE` behavior; Microsoft or Elgato could commoditize the niche. Defense: call-platform-agnostic, privacy-first, fastest-to-polish.
