//! Scenario tests for the alignment engine — the in-repo seed of the
//! regression corpus described in docs/plan/03-voice-following.md. Each test
//! simulates a speaker (chunked final hypotheses, ~200 WPM) and asserts the
//! behavior contract: follow verbatim speech, hold still through ad-libs,
//! re-anchor after skips, jump back for retakes, and never move backward
//! without a Retake event.

use super::*;
use flowly_asr::{HypWord, Hypothesis};

const SCRIPT: &str = "\
Welcome back everyone to the Flowly launch demonstration today.
We built a brand new teleprompter that follows your voice.

Rockets satellites planets orbit gravity fusion energy tomorrow morning.
Distant galaxies collide while astronomers watch quietly tonight.

Closing section wraps everything together with gratitude and applause.
Thank you all for watching this recording goodbye friends.";

struct Sim {
    aligner: Aligner,
    script: CompiledScript,
    now: u64,
}

impl Sim {
    fn new(src: &str) -> Sim {
        let script = CompiledScript::compile(src);
        let aligner = Aligner::new(&script, AlignerConfig::default());
        Sim { aligner, script, now: 0 }
    }

    /// Speak words as chunked final hypotheses (3 words per chunk, 300 ms per
    /// word). Returns all events.
    fn speak(&mut self, words: &[&str]) -> Vec<AlignerEvent> {
        let mut events = Vec::new();
        for chunk in words.chunks(3) {
            let mut hyp_words = Vec::new();
            for w in chunk {
                self.now += 300;
                hyp_words.push(HypWord {
                    text: (*w).to_string(),
                    t_start: self.now - 280,
                    t_end: self.now - 20,
                    stable: true,
                });
            }
            let hyp = Hypothesis { words: hyp_words, is_final: true, t_emitted: self.now };
            events.extend(self.aligner.feed(&hyp, self.now));
        }
        events
    }

    fn silence(&mut self, ms: u64) -> Vec<AlignerEvent> {
        self.now += ms;
        self.aligner.tick(self.now)
    }

    /// Display words of the script tokens in `[from, to)`, as the speaker
    /// would say them.
    fn words(&self, from: usize, to: usize) -> Vec<String> {
        self.script.tokens[from..to].iter().map(|t| t.display.clone()).collect()
    }

    /// Token index of the nth occurrence of a display word.
    fn idx(&self, word: &str, nth: usize) -> usize {
        self.script
            .tokens
            .iter()
            .filter(|t| t.display.trim_matches(|c: char| !c.is_alphanumeric()).eq_ignore_ascii_case(word))
            .nth(nth)
            .map(|t| t.index)
            .unwrap_or_else(|| panic!("word {word:?} #{nth} not in script"))
    }
}

fn speak_refs(sim: &mut Sim, words: &[String]) -> Vec<AlignerEvent> {
    let refs: Vec<&str> = words.iter().map(String::as_str).collect();
    sim.speak(&refs)
}

fn cursor_moves(events: &[AlignerEvent]) -> Vec<usize> {
    events
        .iter()
        .filter_map(|e| match e {
            AlignerEvent::CursorMoved { token_index, .. } => Some(*token_index),
            _ => None,
        })
        .collect()
}

#[test]
fn tracks_verbatim_reading_to_the_end() {
    let mut sim = Sim::new(SCRIPT);
    let n = sim.script.tokens.len();
    let all = sim.words(0, n);
    let events = speak_refs(&mut sim, &all);

    let cursor = sim.aligner.cursor().expect("tracking started");
    assert!(cursor >= n - 2, "cursor {cursor} should reach the end ({n} tokens)");
    assert_eq!(sim.aligner.state(), State::Tracking);

    // Monotone, and every step bounded by the max_advance clamp.
    let moves = cursor_moves(&events);
    for pair in moves.windows(2) {
        assert!(pair[1] >= pair[0], "cursor went backward on verbatim read: {moves:?}");
        assert!(pair[1] - pair[0] <= AlignerConfig::default().max_advance, "oversized step: {moves:?}");
    }
    // No Lost/Searching excursions and no jumps on a clean read.
    assert!(
        !events.iter().any(|e| matches!(e, AlignerEvent::JumpDetected { .. })),
        "clean read must not produce jumps"
    );
    assert!(sim.aligner.wpm() > 100.0 && sim.aligner.wpm() < 350.0, "wpm {}", sim.aligner.wpm());
}

