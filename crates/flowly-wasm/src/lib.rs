//! WASM bindings: the real alignment engine in the browser. JSON strings
//! cross the boundary (payloads are tiny and it keeps the ABI trivial); the
//! TypeScript side owns nothing but transport.

use flowly_align::{Aligner, AlignerConfig};
use flowly_asr::Hypothesis;
use flowly_script::CompiledScript;
use wasm_bindgen::prelude::*;

/// Compile a script and return renderer-facing token info as JSON:
/// `[{display, line, sentenceStart}]`.
#[wasm_bindgen]
pub fn compile_tokens(source: &str) -> String {
    let script = CompiledScript::compile(source);
    let toks: Vec<serde_json::Value> = script
        .tokens
        .iter()
        .map(|t| {
            serde_json::json!({
                "display": t.display,
                "line": t.line,
                "sentenceStart": t.sentence_start,
                "paragraphStart": t.paragraph_start,
            })
        })
        .collect();
    serde_json::to_string(&toks).expect("tokens serialize")
}

#[wasm_bindgen]
pub struct WasmAligner {
    inner: Aligner,
}

#[wasm_bindgen]
impl WasmAligner {
    #[wasm_bindgen(constructor)]
    pub fn new(source: &str) -> WasmAligner {
        let script = CompiledScript::compile(source);
        WasmAligner { inner: Aligner::new(&script, AlignerConfig::default()) }
    }

    /// Feed one hypothesis (flowly-asr JSON shape). Returns the event array.
    pub fn feed(&mut self, hypothesis_json: &str, now_ms: f64) -> String {
        let Ok(hyp) = serde_json::from_str::<Hypothesis>(hypothesis_json) else {
            return "[]".to_string();
        };
        let events = self.inner.feed(&hyp, now_ms as u64);
        serde_json::to_string(&events).expect("events serialize")
    }

    /// Clock tick (drives silence -> PAUSED). Returns the event array.
    pub fn tick(&mut self, now_ms: f64) -> String {
        serde_json::to_string(&self.inner.tick(now_ms as u64)).expect("events serialize")
    }

    /// Manual override; always wins. Returns the event array.
    pub fn jump_to(&mut self, token_index: usize, now_ms: f64) -> String {
        serde_json::to_string(&self.inner.jump_to(token_index, now_ms as u64))
            .expect("events serialize")
    }

    pub fn wpm(&self) -> f32 {
        self.inner.wpm()
    }
}
