use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use vwi_core::auditor::Auditor;

#[cfg(feature = "cascade")]
use std::cell::RefCell;
#[cfg(feature = "cascade")]
use std::sync::OnceLock;

#[cfg(feature = "cascade")]
use vsm_runtime::VsmCascadeScorer;
#[cfg(feature = "cascade")]
use vwi_core::{
    run_tier4_with_bigram, BigramTable, CascadeConfig, NeuralContextScorer, Tier4Config,
};

#[cfg(feature = "cascade")]
static VSM_BYTES: &[u8] = include_bytes!("../../../neural/fixtures/vsm_96_4_curriculum_int8.vsm");
#[cfg(feature = "cascade")]
static VBG_BYTES: &[u8] = include_bytes!("../../../neural/fixtures/vi_bigram.vbg");

#[cfg(feature = "cascade")]
const MAX_CHUNK_BYTES: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub start: usize,
    pub end: usize,
    pub raw: String,
    pub suggestion: String,
    pub source: String,
    pub severity: String,
}

#[cfg(feature = "cascade")]
#[derive(Debug, Clone, Copy)]
struct TextChunk {
    start: usize,
    end: usize,
}

#[cfg(feature = "cascade")]
#[inline]
fn safe_utf8_floor(text: &str, minimum: usize, limit: usize) -> usize {
    let clamped = limit.min(text.len());
    if clamped <= minimum {
        return minimum.min(text.len());
    }
    let mut index = clamped;
    while index > minimum && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

#[cfg(feature = "cascade")]
fn split_text_chunks(text: &str, max_bytes: usize) -> Vec<TextChunk> {
    let mut chunks = Vec::new();
    if text.is_empty() {
        return chunks;
    }
    let capacity = max_bytes.max(16);
    let mut chunk_start = 0usize;
    let mut last_punctuation: Option<usize> = None;
    for (index, character) in text.char_indices() {
        let end = index + character.len_utf8();
        if end - chunk_start > capacity {
            let cut = match last_punctuation {
                Some(position) if position > chunk_start => position,
                _ => safe_utf8_floor(text, chunk_start + 1, chunk_start + capacity),
            };
            if cut > chunk_start {
                chunks.push(TextChunk {
                    start: chunk_start,
                    end: cut,
                });
                chunk_start = cut;
                last_punctuation = None;
            }
        }
        if matches!(character, '.' | '!' | '?' | '\n' | '\r') && end > chunk_start {
            last_punctuation = Some(end);
            if end - chunk_start >= 64 {
                chunks.push(TextChunk {
                    start: chunk_start,
                    end,
                });
                chunk_start = end;
                last_punctuation = None;
            }
        }
    }
    while chunk_start < text.len() {
        let next_end = if text.len() - chunk_start <= capacity {
            text.len()
        } else {
            safe_utf8_floor(text, chunk_start + 1, chunk_start + capacity)
        };
        if next_end <= chunk_start {
            break;
        }
        chunks.push(TextChunk {
            start: chunk_start,
            end: next_end,
        });
        chunk_start = next_end;
    }
    chunks
}

#[cfg(feature = "cascade")]
fn spans_overlap(left_start: usize, left_end: usize, right_start: usize, right_end: usize) -> bool {
    left_start < right_end && right_start < left_end
}

#[cfg(feature = "cascade")]
fn merge_without_overlap(symbolic: Vec<Diagnostic>, context: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let mut merged = symbolic;
    for hint in context {
        if hint.start >= hint.end || hint.raw == hint.suggestion {
            continue;
        }
        let overlaps = merged
            .iter()
            .any(|existing| spans_overlap(existing.start, existing.end, hint.start, hint.end));
        if !overlaps {
            merged.push(hint);
        }
    }
    merged.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| left.end.cmp(&right.end))
    });
    merged
}

fn char_to_byte(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len())
}

#[cfg(feature = "cascade")]
static BIGRAM: OnceLock<Option<BigramTable<'static>>> = OnceLock::new();

#[wasm_bindgen]
pub struct WasmAuditor {
    inner: Auditor,
    #[cfg(feature = "cascade")]
    cascade: RefCell<Option<VsmCascadeScorer>>,
}

