use std::time::Instant;

use vsm_runtime::kernel::conv1d::conv1d_pw_f32;

fn main() {
    let length = 256;
    let in_channels = 160;
    let out_channels = 320;
    let input = vec![0.01f32; length * in_channels];
    let weight = vec![0.01f32; out_channels * in_channels];
    for _ in 0..5 {
        let _ = conv1d_pw_f32(&input, &weight, out_channels, in_channels, length);
    }
    let runs = 100;
    let start = Instant::now();
    for _ in 0..runs {
        let _ = conv1d_pw_f32(&input, &weight, out_channels, in_channels, length);
    }
    let ms = start.elapsed().as_secs_f64() * 1000.0 / runs as f64;
    println!(
        "pw 256x320x160: {ms:.3} ms ({} MACs)",
        length * in_channels * out_channels
    );
}
