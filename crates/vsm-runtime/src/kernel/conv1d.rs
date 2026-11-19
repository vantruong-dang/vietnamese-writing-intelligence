//! Kernel Conv1D W8A32: trá»ng sá»‘ INT8, activation F32.
//!
//! Layout time-major `[T][C]` Ä‘á»ƒ thuáº­n tiá»‡n cho byte-level tagging.
//! Trá»ng sá»‘ dense/PW á»Ÿ layout `[out][k][in]`; DW á»Ÿ `[channel][k]`.
//!
//! `*_w8a32` nháº­n input **Ä‘Ã£ padding** cho DW/dense. HÃ m `pad_time_major`
//! trong `kernel/mod.rs` táº¡o padding chÃ­nh xÃ¡c theo dilation.

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

/// Depthwise Conv1D, xá»­ lÃ½ 4 kÃªnh song song báº±ng f32x4 (map xuá»‘ng simd128 trÃªn wasm).
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
        "DW input quÃ¡ ngáº¯n: {} < {}",
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

/// Dense Conv1D, layout weight `[out][k][in]`. Input Ä‘Ã£ padding.
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
        "dense input quÃ¡ ngáº¯n: {} < {}",
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

#[inline]
fn dot_f32(input: &[f32], weight: &[f32]) -> f32 {
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
            weight[index],
            weight[index + 1],
            weight[index + 2],
            weight[index + 3],
        ]);
        accumulator += x * w;
        index += 4;
    }
    let mut total = accumulator.reduce_add();
    while index < length {
        total += input[index] * weight[index];
        index += 1;
    }
    total
}

/// Pointwise Conv1D F32 với trọng số pack-4 `[out/4][in][4]`.
///
/// Giữ activation `[T, C]`; 4 output channel nằm trên 4 làn SIMD, 4 timestep
/// song song. Không transpose, không horizontal sum.
pub fn conv1d_pw_f32_packed(
    input: &[f32],
    packed_weight: &[f32],
    out_channels: usize,
    in_channels: usize,
    length: usize,
) -> Vec<f32> {
    assert_eq!(input.len(), length * in_channels);
    assert!(out_channels.is_multiple_of(4));
    let blocks = out_channels / 4;
    assert_eq!(packed_weight.len(), blocks * in_channels * 4);
    let mut output = vec![0.0f32; length * out_channels];

    for block in 0..blocks {
        let output_base = block * 4;
        let mut time = 0usize;
        while time + 4 <= length {
            let mut acc0 = f32x4::ZERO;
            let mut acc1 = f32x4::ZERO;
            let mut acc2 = f32x4::ZERO;
            let mut acc3 = f32x4::ZERO;
            for input_channel in 0..in_channels {
                let packed_base = (block * in_channels + input_channel) * 4;
                let w = f32x4::from([
                    packed_weight[packed_base],
                    packed_weight[packed_base + 1],
                    packed_weight[packed_base + 2],
                    packed_weight[packed_base + 3],
                ]);
                let x0 = f32x4::splat(input[time * in_channels + input_channel]);
                let x1 = f32x4::splat(input[(time + 1) * in_channels + input_channel]);
                let x2 = f32x4::splat(input[(time + 2) * in_channels + input_channel]);
                let x3 = f32x4::splat(input[(time + 3) * in_channels + input_channel]);
                acc0 = x0.mul_add(w, acc0);
                acc1 = x1.mul_add(w, acc1);
                acc2 = x2.mul_add(w, acc2);
                acc3 = x3.mul_add(w, acc3);
            }
            let base0 = time * out_channels + output_base;
            output[base0..base0 + 4].copy_from_slice(&acc0.to_array());
            let base1 = base0 + out_channels;
            output[base1..base1 + 4].copy_from_slice(&acc1.to_array());
            let base2 = base1 + out_channels;
            output[base2..base2 + 4].copy_from_slice(&acc2.to_array());
            let base3 = base2 + out_channels;
            output[base3..base3 + 4].copy_from_slice(&acc3.to_array());
            time += 4;
        }
        while time < length {
            let mut total = [0.0f32; 4];
            for input_channel in 0..in_channels {
                let packed_base = (block * in_channels + input_channel) * 4;
                let x = input[time * in_channels + input_channel];
                for lane in 0..4 {
                    total[lane] += x * packed_weight[packed_base + lane];
                }
            }
            let base = time * out_channels + output_base;
            output[base..base + 4].copy_from_slice(&total);
            time += 1;
        }
    }
    output
}

