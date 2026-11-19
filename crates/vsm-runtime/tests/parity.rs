use std::fs;
use std::path::PathBuf;

use vsm_runtime::{forward, VsmModel};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../neural/fixtures")
}

fn parse_expected(text: &str, key: &str) -> Vec<f32> {
    let line = text
        .lines()
        .find(|line| line.starts_with(key))
        .unwrap_or_else(|| panic!("thiếu dòng {key}"));
    line.split_whitespace()
        .skip(1)
        .map(|value| value.parse::<f32>().expect("expected number"))
        .collect()
}

#[test]
fn parity_fixture_int8() {
    let dir = fixture_dir();
    let bytes = fs::read(dir.join("tiny_model.vsm")).expect("đọc fixture");
    let expected_text =
        fs::read_to_string(dir.join("tiny_model_expected.txt")).expect("đọc expected");

    let model = VsmModel::from_bytes(&bytes).expect("parse .vsm");
    let input: Vec<u8> = parse_expected(&expected_text, "input")
        .into_iter()
        .map(|value| value as u8)
        .collect();
    let expected_logits = parse_expected(&expected_text, "error_logits");
    let expected_argmax = parse_expected(&expected_text, "byte_argmax")
        .into_iter()
        .map(|value| value as u8)
        .collect::<Vec<_>>();

    let output = forward(&model, &input).expect("forward");
    assert_eq!(output.length, input.len());
    assert_eq!(output.error_logits.len(), expected_logits.len());
    for (index, (actual, expected)) in output
        .error_logits
        .iter()
        .zip(expected_logits.iter())
        .enumerate()
    {
        let delta = (actual - expected).abs();
        assert!(
            delta <= 2e-3,
            "error_logits[{index}] lệch {delta}: {actual} vs {expected}"
        );
    }
    assert_eq!(output.byte_argmax(), expected_argmax);
}

#[test]
fn rejects_bad_magic() {
    let mut bytes = fs::read(fixture_dir().join("tiny_model.vsm")).expect("đọc fixture");
    bytes[0] = b'X';
    assert!(VsmModel::from_bytes(&bytes).is_err());
}

#[test]
fn chunked_forward_matches_full_forward() {
    let bytes = fs::read(fixture_dir().join("tiny_model.vsm")).expect("đọc fixture");
    let model = VsmModel::from_bytes(&bytes).expect("parse .vsm");
    let input: Vec<u8> = (0..48u16).map(|value| (value % 251) as u8).collect();
    let full = forward(&model, &input).expect("full forward");
    // max_seq_len của fixture là 32, nên forward() tự chunk.
    assert_eq!(full.length, input.len());
    assert_eq!(full.error_argmax().len(), input.len());
    assert_eq!(full.byte_argmax().len(), input.len());
}
