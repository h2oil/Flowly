//! Token normalization and verbalization.
//!
//! Every written token compiles to 1–8 spoken variants (a tiny lattice); the
//! aligner accepts any variant path. Mis-normalized numbers look like permanent
//! mismatches at exactly the points executives care about, so figures get the
//! most attention here: `$3.5M` → "three point five million dollars", `2026` →
//! "twenty twenty six" / "two thousand twenty six", `47%` → "forty seven
//! percent", `3rd` → "third", `Q3` → "q three".

/// Compute the spoken variants for one display word. Empty when the word has
/// no speakable content. `variants[0]` is the primary form.
pub fn variants(display: &str) -> Vec<Vec<String>> {
    let core = trim_outer_punct(display);
    if core.is_empty() {
        return Vec::new();
    }
    let lower = normalize_apostrophes(&core.to_lowercase());

    let mut out: Vec<Vec<String>> = Vec::new();
    let push = |v: Vec<String>, out: &mut Vec<Vec<String>>| {
        if !v.is_empty() && !v.iter().any(|w| w.is_empty()) && !out.contains(&v) {
            out.push(v);
        }
    };

    // Numeric / symbolic forms first: their verbalizations become the primary
    // variant because the written form is never what the ASR hears.
    for v in numeric_variants(&lower) {
        push(v, &mut out);
    }

    let bare: String = lower.chars().filter(|c| c.is_alphanumeric() || *c == '\'' || *c == '-').collect();
    let bare = bare.trim_matches(['\'', '-']).to_string();
    if !bare.is_empty() && out.is_empty() {
        // Plain word: primary is the cleaned lowercase form (without hyphens/apostrophe-s noise).
        push(vec![bare.replace(['-', '\''], "")], &mut out);
    }
    if !bare.is_empty() {
        // Hyphenated words also match as separate words: "voice-over" → "voice over".
        if bare.contains('-') {
            push(bare.split('-').map(str::to_string).collect(), &mut out);
        }
        // Contractions expand: "don't" → "do not".
        if let Some(expansion) = expand_contraction(&bare) {
            push(expansion.split(' ').map(str::to_string).collect(), &mut out);
        }
        // Keep the apostrophe-preserving form too ("it's" vs "its").
        if bare.contains('\'') {
            push(vec![bare.replace(['-', '\''], "")], &mut out);
        }
    }

    // Unknown all-caps initialisms are often read letter-by-letter: SQL → "s q l".
    let alpha_only: String = core.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    if alpha_only.len() >= 2
        && alpha_only.len() <= 5
        && alpha_only.len() == core.len()
        && core.chars().all(|c| c.is_ascii_uppercase())
        && !PRONOUNCEABLE_ACRONYMS.contains(&alpha_only.to_lowercase().as_str())
    {
        push(alpha_only.to_lowercase().chars().map(|c| c.to_string()).collect(), &mut out);
    }

    out
}

/// Strip leading/trailing punctuation that isn't part of the spoken form,
/// keeping `$`/digits/letters and internal `.`/`,`/`%`/`'`/`-`.
fn trim_outer_punct(w: &str) -> String {
    w.trim_matches(|c: char| {
        !(c.is_alphanumeric() || c == '$' || c == '%')
    })
    .to_string()
}

fn normalize_apostrophes(s: &str) -> String {
    s.replace(['\u{2018}', '\u{2019}'], "'")
}

