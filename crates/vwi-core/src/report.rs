use crate::types::{Finding, Severity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Báo cáo kiểm toán văn bản toàn diện kèm điểm số chất lượng (0.0 - 100.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub text_len_chars: usize,
    pub word_count: usize,
    pub score: f32, // Thang điểm 0.0 - 100.0
    pub findings: Vec<Finding>,
    pub finding_counts: HashMap<String, usize>, // Severity -> Count
    pub category_counts: HashMap<String, usize>,
    pub execution_time_ms: f64,
}

impl AuditReport {
    /// Thuật toán tính điểm chất lượng văn bản dựa trên mật độ lỗi phi tuyến (Non-linear Error Density)
    pub fn calculate(
        text: &str,
        findings: Vec<Finding>,
        word_count: usize,
        execution_time_ms: f64,
    ) -> Self {
        let mut finding_counts = HashMap::new();
        let mut category_counts = HashMap::new();

        let mut total_penalty = 0.0f32;

        for f in &findings {
            let sev = Severity::parse(&f.severity);
            *finding_counts.entry(sev.as_str().to_string()).or_insert(0) += 1;
            *category_counts.entry(f.category.clone()).or_insert(0) += 1;

            // Điểm phạt cơ sở theo mức độ nghiêm trọng
            let base_penalty = match sev {
                Severity::Critical => 12.0,
                Severity::Major => 8.0,
                Severity::Minor => 3.0,
                Severity::Suggestion => 1.0,
                Severity::Advisory => 0.0, // Chẩn đoán nhịp điệu / tư vấn: KHÔNG TRỪ ĐIỂM
            };
            total_penalty += base_penalty;
        }

        // Chuẩn hóa theo độ dài văn bản (mật độ lỗi trên mỗi 200 từ)
        let effective_words = (word_count as f32).max(50.0);
        let density_factor = 200.0 / effective_words;
        let scaled_penalty = total_penalty * density_factor;

        // Điểm số tối thiểu là 0.0, tối đa là 100.0
        let score = (100.0 - scaled_penalty).clamp(0.0, 100.0);

        AuditReport {
            text_len_chars: text.chars().count(),
            word_count,
            score: (score * 10.0).round() / 10.0, // Làm tròn 1 chữ số thập phân
            findings,
            finding_counts,
            category_counts,
            execution_time_ms,
        }
    }

    /// Định dạng xuất JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Định dạng xuất Markdown
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!(
            "# BÁO CÁO KIỂM TOÁN VĂN BẢN (Điểm: {}/100)\n\n",
            self.score
        ));
        md.push_str(&format!(
            "- **Số từ**: {}\n- **Số lỗi phát hiện**: {}\n- **Thời gian xử lý**: {:.2} ms\n\n",
            self.word_count,
            self.findings.len(),
            self.execution_time_ms
        ));

        if self.findings.is_empty() {
            md.push_str("✅ **Văn bản đạt chuẩn thuần Việt! Không phát hiện lỗi cấu trúc.**\n");
            return md;
        }

        md.push_str("| Vị trí | Mức độ | Nhóm | Đoạn văn vi phạm | Chi tiết lỗi | Gợi ý khắc phục |\n");
        md.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
        for f in &self.findings {
            let sev = Severity::parse(&f.severity);
            let sev_badge = match sev {
                Severity::Critical => "🚨 Critical",
                Severity::Major => "🔴 Major",
                Severity::Minor => "🟡 Minor",
                Severity::Suggestion => "💡 Suggestion",
                Severity::Advisory => "ℹ️ Advisory",
            };
            let sugg = f.suggestion.as_deref().unwrap_or("—");
            md.push_str(&format!(
                "| {}..{} | {} | {} | `{}` | {} | {} |\n",
                f.start, f.end, sev_badge, f.category, f.matched_span, f.rationale, sugg
            ));
        }
        md
    }
}
