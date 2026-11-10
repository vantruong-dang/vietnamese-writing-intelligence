# `vwi-cli` — Vietnamese Writing Intelligence Native CLI & MCP Server

[![Crates.io](https://img.shields.io/crates/v/vwi-cli.svg)](https://crates.io/crates/vwi-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

`vwi` is a standalone, zero-dependency native binary and Model Context Protocol (MCP) server for auditing Vietnamese prose, detecting translationese (Tây hóa), AI clichés, bureaucratic padding, and functional Topic-Comment violations based on Prof. Cao Xuân Hạo's linguistic framework.

## Installation

```bash
cargo install vwi-cli
```

## Quick Start

```bash
# Quick audit of a file with colored terminal report
vwi check document.md

# Detailed semantic audit with immutable anchor verification & JSON output
vwi audit document.md --anchor "100 Gbps" --anchor "TP.HCM" --format json

# Strict CI/CD gate: fail if quality score drops below 85/100
vwi check document.md --fail-under 85 --major-only

# Tokenize and tag Universal Dependencies POS with Viterbi Trellis
vwi tokenize "Học sinh học sinh học tại trường." --pos

# Start native stdio MCP Server for Claude Desktop / Cursor
vwi mcp
```
