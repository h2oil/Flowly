//! Phonetic key for homophone-tolerant matching.
//!
//! Rev1 uses a Soundex-style key: cheap, deterministic, and it collapses the
//! homophones that actually hurt a prompter (`there/their`, `week/weak`,
//! `to/too/two`). The plan calls for Double Metaphone; this function is the
//! seam where that lands — the aligner only ever compares keys for equality.

/// Soundex-style phonetic key ("their" → "T600"). Empty input → empty key.
pub fn key(word: &str) -> String {
    let letters: Vec<char> = word
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    let Some(&first) = letters.first() else {
        return String::new();
    };

    fn code(c: char) -> u8 {
        match c {
            'B' | 'F' | 'P' | 'V' => 1,
            'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => 2,
            'D' | 'T' => 3,
            'L' => 4,
            'M' | 'N' => 5,
            'R' => 6,
            _ => 0, // vowels + H/W/Y separate, but don't encode
        }
    }

    let mut out = String::new();
    out.push(first);
    let mut last_code = code(first);
    for &c in &letters[1..] {
        let k = code(c);
        // H and W do not reset the previous code (standard Soundex rule).
        if c == 'H' || c == 'W' {
            continue;
        }
        if k != 0 && k != last_code {
            out.push(char::from(b'0' + k));
            if out.len() == 4 {
                break;
            }
        }
        last_code = k;
    }
    while out.len() < 4 {
        out.push('0');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::key;

    #[test]
    fn homophones_share_keys() {
        assert_eq!(key("there"), key("their"));
        assert_eq!(key("week"), key("weak"));
        assert_eq!(key("to"), key("too"));
        assert_eq!(key("two"), key("to"));
        // Known Soundex limitation: first-letter mismatches (write/right)
        // are NOT collapsed — the fuzzy edit-distance path covers those.
        assert_ne!(key("write"), key("right"));
    }

    #[test]
    fn distinct_words_differ() {
        assert_ne!(key("voice"), key("following"));
        assert_ne!(key("cat"), key("dog"));
    }

    #[test]
    fn stable_shape() {
        assert_eq!(key("their").len(), 4);
        assert_eq!(key(""), "");
        assert_eq!(key("a"), "A000");
    }
}