#[test]
fn ignores_fillers() {
    let mut sim = Sim::new(SCRIPT);
    sim.speak(&["welcome", "um", "back", "uh", "everyone", "like", "to", "the", "flowly"]);
    let cursor = sim.aligner.cursor().expect("tracking started");
    assert_eq!(cursor, sim.idx("Flowly", 0));
    assert_eq!(sim.aligner.state(), State::Tracking);
}

#[test]
fn misrecognized_words_still_track() {
    let mut sim = Sim::new(SCRIPT);
    // "teleprompter" mangled, "follows" heard phonetically as "follose".
    sim.speak(&["we", "built", "a", "brand", "new", "teleprompta", "that", "follose", "your", "voice"]);
    let cursor = sim.aligner.cursor().expect("tracking started");
    assert!(cursor >= sim.idx("voice", 0).saturating_sub(1), "cursor {cursor}");
}

#[test]
fn holds_still_during_adlib_then_reanchors() {
    let mut sim = Sim::new(SCRIPT);
    let stop = sim.idx("teleprompter", 0);
    let intro = sim.words(0, stop + 1);
    speak_refs(&mut sim, &intro);
    let frozen = sim.aligner.cursor().expect("tracking started");

    // Six content words of pure ad-lib: must freeze, never advance.
    let events = sim.speak(&["quick", "story", "about", "my", "dog", "chasing", "squirrels", "yesterday", "afternoon"]);
    assert!(
        cursor_moves(&events).iter().all(|&c| c <= frozen + 1),
        "cursor advanced through an ad-lib: {:?}",
        cursor_moves(&events)
    );
    assert_eq!(sim.aligner.state(), State::Lost, "ad-lib should end in LOST");
    let after_adlib = sim.aligner.cursor().unwrap();

    // Resume the script where we left off: re-anchor and continue.
    let resume_words = sim.words(stop + 1, stop + 8);
    speak_refs(&mut sim, &resume_words);
    assert_eq!(sim.aligner.state(), State::Tracking, "should re-anchor after ad-lib");
    assert!(sim.aligner.cursor().unwrap() > after_adlib);
}

#[test]
fn skipping_a_paragraph_reanchors_forward() {
    let mut sim = Sim::new(SCRIPT);
    let intro_end = sim.idx("teleprompter", 0);
    let w = sim.words(0, intro_end);
    speak_refs(&mut sim, &w);

    // Jump straight to the "Distant galaxies" sentence.
    let target = sim.idx("galaxies", 0);
    let words = sim.words(target - 1, target + 8);
    let events = speak_refs(&mut sim, &words);

    let cursor = sim.aligner.cursor().unwrap();
    assert!(cursor >= target + 3, "cursor {cursor} should reach the skipped-to sentence ({target})");
    assert_eq!(sim.aligner.state(), State::Tracking);
    // The engine either jump-detects the skip or crawls with a bounded first
    // step; wrong motion (backward) must never happen.
    for pair in cursor_moves(&events).windows(2) {
        assert!(pair[1] >= pair[0], "backward motion during forward skip");
    }
}

#[test]
fn jumping_back_for_a_retake_emits_retake() {
    let mut sim = Sim::new(SCRIPT);
    let through = sim.idx("tomorrow", 0);
    let w = sim.words(0, through + 1);
    speak_refs(&mut sim, &w);
    let high = sim.aligner.cursor().unwrap();

    // Re-record the rockets sentence from its start, stopping BEFORE the old
    // position — the cursor must come back, not sit still.
    let restart = sim.idx("Rockets", 0);
    let words = sim.words(restart, restart + 4);
    let events = speak_refs(&mut sim, &words);

    assert!(
        events.iter().any(|e| matches!(e, AlignerEvent::JumpDetected { kind: JumpKind::Retake, .. })),
        "expected a Retake jump, got {events:?}"
    );
    let cursor = sim.aligner.cursor().unwrap();
    assert!(cursor < high, "cursor {cursor} should be back before {high}");
    assert!(cursor >= restart, "cursor {cursor} should be inside the retaken sentence");

    // And tracking continues forward from the retaken position.
    let more = sim.words(restart + 4, restart + 8);
    speak_refs(&mut sim, &more);
    assert_eq!(sim.aligner.state(), State::Tracking);
    assert!(sim.aligner.cursor().unwrap() > cursor);
}

