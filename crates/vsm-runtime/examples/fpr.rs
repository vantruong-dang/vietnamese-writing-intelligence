//! Đo FPR của cascade Tầng 3 + Tầng 4 trên văn bản sạch.
//!
//!   cargo run -p vsm-runtime --features cascade --example fpr --release -- \
//!       neural/fixtures/vsm_96_4_int8.vsm data_processed/bench_corpus/journalism.txt 2000

use std::fs;
use std::path::Path;

use vsm_runtime::VsmCascadeScorer;
use vwi_core::{
    run_tier4_with_bigram, BigramTable, CascadeConfig, NeuralContextScorer, Tier4Config,
};

fn main() {
    let mut args = std::env::args().skip(1);
    let model_path = args
        .next()
        .unwrap_or_else(|| "neural/fixtures/vsm_96_4_int8.vsm".to_string());
    let corpus_path = args
        .next()
        .unwrap_or_else(|| "data_processed/bench_corpus/journalism.txt".to_string());
    let max_sentences: usize = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(2_000);

    let model_bytes = fs::read(&model_path).expect("đọc .vsm");
    let scorer = VsmCascadeScorer::new(&model_bytes).expect("parse .vsm");
    let config = CascadeConfig::default();
    let tier4_config = Tier4Config::default();
    let bigram_bytes = fs::read("neural/fixtures/vi_bigram.vbg").ok();
    let bigram = bigram_bytes.as_deref().and_then(BigramTable::from_bytes);
    let text = fs::read_to_string(Path::new(&corpus_path)).expect("đọc corpus");

    let mut sentences = 0usize;
    let mut tokens = 0usize;
    let mut diagnostics = 0usize;
    let mut flagged_sentences = 0usize;
    let mut examples: Vec<String> = Vec::new();

    for line in text.lines() {
        let sentence = line.trim();
        let word_count = sentence
            .split_whitespace()
            .filter(|word| word.chars().any(char::is_alphabetic))
            .count();
        if !(3..=60).contains(&word_count) || sentence.contains("http") {
            continue;
        }
        sentences += 1;
        tokens += word_count;
        let locked = vec![false; sentence.len()];
        let mut hints = Vec::new();
        scorer.score_unmasked_regions(sentence, &locked, &mut hints);
        let found = run_tier4_with_bigram(
            sentence,
            &hints,
            &locked,
            config,
            &[],
            bigram.as_ref(),
            &tier4_config,
        );
        if !found.is_empty() {
            flagged_sentences += 1;
            diagnostics += found.len();
            if examples.len() < 20 {
                examples.push(format!(
                    "{} => {} ({})",
                    found[0].original, found[0].suggestion, sentence
                ));
            }
        }
        if sentences >= max_sentences {
            break;
        }
    }

    println!(
        "{{\"sentences\":{sentences},\"tokens\":{tokens},\"diagnostics\":{diagnostics},\"token_fpr\":{:.6},\"flagged_sentences\":{flagged_sentences},\"sentence_fpr\":{:.6}}}",
        diagnostics as f64 / tokens.max(1) as f64,
        flagged_sentences as f64 / sentences.max(1) as f64,
    );
    for example in examples {
        println!("  {example}");
    }
}
