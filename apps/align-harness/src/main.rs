//! align-harness — offline test rig for the Flowly alignment engine.
//!
//! Two subcommands:
//!
//! ```text
//! align-harness synth --script s.md --out-hyp h.jsonl --out-truth t.tsv \
//!     [--seed 42] [--wer 0.08] [--wpm 180] [--chunk 3]
//! align-harness run --script s.md --hyp h.jsonl [--truth t.tsv] [--gate]
//! ```
//!
//! `synth` simulates a speaker reading the script (word-error injection,
//! fillers, chunked partial+final hypotheses) and writes both the hypothesis
//! log and the ground-truth timeline. `run` replays a hypothesis log through
//! the aligner and reports the corpus metrics from
//! docs/plan/03-voice-following.md (token error, wrong-motion count, re-lock
//! times); `--gate` exits non-zero when they regress — that is the CI hook.
//! Real recorded corpus cases replace synthetic ones as they are collected;
//! the file formats are identical.

use flowly_align::{Aligner, AlignerConfig, AlignerEvent, State};
use flowly_asr::{replay::ReplayAsr, HypWord, Hypothesis};
use flowly_script::CompiledScript;
use std::collections::HashMap;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        eprintln!("usage: align-harness <synth|run> [options]");
        return ExitCode::from(2);
    };
    let opts = parse_opts(&args[1..]);
    match cmd.as_str() {
        "synth" => synth(&opts),
        "run" => run(&opts),
        _ => {
            eprintln!("unknown command {cmd:?}; expected synth or run");
            ExitCode::from(2)
        }
    }
}

fn parse_opts(args: &[String]) -> HashMap<String, String> {
    let mut opts = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        if let Some(key) = args[i].strip_prefix("--") {
            let val = args.get(i + 1).filter(|v| !v.starts_with("--")).cloned();
            match val {
                Some(v) => {
                    opts.insert(key.to_string(), v);
                    i += 2;
                }
                None => {
                    opts.insert(key.to_string(), "true".to_string());
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    opts
}

fn req<'a>(opts: &'a HashMap<String, String>, key: &str) -> &'a str {
    opts.get(key).unwrap_or_else(|| {
        eprintln!("missing required option --{key}");
        std::process::exit(2);
    })
}

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn chance(&mut self, p: f64) -> bool {
        (self.next() % 10_000) as f64 / 10_000.0 < p
    }
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[(self.next() as usize) % items.len()]
    }
}

// ---- synth ----------------------------------------------------------------

fn synth(opts: &HashMap<String, String>) -> ExitCode {
    let script_path = req(opts, "script");
    let source = std::fs::read_to_string(script_path).expect("read script");
    let script = CompiledScript::compile(&source);
    let seed: u64 = opts.get("seed").map(|s| s.parse().expect("--seed")).unwrap_or(42);
    let wer: f64 = opts.get("wer").map(|s| s.parse().expect("--wer")).unwrap_or(0.08);
    let wpm: f64 = opts.get("wpm").map(|s| s.parse().expect("--wpm")).unwrap_or(180.0);
    let chunk: usize = opts.get("chunk").map(|s| s.parse().expect("--chunk")).unwrap_or(3);

    let mut rng = Lcg(seed);
    let word_ms = (60_000.0 / wpm) as u64;
    let fillers = ["um", "uh", "you", "know"];

    // The speaker says each token's primary spoken variant.
    let mut spoken: Vec<(String, usize)> = Vec::new(); // (word, true token index)
    for tok in &script.tokens {
        for w in &tok.variants[0] {
            spoken.push((w.clone(), tok.index));
        }
    }

    let mut events: Vec<Hypothesis> = Vec::new();
    let mut truth: Vec<(u64, usize)> = Vec::new();
    let mut now: u64 = 0;
    let mut pending: Vec<(HypWord, usize)> = Vec::new();

    for (word, tok_idx) in spoken {
        if rng.chance(0.03) {
            now += word_ms;
            pending.push((
                HypWord { text: rng.pick(&fillers).to_string(), t_start: now - word_ms, t_end: now, stable: false },
                tok_idx,
            ));
        }
        now += word_ms;
        let text = if rng.chance(wer) { corrupt(&word, &mut rng) } else { word };
        pending.push((HypWord { text, t_start: now - word_ms, t_end: now, stable: false }, tok_idx));
        truth.push((now, tok_idx));

        if pending.len() >= chunk {
            flush_chunk(&mut events, &mut pending, now, word_ms);
        }
    }
    if !pending.is_empty() {
        flush_chunk(&mut events, &mut pending, now, word_ms);
    }

    let hyp_out = req(opts, "out-hyp");
    std::fs::write(hyp_out, flowly_asr::replay::to_jsonl(&events)).expect("write hypotheses");
    if let Some(truth_out) = opts.get("out-truth") {
        let mut tsv = String::from("time_ms\ttoken_index\n");
        for (t, idx) in &truth {
            tsv.push_str(&format!("{t}\t{idx}\n"));
        }
        std::fs::write(truth_out, tsv).expect("write truth");
    }
    eprintln!("synth: {} hypotheses over {:.1}s -> {}", events.len(), now as f64 / 1000.0, hyp_out);
    ExitCode::SUCCESS
}

