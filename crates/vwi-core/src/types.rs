use serde::{Deserialize, Serialize};

/// Tọa độ và thông tin của một token (từ đơn hoặc từ ghép tiếng Việt).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenSpan {
    pub text: String,
    /// Offset ký tự bắt đầu (tính theo Unicode scalar value / code points)
    pub start: usize,
    /// Offset ký tự kết thúc (tính theo Unicode scalar value / code points)
    pub end: usize,
    pub is_compound: bool,
    /// Nhãn từ loại theo ngữ pháp chức năng (N, V, E, C, Nc, Kin, Asp, R, P, M, CH...)
    pub pos: Option<String>,
}

impl TokenSpan {
    pub fn new(text: impl Into<String>, start: usize, end: usize, is_compound: bool) -> Self {
        Self {
            text: text.into(),
            start,
            end,
            is_compound,
            pos: None,
        }
    }

    pub fn with_pos(
        text: impl Into<String>,
        start: usize,
        end: usize,
        is_compound: bool,
        pos: impl Into<String>,
    ) -> Self {
        Self {
            text: text.into(),
            start,
            end,
            is_compound,
            pos: Some(pos.into()),
        }
    }
}

/// Phát hiện vi phạm ngữ pháp / phong cách ngôn ngữ học.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub pattern_id: String,
    pub category: String,
    pub severity: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f64,
    pub rationale: String,
    pub source: String,
    pub matched_span: String,
}

impl Finding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pattern_id: impl Into<String>,
        category: impl Into<String>,
        severity: impl Into<String>,
        start: usize,
        end: usize,
        confidence: f64,
        rationale: impl Into<String>,
        source: impl Into<String>,
        matched_span: impl Into<String>,
    ) -> Self {
        Self {
            pattern_id: pattern_id.into(),
            category: category.into(),
            severity: severity.into(),
            start,
            end,
            confidence,
            rationale: rationale.into(),
            source: source.into(),
            matched_span: matched_span.into(),
        }
    }
}
