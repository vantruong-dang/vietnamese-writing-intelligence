//! Kernel và activation cho VSM.

pub mod conv1d;

pub use conv1d::{
    conv1d_dense_f32, conv1d_dense_w8a32, conv1d_dw_f32, conv1d_dw_w8a32, conv1d_pw_f32,
    conv1d_pw_w8a32,
};

/// Padding time-major `[T][C]` với `pad` vị trí 0 mỗi phía.
pub fn pad_time_major(input: &[f32], length: usize, channels: usize, pad: usize) -> Vec<f32> {
    assert_eq!(input.len(), length * channels);
    let padded_length = length + 2 * pad;
    let mut output = vec![0.0f32; padded_length * channels];
    output[pad * channels..(pad + length) * channels].copy_from_slice(input);
    output
}

#[inline]
pub fn silu_inplace(values: &mut [f32]) {
    for value in values {
        *value = *value / (1.0 + (-*value).exp());
    }
}

#[inline]
pub fn gelu_tanh_inplace(values: &mut [f32]) {
    const COEFFICIENT: f32 = 0.797_884_6; // sqrt(2/pi)
    for value in values {
        let x = *value;
        *value = 0.5 * x * (1.0 + (COEFFICIENT * (x + 0.044_715 * x * x * x)).tanh());
    }
}

/// RMSNorm trên layout `[T][C]`.
pub fn rms_norm_inplace(values: &mut [f32], weight: &[f32], channels: usize, eps: f32) {
    assert_eq!(values.len() % channels, 0);
    assert_eq!(weight.len(), channels);
    for row in values.chunks_exact_mut(channels) {
        let mean_square = row.iter().map(|value| value * value).sum::<f32>() / channels as f32;
        let scale = 1.0 / (mean_square + eps).sqrt();
        for (value, weight) in row.iter_mut().zip(weight) {
            *value = *value * scale * *weight;
        }
    }
}

pub fn add_inplace(target: &mut [f32], residual: &[f32]) {
    assert_eq!(target.len(), residual.len());
    for (target, residual) in target.iter_mut().zip(residual) {
        *target += *residual;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_and_rms() {
        let padded = pad_time_major(&[1.0, 2.0], 2, 1, 1);
        assert_eq!(padded, vec![0.0, 1.0, 2.0, 0.0]);
        let mut values = vec![3.0, 4.0];
        rms_norm_inplace(&mut values, &[1.0, 1.0], 2, 0.0);
        let scale = 1.0 / (12.5f32).sqrt();
        assert!((values[0] - 3.0 * scale).abs() < 1e-5);
        assert!((values[1] - 4.0 * scale).abs() < 1e-5);
    }
}