/// Emit a revising partial (all but the last word) followed by the final —
/// exercises the aligner's stable-prefix handling like a real streaming ASR.
fn flush_chunk(events: &mut Vec<Hypothesis>, pending: &mut Vec<(HypWord, usize)>, now: u64, word_ms: u64) {
    let words: Vec<HypWord> = pending.drain(..).map(|(w, _)| w).collect();
    if words.len() > 1 {
        events.push(Hypothesis {
            words: words[..words.len() - 1].to_vec(),
            is_final: false,
            t_emitted: now.saturating_sub(word_ms / 2),
        });
    }
    events.push(Hypothesis { words, is_final: true, t_emitted: now });
}

fn corrupt(word: &str, rng: &mut Lcg) -> String {
    let chars: Vec<char> = word.chars().collect();
    match rng.next() % 3 {
        0 if chars.len() > 3 => {
            // Drop a middle character.
            let i = 1 + (rng.next() as usize) % (chars.len() - 2);
            chars.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, c)| c).collect()
        }
        1 => format!("{word}s"),
        _ if chars.len() > 3 => {
            // Swap two adjacent middle characters.
            let mut c = chars;
            let i = 1 + (rng.next() as usize) % (c.len() - 2);
            c.swap(i, i - 1);
            c.into_iter().collect()
        }
        _ => format!("{word}a"),
    }
}

// ---- run --------------------------------------------------------------------

fn run(opts: &HashMap<String, String>) -> ExitCode {
    let source = std::fs::read_to_string(req(opts, "script")).expect("read script");
    let script = CompiledScript::compile(&source);
    let hyp_text = std::fs::read_to_string(req(opts, "hyp")).expect("read hypotheses");
    let mut replay = ReplayAsr::from_jsonl(&hyp_text).expect("parse hypotheses");
    let truth: Option<Vec<(u64, usize)>> = opts.get("truth").map(|p| {
        std::fs::read_to_string(p)
            .expect("read truth")
            .lines()
            .skip(1)
            .filter_map(|l| {
                let mut parts = l.split('\t');
                Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
            })
            .collect()
    });

    let mut aligner = Aligner::new(&script, AlignerConfig::default());
    let mut errors: Vec<i64> = Vec::new();
    let mut wrong_motion = 0usize;
    let mut lock_losses = 0usize;
    let mut relock_ms: Vec<u64> = Vec::new();
    let mut lost_at: Option<u64> = None;
    let mut prev_cursor: Option<usize> = None;
    let mut moves = 0usize;

    while let Some(hyp) = replay.next_event() {
        let now = hyp.t_emitted;
        let mut events = aligner.tick(now);
        events.extend(aligner.feed(&hyp, now));
        for e in &events {
            match e {
                AlignerEvent::StateChanged { to: State::Lost, .. } => {
                    lock_losses += 1;
                    lost_at = Some(now);
                }
                AlignerEvent::StateChanged { to: State::Tracking, .. } => {
                    if let Some(t0) = lost_at.take() {
                        relock_ms.push(now - t0);
                    }
                }
                AlignerEvent::CursorMoved { token_index, .. } => {
                    moves += 1;
                    if let Some(truth) = &truth {
                        let want = truth_at(truth, now);
                        let err = *token_index as i64 - want as i64;
                        errors.push(err.abs());
                        if let Some(prev) = prev_cursor {
                            let prev_err = (prev as i64 - want as i64).abs();
                            if err.abs() > prev_err + 2 {
                                wrong_motion += 1;
                            }
                        }
                    }
                    prev_cursor = Some(*token_index);
                }
                _ => {}
            }
        }
    }

    errors.sort_unstable();
    let median = errors.get(errors.len() / 2).copied().unwrap_or(0);
    let p95 = errors.get(errors.len() * 95 / 100).copied().unwrap_or(0);
    let final_cursor = aligner.cursor();
    let relock_max = relock_ms.iter().max().copied().unwrap_or(0);

    println!("tokens          : {}", script.tokens.len());
    println!("cursor moves    : {moves}");
    println!("final cursor    : {final_cursor:?}");
    println!("lock losses     : {lock_losses}");
    println!("re-lock max     : {relock_max} ms");
    if truth.is_some() {
        println!("token error med : {median}");
        println!("token error p95 : {p95}");
        println!("wrong motion    : {wrong_motion}");
    }

    if opts.contains_key("gate") {
        let reached_end = final_cursor.map(|c| c + 5 >= script.tokens.len()).unwrap_or(false);
        let ok = wrong_motion == 0 && p95 <= 8 && reached_end;
        if !ok {
            eprintln!("GATE FAILED: wrong_motion={wrong_motion} p95={p95} reached_end={reached_end}");
            return ExitCode::FAILURE;
        }
        eprintln!("gate passed");
    }
    ExitCode::SUCCESS
}

fn truth_at(truth: &[(u64, usize)], t: u64) -> usize {
    match truth.binary_search_by_key(&t, |(tm, _)| *tm) {
        Ok(i) => truth[i].1,
        Err(0) => 0,
        Err(i) => truth[i - 1].1,
    }
}
