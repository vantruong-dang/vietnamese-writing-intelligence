use crate::report::AuditReport;
use crate::rules::*;
use crate::types::{Finding, Register, Severity};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Cấu hình kiểm toán ngôn ngữ học (Auditor Configuration).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditorConfig {
    /// Danh sách các mã luật bị vô hiệu hóa (disabled).
    #[serde(default)]
    pub disabled_rules: HashSet<String>,
    /// Nhóm thể loại được kích hoạt (nếu None thì cho phép toàn bộ).
    #[serde(default)]
    pub enabled_categories: Option<HashSet<String>>,
    /// Ghi đè mức độ nghiêm trọng (pattern_id -> new severity, e.g. "warning", "critical", "minor", "suggestion").
    #[serde(default)]
    pub severity_overrides: HashMap<String, String>,
    /// Ngưỡng mức độ nghiêm trọng tối thiểu (nếu Some, bỏ qua các finding dưới ngưỡng này).
    #[serde(default)]
    pub min_severity: Option<Severity>,
    /// Ngữ vực của văn bản (Text Register). Mặc định là General.
    #[serde(default)]
    pub register: Register,
}

impl AuditorConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_register(&mut self, register: Register) -> &mut Self {
        self.register = register;
        self
    }

    pub fn disable_rule(&mut self, rule_id: impl Into<String>) -> &mut Self {
        self.disabled_rules.insert(rule_id.into());
        self
    }

    pub fn enable_rule(&mut self, rule_id: &str) -> &mut Self {
        self.disabled_rules.remove(rule_id);
        self
    }

    pub fn set_enabled_categories(&mut self, categories: Vec<String>) -> &mut Self {
        self.enabled_categories = Some(categories.into_iter().collect());
        self
    }

    pub fn override_severity(
        &mut self,
        rule_id: impl Into<String>,
        severity: impl Into<String>,
    ) -> &mut Self {
        self.severity_overrides
            .insert(rule_id.into(), severity.into());
        self
    }

    pub fn set_min_severity(&mut self, min: Severity) -> &mut Self {
        self.min_severity = Some(min);
        self
    }

    #[inline]
    pub fn is_rule_enabled(&self, rule_id: &str, category: &str) -> bool {
        if self.disabled_rules.contains(rule_id) {
            return false;
        }
        if let Some(ref cats) = self.enabled_categories {
            if !cats.contains(category) {
                return false;
            }
        }
        true
    }
}

/// Động cơ kiểm toán ngôn ngữ học cấp cao (High-Level Auditor).
/// Hỗ trợ cấu hình bộ lọc luật, nhóm thể loại, ghi đè mức độ nghiêm trọng (Voice Profile),
/// và xử lý tài liệu lớn song song bằng Rayon đa luồng.
#[derive(Debug, Clone, Default)]
pub struct Auditor {
    pub config: AuditorConfig,
}

impl Auditor {
    pub fn new() -> Self {
        Self {
            config: AuditorConfig::default(),
        }
    }

