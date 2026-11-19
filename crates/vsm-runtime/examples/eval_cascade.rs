//! Đo End-to-End Cascade Recall + Suggestion Top-1 trên test JSONL.
//!
//!   cargo run -p vsm-runtime --features cascade --example eval_cascade --release -- \
//!       neural/fixtures/vsm_96_4_int8.vsm data_processed/neural_diverse/test.jsonl 10000
//!
//! Tier 1 hard guard chưa bật; Tier 2 dùng Auditor của vwi-core. Vì vậy recall
//! là **upper bound** (Tier 1 chỉ có thể khóa thêm, không thể tăng recall).

use std::fs;

use serde_json::Value;
use vsm_runtime::VsmCascadeScorer;
use vwi_core::{run_tier4_with_candidates, Auditor, CascadeConfig, NeuralContextScorer};

fn char_to_byte(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len())
}

fn normalize(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn spans_overlap(left: (usize, usize), right: (usize, usize)) -> bool {
    left.0 < right.1 && right.0 < left.1
}

fn main() {
    let mut args = std::env::args().skip(1);
    let model_path = args
        .next()
        .unwrap_or_else(|| "neural/fixtures/vsm_96_4_int8.vsm".to_string());
    let data_path = args
        .next()
        .unwrap_or_else(|| "data_processed/neural_diverse/test.jsonl".to_string());
    let limit: usize = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(10_000);
    let no_lock = std::env::args().any(|value| value == "--no-lock");
    let debug = std::env::args().any(|value| value == "--debug");

    let model_bytes = fs::read(&model_path).expect("đọc .vsm");
    let scorer = VsmCascadeScorer::new(&model_bytes).expect("parse .vsm");
    let config = CascadeConfig::default();
    let auditor = Auditor::new();
    let data = fs::read_to_string(&data_path).expect("đọc test jsonl");

    let mut rows = 0usize;
    let mut gold_spans = 0usize;
    let mut detected = 0usize;
    let mut detected_by_neural = 0usize;
    let mut suggestion_correct = 0usize;
    let mut error_records = 0usize;
    let mut record_exact = 0usize;
    let mut diagnostics = 0usize;
    let mut clean_records = 0usize;
    let mut clean_false_positives = 0usize;
    let mut debug_misses = 0usize;

    for line in data.lines().take(limit) {
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(text) = record["text"].as_str() else {
            continue;
        };
        let Some(clean_text) = record["clean_text"].as_str() else {
            continue;
        };
        rows += 1;

        let findings = if no_lock {
            Vec::new()
        } else {
            auditor.audit(text)
        };
        let mut locked = vec![false; text.len()];
        let mut tier2_spans: Vec<(usize, usize, String)> = Vec::new();
        for finding in &findings {
            if finding.category != "orthography" && finding.category != "spelling" {
                continue;
            }
            let start = char_to_byte(text, finding.start);
            let end = char_to_byte(text, finding.end);
            if start >= end || end > text.len() {
                continue;
            }
            for value in &mut locked[start..end] {
                *value = true;
            }
            tier2_spans.push((start, end, finding.suggestion.clone().unwrap_or_default()));
        }

        let mut hints = Vec::new();
        scorer.score_unmasked_regions(text, &locked, &mut hints);
        let mut external = Vec::new();
        for hint in &hints {
            let mut decoded = Vec::new();
            scorer.decode_top_candidates(hint.start, hint.end, 1, &mut decoded);
            for candidate in decoded {
                external.push((hint.start, hint.end, candidate));
            }
        }
        let found = run_tier4_with_candidates(text, &hints, &locked, config, &external);
        diagnostics += found.len();
        for diagnostic in &found {
            tier2_spans.push((
                diagnostic.start,
                diagnostic.end,
                diagnostic.suggestion.clone(),
            ));
        }

        let gold: Vec<(usize, usize)> = record["spans"]
            .as_array()
            .map(|spans| {
                spans
                    .iter()
                    .filter_map(|span| {
                        Some((
                            span["start"].as_u64()? as usize,
                            span["end"].as_u64()? as usize,
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();

        if gold.is_empty() {
            clean_records += 1;
            if !found.is_empty() {
                clean_false_positives += 1;
            }
            continue;
        }
        gold_spans += gold.len();
        error_records += 1;
        let mut corrected = text.as_bytes().to_vec();
        for diagnostic in found.iter().rev() {
            if diagnostic.start < corrected.len() && diagnostic.end <= corrected.len() {
                corrected.splice(
                    diagnostic.start..diagnostic.end,
                    diagnostic.suggestion.as_bytes().iter().copied(),
                );
            }
        }
        let corrected = String::from_utf8_lossy(&corrected);
        if normalize(&corrected) == normalize(clean_text) {
            record_exact += 1;
        }
        for (start, end) in &gold {
            let matching: Vec<&(usize, usize, String)> = tier2_spans
                .iter()
                .filter(|(left, right, _)| spans_overlap((*left, *right), (*start, *end)))
                .collect();
            if !matching.is_empty() {
                detected += 1;
            }
            let neural_match = found.iter().any(|diagnostic| {
                spans_overlap((diagnostic.start, diagnostic.end), (*start, *end))
            });
            if neural_match {
                detected_by_neural += 1;
            } else if debug && debug_misses < 20 {
                debug_misses += 1;
                eprintln!(
                    "MISS gold={start}..{end} {:?} hints={:?} found={:?} text={:?}",
                    &text[*start..*end],
                    hints,
                    found,
                    text
                );
            }
            let prefix = &text[..*start];
            let suffix = &text[*end..];
            if clean_text.starts_with(prefix) && clean_text.ends_with(suffix) {
                let clean_segment = &clean_text[prefix.len()..clean_text.len() - suffix.len()];
                let expected = normalize(clean_segment);
                if matching
                    .iter()
                    .any(|(_, _, suggestion)| normalize(suggestion) == expected)
                {
                    suggestion_correct += 1;
                }
            }
        }
    }

    let recall = detected as f64 / gold_spans.max(1) as f64;
    let neural_recall = detected_by_neural as f64 / gold_spans.max(1) as f64;
    let top1 = suggestion_correct as f64 / gold_spans.max(1) as f64;
    let record_top1 = record_exact as f64 / error_records.max(1) as f64;
    let clean_fpr = clean_false_positives as f64 / clean_records.max(1) as f64;
    println!(
        "{{\"rows\":{rows},\"gold_spans\":{gold_spans},\"detected\":{detected},\"cascade_recall\":{recall:.6},\"neural_detected\":{detected_by_neural},\"neural_recall\":{neural_recall:.6},\"suggestion_top1\":{top1:.6},\"record_top1\":{record_top1:.6},\"diagnostics\":{diagnostics},\"clean_records\":{clean_records},\"clean_false_positive_records\":{clean_false_positives},\"clean_fpr\":{clean_fpr:.6}}}"
    );
}
