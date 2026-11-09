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