pub fn conv1d_pw_f32_packed_residual(
    input: &[f32],
    packed_weight: &[f32],
    residual: &[f32],
    out_channels: usize,
    in_channels: usize,
    length: usize,
) -> Vec<f32> {
    assert_eq!(input.len(), length * in_channels);
    assert_eq!(residual.len(), length * out_channels);
    assert!(out_channels.is_multiple_of(4));
    let blocks = out_channels / 4;
    assert_eq!(packed_weight.len(), blocks * in_channels * 4);
    let mut output = vec![0.0f32; length * out_channels];

    for block in 0..blocks {
        let output_base = block * 4;
        let mut time = 0usize;
        while time + 4 <= length {
            let mut acc0 = f32x4::ZERO;
            let mut acc1 = f32x4::ZERO;
            let mut acc2 = f32x4::ZERO;
            let mut acc3 = f32x4::ZERO;
            for input_channel in 0..in_channels {
                let packed_base = (block * in_channels + input_channel) * 4;
                let w = f32x4::from([
                    packed_weight[packed_base],
                    packed_weight[packed_base + 1],
                    packed_weight[packed_base + 2],
                    packed_weight[packed_base + 3],
                ]);
                acc0 = f32x4::splat(input[time * in_channels + input_channel]).mul_add(w, acc0);
                acc1 =
                    f32x4::splat(input[(time + 1) * in_channels + input_channel]).mul_add(w, acc1);
                acc2 =
                    f32x4::splat(input[(time + 2) * in_channels + input_channel]).mul_add(w, acc2);
                acc3 =
                    f32x4::splat(input[(time + 3) * in_channels + input_channel]).mul_add(w, acc3);
            }
            for (offset, accumulator) in [acc0, acc1, acc2, acc3].into_iter().enumerate() {
                let base = (time + offset) * out_channels + output_base;
                let residual_base = time * out_channels + output_base;
                let residual_values = f32x4::from([
                    residual[residual_base + offset * out_channels],
                    residual[residual_base + offset * out_channels + 1],
                    residual[residual_base + offset * out_channels + 2],
                    residual[residual_base + offset * out_channels + 3],
                ]);
                output[base..base + 4].copy_from_slice(&(accumulator + residual_values).to_array());
            }
            time += 4;
        }
        while time < length {
            let mut total = [0.0f32; 4];
            for input_channel in 0..in_channels {
                let packed_base = (block * in_channels + input_channel) * 4;
                let x = input[time * in_channels + input_channel];
                for lane in 0..4 {
                    total[lane] += x * packed_weight[packed_base + lane];
                }
            }
            let base = time * out_channels + output_base;
            for lane in 0..4 {
                total[lane] += residual[base + lane];
            }
            output[base..base + 4].copy_from_slice(&total);
            time += 1;
        }
    }
    output
}

fn transpose_time_to_channel(input: &[f32], channels: usize, length: usize) -> Vec<f32> {
    let mut output = vec![0.0f32; channels * length];
    for time in 0..length {
        for channel in 0..channels {
            output[channel * length + time] = input[time * channels + channel];
        }
    }
    output
}

fn transpose_channel_to_time(input: &[f32], channels: usize, length: usize) -> Vec<f32> {
    let mut output = vec![0.0f32; channels * length];
    for channel in 0..channels {
        for time in 0..length {
            output[time * channels + channel] = input[channel * length + time];
        }
    }
    output
}

/// Pointwise Conv1D F32 (fast path sau khi dequantize má»™t láº§n khi load).
///
/// Transpose activation sang `[in][T]` Ä‘á»ƒ load 4 time step liÃªn tiáº¿p báº±ng má»™t
/// `v128`, rá»“i transpose káº¿t quáº£ vá» `[T][out]`.
pub fn conv1d_pw_f32(
    input: &[f32],
    weight: &[f32],
    out_channels: usize,
    in_channels: usize,
    length: usize,
) -> Vec<f32> {
    assert_eq!(input.len(), length * in_channels);
    assert_eq!(weight.len(), out_channels * in_channels);
    let input_t = transpose_time_to_channel(input, in_channels, length);
    let mut output_t = vec![0.0f32; out_channels * length];
    for out_channel in 0..out_channels {
        let weight_row = &weight[out_channel * in_channels..(out_channel + 1) * in_channels];
        let output_base = out_channel * length;
        let mut time = 0usize;
        while time + 8 <= length {
            let mut accumulator0 = f32x4::ZERO;
            let mut accumulator1 = f32x4::ZERO;
            for (in_channel, weight_value) in weight_row.iter().enumerate() {
                let base = in_channel * length + time;
                let x0 = f32x4::from([
                    input_t[base],
                    input_t[base + 1],
                    input_t[base + 2],
                    input_t[base + 3],
                ]);
                let x1 = f32x4::from([
                    input_t[base + 4],
                    input_t[base + 5],
                    input_t[base + 6],
                    input_t[base + 7],
                ]);
                let weight = f32x4::splat(*weight_value);
                accumulator0 = x0.mul_add(weight, accumulator0);
                accumulator1 = x1.mul_add(weight, accumulator1);
            }
            output_t[output_base + time..output_base + time + 4]
                .copy_from_slice(&accumulator0.to_array());
            output_t[output_base + time + 4..output_base + time + 8]
                .copy_from_slice(&accumulator1.to_array());
            time += 8;
        }
        while time + 4 <= length {
            let mut accumulator = f32x4::ZERO;
            for (in_channel, weight_value) in weight_row.iter().enumerate() {
                let base = in_channel * length + time;
                let x = f32x4::from([
                    input_t[base],
                    input_t[base + 1],
                    input_t[base + 2],
                    input_t[base + 3],
                ]);
                accumulator = x.mul_add(f32x4::splat(*weight_value), accumulator);
            }
            let values = accumulator.to_array();
            output_t[output_base + time..output_base + time + 4].copy_from_slice(&values);
            time += 4;
        }
        while time < length {
            let mut total = 0.0f32;
            for (in_channel, weight_value) in weight_row.iter().enumerate() {
                total += input_t[in_channel * length + time] * weight_value;
            }
            output_t[output_base + time] = total;
            time += 1;
        }
    }
    transpose_channel_to_time(&output_t, out_channels, length)
}

