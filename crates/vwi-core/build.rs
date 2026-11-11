//! Chọn nguồn bigram LM lúc compile:
//! - `data/bigram_lm.tsv` + `data/unigram_lm.tsv` nếu có (người dùng cung cấp),
//! - nếu không, dùng fixture nhỏ `*_test.tsv` đã commit để CI build được
//!   feature `context-lm` trên checkout sạch.
//!
//! Fixture chỉ để compile/test; không dùng cho production.

use std::path::PathBuf;

fn main() {
    // Khi feature tắt, bigram_lm.rs không được biên dịch nên không cần data.
    if std::env::var_os("CARGO_FEATURE_CONTEXT_LM").is_none() {
        return;
    }

    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let real_bigram = manifest.join("data/bigram_lm.tsv");
    let real_unigram = manifest.join("data/unigram_lm.tsv");
    let fixture_bigram = manifest.join("data/bigram_lm_test.tsv");
    let fixture_unigram = manifest.join("data/unigram_lm_test.tsv");

    let (bigram, unigram) = if real_bigram.exists() && real_unigram.exists() {
        (&real_bigram, &real_unigram)
    } else {
        if !fixture_bigram.exists() || !fixture_unigram.exists() {
            panic!(
                "context-lm cần data/bigram_lm.tsv + data/unigram_lm.tsv, \
                 hoặc fixture data/bigram_lm_test.tsv + data/unigram_lm_test.tsv"
            );
        }
        println!(
            "cargo:warning=context-lm fixture: using data/bigram_lm_test.tsv; \
             production needs real bigram_lm.tsv + unigram_lm.tsv"
        );
        (&fixture_bigram, &fixture_unigram)
    };

    for path in [
        &real_bigram,
        &real_unigram,
        &fixture_bigram,
        &fixture_unigram,
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rustc-env=VWI_BIGRAM_TSV={}", bigram.display());
    println!("cargo:rustc-env=VWI_UNIGRAM_TSV={}", unigram.display());
}
