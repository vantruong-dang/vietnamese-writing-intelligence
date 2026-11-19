//! Forward pass VSM trên runtime Rust (W8A32).

use crate::format::{Tensor, VsmError, VsmModel};
use crate::kernel::{
    add_inplace, conv1d_dense_w8a32, conv1d_dw_w8a32, conv1d_pw_w8a32, gelu_tanh_inplace,
    pad_time_major, rms_norm_inplace, silu_inplace,
};

#[derive(Debug, Clone)]
pub struct ForwardOutput {
    pub length: usize,
    pub error_logits: Vec<f32>,
    pub byte_logits: Vec<f32>,
}

impl ForwardOutput {
    pub fn error_argmax(&self) -> Vec<u8> {
        self.error_logits
            .as_chunks::<4>()
            .0
            .iter()
            .map(|row| argmax(row))
            .collect()
    }

    pub fn byte_argmax(&self) -> Vec<u8> {
        self.byte_logits
            .as_chunks::<256>()
            .0
            .iter()
            .map(|row| argmax(row))
            .collect()
    }
}

fn argmax(row: &[f32]) -> u8 {
    let mut best = 0usize;
    for index in 1..row.len() {
        if row[index] > row[best] {
            best = index;
        }
    }
    best as u8
}

fn scales<'a>(model: &'a VsmModel, name: &str, expected: usize) -> Result<&'a [f32], VsmError> {
    let tensor = model.tensor(name)?;
    let values = tensor.as_f32()?;
    if values.len() != expected {
        return Err(VsmError::ShapeMismatch(name.to_string()));
    }
    Ok(values)
}

fn require_numel(tensor: &Tensor, expected: usize, name: &str) -> Result<(), VsmError> {
    if tensor.numel() != expected {
        return Err(VsmError::ShapeMismatch(name.to_string()));
    }
    Ok(())
}

fn add_bias(logits: &mut [f32], bias: &[f32], channels: usize) {
    for row in logits.chunks_exact_mut(channels) {
        for (value, bias) in row.iter_mut().zip(bias) {
            *value += *bias;
        }
    }
}

