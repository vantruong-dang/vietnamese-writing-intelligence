//! WASM binding tối giản cho VSM runtime.
//!
//! Dùng cho benchmark V8 và để `vwi-wasm` nối cascade sau này.

use wasm_bindgen::prelude::*;

use vsm_runtime::{forward, VsmModel};

#[wasm_bindgen]
pub struct VsmScorer {
    model: VsmModel,
}

#[wasm_bindgen]
impl VsmScorer {
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> Result<VsmScorer, JsValue> {
        VsmModel::from_bytes(bytes)
            .map(|model| Self { model })
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    /// Trả về argmax nhãn lỗi cho từng byte (0..3).
    pub fn error_argmax(&self, input: &[u8]) -> Result<Vec<u8>, JsValue> {
        forward(&self.model, input)
            .map(|output| output.error_argmax())
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    /// Trả về argmax byte đúng cho từng byte (0..255).
    pub fn byte_argmax(&self, input: &[u8]) -> Result<Vec<u8>, JsValue> {
        forward(&self.model, input)
            .map(|output| output.byte_argmax())
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    /// Trả về logits lỗi thô (length * 4) để đo chi phí forward đầy đủ.
    pub fn error_logits(&self, input: &[u8]) -> Result<Vec<f32>, JsValue> {
        forward(&self.model, input)
            .map(|output| output.error_logits)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }
}