/// Verbalizations for numeric/symbolic tokens. Empty if the token isn't numeric.
fn numeric_variants(lower: &str) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let s = lower.trim_end_matches([',', '.', ';', ':']);

    // $3.5M / $5,000 / $8
    if let Some(rest) = s.strip_prefix('$') {
        let (num, scale) = split_scale_suffix(rest);
        if let Some(words) = number_words(&num) {
            for mut base in words {
                if let Some(sc) = scale {
                    base.push(sc.to_string());
                }
                let singular = num == "1" && scale.is_none();
                let mut with_unit = base.clone();
                with_unit.push(if singular { "dollar" } else { "dollars" }.to_string());
                out.push(with_unit);
                out.push(base);
            }
        }
        return out;
    }
    // 47% / 3.5%
    if let Some(rest) = s.strip_suffix('%') {
        if let Some(words) = number_words(rest) {
            for mut base in words {
                base.push("percent".to_string());
                out.push(base);
            }
        }
        return out;
    }
    // Ordinals: 3rd, 21st
    if let Some(n) = parse_ordinal(s) {
        out.push(vec![ordinal_words(n)]);
        return out;
    }
    // Scale suffix without currency: 10k, 3.5B
    let (num, scale) = split_scale_suffix(s);
    if let Some(sc) = scale {
        if let Some(words) = number_words(&num) {
            for mut base in words {
                base.push(sc.to_string());
                out.push(base);
            }
            return out;
        }
    }
    // Plain number (integer, comma-grouped, or decimal) — includes years.
    if let Some(words) = number_words(s) {
        out.extend(words);
        return out;
    }
    // Letter+digits mix: Q3 → "q three", E3 → "e three".
    let alpha: String = s.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    let digits: String = s[alpha.len()..].to_string();
    if !alpha.is_empty() && alpha.len() <= 2 && !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(n) = digits.parse::<u64>() {
            let mut v: Vec<String> = alpha.chars().map(|c| c.to_string()).collect();
            v.extend(cardinal_words(n));
            out.push(v);
        }
    }
    out
}

/// Split a trailing k/m/b/t scale suffix: "3.5m" → ("3.5", Some("million")).
fn split_scale_suffix(s: &str) -> (String, Option<&'static str>) {
    let scale = match s.chars().last() {
        Some('k') => Some("thousand"),
        Some('m') => Some("million"),
        Some('b') => Some("billion"),
        Some('t') => Some("trillion"),
        _ => None,
    };
    match scale {
        Some(_) => {
            let head = &s[..s.len() - 1];
            if !head.is_empty() && head.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
                (head.replace(',', ""), scale)
            } else {
                (s.replace(',', ""), None)
            }
        }
        None => (s.replace(',', ""), None),
    }
}

/// All spoken forms of a bare number string, or None if it isn't a number.
/// Four-digit years get both readings ("twenty twenty six", "two thousand
/// twenty six").
fn number_words(s: &str) -> Option<Vec<Vec<String>>> {
    // "1,234" is comma-grouped — never a year.
    let had_commas = s.contains(',');
    let s = s.replace(',', "");
    if s.is_empty() {
        return None;
    }
    if let Some((int_part, frac_part)) = s.split_once('.') {
        if int_part.chars().all(|c| c.is_ascii_digit())
            && !frac_part.is_empty()
            && frac_part.chars().all(|c| c.is_ascii_digit())
        {
            let n: u64 = int_part.parse().ok()?;
            let mut words = cardinal_words(n);
            words.push("point".to_string());
            for d in frac_part.chars() {
                words.push(digit_word(d).to_string());
            }
            return Some(vec![words]);
        }
        return None;
    }
    if !s.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let n: u64 = s.parse().ok()?;
    let mut forms = Vec::new();
    if s.len() == 4 && !had_commas && (1000..=2199).contains(&n) {
        forms.extend(year_words(n));
    }
    let cardinal = cardinal_words(n);
    if !forms.contains(&cardinal) {
        forms.push(cardinal);
    }
    Some(forms)
}

fn parse_ordinal(s: &str) -> Option<u64> {
    let digits = s.strip_suffix("st").or_else(|| s.strip_suffix("nd"))
        .or_else(|| s.strip_suffix("rd")).or_else(|| s.strip_suffix("th"))?;
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

fn digit_word(d: char) -> &'static str {
    ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"]
        [d as usize - '0' as usize]
}