fn forward_chunk(model: &VsmModel, input_bytes: &[u8]) -> Result<ForwardOutput, VsmError> {
    let config = model.config;
    let length = input_bytes.len();
    let channels = config.d_model;
    let kernel = config.kernel_size;
    let expanded = config.expanded_dim();

    let embedding = model.tensor("embedding.weight")?;
    require_numel(embedding, config.vocab_size * channels, "embedding.weight")?;
    let embedding_values = embedding.as_i8()?;
    let embedding_scales = scales(model, "embedding.scales", config.vocab_size)?;
    let mut x = Vec::with_capacity(length * channels);
    for byte in input_bytes {
        let byte = *byte as usize;
        if byte >= config.vocab_size {
            return Err(VsmError::ShapeMismatch("input byte >= vocab_size".into()));
        }
        let base = byte * channels;
        let scale = embedding_scales[byte];
        x.extend(
            embedding_values[base..base + channels]
                .iter()
                .map(|value| *value as f32 * scale),
        );
    }

    let stem = model.tensor("stem.weight")?;
    require_numel(stem, channels * kernel * channels, "stem.weight")?;
    let stem_scales = scales(model, "stem.scales", channels)?;
    let stem_pad = kernel / 2;
    let stem_input = pad_time_major(&x, length, channels, stem_pad);
    x = conv1d_dense_w8a32(
        &stem_input,
        stem.as_i8()?,
        stem_scales,
        channels,
        channels,
        kernel,
        1,
        length,
    );
    silu_inplace(&mut x);

    for index in 0..config.n_blocks {
        let prefix = format!("blocks.{index}");
        let dilation = model.dilations()[index];
        let residual = x.clone();

        let dw_name = format!("{prefix}.dw.weight");
        let dw = model.tensor(&dw_name)?;
        require_numel(dw, channels * kernel, &dw_name)?;
        let dw_scales = scales(model, &format!("{prefix}.dw.scales"), channels)?;
        let dw_pad = (kernel / 2) * dilation;
        let dw_input = pad_time_major(&x, length, channels, dw_pad);
        let mut h = conv1d_dw_w8a32(
            &dw_input,
            dw.as_i8()?,
            dw_scales,
            channels,
            kernel,
            dilation,
            length,
        );

        let norm = model.tensor(&format!("{prefix}.norm.weight"))?;
        require_numel(norm, channels, &format!("{prefix}.norm.weight"))?;
        rms_norm_inplace(&mut h, norm.as_f32()?, channels, 1e-5);

        let pw1_name = format!("{prefix}.pw1.weight");
        let pw1 = model.tensor(&pw1_name)?;
        require_numel(pw1, expanded * channels, &pw1_name)?;
        let pw1_scales = scales(model, &format!("{prefix}.pw1.scales"), expanded)?;
        h = conv1d_pw_w8a32(&h, pw1.as_i8()?, pw1_scales, expanded, channels, length);
        gelu_tanh_inplace(&mut h);

        let pw2_name = format!("{prefix}.pw2.weight");
        let pw2 = model.tensor(&pw2_name)?;
        require_numel(pw2, channels * expanded, &pw2_name)?;
        let pw2_scales = scales(model, &format!("{prefix}.pw2.scales"), channels)?;
        h = conv1d_pw_w8a32(&h, pw2.as_i8()?, pw2_scales, channels, expanded, length);
        add_inplace(&mut h, &residual);
        x = h;
    }

    let error_tensor = model.tensor("head_err.weight")?;
    require_numel(
        error_tensor,
        config.n_error_classes * channels,
        "head_err.weight",
    )?;
    let error_scales = scales(model, "head_err.scales", config.n_error_classes)?;
    let mut error_logits = conv1d_pw_w8a32(
        &x,
        error_tensor.as_i8()?,
        error_scales,
        config.n_error_classes,
        channels,
        length,
    );
    let error_bias = model.tensor("head_err.bias")?.as_f32()?;
    if error_bias.len() != config.n_error_classes {
        return Err(VsmError::ShapeMismatch("head_err.bias".into()));
    }
    add_bias(&mut error_logits, error_bias, config.n_error_classes);

    let byte_tensor = model.tensor("head_byte.weight")?;
    require_numel(
        byte_tensor,
        config.vocab_size * channels,
        "head_byte.weight",
    )?;
    let byte_scales = scales(model, "head_byte.scales", config.vocab_size)?;
    let mut byte_logits = conv1d_pw_w8a32(
        &x,
        byte_tensor.as_i8()?,
        byte_scales,
        config.vocab_size,
        channels,
        length,
    );
    let byte_bias = model.tensor("head_byte.bias")?.as_f32()?;
    if byte_bias.len() != config.vocab_size {
        return Err(VsmError::ShapeMismatch("head_byte.bias".into()));
    }
    add_bias(&mut byte_logits, byte_bias, config.vocab_size);

    Ok(ForwardOutput {
        length,
        error_logits,
        byte_logits,
    })
}

pub fn forward(model: &VsmModel, input_bytes: &[u8]) -> Result<ForwardOutput, VsmError> {
    let config = model.config;
    let length = input_bytes.len();
    if length == 0 {
        return Ok(ForwardOutput {
            length: 0,
            error_logits: Vec::new(),
            byte_logits: Vec::new(),
        });
    }
    if length <= config.max_seq_len {
        return forward_chunk(model, input_bytes);
    }

    let radius = config.rf_radius_bytes;
    let stride = config.max_seq_len.saturating_sub(2 * radius).clamp(1, 128);
    let mut error_logits = vec![0.0f32; length * config.n_error_classes];
    let mut byte_logits = vec![0.0f32; length * config.vocab_size];
    let mut global_start = 0usize;
    while global_start < length {
        let chunk_start = global_start.saturating_sub(radius);
        let chunk_end = (global_start + stride + radius).min(length);
        let chunk = forward_chunk(model, &input_bytes[chunk_start..chunk_end])?;
        let keep_start = global_start - chunk_start;
        let keep_length = (length - global_start).min(stride);
        let error_offset = keep_start * config.n_error_classes;
        let byte_offset = keep_start * config.vocab_size;
        let target_error = global_start * config.n_error_classes;
        let target_byte = global_start * config.vocab_size;
        error_logits[target_error..target_error + keep_length * config.n_error_classes]
            .copy_from_slice(
                &chunk.error_logits
                    [error_offset..error_offset + keep_length * config.n_error_classes],
            );
        byte_logits[target_byte..target_byte + keep_length * config.vocab_size].copy_from_slice(
            &chunk.byte_logits[byte_offset..byte_offset + keep_length * config.vocab_size],
        );
        global_start += stride;
    }

    Ok(ForwardOutput {
        length,
        error_logits,
        byte_logits,
    })
}
