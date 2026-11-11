//! # BIGRAM LANGUAGE MODEL (WIKIPEDIA + HUMAN CORPUS)
//!
//! Dữ liệu được sinh bởi `tools/build_bigram_lm.py`, prune count >= 3.
//! Lưu dạng mảng phẳng đã sort để tra cứu binary search, không dùng HashMap
//! cho bảng bigram (tiết kiệm bộ nhớ cho WASM).
//!
//! Unigram count được suy ra từ tổng bigram đi ra (`out_count`), đủ cho
//! conditional probability P(w|prev) = c(prev,w) / c(prev).

use std::collections::HashMap;
use std::sync::LazyLock;

const BIGRAM_TSV: &str = include_str!("../../data/bigram_lm.tsv");

pub const SMOOTHING_K: f32 = 0.1;

static BIGRAMS: LazyLock<Vec<(&'static str, &'static str, u32)>> = LazyLock::new(|| {
    let mut rows: Vec<(&'static str, &'static str, u32)> = BIGRAM_TSV
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let a = parts.next()?;
            let b = parts.next()?;
            let c: u32 = parts.next()?.parse().ok()?;
            Some((a, b, c))
        })
        .collect();
    rows.sort_unstable_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    rows
});

/// Mọi từ xuất hiện ở bất kỳ phía nào của bigram (kể cả w2). Dùng để biết
/// một từ có trong LM hay không — `out_count` chỉ đúng cho w1.
static KNOWN_WORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let mut words: Vec<&'static str> = Vec::with_capacity(BIGRAMS.len() * 2);
    for (a, b, _) in BIGRAMS.iter() {
        words.push(*a);
        words.push(*b);
    }
    words.sort_unstable();
    words.dedup();
    words
});

static OUT_COUNTS: LazyLock<Vec<(&'static str, u32)>> = LazyLock::new(|| {
    let mut map: HashMap<&'static str, u32> = HashMap::new();
    for (a, _, c) in BIGRAMS.iter() {
        *map.entry(*a).or_insert(0) += *c;
    }
    let mut rows: Vec<(&'static str, u32)> = map.into_iter().collect();
    rows.sort_unstable_by(|a, b| a.0.cmp(b.0));
    rows
});

/// Số cặp bigram (đã prune) trong LM.
pub fn bigram_size() -> usize {
    BIGRAMS.len()
}

/// Số từ có bigram đi ra (xấp xỉ vocabulary của LM).
pub fn vocab_size() -> usize {
    OUT_COUNTS.len()
}

pub fn bigram_count(w1: &str, w2: &str) -> u32 {
    match BIGRAMS.binary_search_by(|probe| (probe.0, probe.1).cmp(&(w1, w2))) {
        Ok(idx) => BIGRAMS[idx].2,
        Err(_) => 0,
    }
}

pub fn out_count(w1: &str) -> u32 {
    match OUT_COUNTS.binary_search_by(|probe| probe.0.cmp(w1)) {
        Ok(idx) => OUT_COUNTS[idx].1,
        Err(_) => 0,
    }
}

pub fn contains_word(w: &str) -> bool {
    KNOWN_WORDS.binary_search(&w).is_ok()
}

/// log P(w | prev) với add-k; None nếu bigram chưa từng thấy.
pub fn log_p_bigram(prev: &str, w: &str) -> Option<f32> {
    let c = bigram_count(prev, w);
    if c == 0 {
        return None;
    }
    let denom = out_count(prev);
    if denom == 0 {
        return None;
    }
    let k = SMOOTHING_K;
    let p = (c as f32 + k) / (denom as f32 + k * vocab_size() as f32);
    Some(p.ln())
}

/// Backoff uniform khi chưa có bigram.
pub fn log_uniform() -> f32 {
    (1.0 / vocab_size() as f32).ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lm_loaded() {
        assert!(bigram_size() > 400_000);
        assert!(vocab_size() > 40_000);
    }

    #[test]
    fn test_known_bigrams() {
        assert!(bigram_count("cơn", "bão") > 0);
        assert!(bigram_count("buổi", "sáng") > 0);
        assert!(bigram_count("giấy", "tờ") > 0);
    }
}
