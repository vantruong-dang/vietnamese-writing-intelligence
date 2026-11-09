# `vwi-core` — Vietnamese Writing Intelligence Core Engine

[![Crates.io](https://img.shields.io/crates/v/vwi-core.svg)](https://crates.io/crates/vwi-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

`vwi-core` is the pure-Rust linguistic engine powering **Vietnamese Writing Intelligence**. It combines a statistical Viterbi Trellis word segmenter (72,564 compound dictionary), Universal Dependencies (UPOS) part-of-speech tagger, and a functional Topic-Comment shallow parser inspired by Prof. Cao Xuân Hạo's framework.

## Key Features

- **Viterbi Trellis Word Segmentation**: Disambiguates overlapping Vietnamese compounds using Unigram/Bigram log-probabilities and Double-Array Aho-Corasick (`daachorse`).
- **Universal Dependencies POS Tagging**: $O(\log N)$ binary search lexicon with $13 \times 13$ transition matrix and morphological OOV heuristics.
- **Topic-Comment Shallow Parser**: Detects Crippled Prepositional Topics (*Khung đề què*) and Preposition-Conjunction Conflicts without fragile token-limit regexes.
- **25+ Linguistic & Stylistic Rules**: Covers artificial passives, translationese, pleonasms, concessive mood misuse, bureaucratic verb padding, and statistically mined AI clichés ($G^2$ Log-Likelihood).
- **Parallel Batch Auditing**: Multi-threaded document auditing via `rayon` with configurable rule overrides and non-linear 0–100 quality scoring (`AuditReport`).

## Quick Start

```rust
use vwi_core::{Auditor, tokenize_viterbi};

fn main() {
    let auditor = Auditor::default();
    let report = auditor.audit_document(
        "Trong tình hình kinh tế hiện nay đòi hỏi doanh nghiệp phải tái cơ cấu toàn diện."
    );
    println!("Quality Score: {}/100 ({})", report.score, report.grade);
    for f in &report.findings {
        println!("[{}] {}: {}", f.severity, f.pattern_id, f.matched_span);
    }

    let tokens = tokenize_viterbi("Học sinh học sinh học tại trường.");
    for t in tokens {
        println!("{} -> {:?}", t.text, t.pos);
    }
}
```
