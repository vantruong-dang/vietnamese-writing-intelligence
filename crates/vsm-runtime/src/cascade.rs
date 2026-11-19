//! Adapter Tầng 3: implement `NeuralContextScorer` cho VSM runtime.
//!
//! Feature `cascade` kéo `vwi-core` để dùng `hints_from_byte_probs` và
//! `NeuralContextScorer`. Không bật feature thì `vsm-runtime` vẫn standalone.

use std::cell::RefCell;

use vwi_core::{hints_from_byte_probs, CascadeConfig, NeuralContextScorer, NeuralSpanHint};

use crate::{forward, VsmModel};

pub struct VsmCascadeScorer {
    model: VsmModel,
    config: CascadeConfig,
    last_byte_logits: RefCell<Vec<f32>>,
}

impl VsmCascadeScorer {
    pub fn new(bytes: &[u8]) -> Result<Self, String> {
        VsmModel::from_bytes(bytes)
            .map(|model| Self {
                model,
                config: CascadeConfig::default(),
                last_byte_logits: RefCell::new(Vec::new()),
            })
            .map_err(|error| error.to_string())
    }

    pub fn with_config(mut self, config: CascadeConfig) -> Self {
        self.config = config;
        self
    }

    pub fn model(&self) -> &VsmModel {
        &self.model
    }
}

fn softmax4(row: &[f32]) -> [f32; 4] {
    let mut values = [0.0f32; 4];
    values.copy_from_slice(&row[..4]);
    let maximum = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    for value in &mut values {
        *value = (*value - maximum).exp();
        sum += *value;
    }
    for value in &mut values {
        *value /= sum.max(1e-9);
    }
    values
}

impl NeuralContextScorer for VsmCascadeScorer {
    fn score_unmasked_regions(
        &self,
        text: &str,
        locked_mask: &[bool],
        out_hints: &mut Vec<NeuralSpanHint>,
    ) {
        let Ok(output) = forward(&self.model, text.as_bytes()) else {
            return;
        };
        let probabilities: Vec<[f32; 4]> = output
            .error_logits
            .as_chunks::<4>()
            .0
            .iter()
            .map(|row| softmax4(row))
            .collect();
        let hints = hints_from_byte_probs(text, &probabilities, locked_mask, self.config);
        out_hints.extend(hints);
        *self.last_byte_logits.borrow_mut() = output.byte_logits;
    }

    fn decode_top_candidates(
        &self,
        span_start: usize,
        span_end: usize,
        top_k: usize,
        out_buf: &mut Vec<String>,
    ) {
        let logits = self.last_byte_logits.borrow();
        if logits.is_empty() || top_k == 0 || span_start >= span_end {
            return;
        }
        let byte_count = logits.len() / 256;
        let start = span_start.min(byte_count);
        let end = span_end.min(byte_count);
        if start >= end {
            return;
        }
        let mut decoded = Vec::with_capacity(end - start);
        for byte_index in start..end {
            let row = &logits[byte_index * 256..(byte_index + 1) * 256];
            let mut best = 0usize;
            for index in 1..row.len() {
                if row[index] > row[best] {
                    best = index;
                }
            }
            decoded.push(best as u8);
        }
        if let Ok(candidate) = String::from_utf8(decoded) {
            out_buf.push(candidate);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cascade_scorer_does_not_panic() {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../neural/fixtures/tiny_model.vsm"
        ))
        .expect("fixture");
        let scorer = VsmCascadeScorer::new(&bytes).expect("model");
        let text = "chính tã";
        let locked = vec![false; text.len()];
        let mut hints = Vec::new();
        scorer.score_unmasked_regions(text, &locked, &mut hints);
        let mut candidates = Vec::new();
        scorer.decode_top_candidates(0, text.len(), 1, &mut candidates);
        assert!(hints.len() <= text.len());
        assert!(candidates.len() <= 1);
    }
}
