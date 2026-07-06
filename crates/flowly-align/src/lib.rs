//! The Flowly alignment engine.
//!
//! Pure and deterministic: no I/O, no threads, no clocks — time arrives as
//! `now_ms` parameters, hypotheses arrive via [`Aligner::feed`], and position
//! comes out as [`AlignerEvent`]s. Determinism is what makes the hypothesis-
//! replay regression corpus meaningful (docs/plan/03-voice-following.md).
//!
//! Behavior contract (the hysteresis rules):
//! - The cursor never moves backward in TRACKING except through a
//!   double-confirmed retake anchor, which emits `JumpDetected{kind: Retake}`.
//! - Forward advances are clamped to `max_advance` tokens per update unless a
//!   strong trailing anchor justifies a larger jump.
//! - Ad-libs freeze the cursor (LOST); prolonged loss widens to SEARCHING;
//!   silence >2 s parks in PAUSED. Wrong motion is the cardinal sin — when in
//!   doubt, hold still.

mod matcher;

pub use matcher::Heard;

use flowly_asr::Hypothesis;
use flowly_script::CompiledScript;
use matcher::{align_window, prepare, AlignOutcome, PrepToken};
use serde::Serialize;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum State {
    Tracking,
    Lost,
    Searching,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum JumpKind {
    Retake,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AlignerEvent {
    CursorMoved { token_index: usize, confidence: f32, wpm: f32 },
    StateChanged { from: State, to: State },
    JumpDetected { from: usize, to: usize, kind: JumpKind },
}

#[derive(Debug, Clone)]
pub struct AlignerConfig {
    /// Confidence needed to advance the cursor.
    pub theta_track: f32,
    /// Confidence below which content words count toward loss-of-lock.
    pub theta_lost: f32,
    /// Max forward tokens per update without a strong anchor.
    pub max_advance: usize,
    /// Consecutive low-score content words before TRACKING → LOST.
    pub lost_after: usize,
    /// LOST → SEARCHING after this long without a re-anchor.
    pub lost_to_search_ms: u64,
    /// Silence before any state → PAUSED.
    pub pause_after_ms: u64,
    /// Tracking window behind the cursor (covers near retakes).
    pub window_back: usize,
    /// Tracking window ahead of the cursor.
    pub window_fwd: usize,
    /// Forward window growth while LOST (covers skipped paragraphs).
    pub search_fwd: usize,
    /// Heard words fed to each alignment (K).
    pub context_words: usize,
    /// Trailing exact/phonetic content matches to re-anchor from LOST.
    pub reanchor_content: usize,
    /// Trailing matched words required to accept a SEARCHING anchor.
    pub search_anchor_words: usize,
}

impl Default for AlignerConfig {
    fn default() -> Self {
        AlignerConfig {
            theta_track: 0.55,
            theta_lost: 0.30,
            max_advance: 6,
            lost_after: 4,
            lost_to_search_ms: 5000,
            pause_after_ms: 2000,
            window_back: 40,
            window_fwd: 120,
            search_fwd: 400,
            context_words: 10,
            reanchor_content: 3,
            search_anchor_words: 4,
        }
    }
}

pub struct Aligner {
    cfg: AlignerConfig,
    tokens: Vec<PrepToken>,
    state: State,
    /// State to restore when speech resumes after PAUSED.
    stashed: State,
    cursor: Option<usize>,
    heard: VecDeque<Heard>,
    prev_partial: Vec<String>,
    stable_count: usize,
    low_streak: usize,
    lost_since: Option<u64>,
    /// Backward-jump candidate awaiting its second confirmation.
    pending_back: Option<(usize, u8)>,
    /// SEARCHING candidate awaiting its second confirmation.
    pending_search: Option<(usize, u8)>,
    last_heard_ms: Option<u64>,
    wpm: f32,
    last_advance: Option<(u64, usize)>,
}

impl Aligner {
    pub fn new(script: &CompiledScript, cfg: AlignerConfig) -> Aligner {
        Aligner {
            cfg,
            tokens: prepare(script),
            state: State::Tracking,
            stashed: State::Tracking,
            cursor: None,
            heard: VecDeque::new(),
            prev_partial: Vec::new(),
            stable_count: 0,
            low_streak: 0,
            lost_since: None,
            pending_back: None,
            pending_search: None,
            last_heard_ms: None,
            wpm: 0.0,
            last_advance: None,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    /// Last confirmed token, if tracking has started.
    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    pub fn wpm(&self) -> f32 {
        self.wpm
    }

    /// Manual override (hotkey nudge / click): always wins, instantly.
    pub fn jump_to(&mut self, token_index: usize, now_ms: u64) -> Vec<AlignerEvent> {
        let token_index = token_index.min(self.tokens.len().saturating_sub(1));
        let mut events = Vec::new();
        self.set_state(State::Tracking, &mut events);
        self.cursor = Some(token_index);
        self.low_streak = 0;
        self.lost_since = None;
        self.pending_back = None;
        self.pending_search = None;
        self.heard.clear();
        self.prev_partial.clear();
        self.stable_count = 0;
        self.last_advance = Some((now_ms, token_index));
        events.push(AlignerEvent::CursorMoved {
            token_index,
            confidence: 1.0,
            wpm: self.wpm,
        });
        events
    }

    /// Clock tick with no new hypothesis: drives silence → PAUSED.
    pub fn tick(&mut self, now_ms: u64) -> Vec<AlignerEvent> {
        let mut events = Vec::new();
        if self.state != State::Paused {
            if let Some(last) = self.last_heard_ms {
                if now_ms.saturating_sub(last) > self.cfg.pause_after_ms {
                    self.stashed = self.state;
                    self.set_state(State::Paused, &mut events);
                }
            }
        }
        events
    }

    /// Feed one streaming hypothesis. Returns the events it caused.
    pub fn feed(&mut self, hyp: &Hypothesis, now_ms: u64) -> Vec<AlignerEvent> {
        let mut events = Vec::new();
        let pushed = self.ingest(hyp);
        if pushed == 0 {
            return events;
        }
        self.last_heard_ms = Some(now_ms);

        if self.state == State::Paused {
            let to = self.stashed;
            self.set_state(to, &mut events);
        }

        let outcome = self.align();
        match self.state {
            State::Tracking => self.on_tracking(outcome, now_ms, pushed, &mut events),
            State::Lost => self.on_lost(outcome, now_ms, &mut events),
            State::Searching => self.on_searching(outcome, now_ms, &mut events),
            State::Paused => unreachable!("resumed above"),
        }
        events
    }

    // ---- ingestion ----------------------------------------------------

    /// Append newly stable words to the heard buffer. Returns how many were
    /// pushed (fillers and empty husks are dropped).
    fn ingest(&mut self, hyp: &Hypothesis) -> usize {
        let texts: Vec<String> = hyp.words.iter().map(|w| w.text.clone()).collect();
        let mut pushed = 0usize;
        if hyp.is_final {
            for w in hyp.words.iter().skip(self.stable_count) {
                pushed += usize::from(self.push_heard(&w.text, w.t_end));
            }
            self.stable_count = 0;
            self.prev_partial.clear();
        } else {
            let lcp = self
                .prev_partial
                .iter()
                .zip(texts.iter())
                .take_while(|(a, b)| a == b)
                .count();
            for w in hyp.words.iter().take(lcp).skip(self.stable_count) {
                pushed += usize::from(self.push_heard(&w.text, w.t_end));
            }
            self.stable_count = self.stable_count.max(lcp);
            self.prev_partial = texts;
        }
        pushed
    }

    fn push_heard(&mut self, raw: &str, t_ms: u64) -> bool {
        match Heard::from_raw(raw, t_ms) {
            Some(h) => {
                self.heard.push_back(h);
                while self.heard.len() > 24 {
                    self.heard.pop_front();
                }
                true
            }
            None => false,
        }
    }

    // ---- alignment ----------------------------------------------------

    fn align(&self) -> Option<AlignOutcome> {
        let context: Vec<Heard> = self
            .heard
            .iter()
            .rev()
            .take(self.cfg.context_words)
            .rev()
            .cloned()
            .collect();
        let anchor = self.cursor.unwrap_or(0);
        match self.state {
            State::Searching => {
                align_window(&self.tokens, &context, 0, self.tokens.len(), Some(anchor))
            }
            State::Lost => align_window(
                &self.tokens,
                &context,
                anchor.saturating_sub(self.cfg.window_back),
                anchor + self.cfg.search_fwd,
                None,
            ),
            _ => align_window(
                &self.tokens,
                &context,
                anchor.saturating_sub(self.cfg.window_back),
                anchor + self.cfg.window_fwd,
                None,
            ),
        }
    }

    // ---- state handlers -------------------------------------------------

    fn on_tracking(
        &mut self,
        outcome: Option<AlignOutcome>,
        now_ms: u64,
        pushed: usize,
        events: &mut Vec<AlignerEvent>,
    ) {
        let confident = outcome
            .as_ref()
            .map(|o| o.confidence >= self.cfg.theta_track)
            .unwrap_or(false);

        if confident {
            let o = outcome.expect("confident implies outcome");
            self.low_streak = 0;
            match self.cursor {
                None => {
                    self.cursor = Some(o.cand);
                    self.last_advance = Some((now_ms, o.cand));
                    events.push(AlignerEvent::CursorMoved {
                        token_index: o.cand,
                        confidence: o.confidence,
                        wpm: self.wpm,
                    });
                }
                Some(cur) if o.cand >= cur => {
                    self.pending_back = None;
                    let delta = o.cand - cur;
                    let strong_anchor = o.trail_words >= self.cfg.search_anchor_words;
                    let target = if delta <= self.cfg.max_advance || strong_anchor {
                        o.cand
                    } else {
                        cur + self.cfg.max_advance
                    };
                    if target > cur {
                        if target - cur > 2 * self.cfg.max_advance {
                            events.push(AlignerEvent::JumpDetected {
                                from: cur,
                                to: target,
                                kind: JumpKind::Skip,
                            });
                        }
                        self.update_wpm(now_ms, target);
                        self.cursor = Some(target);
                        events.push(AlignerEvent::CursorMoved {
                            token_index: target,
                            confidence: o.confidence,
                            wpm: self.wpm,
                        });
                    }
                }
                Some(_) => {
                    self.try_backward(&o, now_ms, events);
                }
            }
        } else {
            // The full-context alignment is polluted right after a retake:
            // pre-retake words anchor the DP forward and drown the backward
            // candidate. A recent-words-only alignment sees the retake.
            if let Some(o) = self.align_recent() {
                if o.confidence >= self.cfg.theta_track && o.trail_content >= self.cfg.reanchor_content {
                    self.try_backward(&o, now_ms, events);
                    if self.state == State::Tracking
                        && self.cursor.map(|c| c + 2 >= o.cand).unwrap_or(false)
                    {
                        self.low_streak = 0;
                        return;
                    }
                }
            }
            // Count content words toward loss-of-lock only when the score is
            // truly low; the middle zone just holds position.
            let low = outcome
                .as_ref()
                .map(|o| o.confidence < self.cfg.theta_lost)
                .unwrap_or(true);
            if low && self.cursor.is_some() {
                let content_pushed = self
                    .heard
                    .iter()
                    .rev()
                    .take(pushed)
                    .filter(|h| h.is_content)
                    .count();
                self.low_streak += content_pushed;
                if self.low_streak >= self.cfg.lost_after {
                    self.low_streak = 0;
                    self.lost_since = Some(now_ms);
                    self.set_state(State::Lost, events);
                }
            }
        }
    }

    /// Alignment over only the most recent words — immune to pre-retake
    /// context pollution.
    fn align_recent(&self) -> Option<AlignOutcome> {
        if self.heard.len() < 3 {
            return None;
        }
        let context: Vec<Heard> = self.heard.iter().rev().take(5).rev().cloned().collect();
        let anchor = self.cursor.unwrap_or(0);
        align_window(
            &self.tokens,
            &context,
            anchor.saturating_sub(self.cfg.window_back),
            anchor + self.cfg.window_fwd,
            None,
        )
    }

    /// Backward (retake) candidate: needs a real anchor — at least 3
    /// consecutive exact/phonetic content-word matches (fuzzy matches break
    /// the trail, so ad-lib garbage can't fabricate one) — plus a second
    /// confirmation before the cursor may ever move back.
    fn try_backward(&mut self, o: &AlignOutcome, now_ms: u64, events: &mut Vec<AlignerEvent>) {
        let Some(cur) = self.cursor else { return };
        if cur <= o.cand + 2 || o.trail_content < self.cfg.reanchor_content || o.trail_words < 3 {
            return;
        }
        match self.pending_back {
            Some((pos, _)) if o.cand.abs_diff(pos) <= 3 => {
                self.pending_back = None;
                events.push(AlignerEvent::JumpDetected { from: cur, to: o.cand, kind: JumpKind::Retake });
                self.cursor = Some(o.cand);
                self.last_advance = Some((now_ms, o.cand));
                events.push(AlignerEvent::CursorMoved {
                    token_index: o.cand,
                    confidence: o.confidence,
                    wpm: self.wpm,
                });
            }
            _ => self.pending_back = Some((o.cand, 1)),
        }
    }

    fn on_lost(&mut self, outcome: Option<AlignOutcome>, now_ms: u64, events: &mut Vec<AlignerEvent>) {
        if let Some(o) = outcome {
            if o.confidence >= self.cfg.theta_track && o.trail_content >= self.cfg.reanchor_content {
                self.commit_reanchor(o, now_ms, events);
                return;
            }
        }
        if let Some(since) = self.lost_since {
            if now_ms.saturating_sub(since) > self.cfg.lost_to_search_ms {
                self.lost_since = None;
                self.pending_search = None;
                self.set_state(State::Searching, events);
            }
        }
    }

    fn on_searching(
        &mut self,
        outcome: Option<AlignOutcome>,
        now_ms: u64,
        events: &mut Vec<AlignerEvent>,
    ) {
        let Some(o) = outcome else { return };
        let strong = o.confidence >= self.cfg.theta_track
            && o.trail_words >= self.cfg.search_anchor_words
            && o.trail_content >= 2;
        if !strong {
            return;
        }
        match self.pending_search {
            Some((pos, _)) if o.cand.abs_diff(pos) <= 3 => {
                self.pending_search = None;
                self.commit_reanchor(o, now_ms, events);
            }
            _ => self.pending_search = Some((o.cand, 1)),
        }
    }

    fn commit_reanchor(&mut self, o: AlignOutcome, now_ms: u64, events: &mut Vec<AlignerEvent>) {
        let from = self.cursor;
        self.set_state(State::Tracking, events);
        self.low_streak = 0;
        self.lost_since = None;
        if let Some(cur) = from {
            if o.cand + 2 < cur {
                events.push(AlignerEvent::JumpDetected { from: cur, to: o.cand, kind: JumpKind::Retake });
            } else if o.cand > cur + self.cfg.max_advance {
                events.push(AlignerEvent::JumpDetected { from: cur, to: o.cand, kind: JumpKind::Skip });
            }
        }
        self.cursor = Some(o.cand);
        self.last_advance = Some((now_ms, o.cand));
        events.push(AlignerEvent::CursorMoved {
            token_index: o.cand,
            confidence: o.confidence,
            wpm: self.wpm,
        });
    }

    fn set_state(&mut self, to: State, events: &mut Vec<AlignerEvent>) {
        if self.state != to {
            events.push(AlignerEvent::StateChanged { from: self.state, to });
            self.state = to;
        }
    }

    fn update_wpm(&mut self, now_ms: u64, new_cursor: usize) {
        if let Some((t0, c0)) = self.last_advance {
            let dt_min = (now_ms.saturating_sub(t0)) as f32 / 60_000.0;
            let words = new_cursor.saturating_sub(c0) as f32;
            if dt_min > 0.0005 && words > 0.0 {
                let inst = (words / dt_min).clamp(40.0, 400.0);
                self.wpm = if self.wpm == 0.0 { inst } else { 0.85 * self.wpm + 0.15 * inst };
            }
        }
        self.last_advance = Some((now_ms, new_cursor));
    }
}

#[cfg(test)]
mod tests;