const ONES: [&str; 20] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen",
    "nineteen",
];
const TENS: [&str; 10] =
    ["", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"];

/// English cardinal, one word per Vec entry ("one hundred twenty three").
pub fn cardinal_words(n: u64) -> Vec<String> {
    fn under_1000(n: u64, out: &mut Vec<String>) {
        let n = n as usize;
        if n >= 100 {
            out.push(ONES[n / 100].to_string());
            out.push("hundred".to_string());
        }
        let rem = n % 100;
        if rem == 0 {
            return;
        }
        if rem < 20 {
            out.push(ONES[rem].to_string());
        } else {
            out.push(TENS[rem / 10].to_string());
            if rem % 10 != 0 {
                out.push(ONES[rem % 10].to_string());
            }
        }
    }
    if n == 0 {
        return vec!["zero".to_string()];
    }
    let mut out = Vec::new();
    let groups: [(u64, &str); 4] = [
        (1_000_000_000_000, "trillion"),
        (1_000_000_000, "billion"),
        (1_000_000, "million"),
        (1_000, "thousand"),
    ];
    let mut rest = n;
    for (size, name) in groups {
        if rest >= size {
            under_1000(rest / size, &mut out);
            out.push(name.to_string());
            rest %= size;
        }
    }
    if rest > 0 {
        under_1000(rest, &mut out);
    }
    out
}

/// Year readings: 1984 → "nineteen eighty four"; 2007 → "two thousand seven",
/// "twenty oh seven"; 2026 → "twenty twenty six", "two thousand twenty six".
fn year_words(n: u64) -> Vec<Vec<String>> {
    let hi = n / 100;
    let lo = n % 100;
    let mut forms = Vec::new();
    if lo == 0 {
        let mut v = cardinal_words(hi);
        v.push("hundred".to_string());
        forms.push(v);
    } else if (2000..2010).contains(&n) {
        let mut a = vec!["two".to_string(), "thousand".to_string()];
        a.extend(cardinal_words(lo));
        forms.push(a);
        let mut b = vec!["twenty".to_string(), "oh".to_string()];
        b.extend(cardinal_words(lo));
        forms.push(b);
    } else {
        let mut a = cardinal_words(hi);
        if lo < 10 {
            a.push("oh".to_string());
        }
        a.extend(cardinal_words(lo));
        forms.push(a);
        if (2000..2200).contains(&n) {
            let mut b = vec!["two".to_string(), "thousand".to_string()];
            b.extend(cardinal_words(lo));
            forms.push(b);
        }
    }
    forms
}

/// English ordinal as a single joined word sequence ("twenty first").
pub fn ordinal_words(n: u64) -> String {
    let mut words = cardinal_words(n);
    let last = words.pop().unwrap_or_default();
    let ord = match last.as_str() {
        "one" => "first".to_string(),
        "two" => "second".to_string(),
        "three" => "third".to_string(),
        "five" => "fifth".to_string(),
        "eight" => "eighth".to_string(),
        "nine" => "ninth".to_string(),
        "twelve" => "twelfth".to_string(),
        w if w.ends_with('y') => format!("{}ieth", &w[..w.len() - 1]),
        w => format!("{w}th"),
    };
    words.push(ord);
    words.join(" ")
}

fn expand_contraction(w: &str) -> Option<&'static str> {
    Some(match w {
        "don't" => "do not",
        "doesn't" => "does not",
        "didn't" => "did not",
        "can't" => "cannot",
        "won't" => "will not",
        "wouldn't" => "would not",
        "couldn't" => "could not",
        "shouldn't" => "should not",
        "isn't" => "is not",
        "aren't" => "are not",
        "wasn't" => "was not",
        "weren't" => "were not",
        "hasn't" => "has not",
        "haven't" => "have not",
        "hadn't" => "had not",
        "it's" => "it is",
        "that's" => "that is",
        "there's" => "there is",
        "here's" => "here is",
        "what's" => "what is",
        "who's" => "who is",
        "let's" => "let us",
        "i'm" => "i am",
        "you're" => "you are",
        "we're" => "we are",
        "they're" => "they are",
        "i've" => "i have",
        "you've" => "you have",
        "we've" => "we have",
        "they've" => "they have",
        "i'll" => "i will",
        "you'll" => "you will",
        "he'll" => "he will",
        "she'll" => "she will",
        "we'll" => "we will",
        "they'll" => "they will",
        "i'd" => "i would",
        "you'd" => "you would",
        "he'd" => "he would",
        "she'd" => "she would",
        "we'd" => "we would",
        "they'd" => "they would",
        _ => return None,
    })
}

