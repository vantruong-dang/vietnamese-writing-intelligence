# `vwi-cli` — Vietnamese Writing Intelligence Native CLI

[![Crates.io](https://img.shields.io/crates/v/vwi-cli.svg)](https://crates.io/crates/vwi-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

`vwi` is a standalone, zero-dependency native binary for auditing Vietnamese prose, detecting translationese (Tây hóa), AI clichés, bureaucratic padding, and functional Topic-Comment violations based on Prof. Cao Xuân Hạo's linguistic framework.

## Installation

```bash
cargo install vwi-cli
```

## Quick Start

```bash
# Audit a file or directory with colored terminal report
vwi check document.md

# Audit with Markdown table output
vwi check document.md --format markdown

# Strict CI/CD gate: fail if quality score drops below 85/100
vwi check docs/ --min-score 85 --fail-on-major

# Tokenize and tag Universal Dependencies POS with Viterbi Trellis
vwi tokenize "Học sinh học sinh học tại trường."
```

## Scope & Current Limitations

### Strengths:
* **Standalone Executable:** ~5MB single binary with zero external dependencies (no Python, no C++ runtime).
* **CI/CD Quality Gate:** Strict `--fail-under` flag exiting with code 1 when quality score falls below threshold.
* **Deterministic Grammar Linter:** Catches common translationese, false passives, and broken topic-comment structures in journalistic, formal, and corporate writing.

### Current Limitations:
* **Not a Deep Learning Transformer:** Does not perform abstractive summarization, neural translation, or semantic search.
* **Standard Vietnamese Focus:** Teencode, social media slang, or heavy spelling distortions fall back to character/syllable matching.
* **Shallow Parsing:** Analyzes topic-comment boundary violations without constructing complete phrase-structure trees.

## License

Distributed under the [MIT License](https://opensource.org/licenses/MIT).

