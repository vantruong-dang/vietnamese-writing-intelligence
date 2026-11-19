//! Benchmark native (host) cho VSM runtime.
//!
//!   cargo run -p vsm-runtime --example bench --release -- \
//!       neural/fixtures/full_random.vsm 2000

use std::time::Instant;

use vsm_runtime::{forward, VsmModel};

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "neural/fixtures/full_random.vsm".to_string());
    let bytes = std::fs::read(&path).expect("đọc .vsm");
    let model = VsmModel::from_bytes(&bytes).expect("parse .vsm");
    let length: usize = std::env::args()
        .nth(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(2_000);
    let input = vec![b'a'; length];

    for _ in 0..3 {
        let _ = forward(&model, &input).expect("warmup");
    }
    let runs = 20;
    let start = Instant::now();
    for _ in 0..runs {
        let _ = forward(&model, &input).expect("forward");
    }
    let elapsed = start.elapsed().as_secs_f64() * 1000.0 / runs as f64;
    println!(
        "{{\"bytes\":{length},\"runs\":{runs},\"average_ms\":{elapsed:.3},\"chunks\":{}}}",
        length.div_ceil(128)
    );
}
