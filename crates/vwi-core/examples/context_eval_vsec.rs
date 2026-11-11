#![cfg(feature = "context-lm")]

use std::fs;
use vwi_core::rules::context_spellcheck::evaluate_context_spellcheck_with;

fn main() {
    let dev_text = fs::read_to_string("data_processed/dev_hj.txt").expect("dev_hj.txt");
    let dev: Vec<&str> = dev_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(300)
        .collect();

    let vsec_text =
        fs::read_to_string("data_processed/vsec_realword.tsv").expect("vsec_realword.tsv");
    let vsec: Vec<(String, String, String)> = vsec_text
        .lines()
        .filter_map(|line| {
            let mut p = line.split('\t');
            let _id = p.next()?;
            let sent = p.next()?.to_string();
            let wrong = p.next()?.to_string();
            let correct = p.next()?.to_string();
            Some((sent, wrong, correct))
        })
        .take(500)
        .collect();

    println!("dev_hj={} vsec={}", dev.len(), vsec.len());

    for min_abs in [-3.0f32, -4.0, -5.0, -6.0, -7.0] {
        for margin in [2.0f32, 4.0, 6.0, 8.0, 10.0] {
            let fp = dev
                .iter()
                .filter(|line| !evaluate_context_spellcheck_with(line, min_abs, margin).is_empty())
                .count();
            let mut tp = 0usize;
            for (sent, wrong, correct) in &vsec {
                let findings = evaluate_context_spellcheck_with(sent, min_abs, margin);
                if findings.iter().any(|f| {
                    f.matched_span.to_lowercase() == wrong.to_lowercase()
                        && f.suggestion
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&correct.to_lowercase())
                }) {
                    tp += 1;
                }
            }
            let recall = tp as f64 / vsec.len() as f64;
            let fpr = fp as f64 / dev.len() as f64;
            let precision = if tp + fp == 0 {
                0.0
            } else {
                tp as f64 / (tp + fp) as f64
            };
            let f1 = if precision + recall == 0.0 {
                0.0
            } else {
                2.0 * precision * recall / (precision + recall)
            };
            println!(
                "min={:<4} margin={:<4} TP={:<3} FP={:<3} recall={:>6.2}% FPR={:>6.2}% P={:>5.1}% F1={:>5.1}%",
                min_abs,
                margin,
                tp,
                fp,
                recall * 100.0,
                fpr * 100.0,
                precision * 100.0,
                f1 * 100.0
            );
        }
    }
}
