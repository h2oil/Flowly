//! ASR abstraction for Flowly.
//!
//! The aligner never talks to a concrete recognizer: it consumes [`Hypothesis`]
//! values from anything implementing [`AsrStream`]. Rev1 ships two impls:
//! [`replay::ReplayAsr`] (reads recorded hypothesis logs — the backbone of the
//! regression corpus) and, on Windows in a later rev, a sherpa-onnx streaming
//! Zipformer. A cloud engine slot is reserved but deliberately not built.

use serde::{Deserialize, Serialize};

/// One recognized word inside a streaming hypothesis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HypWord {
    pub text: String,
    /// Start time in milliseconds from session start.
    pub t_start: u64,
    /// End time in milliseconds from session start.
    pub t_end: u64,
    /// True once this word has survived two consecutive partials unchanged.
    /// Recognizers that don't track stability leave this false; the aligner
    /// computes its own stable prefix in that case.
    #[serde(default)]
    pub stable: bool,
}

/// A streaming recognition hypothesis. Partials (`is_final == false`) may be
/// revised by later partials; a final commits the utterance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hypothesis {
    pub words: Vec<HypWord>,
    pub is_final: bool,
    /// Emission time in milliseconds from session start.
    pub t_emitted: u64,
}

/// A push-based streaming recognizer.
///
/// `push_samples` feeds 16 kHz mono i16 PCM; hypotheses are pulled with
/// `poll`. `set_hotwords` re-biases decoding toward upcoming script tokens
/// (a no-op for engines without contextual biasing).
pub trait AsrStream {
    fn push_samples(&mut self, samples: &[i16]);
    fn poll(&mut self) -> Option<Hypothesis>;
    fn set_hotwords(&mut self, words: &[String]);
}

pub mod replay {
    use super::Hypothesis;

    /// Replays a recorded hypothesis log (JSON Lines, one [`Hypothesis`] per
    /// line). Deterministic input for the aligner regression suite; audio is
    /// not needed and never stored.
    pub struct ReplayAsr {
        events: std::vec::IntoIter<Hypothesis>,
    }

    impl ReplayAsr {
        pub fn from_jsonl(text: &str) -> Result<Self, serde_json::Error> {
            let events = text
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(serde_json::from_str::<Hypothesis>)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Self { events: events.into_iter() })
        }

        pub fn next_event(&mut self) -> Option<Hypothesis> {
            self.events.next()
        }
    }

    pub fn to_jsonl(events: &[Hypothesis]) -> String {
        let mut out = String::new();
        for e in events {
            out.push_str(&serde_json::to_string(e).expect("hypothesis serializes"));
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_roundtrip() {
        let events = vec![
            Hypothesis {
                words: vec![HypWord { text: "hello".into(), t_start: 0, t_end: 300, stable: false }],
                is_final: false,
                t_emitted: 350,
            },
            Hypothesis {
                words: vec![
                    HypWord { text: "hello".into(), t_start: 0, t_end: 300, stable: true },
                    HypWord { text: "world".into(), t_start: 320, t_end: 600, stable: false },
                ],
                is_final: true,
                t_emitted: 700,
            },
        ];
        let jsonl = replay::to_jsonl(&events);
        let mut r = replay::ReplayAsr::from_jsonl(&jsonl).unwrap();
        assert_eq!(r.next_event().unwrap(), events[0]);
        assert_eq!(r.next_event().unwrap(), events[1]);
        assert!(r.next_event().is_none());
    }
}