    pub fn with_config(config: AuditorConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &AuditorConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut AuditorConfig {
        &mut self.config
    }

    /// Kiểm toán một đoạn văn bản đơn lẻ, tự động áp dụng cấu hình kích hoạt/vô hiệu hóa và ghi đè severity.
    pub fn audit(&self, text: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        // 1. Nhóm bị động nhân tạo & thụ hưởng dư thừa
        if self.config.is_rule_enabled(ARTIFICIAL_PASSIVE_RULE_ID, "syntax") {
            findings.extend(evaluate_artificial_passive_fast(text));
        }
        if self.config.is_rule_enabled(REDUNDANT_BENEFICIARY_RULE_ID, "syntax") {
            findings.extend(evaluate_redundant_beneficiary_passive(text));
        }

        // 2. Nhóm cấu trúc Khung đề - Chủ đề
        if self.config.is_rule_enabled(CRIPPLED_TOPIC_RULE_ID, "syntax") {
            findings.extend(evaluate_crippled_prepositional_topic(text));
        }
        if self.config.is_rule_enabled(PREP_CONJ_RULE_ID, "syntax") {
            findings.extend(evaluate_preposition_conjunction_conflict(text));
        }

        // 3. Nhóm trùng ngữ logic
        if self.config.is_rule_enabled(LOGICAL_PLEONASM_RULE_ID, "pleonasm") {
            findings.extend(evaluate_logical_pleonasm(text));
        }

        // 4. Nhóm dịch ngữ & danh từ hóa & chủ ngữ giả
        if self.config.is_rule_enabled(NOMINALIZATION_HEAVY_RULE_ID, "translationese") {
            findings.extend(evaluate_nominalization_heavy(text));
        }
        if self.config.is_rule_enabled(TEMPORAL_FRAMING_RULE_ID, "translationese") {
            findings.extend(evaluate_temporal_framing(text));
        }
        if self.config.is_rule_enabled(TRANSLATIONESE_SOLEIL_RULE_ID, "translationese") {
            findings.extend(evaluate_translationese_soleil(text));
        }
        if self.config.is_rule_enabled(DUMMY_IT_RULE_ID, "translationese")
            || self.config.is_rule_enabled(ADJECTIVAL_SUFFIX_FAKE_RULE_ID, "translationese")
        {
            let dummy_findings = evaluate_dummy_it_translationese(text);
            findings.extend(
                dummy_findings
                    .into_iter()
                    .filter(|f| self.config.is_rule_enabled(&f.pattern_id, &f.category)),
            );
        }

        // 5. Nhóm liên từ nhượng bộ, xung đột kép & thời thể
        if self.config.is_rule_enabled(FACTUAL_CONCESSIVE_RULE_ID, "syntax") {
            findings.extend(evaluate_factual_concessive_misuse(text));
        }
        if self.config.is_rule_enabled(HYPOTHETICAL_CONCESSIVE_RULE_ID, "syntax") {
            findings.extend(evaluate_hypothetical_concessive_misuse(text));
        }
        if self.config.is_rule_enabled(CONCESSIVE_INVERSION_RULE_ID, "syntax") {
            findings.extend(evaluate_concessive_inversion(text));
        }
        if self.config.is_rule_enabled(ASPECT_MECHANICS_RULE_ID, "aspect") {
            findings.extend(evaluate_aspect_mechanics(text));
        }

        // 6. Nhóm cú pháp đối ngẫu & trần thuật
        if self.config.is_rule_enabled(SYNTAX_REPORTING_RULE_ID, "syntax") {
            findings.extend(evaluate_syntax_reporting(text));
        }
        if self.config.is_rule_enabled(SYNTAX_PARALLEL_RULE_ID, "syntax") {
            findings.extend(evaluate_syntax_parallel(text));
        }
        if self.config.is_rule_enabled(SYNTAX_CLICHE_RULE_ID, "cliche") {
            findings.extend(evaluate_syntax_cliche(text));
        }

        // 7. Nhóm từ vựng lai căng
        if self.config.is_rule_enabled(SUPERLATIVE_CHI_IT_RULE_ID, "lexical") {
            findings.extend(evaluate_superlative_misuse(text));
        }
        if self.config.is_rule_enabled(SUYT_NUA_THI_RULE_ID, "lexical") {
            findings.extend(evaluate_suyt_nua_thi(text));
        }

        // 8. Nhóm công thức & sáo ngữ AI (Humanizer)
        if self.config.is_rule_enabled(AI_OPENING_RULE_ID, "humanizer") {
            findings.extend(evaluate_ai_opening_cliche(text));
        }
        if self.config.is_rule_enabled(AI_SUMMARY_RULE_ID, "humanizer") {
            findings.extend(evaluate_ai_summary_tail(text));
        }
        if self.config.is_rule_enabled(FORMULAIC_METAPHOR_RULE_ID, "humanizer") {
            findings.extend(evaluate_formulaic_metaphor(text));
        }
        if self.config.is_rule_enabled(AI_DISCOVERED_CLICHE_RULE_ID, "humanizer") {
            findings.extend(evaluate_discovered_ai_cliches(text));
        }

        // 9. Nhóm sáo ngữ hành chính & đệm từ rỗng (Aho-Corasick)
        if self.config.is_rule_enabled(BUREAUCRATIC_VERB_PADDING_RULE_ID, "bureaucracy")
            || self.config.is_rule_enabled(REDUNDANT_CONNECTORS_RULE_ID, "bureaucracy")
            || self.config.is_rule_enabled(VAGUE_INTENSIFIER_RULE_ID, "bureaucracy")
        {
            let bureau_findings = evaluate_bureaucratic_rules(text);
            findings.extend(
                bureau_findings
                    .into_iter()
                    .filter(|f| self.config.is_rule_enabled(&f.pattern_id, &f.category)),
            );
        }

        // 10. Cảm biến ngữ âm & tiết tấu định lượng (Prosodic Advisory Sensor)
        if self.config.is_rule_enabled(PROSODIC_EXCESSIVE_TRAC_RULE_ID, "prosody") {
            findings.extend(evaluate_prosody(text, self.config.register));
        }

        // Áp dụng ghi đè severity nếu có
        if !self.config.severity_overrides.is_empty() {
            for f in &mut findings {
                if let Some(new_sev) = self.config.severity_overrides.get(&f.pattern_id) {
                    f.severity = new_sev.clone();
                }
            }
        }

        // Lọc theo min_severity nếu có
        if let Some(min_sev) = self.config.min_severity {
            findings.retain(|f| Severity::parse(&f.severity) >= min_sev);
        }

        // Sắp xếp các phát hiện theo thứ tự xuất hiện trong văn bản
        findings.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.end.cmp(&b.end)));
        findings
    }

    /// Kiểm toán danh sách nhiều tài liệu/đoạn văn bản song song trên đa nhân CPU thông qua Rayon.
    pub fn audit_batch(&self, texts: &[&str]) -> Vec<Vec<Finding>> {
        texts.par_iter().map(|text| self.audit(text)).collect()
    }

    /// Kiểm toán toàn diện văn bản, tự động tách từ bằng Viterbi để đếm word count và sinh AuditReport hoàn chỉnh.
    pub fn audit_document(&self, text: &str) -> AuditReport {
        let start_time = std::time::Instant::now();
        let tokens = crate::segmentation::viterbi_segmenter::tokenize_viterbi(text);
        let word_count = tokens.len();
        let findings = self.audit(text);
        let elapsed = start_time.elapsed().as_secs_f64() * 1000.0;
        AuditReport::calculate(text, findings, word_count, elapsed)
    }

    /// Kiểm toán toàn diện danh sách nhiều tài liệu song song trên đa nhân CPU thông qua Rayon.
    pub fn audit_document_batch(&self, texts: &[&str]) -> Vec<AuditReport> {
        texts.par_iter().map(|text| self.audit_document(text)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auditor_default() {
        let auditor = Auditor::new();
        let text = "Trong tình hình hiện nay đòi hỏi giải pháp tối ưu nhất được triển khai bởi nhóm.";
        let findings = auditor.audit(text);

        assert!(findings.len() >= 3);
        let ids: Vec<&str> = findings.iter().map(|f| f.pattern_id.as_str()).collect();
        assert!(ids.contains(&"CRIPPLED_PREPOSITIONAL_TOPIC"));
        assert!(ids.contains(&"LOGICAL_PLEONASM"));
        assert!(ids.contains(&"ARTIFICIAL_PASSIVE_BE_BY"));
    }

    #[test]
    fn test_auditor_disabled_rule() {
        let mut config = AuditorConfig::new();
        config.disable_rule("ARTIFICIAL_PASSIVE_BE_BY");
        let auditor = Auditor::with_config(config);

        let text = "Dự án được triển khai bởi nhóm.";
        let findings = auditor.audit(text);
        let ids: Vec<&str> = findings.iter().map(|f| f.pattern_id.as_str()).collect();
        assert!(!ids.contains(&"ARTIFICIAL_PASSIVE_BE_BY"));
    }

    #[test]
    fn test_auditor_category_filter() {
        let mut config = AuditorConfig::new();
        config.set_enabled_categories(vec!["bureaucracy".to_string()]);
        let auditor = Auditor::with_config(config);

        let text = "Nó là rất quan trọng để tiến hành thực hiện phương án này được duyệt bởi sếp.";
        let findings = auditor.audit(text);

        // Chỉ cho phép bureaucracy, các lỗi syntax/translationese sẽ bị lọc bỏ
        for f in &findings {
            assert_eq!(f.category, "bureaucracy");
        }
        let ids: Vec<&str> = findings.iter().map(|f| f.pattern_id.as_str()).collect();
        assert!(ids.contains(&"BUREAUCRATIC_VERB_PADDING"));
        assert!(!ids.contains(&"DUMMY_IT_TRANSLATIONESE"));
        assert!(!ids.contains(&"ARTIFICIAL_PASSIVE_BE_BY"));
    }

    #[test]
    fn test_auditor_severity_override() {
        let mut config = AuditorConfig::new();
        config.override_severity("BUREAUCRATIC_VERB_PADDING", "critical");
        let auditor = Auditor::with_config(config);

        let text = "Chúng tôi tiến hành thực hiện kế hoạch.";
        let findings = auditor.audit(text);
        let finding = findings
            .iter()
            .find(|f| f.pattern_id == "BUREAUCRATIC_VERB_PADDING")
            .unwrap();
        assert_eq!(finding.severity, "critical");
    }

    #[test]
    fn test_auditor_audit_batch_rayon() {
        let auditor = Auditor::new();
        let docs = vec![
            "Trong tình hình hiện nay đòi hỏi giải pháp tối ưu nhất.",
            "Dự án được triển khai bởi ban giám đốc.",
            "Chúng tôi tiến hành thực hiện công tác kiểm tra.",
        ];

        let batch_findings = auditor.audit_batch(&docs);
        assert_eq!(batch_findings.len(), 3);
        assert!(!batch_findings[0].is_empty());
        assert!(!batch_findings[1].is_empty());
        assert!(!batch_findings[2].is_empty());
    }

    #[test]
    fn test_auditor_document_report() {
        let auditor = Auditor::new();
        let text = "Không thể phủ nhận rằng trong tình hình hiện nay đòi hỏi giải pháp tối ưu nhất được triển khai bởi nhóm.";
        let report = auditor.audit_document(text);

        assert!(report.word_count > 10);
        assert!(!report.findings.is_empty());
        assert!(report.score < 90.0);
        assert!(report.execution_time_ms >= 0.0);
        let md = report.to_markdown();
        assert!(md.contains("# BÁO CÁO KIỂM TOÁN VĂN BẢN"));
        let json = report.to_json();
        assert!(json.contains("\"score\":"));
    }

    #[test]
    fn test_auditor_min_severity() {
        let mut config = AuditorConfig::new();
        config.set_min_severity(Severity::Major);
        let auditor = Auditor::with_config(config);

        // Văn bản có cả Minor (tối ưu nhất) và Major (bị động nhân tạo, khung đề)
        let text = "Giải pháp tối ưu nhất được triển khai bởi nhóm.";
        let findings = auditor.audit(text);

        for f in &findings {
            let sev = Severity::parse(&f.severity);
            assert!(sev >= Severity::Major);
        }
    }
}
