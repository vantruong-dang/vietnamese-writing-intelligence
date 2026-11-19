//! Kernel Conv1D W8A32: trọng số INT8, activation F32.
//!
//! Layout time-major `[T][C]` để thuận tiện cho byte-level tagging.
//! Trọng số dense/PW ở layout `[out][k][in]`; DW ở `[channel][k]`.
//!
//! `*_w8a32` nhận input **đã padding** cho DW/dense. Hàm `pad_time_major`
//! trong `kernel/mod.rs` tạo padding chính xác theo dilation.

use wide::f32x4;

#[inline]
fn dot_i8_f32(input: &[f32], weight: &[i8]) -> f32 {
    let length = input.len().min(weight.len());
    let mut accumulator = f32x4::ZERO;
    let mut index = 0usize;
    while index + 4 <= length {
        let x = f32x4::from([
            input[index],
            input[index + 1],
            input[index + 2],
            input[index + 3],
        ]);
        let w = f32x4::from([
            weight[index] as f32,
            weight[index + 1] as f32,
            weight[index + 2] as f32,
            weight[index + 3] as f32,
        ]);
        accumulator += x * w;
        index += 4;
    }
    let mut total = accumulator.reduce_add();
    while index < length {
        total += input[index] * weight[index] as f32;
        index += 1;
    }
    total
}

/// Depthwise Conv1D, xử lý 4 kênh song song bằng f32x4 (map xuống simd128 trên wasm).
pub fn conv1d_dw_w8a32(
    padded: &[f32],
    weight: &[i8],
    scales: &[f32],
    channels: usize,
    kernel: usize,
    dilation: usize,
    length: usize,
) -> Vec<f32> {
    let required = (length + (kernel.saturating_sub(1)) * dilation) * channels;
    assert!(
        padded.len() >= required,
        "DW input quá ngắn: {} < {}",
        padded.len(),
        required
    );
    assert_eq!(weight.len(), channels * kernel);
    assert_eq!(scales.len(), channels);

    let mut output = vec![0.0f32; length * channels];
    for time in 0..length {
        let mut channel = 0usize;
        while channel + 4 <= channels {
            let mut accumulator = f32x4::ZERO;
            for tap in 0..kernel {
                let source = time + tap * dilation;
                let base = source * channels + channel;
                let x = f32x4::from([
                    padded[base],
                    padded[base + 1],
                    padded[base + 2],
                    padded[base + 3],
                ]);
                let weight_base = channel * kernel + tap;
                let w = f32x4::from([
                    weight[weight_base] as f32,
                    weight[weight_base + kernel] as f32,
                    weight[weight_base + 2 * kernel] as f32,
                    weight[weight_base + 3 * kernel] as f32,
                ]);
                accumulator += x * w;
            }
            let scale = f32x4::from([
                scales[channel],
                scales[channel + 1],
                scales[channel + 2],
                scales[channel + 3],
            ]);
            let result = accumulator * scale;
            let array = result.to_array();
            let output_base = time * channels + channel;
            output[output_base..output_base + 4].copy_from_slice(&array);
            channel += 4;
        }
        while channel < channels {
            let mut total = 0.0f32;
            for tap in 0..kernel {
                let source = time + tap * dilation;
                total +=
                    padded[source * channels + channel] * weight[channel * kernel + tap] as f32;
            }
            output[time * channels + channel] = total * scales[channel];
            channel += 1;
        }
    }
    output
}

/// Pointwise Conv1D (k=1), layout weight `[out][in]`.
pub fn conv1d_pw_w8a32(
    input: &[f32],
    weight: &[i8],
    scales: &[f32],
    out_channels: usize,
    in_channels: usize,
    length: usize,
) -> Vec<f32> {
    assert_eq!(input.len(), length * in_channels);
    assert_eq!(weight.len(), out_channels * in_channels);
    assert_eq!(scales.len(), out_channels);

    let mut output = vec![0.0f32; length * out_channels];
    for time in 0..length {
        let input_base = time * in_channels;
        let output_base = time * out_channels;
        for out_channel in 0..out_channels {
            let weight_base = out_channel * in_channels;
            let value = dot_i8_f32(
                &input[input_base..input_base + in_channels],
                &weight[weight_base..weight_base + in_channels],
            );
            output[output_base + out_channel] = value * scales[out_channel];
        }
    }
    output
}

/// Dense Conv1D, layout weight `[out][k][in]`. Input đã padding.
#[allow(clippy::too_many_arguments)]
pub fn conv1d_dense_w8a32(
    padded: &[f32],
    weight: &[i8],
    scales: &[f32],
    out_channels: usize,
    in_channels: usize,
    kernel: usize,
    dilation: usize,
    length: usize,
) -> Vec<f32> {
    let required = (length + (kernel.saturating_sub(1)) * dilation) * in_channels;
    assert!(
        padded.len() >= required,
        "dense input quá ngắn: {} < {}",
        padded.len(),
        required
    );
    assert_eq!(weight.len(), out_channels * kernel * in_channels);
    assert_eq!(scales.len(), out_channels);

    let mut output = vec![0.0f32; length * out_channels];
    for time in 0..length {
        for out_channel in 0..out_channels {
            let mut total = 0.0f32;
            for tap in 0..kernel {
                let source = time + tap * dilation;
                let input_base = source * in_channels;
                let weight_base = (out_channel * kernel + tap) * in_channels;
                total += dot_i8_f32(
                    &padded[input_base..input_base + in_channels],
                    &weight[weight_base..weight_base + in_channels],
                );
            }
            output[time * out_channels + out_channel] = total * scales[out_channel];
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dw_identity() {
        // 1 kênh, kernel 1, weight [1] -> copy input.
        let input = vec![1.0, 2.0, 3.0];
        let output = conv1d_dw_w8a32(&input, &[127], &[1.0 / 127.0], 1, 1, 1, 3);
        assert!((output[0] - 1.0).abs() < 1e-5);
        assert!((output[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_pointwise_matches_scalar() {
        let input = vec![0.5, -0.25, 1.0, 0.75];
        let weight = vec![127i8, 0, 0, 127];
        let out = conv1d_pw_w8a32(&input, &weight, &[1.0 / 127.0, 1.0 / 127.0], 2, 2, 2);
        assert!((out[0] - 0.5).abs() < 1e-5);
        assert!((out[3] - 0.75).abs() < 1e-5);
    }
}
