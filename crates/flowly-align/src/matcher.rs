//! Banded local alignment of heard words against script tokens.
//!
//! A Smith-Waterman-style DP (score-maximizing, local) aligns the last K stable
//! heard words against a window of script tokens. Script tokens match through
//! any of their spoken variants (a variant with L words consumes L heard
//! words), with exact / phonetic / fuzzy word costs, near-free gaps for dropped
//! script words, and penalties for ASR insertions. The DP endpoint is the
//! cursor candidate; the backtrack yields the confidence signal and the
//! trailing exact/phonetic anchor run used by the state machine's hysteresis.

use flowly_script::normalize;
use flowly_script::phonetic;
use flowly_script::CompiledScript;

const MATCH_EXACT: f32 = 1.0;
const MATCH_PHONETIC: f32 = 0.8;
const MATCH_FUZZY: f32 = 0.55;
const COST_SUBSTITUTION: f32 = -0.6;
const COST_SKIP_SCRIPT: f32 = -0.35;
const COST_EXTRA_HYP: f32 = -0.5;
/// Positional prior for whole-script search: max penalty at >=200 tokens away.
const SEARCH_DISTANCE_PENALTY: f32 = 0.3;
const SEARCH_SENTENCE_START_BONUS: f32 = 0.15;

/// One stable heard word, normalized.
#[derive(Debug, Clone)]
pub struct Heard {
    pub text: String,
    pub key: String,
    pub t_ms: u64,
    pub is_content: bool,
}

impl Heard {
    /// Normalize a raw ASR word. Returns None for fillers and empty husks.
    pub fn from_raw(raw: &str, t_ms: u64) -> Option<Heard> {
        let lower = raw.to_lowercase().replace(['\u{2018}', '\u{2019}'], "'");
        let kept: String = lower.chars().filter(|c| c.is_alphanumeric() || *c == '\'').collect();
        let trimmed = kept.trim_matches('\'');
        if trimmed.is_empty() || normalize::is_filler(trimmed) {
            return None;
        }
        let text = trimmed.replace('\'', "");
        let key = phonetic::key(&text);
        let is_content = !normalize::is_stopword(&text);
        Some(Heard { text, key, t_ms, is_content })
    }
}

#[derive(Debug, Clone)]
struct MatchWord {
    text: String,
    key: String,
}

/// Script token preprocessed for matching.
#[derive(Debug, Clone)]
pub struct PrepToken {
    variants: Vec<Vec<MatchWord>>,
    pub sentence_start: bool,
    pub is_content: bool,
}

