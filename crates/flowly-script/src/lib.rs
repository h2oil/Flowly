//! Flowly Prompter Markdown compiler.
//!
//! Compiles a script into the **TokenMap**: the single coordinate system shared
//! by the renderer, the voice aligner, and take markers. The contract (see
//! docs/plan/05-data-and-sync.md): compilation is deterministic; spoken tokens
//! carry byte-offset source spans plus a lattice of normalized spoken variants;
//! directives and `(( stage notes ))` are excluded from alignment.
//!
//! Grammar (line-oriented):
//! - `# Heading`            → section marker (nav/jump target, not spoken)
//! - `[PAUSE 2s]` `[SLOW]`… → directives (not spoken)
//! - `//…` to end of line   → directive comment, e.g. `//pause` (not spoken)
//! - `(( note ))`           → unspoken stage note, rendered dim
//! - `**bold**`             → emphasis; markers stripped from spoken tokens
//! - blank line             → paragraph break

pub mod normalize;
pub mod phonetic;

use serde::Serialize;

/// One spoken token: a display word with its source span and the spoken-word
/// variants the aligner may match against (each variant is a word sequence —
/// e.g. `$3.5M` → ["three","point","five","million","dollars"]).
#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub index: usize,
    /// Byte offsets into the source text.
    pub char_start: usize,
    pub char_end: usize,
    /// The display form (punctuation kept, emphasis markers stripped).
    pub display: String,
    /// Normalized spoken variants; `variants[0]` is the primary form.
    pub variants: Vec<Vec<String>>,
    pub sentence_start: bool,
    pub paragraph_start: bool,
    /// False for stopwords — anchor trigrams use content words only.
    pub is_content: bool,
    /// Display line this token belongs to (source line index).
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SpanKind {
    /// Spoken text covering tokens `[first_token, last_token]`.
    Speech { first_token: usize, last_token: usize },
    Heading { level: u8 },
    StageNote,
    Directive,
}

/// A renderable region of the source. Spans are non-overlapping and ordered.
#[derive(Debug, Clone, Serialize)]
pub struct DisplaySpan {
    pub char_start: usize,
    pub char_end: usize,
    pub kind: SpanKind,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Section {
    pub title: String,
    /// Index of the first spoken token after the heading (== tokens.len() if none).
    pub first_token: usize,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledScript {
    pub tokens: Vec<Token>,
    pub spans: Vec<DisplaySpan>,
    pub sections: Vec<Section>,
}

impl CompiledScript {
    /// Deterministically compile a script. Never fails: unrecognized syntax is
    /// treated as spoken text — a prompter must render whatever it is given.
    pub fn compile(source: &str) -> CompiledScript {
        Compiler::new(source).run()
    }
}

struct Compiler<'a> {
    src: &'a str,
    tokens: Vec<Token>,
    spans: Vec<DisplaySpan>,
    sections: Vec<Section>,
    /// Next spoken token starts a sentence.
    sentence_pending: bool,
    /// Next spoken token starts a paragraph.
    paragraph_pending: bool,
}

impl<'a> Compiler<'a> {
    fn new(src: &'a str) -> Self {
        Compiler {
            src,
            tokens: Vec::new(),
            spans: Vec::new(),
            sections: Vec::new(),
            sentence_pending: true,
            paragraph_pending: true,
        }
    }

    fn run(mut self) -> CompiledScript {
        let mut offset = 0usize;
        for (line_idx, line) in self.src.split('\n').enumerate() {
            self.compile_line(line, offset, line_idx);
            offset += line.len() + 1; // '\n'
        }
        CompiledScript { tokens: self.tokens, spans: self.spans, sections: self.sections }
    }

    fn compile_line(&mut self, line: &str, line_start: usize, line_idx: usize) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            self.paragraph_pending = true;
            self.sentence_pending = true;
            return;
        }
        if let Some(rest) = trimmed.strip_prefix('#') {
            let level = 1 + rest.len().saturating_sub(rest.trim_start_matches('#').len()) as u8;
            let title = rest.trim_start_matches('#').trim().to_string();
            self.spans.push(DisplaySpan {
                char_start: line_start,
                char_end: line_start + line.len(),
                kind: SpanKind::Heading { level },
                line: line_idx,
            });
            self.sections.push(Section { title, first_token: self.tokens.len(), line: line_idx });
            self.paragraph_pending = true;
            self.sentence_pending = true;
            return;
        }