#[test]
fn pauses_on_silence_and_resumes() {
    let mut sim = Sim::new(SCRIPT);
    let w = sim.words(0, 6);
    speak_refs(&mut sim, &w);
    let before = sim.aligner.cursor().unwrap();

    let events = sim.silence(3000);
    assert!(events
        .iter()
        .any(|e| matches!(e, AlignerEvent::StateChanged { to: State::Paused, .. })));
    assert_eq!(sim.aligner.cursor().unwrap(), before, "PAUSED must freeze the cursor");

    let resume_words = sim.words(6, 12);
    speak_refs(&mut sim, &resume_words);
    assert_eq!(sim.aligner.state(), State::Tracking);
    assert!(sim.aligner.cursor().unwrap() > before);
}

#[test]
fn follows_spoken_numbers() {
    let mut sim = Sim::new("Revenue reached $3.5M this quarter, growing 47% since 2024.");
    sim.speak(&[
        "revenue", "reached", "three", "point", "five", "million", "dollars", "this", "quarter",
        "growing", "forty", "seven", "percent", "since", "twenty", "twenty", "four",
    ]);
    let n = sim.script.tokens.len();
    let cursor = sim.aligner.cursor().expect("tracking started");
    assert!(cursor >= n - 1, "cursor {cursor} of {n} — number verbalization stalled");
}

#[test]
fn manual_jump_always_wins() {
    let mut sim = Sim::new(SCRIPT);
    let w = sim.words(0, 9);
    speak_refs(&mut sim, &w);
    let target = sim.idx("Closing", 0);
    let events = sim.aligner.jump_to(target, sim.now);
    assert!(matches!(events.last(), Some(AlignerEvent::CursorMoved { token_index, .. }) if *token_index == target));
    assert_eq!(sim.aligner.cursor(), Some(target));

    // Tracking continues from the jumped-to position.
    let words = sim.words(target + 1, target + 6);
    speak_refs(&mut sim, &words);
    assert!(sim.aligner.cursor().unwrap() > target);
}

/// Deterministic fuzz: random interleavings of script chunks, garbage, and
/// silence must never panic, and the cursor must never move backward without
/// a Retake jump in the same batch.
#[test]
fn fuzz_cursor_never_moves_backward_without_retake() {
    let garbage = [
        "banana", "helicopter", "purple", "sandwich", "wombat", "guitar", "puddle", "cactus",
        "marble", "toaster", "velvet", "compass",
    ];
    let mut seed: u64 = 0x5eed_f10e;
    let mut rng = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };

    for round in 0..30 {
        let mut sim = Sim::new(SCRIPT);
        let n = sim.script.tokens.len();
        let mut last_cursor: Option<usize> = None;
        let mut pos = 0usize;
        for _ in 0..40 {
            let events = match rng() % 4 {
                0 | 1 => {
                    // Speak the next script chunk (sometimes with a corrupted word).
                    let len = 3 + rng() % 5;
                    let end = (pos + len).min(n);
                    if pos >= end {
                        pos = rng() % n;
                        continue;
                    }
                    let mut words = sim.words(pos, end);
                    if rng() % 3 == 0 && !words.is_empty() {
                        let i = rng() % words.len();
                        words[i] = format!("{}x", words[i]);
                    }
                    pos = end;
                    speak_refs(&mut sim, &words)
                }
                2 => {
                    let count = 2 + rng() % 6;
                    let picks: Vec<&str> = (0..count).map(|_| garbage[rng() % garbage.len()]).collect();
                    sim.speak(&picks)
                }
                _ => sim.silence(500 + (rng() % 3000) as u64),
            };
            let mut retake_in_batch = false;
            for e in &events {
                match e {
                    AlignerEvent::JumpDetected { kind: JumpKind::Retake, .. } => retake_in_batch = true,
                    AlignerEvent::CursorMoved { token_index, .. } => {
                        if let Some(last) = last_cursor {
                            assert!(
                                *token_index >= last || retake_in_batch,
                                "round {round}: cursor moved backward {last} -> {token_index} without Retake"
                            );
                        }
                        last_cursor = Some(*token_index);
                    }
                    _ => {}
                }
            }
        }
    }
}
