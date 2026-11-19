//! Đọc định dạng `.vsm` v1 (header + tensor descriptor + INT8/F32 payload).

use std::collections::HashMap;
use std::fmt;

pub const MAGIC: &[u8; 4] = b"VSM1";
pub const VERSION: u16 = 1;
pub const HEADER_SIZE: usize = 48;
pub const FLAG_INT8: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DType {
    F32,
    I8,
}

impl DType {
    fn from_id(value: u8) -> Result<Self, VsmError> {
        match value {
            0 => Ok(Self::F32),
            1 => Ok(Self::I8),
            other => Err(VsmError::Malformed(format!("dtype không hỗ trợ: {other}"))),
        }
    }

    pub fn size(self) -> usize {
        match self {
            Self::F32 => 4,
            Self::I8 => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TensorData {
    F32(Vec<f32>),
    I8(Vec<i8>),
}

#[derive(Debug, Clone)]
pub struct Tensor {
    pub name: String,
    pub dtype: DType,
    pub shape: Vec<usize>,
    pub scale: f32,
    pub zero_point: i32,
    pub data: TensorData,
}

impl Tensor {
    pub fn numel(&self) -> usize {
        self.shape.iter().product()
    }

    pub fn as_f32(&self) -> Result<&[f32], VsmError> {
        match &self.data {
            TensorData::F32(values) => Ok(values),
            TensorData::I8(_) => Err(VsmError::WrongDType(self.name.clone())),
        }
    }

    pub fn as_i8(&self) -> Result<&[i8], VsmError> {
        match &self.data {
            TensorData::I8(values) => Ok(values),
            TensorData::F32(_) => Err(VsmError::WrongDType(self.name.clone())),
        }
    }

    pub fn f32_at(&self, index: usize) -> Result<f32, VsmError> {
        self.as_f32()?
            .get(index)
            .copied()
            .ok_or_else(|| VsmError::ShapeMismatch(format!("{} index {index}", self.name)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VsmConfig {
    pub vocab_size: usize,
    pub d_model: usize,
    pub n_blocks: usize,
    pub kernel_size: usize,
    pub expand: usize,
    pub n_error_classes: usize,
    pub max_seq_len: usize,
    pub rf_radius_bytes: usize,
    pub int8: bool,
}

impl VsmConfig {
    pub fn expanded_dim(&self) -> usize {
        self.d_model * self.expand
    }

    fn validate(&self) -> Result<(), VsmError> {
        if self.vocab_size != 256 {
            return Err(VsmError::Malformed("vocab_size phải bằng 256".into()));
        }
        if self.d_model == 0 || self.d_model > 4096 {
            return Err(VsmError::Malformed("d_model ngoài phạm vi".into()));
        }
        if self.n_blocks == 0 || self.n_blocks > 32 {
            return Err(VsmError::Malformed("n_blocks ngoài phạm vi".into()));
        }
        if self.kernel_size == 0 || self.kernel_size.is_multiple_of(2) || self.kernel_size > 15 {
            return Err(VsmError::Malformed(
                "kernel_size phải là số lẻ 1..=15".into(),
            ));
        }
        if self.expand == 0 || self.expand > 8 {
            return Err(VsmError::Malformed("expand ngoài phạm vi".into()));
        }
        if self.n_error_classes == 0 || self.n_error_classes > 16 {
            return Err(VsmError::Malformed("n_error_classes ngoài phạm vi".into()));
        }
        if self.max_seq_len == 0 {
            return Err(VsmError::Malformed("max_seq_len phải > 0".into()));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum VsmError {
    TooShort,
    BadMagic([u8; 4]),
    UnsupportedVersion(u16),
    Malformed(String),
    Utf8,
    MissingTensor(String),
    WrongDType(String),
    ShapeMismatch(String),
}

impl fmt::Display for VsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => write!(f, "file .vsm quá ngắn"),
            Self::BadMagic(magic) => write!(f, "sai magic: {magic:?}"),
            Self::UnsupportedVersion(version) => write!(f, "không hỗ trợ version {version}"),
            Self::Malformed(message) => write!(f, "file .vsm hỏng: {message}"),
            Self::Utf8 => write!(f, "tên tensor không phải UTF-8"),
            Self::MissingTensor(name) => write!(f, "thiếu tensor {name}"),
            Self::WrongDType(name) => write!(f, "sai dtype cho tensor {name}"),
            Self::ShapeMismatch(name) => write!(f, "shape không khớp cho tensor {name}"),
        }
    }
}

impl std::error::Error for VsmError {}

struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], VsmError> {
        let end = self.cursor.checked_add(count).ok_or(VsmError::TooShort)?;
        let slice = self.bytes.get(self.cursor..end).ok_or(VsmError::TooShort)?;
        self.cursor = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, VsmError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, VsmError> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn u32(&mut self) -> Result<u32, VsmError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn u64(&mut self) -> Result<u64, VsmError> {
        let bytes = self.take(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn i32(&mut self) -> Result<i32, VsmError> {
        Ok(self.u32()? as i32)
    }

    fn f32(&mut self) -> Result<f32, VsmError> {
        Ok(f32::from_bits(self.u32()?))
    }
}

#[derive(Debug, Clone)]
struct Descriptor {
    name: String,
    dtype: DType,
    shape: Vec<usize>,
    scale: f32,
    zero_point: i32,
    data_offset: usize,
    data_len: usize,
}

#[derive(Debug, Clone)]
pub struct VsmModel {
    pub config: VsmConfig,
    tensors: HashMap<String, Tensor>,
    weights_f32: HashMap<String, Vec<f32>>,
    packed_pw: HashMap<String, Vec<f32>>,
    dilations: Vec<usize>,
}

impl VsmModel {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, VsmError> {
        if bytes.len() < HEADER_SIZE {
            return Err(VsmError::TooShort);
        }
        let mut reader = Reader::new(bytes);
        let mut magic = [0u8; 4];
        magic.copy_from_slice(reader.take(4)?);
        if &magic != MAGIC {
            return Err(VsmError::BadMagic(magic));
        }
        let version = reader.u16()?;
        if version != VERSION {
            return Err(VsmError::UnsupportedVersion(version));
        }
        let flags = reader.u16()?;
        let config = VsmConfig {
            vocab_size: reader.u32()? as usize,
            d_model: reader.u32()? as usize,
            n_blocks: reader.u32()? as usize,
            kernel_size: reader.u32()? as usize,
            expand: reader.u32()? as usize,
            n_error_classes: reader.u32()? as usize,
            max_seq_len: reader.u32()? as usize,
            rf_radius_bytes: reader.u32()? as usize,
            int8: flags & FLAG_INT8 != 0,
        };
        config.validate()?;
        let tensor_count = reader.u32()? as usize;
        let _reserved = reader.u32()?;
        if tensor_count == 0 || tensor_count > 4096 {
            return Err(VsmError::Malformed("tensor_count ngoài phạm vi".into()));
        }

        let mut descriptors = Vec::with_capacity(tensor_count);
        for _ in 0..tensor_count {
            let name_len = reader.u16()? as usize;
            let name_bytes = reader.take(name_len)?;
            let name = std::str::from_utf8(name_bytes)
                .map_err(|_| VsmError::Utf8)?
                .to_string();
            let dtype = DType::from_id(reader.u8()?)?;
            let ndim = reader.u8()? as usize;
            if ndim == 0 || ndim > 4 {
                return Err(VsmError::Malformed(format!("{name}: ndim {ndim}")));
            }
            let mut shape = Vec::with_capacity(ndim);
            for _ in 0..ndim {
                let dimension = reader.u32()? as usize;
                if dimension == 0 {
                    return Err(VsmError::Malformed(format!("{name}: shape chứa 0")));
                }
                shape.push(dimension);
            }
            let scale = reader.f32()?;
            let zero_point = reader.i32()?;
            let _unused = reader.u32()?;
            let data_offset = reader.u64()? as usize;
            let data_len = reader.u64()? as usize;
            descriptors.push(Descriptor {
                name,
                dtype,
                shape,
                scale,
                zero_point,
                data_offset,
                data_len,
            });
        }

        let data_start = align16(reader.cursor);
        let mut tensors = HashMap::with_capacity(descriptors.len());
        for descriptor in descriptors {
            let start = data_start
                .checked_add(descriptor.data_offset)
                .ok_or_else(|| VsmError::Malformed(format!("{}: offset tràn", descriptor.name)))?;
            let end = start
                .checked_add(descriptor.data_len)
                .ok_or_else(|| VsmError::Malformed(format!("{}: độ dài tràn", descriptor.name)))?;
            let raw = bytes.get(start..end).ok_or_else(|| {
                VsmError::Malformed(format!("{}: dữ liệu bị cắt", descriptor.name))
            })?;
            let numel: usize = descriptor.shape.iter().product();
            let expected_len = numel
                .checked_mul(descriptor.dtype.size())
                .ok_or_else(|| VsmError::Malformed(format!("{}: numel tràn", descriptor.name)))?;
            if expected_len != descriptor.data_len {
                return Err(VsmError::ShapeMismatch(descriptor.name));
            }
            let data = match descriptor.dtype {
                DType::F32 => TensorData::F32(
                    raw.as_chunks::<4>()
                        .0
                        .iter()
                        .map(|chunk| f32::from_le_bytes(*chunk))
                        .collect(),
                ),
                DType::I8 => TensorData::I8(raw.iter().map(|value| *value as i8).collect()),
            };
            let tensor = Tensor {
                name: descriptor.name.clone(),
                dtype: descriptor.dtype,
                shape: descriptor.shape,
                scale: descriptor.scale,
                zero_point: descriptor.zero_point,
                data,
            };
            tensors.insert(descriptor.name, tensor);
        }

        let dilations = match tensors.get("model.dilations") {
            Some(tensor) => tensor
                .as_i8()?
                .iter()
                .map(|value| {
                    let value = *value as i32;
                    if value <= 0 {
                        Err(VsmError::Malformed("dilation phải > 0".into()))
                    } else {
                        Ok(value as usize)
                    }
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => {
                if config.n_blocks == 6 {
                    vec![1, 2, 4, 8, 4, 2]
                } else {
                    vec![1; config.n_blocks]
                }
            }
        };
        if dilations.len() != config.n_blocks {
            return Err(VsmError::Malformed(
                "model.dilations không khớp n_blocks".into(),
            ));
        }

        let mut weights_f32 = HashMap::new();
        for (name, tensor) in &tensors {
            if !name.ends_with(".weight") {
                continue;
            }
            if tensor.dtype == DType::F32 {
                weights_f32.insert(name.clone(), tensor.as_f32()?.to_vec());
                continue;
            }
            if tensor.dtype != DType::I8 {
                continue;
            }
            let scales_name = format!("{}.scales", &name[..name.len() - ".weight".len()]);
            let Some(scale_tensor) = tensors.get(&scales_name) else {
                continue;
            };
            let scales = scale_tensor.as_f32()?;
            let channels = tensor.shape[0];
            if scales.len() != channels {
                return Err(VsmError::ShapeMismatch(scales_name));
            }
            let inner = tensor.numel() / channels;
            let values = tensor.as_i8()?;
            let mut dequantized = Vec::with_capacity(tensor.numel());
            for channel in 0..channels {
                let scale = scales[channel];
                for offset in 0..inner {
                    dequantized.push(values[channel * inner + offset] as f32 * scale);
                }
            }
            weights_f32.insert(name.clone(), dequantized);
        }

        let mut packed_pw = HashMap::new();
        for (name, values) in &weights_f32 {
            if !(name.ends_with(".pw1.weight")
                || name.ends_with(".pw2.weight")
                || name == "head_err.weight"
                || name == "head_byte.weight")
            {
                continue;
            }
            let Some(tensor) = tensors.get(name) else {
                continue;
            };
            let out_channels = tensor.shape[0];
            let in_channels = values.len() / out_channels;
            if let Some(packed) = pack_pw_weight(values, out_channels, in_channels) {
                packed_pw.insert(name.clone(), packed);
            }
        }

        Ok(Self {
            config,
            tensors,
            weights_f32,
            packed_pw,
            dilations,
        })
    }

    pub fn packed_weight(&self, name: &str) -> Option<&[f32]> {
        self.packed_pw.get(name).map(Vec::as_slice)
    }

    pub fn weight_f32(&self, name: &str) -> Result<&[f32], VsmError> {
        self.weights_f32
            .get(name)
            .map(Vec::as_slice)
            .ok_or_else(|| VsmError::MissingTensor(format!("{name} (dequantized)")))
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        Ok(Self::from_bytes(&bytes)?)
    }

    pub fn tensor(&self, name: &str) -> Result<&Tensor, VsmError> {
        self.tensors
            .get(name)
            .ok_or_else(|| VsmError::MissingTensor(name.to_string()))
    }

    pub fn dilations(&self) -> &[usize] {
        &self.dilations
    }

    pub fn scalar_f32(&self, name: &str) -> Result<f32, VsmError> {
        let tensor = self.tensor(name)?;
        if tensor.numel() != 1 {
            return Err(VsmError::ShapeMismatch(name.to_string()));
        }
        tensor.f32_at(0)
    }
}

fn pack_pw_weight(values: &[f32], out_channels: usize, in_channels: usize) -> Option<Vec<f32>> {
    if !out_channels.is_multiple_of(4) || values.len() != out_channels * in_channels {
        return None;
    }
    let mut packed = vec![0.0f32; values.len()];
    for block in 0..out_channels / 4 {
        for input_channel in 0..in_channels {
            for lane in 0..4 {
                let output_channel = block * 4 + lane;
                packed[(block * in_channels + input_channel) * 4 + lane] =
                    values[output_channel * in_channels + input_channel];
            }
        }
    }
    Some(packed)
}

fn align16(value: usize) -> usize {
    value.div_ceil(16) * 16
}
