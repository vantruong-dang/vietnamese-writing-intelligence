use wasm_bindgen::prelude::*;
use vwi_core::auditor::Auditor;

#[wasm_bindgen]
pub struct WasmAuditor {
    inner: Auditor,
}

#[wasm_bindgen]
impl WasmAuditor {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: Auditor::default(),
        }
    }

    /// Kiểm toán văn bản và trả về JSON Report (Điểm số, danh sách lỗi, tọa độ span, gợi ý sửa)
    #[wasm_bindgen]
    pub fn audit(&self, text: &str) -> String {
        let report = self.inner.audit_document(text);
        report.to_json()
    }

    /// Tách từ tiếng Việt kèm nhãn từ loại UPOS
    #[wasm_bindgen]
    pub fn tokenize(&self, text: &str) -> String {
        let tokens = vwi_core::segmentation::viterbi_segmenter::tokenize_viterbi(text);
        serde_json::to_string(&tokens).unwrap_or_else(|_| "[]".to_string())
    }
}