        // Split the line into speech / stage-note / directive segments.
        let bytes = line.as_bytes();
        let mut i = 0usize;
        let mut speech_start = 0usize;
        while i < bytes.len() {
            if bytes[i..].starts_with(b"((") {
                self.flush_speech(line, speech_start, i, line_start, line_idx);
                let end = find_from(line, i + 2, "))").map(|e| e + 2).unwrap_or(line.len());
                self.spans.push(DisplaySpan {
                    char_start: line_start + i,
                    char_end: line_start + end,
                    kind: SpanKind::StageNote,
                    line: line_idx,
                });
                i = end;
                speech_start = i;
            } else if bytes[i] == b'[' {
                let close = find_from(line, i + 1, "]");
                if let Some(end) = close {
                    self.flush_speech(line, speech_start, i, line_start, line_idx);
                    self.spans.push(DisplaySpan {
                        char_start: line_start + i,
                        char_end: line_start + end + 1,
                        kind: SpanKind::Directive,
                        line: line_idx,
                    });
                    i = end + 1;
                    speech_start = i;
                } else {
                    i += 1;
                }
            } else if bytes[i..].starts_with(b"//") {
                self.flush_speech(line, speech_start, i, line_start, line_idx);
                self.spans.push(DisplaySpan {
                    char_start: line_start + i,
                    char_end: line_start + line.len(),
                    kind: SpanKind::Directive,
                    line: line_idx,
                });
                i = line.len();
                speech_start = i;
            } else {
                i += 1;
            }
        }
        self.flush_speech(line, speech_start, line.len(), line_start, line_idx);
    }

    fn flush_speech(&mut self, line: &str, from: usize, to: usize, line_start: usize, line_idx: usize) {
        if from >= to {
            return;
        }
        let segment = &line[from..to];
        if segment.trim().is_empty() {
            return;
        }
        let first_new = self.tokens.len();
        // Tokenize on whitespace, tracking byte offsets within the segment.
        let mut pos = 0usize;
        for word in segment.split_whitespace() {
            let rel = find_from(segment, pos, word).expect("word came from segment");
            pos = rel + word.len();
            let display = word.replace("**", "");
            if display.chars().all(|c| !c.is_alphanumeric()) {
                // Pure punctuation (e.g. a stray "—"): ends a sentence, not a token.
                if display.contains(['.', '!', '?']) {
                    self.sentence_pending = true;
                }
                continue;
            }
            let variants = normalize::variants(&display);
            if variants.is_empty() {
                continue;
            }
            let is_content = !normalize::is_stopword(&variants[0].join(" "));
            let index = self.tokens.len();
            self.tokens.push(Token {
                index,
                char_start: line_start + from + rel,
                char_end: line_start + from + rel + word.len(),
                display: display.clone(),
                variants,
                sentence_start: self.sentence_pending,
                paragraph_start: self.paragraph_pending,
                is_content,
                line: line_idx,
            });
            self.sentence_pending = display.ends_with(['.', '!', '?'])
                || display.ends_with(".\"")
                || display.ends_with(".)")
                || display.ends_with("?\"")
                || display.ends_with("!\"");
            self.paragraph_pending = false;
        }
        if self.tokens.len() > first_new {
            self.spans.push(DisplaySpan {
                char_start: line_start + from,
                char_end: line_start + to,
                kind: SpanKind::Speech { first_token: first_new, last_token: self.tokens.len() - 1 },
                line: line_idx,
            });
        }
    }
}

fn find_from(haystack: &str, from: usize, needle: &str) -> Option<usize> {
    haystack.get(from..).and_then(|s| s.find(needle)).map(|p| p + from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(src: &str) -> CompiledScript {
        CompiledScript::compile(src)
    }

    fn primaries(s: &CompiledScript) -> Vec<String> {
        s.tokens.iter().map(|t| t.variants[0].join(" ")).collect()
    }

    #[test]
    fn plain_sentence() {
        let s = compile("Welcome back everyone.");
        assert_eq!(primaries(&s), vec!["welcome", "back", "everyone"]);
        assert!(s.tokens[0].sentence_start);
        assert!(!s.tokens[1].sentence_start);
        assert!(s.tokens[0].paragraph_start);
    }

    #[test]
    fn headings_notes_directives_are_not_spoken() {
        let src = "# Intro\nHello there. //pause\n[SLOW] Big news (( smile here )) today.";
        let s = compile(src);
        assert_eq!(primaries(&s), vec!["hello", "there", "big", "news", "today"]);
        assert_eq!(s.sections.len(), 1);
        assert_eq!(s.sections[0].title, "Intro");
        assert_eq!(s.sections[0].first_token, 0);
        assert!(s.spans.iter().any(|sp| sp.kind == SpanKind::StageNote));
        assert!(s.spans.iter().any(|sp| sp.kind == SpanKind::Directive));
    }

    #[test]
    fn sentence_and_paragraph_boundaries() {
        let s = compile("One two. Three!\n\nFour");
        let starts: Vec<bool> = s.tokens.iter().map(|t| t.sentence_start).collect();
        assert_eq!(starts, vec![true, false, true, true]);
        assert!(s.tokens[3].paragraph_start);
    }

    #[test]
    fn source_spans_point_at_the_words() {
        let src = "Say **this** now.";
        let s = compile(src);
        let tok = &s.tokens[1];
        assert_eq!(&src[tok.char_start..tok.char_end], "**this**");
        assert_eq!(tok.display, "this");
    }

    #[test]
    fn emphasis_is_stripped_from_spoken_form() {
        let s = compile("The **voice following** feature.");
        assert_eq!(primaries(&s), vec!["the", "voice", "following", "feature"]);
    }

    #[test]
    fn compile_is_deterministic() {
        let src = "# A\nNumbers like $3.5M and 47% in Q3 2026. //pause\n(( breathe ))\nDone.";
        let a = serde_json::to_string(&compile(src)).unwrap();
        let b = serde_json::to_string(&compile(src)).unwrap();
        assert_eq!(a, b);
    }
}