/// Depthwise Conv1D F32, input Ä‘Ã£ padding.
pub fn conv1d_dw_f32(
    padded: &[f32],
    weight: &[f32],
    channels: usize,
    kernel: usize,
    dilation: usize,
    length: usize,
) -> Vec<f32> {
    let required = (length + kernel.saturating_sub(1) * dilation) * channels;
    assert!(padded.len() >= required);
    assert_eq!(weight.len(), channels * kernel);
    let mut output = vec![0.0f32; length * channels];
    for channel in 0..channels {
        let mut time = 0usize;
        while time + 4 <= length {
            let mut accumulator = f32x4::ZERO;
            for tap in 0..kernel {
                let source = time + tap * dilation;
                let x = f32x4::from([
                    padded[source * channels + channel],
                    padded[(source + 1) * channels + channel],
                    padded[(source + 2) * channels + channel],
                    padded[(source + 3) * channels + channel],
                ]);
                accumulator += x * f32x4::splat(weight[channel * kernel + tap]);
            }
            let values = accumulator.to_array();
            for offset in 0..4 {
                output[(time + offset) * channels + channel] = values[offset];
            }
            time += 4;
        }
        while time < length {
            let mut total = 0.0f32;
            for tap in 0..kernel {
                let source = time + tap * dilation;
                total += padded[source * channels + channel] * weight[channel * kernel + tap];
            }
            output[time * channels + channel] = total;
            time += 1;
        }
    }
    output
}

/// Dense Conv1D F32, input Ä‘Ã£ padding, weight `[out][k][in]`.
#[allow(clippy::too_many_arguments)]
pub fn conv1d_dense_f32(
    padded: &[f32],
    weight: &[f32],
    out_channels: usize,
    in_channels: usize,
    kernel: usize,
    dilation: usize,
    length: usize,
) -> Vec<f32> {
    let required = (length + kernel.saturating_sub(1) * dilation) * in_channels;
    assert!(padded.len() >= required);
    assert_eq!(weight.len(), out_channels * kernel * in_channels);
    let mut output = vec![0.0f32; length * out_channels];
    for out_channel in 0..out_channels {
        let out_weight_base = out_channel * kernel * in_channels;
        let mut time = 0usize;
        while time + 4 <= length {
            let mut accumulator = f32x4::ZERO;
            for tap in 0..kernel {
                let weight_base = out_weight_base + tap * in_channels;
                for in_channel in 0..in_channels {
                    let source = time + tap * dilation;
                    let x = f32x4::from([
                        padded[source * in_channels + in_channel],
                        padded[(source + 1) * in_channels + in_channel],
                        padded[(source + 2) * in_channels + in_channel],
                        padded[(source + 3) * in_channels + in_channel],
                    ]);
                    accumulator += x * f32x4::splat(weight[weight_base + in_channel]);
                }
            }
            let values = accumulator.to_array();
            for offset in 0..4 {
                output[(time + offset) * out_channels + out_channel] = values[offset];
            }
            time += 4;
        }
        while time < length {
            let mut total = 0.0f32;
            for tap in 0..kernel {
                let source = time + tap * dilation;
                let input_base = source * in_channels;
                let weight_base = out_weight_base + tap * in_channels;
                total += dot_f32(
                    &padded[input_base..input_base + in_channels],
                    &weight[weight_base..weight_base + in_channels],
                );
            }
            output[time * out_channels + out_channel] = total;
            time += 1;
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dw_identity() {
        // 1 kÃªnh, kernel 1, weight [1] -> copy input.
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
