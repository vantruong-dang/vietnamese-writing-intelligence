//! Đánh giá F1 span của file .vsm INT8 trên JSONL test.
//!
//!   cargo run -p vsm-runtime --example eval_int8 --release -- \
//!       neural/fixtures/vsm_96_4_int8.vsm data_processed/neural_large/test.jsonl 10000

use std::fs;

use serde_json::Value;
use vsm_runtime::{forward, VsmModel};

fn decode(labels: &[u8]) -> Vec<(usize, usize, u8)> {
    let mut spans = Vec::new();
    let mut start = None;
    let mut current = 0u8;
    for (index, label) in labels.iter().copied().enumerate() {
        if label != current {
            if current != 0 {
                if let Some(start) = start {
                    spans.push((start, index, current));
                }
            }
            start = if label != 0 { Some(index) } else { None };
            current = label;
        }
    }
    if current != 0 {
        if let Some(start) = start {
            spans.push((start, labels.len(), current));
        }
    }
    spans
}

fn overlap(left: (usize, usize, u8), right: (usize, usize, u8)) -> bool {
    left.2 == right.2 && left.0 < right.1 && right.0 < left.1
}

fn main() {
    let mut args = std::env::args().skip(1);
    let model_path = args
        .next()
        .unwrap_or_else(|| "neural/fixtures/vsm_96_4_int8.vsm".to_string());
    let data_path = args
        .next()
        .unwrap_or_else(|| "data_processed/neural_large/test.jsonl".to_string());
    let limit: usize = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(10_000);

    let bytes = fs::read(&model_path).expect("đọc .vsm");
    let model = VsmModel::from_bytes(&bytes).expect("parse .vsm");
    let data = fs::read_to_string(&data_path).expect("đọc test jsonl");

    let mut true_positive = 0usize;
    let mut false_positive = 0usize;
    let mut false_negative = 0usize;
    let mut rows = 0usize;
    for line in data.lines().take(limit) {
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(text) = record["text"].as_str() else {
            continue;
        };
        let gold: Vec<(usize, usize, u8)> = record["spans"]
            .as_array()
            .map(|spans| {
                spans
                    .iter()
                    .filter_map(|span| {
                        Some((
                            span["start"].as_u64()? as usize,
                            span["end"].as_u64()? as usize,
                            span["label"].as_u64()? as u8,
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let output = forward(&model, text.as_bytes()).expect("forward");
        let predicted = decode(&output.error_argmax());
        let mut matched = vec![false; gold.len()];
        for prediction in &predicted {
            if let Some(index) = gold
                .iter()
                .enumerate()
                .find(|(index, span)| !matched[*index] && overlap(*prediction, **span))
                .map(|(index, _)| index)
            {
                matched[index] = true;
                true_positive += 1;
            } else {
                false_positive += 1;
            }
        }
        false_negative += matched.iter().filter(|value| !**value).count();
        rows += 1;
    }

    let precision = true_positive as f64 / (true_positive + false_positive).max(1) as f64;
    let recall = true_positive as f64 / (true_positive + false_negative).max(1) as f64;
    let f1 = 2.0 * precision * recall / (precision + recall).max(1e-9);
    println!(
        "{{\"rows\":{rows},\"precision\":{precision:.6},\"recall\":{recall:.6},\"f1\":{f1:.6},\"tp\":{true_positive},\"fp\":{false_positive},\"fn\":{false_negative}}}"
    );
}