/// Acronyms commonly pronounced as words, never letter-by-letter.
const PRONOUNCEABLE_ACRONYMS: [&str; 12] =
    ["nasa", "saas", "laser", "radar", "scuba", "nato", "fomo", "yolo", "gif", "ram", "asap", "arr"];

/// Function words excluded from anchor trigrams (they appear everywhere).
pub fn is_stopword(w: &str) -> bool {
    matches!(
        w,
        "a" | "an" | "the" | "and" | "or" | "but" | "of" | "to" | "in" | "on" | "at" | "by"
            | "for" | "with" | "as" | "is" | "are" | "was" | "were" | "be" | "been" | "am"
            | "it" | "its" | "this" | "that" | "these" | "those" | "we" | "you" | "i" | "he"
            | "she" | "they" | "my" | "our" | "your" | "so" | "if" | "then" | "than" | "not"
            | "do" | "does" | "did" | "have" | "has" | "had" | "will" | "would" | "can"
            | "could" | "there" | "here" | "what" | "who" | "how" | "when" | "just" | "very"
    )
}

/// Hesitation fillers: free insertions for the aligner, never mismatches.
pub fn is_filler(w: &str) -> bool {
    matches!(w, "uh" | "um" | "er" | "ah" | "erm" | "hmm" | "mhm" | "uhh" | "umm" | "like" | "okay" | "right" | "well" | "actually" | "basically" | "y'know")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joined(display: &str) -> Vec<String> {
        variants(display).into_iter().map(|v| v.join(" ")).collect()
    }

    #[test]
    fn plain_words() {
        assert_eq!(joined("Hello,"), vec!["hello"]);
        assert_eq!(joined("world."), vec!["world"]);
        assert_eq!(joined("“Quote”"), vec!["quote"]);
    }

    #[test]
    fn currency() {
        let v = joined("$3.5M");
        assert!(v.contains(&"three point five million dollars".to_string()), "{v:?}");
        assert!(v.contains(&"three point five million".to_string()));
        assert_eq!(joined("$5,000")[0], "five thousand dollars");
    }

    #[test]
    fn percent_and_decimal() {
        assert_eq!(joined("47%")[0], "forty seven percent");
        assert_eq!(joined("3.5")[0], "three point five");
    }

    #[test]
    fn years() {
        let v = joined("2026");
        assert!(v.contains(&"twenty twenty six".to_string()), "{v:?}");
        assert!(v.contains(&"two thousand twenty six".to_string()));
        assert!(joined("1984").contains(&"nineteen eighty four".to_string()));
        assert!(joined("2007").contains(&"twenty oh seven".to_string()));
    }

    #[test]
    fn plain_number_is_not_only_a_year() {
        assert!(joined("1984").contains(&"one thousand nine hundred eighty four".to_string()));
        assert_eq!(joined("47")[0], "forty seven");
        assert_eq!(joined("1,234")[0], "one thousand two hundred thirty four");
    }

    #[test]
    fn ordinals() {
        assert_eq!(joined("3rd")[0], "third");
        assert_eq!(joined("21st")[0], "twenty first");
        assert_eq!(joined("20th")[0], "twentieth");
    }

    #[test]
    fn contractions() {
        let v = joined("don't");
        assert!(v.contains(&"do not".to_string()), "{v:?}");
        assert!(v.contains(&"dont".to_string()));
    }

    #[test]
    fn acronyms() {
        let v = joined("SQL");
        assert!(v.contains(&"s q l".to_string()), "{v:?}");
        assert_eq!(joined("NASA"), vec!["nasa"]); // pronounceable: no letter form
    }

    #[test]
    fn quarter_labels() {
        assert!(joined("Q3").contains(&"q three".to_string()));
    }

    #[test]
    fn hyphenated() {
        let v = joined("voice-over");
        assert!(v.contains(&"voice over".to_string()), "{v:?}");
        assert!(v.contains(&"voiceover".to_string()));
    }

    #[test]
    fn scale_suffix() {
        assert_eq!(joined("10k")[0], "ten thousand");
    }
}