pub fn prepare(script: &CompiledScript) -> Vec<PrepToken> {
    script
        .tokens
        .iter()
        .map(|t| PrepToken {
            variants: t
                .variants
                .iter()
                .map(|v| {
                    v.iter()
                        .map(|w| MatchWord { key: phonetic::key(w), text: w.clone() })
                        .collect()
                })
                .collect(),
            sentence_start: t.sentence_start,
            is_content: t.is_content,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Quality {
    Fuzzy,
    Phonetic,
    Exact,
}

#[derive(Debug, Clone, Copy)]
enum Step {
    Start,
    /// Script token skipped (speaker dropped it).
    GapScript,
    /// Extra heard word (ASR insertion).
    GapHyp,
    /// Token consumed against one heard word without a real match.
    Substitution,
    /// Token matched via a variant consuming `len` heard words.
    Match { len: u8, quality: Quality },
}

#[derive(Debug, Clone)]
pub struct AlignOutcome {
    /// Absolute script index of the last matched/consumed token.
    pub cand: usize,
    /// Matched fraction of the last <=6 content words heard (0..=1).
    pub confidence: f32,
    /// Heard words in the trailing consecutive exact/phonetic match run.
    pub trail_words: usize,
    /// Content script tokens inside that trailing run.
    pub trail_content: usize,
}

fn word_score(h: &Heard, v: &MatchWord) -> Option<(f32, Quality)> {
    if h.text == v.text {
        return Some((MATCH_EXACT, Quality::Exact));
    }
    if !h.key.is_empty() && h.key == v.key {
        return Some((MATCH_PHONETIC, Quality::Phonetic));
    }
    let dist = levenshtein(&h.text, &v.text);
    let max_len = h.text.len().max(v.text.len());
    if max_len >= 3 && (dist as f32) / (max_len as f32) <= 0.4 {
        return Some((MATCH_FUZZY, Quality::Fuzzy));
    }
    None
}

/// Align `heard` (the last K stable words) against script tokens
/// `[w0, w1)`. `prior_center` enables the positional prior + sentence-start
/// bonus used for whole-script SEARCHING.
pub fn align_window(
    tokens: &[PrepToken],
    heard: &[Heard],
    w0: usize,
    w1: usize,
    prior_center: Option<usize>,
) -> Option<AlignOutcome> {
    let w1 = w1.min(tokens.len());
    if w0 >= w1 || heard.is_empty() {
        return None;
    }
    let window = &tokens[w0..w1];
    let m = heard.len();
    let n = window.len();
    let width = n + 1;
    let mut score = vec![0f32; (m + 1) * width];
    let mut step = vec![Step::Start; (m + 1) * width];
    let at = |i: usize, j: usize| i * width + j;

    for i in 1..=m {
        for j in 1..=n {
            let mut best = 0f32;
            let mut best_step = Step::Start;
            let skip = score[at(i, j - 1)] + COST_SKIP_SCRIPT;
            if skip > best {
                best = skip;
                best_step = Step::GapScript;
            }
            let extra = score[at(i - 1, j)] + COST_EXTRA_HYP;
            if extra > best {
                best = extra;
                best_step = Step::GapHyp;
            }
            let sub = score[at(i - 1, j - 1)] + COST_SUBSTITUTION;
            if sub > best {
                best = sub;
                best_step = Step::Substitution;
            }
            for variant in &window[j - 1].variants {
                let len = variant.len();
                if len == 0 || len > i {
                    continue;
                }
                let mut total = 0f32;
                let mut min_quality = Quality::Exact;
                let mut ok = true;
                for (k, vw) in variant.iter().enumerate() {
                    match word_score(&heard[i - len + k], vw) {
                        Some((s, q)) => {
                            total += s;
                            min_quality = min_quality.min(q);
                        }
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    continue;
                }
                let cand = score[at(i - len, j - 1)] + total;
                if cand > best {
                    best = cand;
                    best_step = Step::Match { len: len as u8, quality: min_quality };
                }
            }
            score[at(i, j)] = best;
            step[at(i, j)] = best_step;
        }
    }

    // Endpoint: the alignment must end at the last heard word (i == m).
    let mut best_j = 0usize;
    let mut best_val = f32::MIN;
    for j in 1..=n {
        let mut v = score[at(m, j)];
        if let Some(center) = prior_center {
            let dist = (w0 + j - 1).abs_diff(center) as f32;
            v -= SEARCH_DISTANCE_PENALTY * (dist / 200.0).min(1.0);
            if window[j - 1].sentence_start {
                v += SEARCH_SENTENCE_START_BONUS;
            }
        }
        if v > best_val {
            best_val = v;
            best_j = j;
        }
    }
    if best_j == 0 || score[at(m, best_j)] <= 0.0 {
        return None;
    }

    // Backtrack.
    let mut i = m;
    let mut j = best_j;
    let mut cand: Option<usize> = None;
    let mut matched_heard = vec![false; m];
    let mut trail_words = 0usize;
    let mut trail_content = 0usize;
    let mut trail_alive = true;
    loop {
        match step[at(i, j)] {
            Step::Start => break,
            Step::GapScript => {
                j -= 1;
                // Trailing skipped script tokens don't break the anchor run —
                // the speaker simply hasn't reached them.
            }
            Step::GapHyp => {
                i -= 1;
                trail_alive = false;
            }
            Step::Substitution => {
                if cand.is_none() {
                    cand = Some(w0 + j - 1);
                }
                i -= 1;
                j -= 1;
                trail_alive = false;
            }
            Step::Match { len, quality } => {
                if cand.is_none() {
                    cand = Some(w0 + j - 1);
                }
                let len = len as usize;
                for k in (i - len)..i {
                    matched_heard[k] = true;
                }
                if trail_alive {
                    if quality >= Quality::Phonetic {
                        trail_words += len;
                        if window[j - 1].is_content {
                            trail_content += 1;
                        }
                    } else {
                        trail_alive = false;
                    }
                }
                i -= len;
                j -= 1;
            }
        }
        if i == 0 && j == 0 {
            break;
        }
        if i == 0 || j == 0 {
            break;
        }
    }
    let cand = cand?;

    // Confidence: matched fraction of the last <=6 content words heard.
    let mut considered = 0usize;
    let mut matched = 0usize;
    for k in (0..m).rev() {
        if !heard[k].is_content {
            continue;
        }
        considered += 1;
        if matched_heard[k] {
            matched += 1;
        }
        if considered == 6 {
            break;
        }
    }
    let confidence = if considered == 0 {
        // Only stopwords heard: fall back to overall matched fraction.
        matched_heard.iter().filter(|&&b| b).count() as f32 / m as f32
    } else {
        matched as f32 / considered as f32
    };

    Some(AlignOutcome { cand, confidence, trail_words, trail_content })
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != cb);
            cur[j + 1] = sub.min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heard(words: &[&str]) -> Vec<Heard> {
        words
            .iter()
            .enumerate()
            .filter_map(|(i, w)| Heard::from_raw(w, (i as u64) * 300))
            .collect()
    }

    fn prep(src: &str) -> Vec<PrepToken> {
        prepare(&CompiledScript::compile(src))
    }

    #[test]
    fn verbatim_match_lands_on_last_word() {
        let tokens = prep("the quick brown fox jumps over the lazy dog");
        let out = align_window(&tokens, &heard(&["quick", "brown", "fox"]), 0, tokens.len(), None)
            .expect("aligns");
        assert_eq!(out.cand, 3); // "fox"
        assert!(out.confidence > 0.9, "confidence {}", out.confidence);
        assert!(out.trail_words >= 3);
    }

    #[test]
    fn fuzzy_and_phonetic_words_still_match() {
        let tokens = prep("we shipped the voice following feature this week");
        let out = align_window(
            &tokens,
            &heard(&["shipped", "the", "voice", "followin", "feature"]),
            0,
            tokens.len(),
            None,
        )
        .expect("aligns");
        assert_eq!(out.cand, 5); // "feature"
        assert!(out.confidence > 0.9);
    }

    #[test]
    fn dropped_script_word_is_a_cheap_gap() {
        let tokens = prep("please welcome our brand new product demo");
        // Speaker skips "brand".
        let out = align_window(&tokens, &heard(&["welcome", "our", "new", "product"]), 0, tokens.len(), None)
            .expect("aligns");
        assert_eq!(out.cand, 5); // "product"
    }

    #[test]
    fn multiword_variant_consumes_multiple_heard_words() {
        let tokens = prep("revenue hit $3.5M last quarter");
        let out = align_window(
            &tokens,
            &heard(&["hit", "three", "point", "five", "million", "dollars", "last"]),
            0,
            tokens.len(),
            None,
        )
        .expect("aligns");
        assert_eq!(out.cand, 3); // "last"
    }

    #[test]
    fn garbage_does_not_align() {
        let tokens = prep("the quarterly report shows strong growth");
        let out = align_window(
            &tokens,
            &heard(&["bananas", "helicopter", "purple", "sandwich"]),
            0,
            tokens.len(),
            None,
        );
        assert!(out.is_none() || out.unwrap().confidence < 0.3);
    }

    #[test]
    fn homophone_matches_phonetically() {
        let tokens = prep("their team won the game");
        let out = align_window(&tokens, &heard(&["there", "team", "won"]), 0, tokens.len(), None)
            .expect("aligns");
        assert_eq!(out.cand, 2);
        assert!(out.trail_words >= 3);
    }
}