#[wasm_bindgen]
impl WasmAuditor {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: Auditor::default(),
            #[cfg(feature = "cascade")]
            cascade: RefCell::new(None),
        }
    }

    /// Diagnostics cho extension: symbolic luôn chạy; context hints chỉ khi opt-in.
    #[wasm_bindgen]
    pub fn check_text(&self, text: &str, enable_context_hints: bool) -> String {
        let symbolic = self.symbolic_diagnostics(text);
        let final_diagnostics = if enable_context_hints {
            #[cfg(feature = "cascade")]
            {
                merge_without_overlap(symbolic, self.context_hints(text))
            }
            #[cfg(not(feature = "cascade"))]
            {
                symbolic
            }
        } else {
            symbolic
        };
        serde_json::to_string(&final_diagnostics).unwrap_or_else(|_| "[]".to_string())
    }

    fn symbolic_diagnostics(&self, text: &str) -> Vec<Diagnostic> {
        self.inner
            .audit(text)
            .into_iter()
            .filter_map(|finding| {
                let start = char_to_byte(text, finding.start);
                let end = char_to_byte(text, finding.end);
                if start >= end || end > text.len() {
                    return None;
                }
                let suggestion = finding.suggestion?;
                if suggestion.is_empty() {
                    return None;
                }
                Some(Diagnostic {
                    start,
                    end,
                    raw: finding.matched_span,
                    suggestion,
                    source: "symbolic".to_string(),
                    severity: if finding.severity == "advisory" || finding.severity == "minor" {
                        "advisory".to_string()
                    } else {
                        "error".to_string()
                    },
                })
            })
            .collect()
    }

    #[cfg(feature = "cascade")]
    fn context_hints(&self, text: &str) -> Vec<Diagnostic> {
        let bigram = BIGRAM
            .get_or_init(|| BigramTable::from_bytes(VBG_BYTES))
            .as_ref();
        let mut cascade = self.cascade.borrow_mut();
        if cascade.is_none() {
            *cascade = VsmCascadeScorer::new(VSM_BYTES).ok();
        }
        let Some(scorer) = cascade.as_ref() else {
            return Vec::new();
        };
        let config = CascadeConfig::default();
        let tier4 = Tier4Config::default();
        let mut output = Vec::new();
        for chunk in split_text_chunks(text, MAX_CHUNK_BYTES) {
            let slice = &text[chunk.start..chunk.end];
            if slice.trim().is_empty() {
                continue;
            }
            let locked = vec![false; slice.len()];
            let mut hints = Vec::new();
            scorer.score_unmasked_regions(slice, &locked, &mut hints);
            let mut external = Vec::new();
            for hint in &hints {
                let mut candidates = Vec::new();
                scorer.decode_top_candidates(hint.start, hint.end, 1, &mut candidates);
                for candidate in candidates {
                    external.push((hint.start, hint.end, candidate));
                }
            }
            let found =
                run_tier4_with_bigram(slice, &hints, &locked, config, &external, bigram, &tier4);
            for diagnostic in found {
                let start = chunk.start + diagnostic.start;
                let end = chunk.start + diagnostic.end;
                if start >= end
                    || end > text.len()
                    || !text.is_char_boundary(start)
                    || !text.is_char_boundary(end)
                {
                    continue;
                }
                output.push(Diagnostic {
                    start,
                    end,
                    raw: diagnostic.original,
                    suggestion: diagnostic.suggestion,
                    source: "context_hint".to_string(),
                    severity: "advisory".to_string(),
                });
            }
        }
        output
    }

    /// Kiểm toán/soát lỗi văn bản và trả về JSON Report (Điểm số, danh sách lỗi, tọa độ span, gợi ý sửa)
    /// `register`: Tùy chọn ngữ vực ("general", "literary", "administrative", "technical", "journalism")
    #[wasm_bindgen]
    pub fn audit(&self, text: &str, register: Option<String>) -> String {
        let mut auditor = self.inner.clone();
        if let Some(reg_str) = register {
            auditor
                .config_mut()
                .set_register(vwi_core::types::Register::parse(&reg_str));
        }
        let report = auditor.audit_document(text);
        report.to_json()
    }

    /// Tách từ tiếng Việt kèm nhãn từ loại UPOS
    #[wasm_bindgen]
    pub fn tokenize(&self, text: &str) -> String {
        let tokens = vwi_core::segmentation::viterbi_segmenter::tokenize_viterbi(text);
        serde_json::to_string(&tokens).unwrap_or_else(|_| "[]".to_string())
    }
}
