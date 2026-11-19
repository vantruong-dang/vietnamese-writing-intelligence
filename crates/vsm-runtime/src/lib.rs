//! VSM runtime: đọc `.vsm`, chạy Conv1D W8A32 và forward pass.

pub mod format;
pub mod kernel;
pub mod model;

pub use format::{DType, Tensor, TensorData, VsmConfig, VsmError, VsmModel};
pub use model::{forward, ForwardOutput};
