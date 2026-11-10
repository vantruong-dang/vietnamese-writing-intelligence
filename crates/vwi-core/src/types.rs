use serde::{Deserialize, Serialize};

/// Mức độ nghiêm trọng của vi phạm ngôn ngữ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Advisory,   // Không trừ điểm: Chỉ số chẩn đoán nhịp điệu / tư vấn phong cách
    Suggestion, // Gợi ý phong cách nhẹ
    Minor,      // Vi phạm nhẹ
    Major,      // Vi phạm cấu trúc nặng
    Critical,   // Vi phạm nghiêm trọng
}

impl Severity {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => Severity::Critical,
            "major" => Severity::Major,
            "minor" => Severity::Minor,
            "advisory" => Severity::Advisory,
            _ => Severity::Suggestion,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::Major => "major",
            Severity::Minor => "minor",
            Severity::Suggestion => "suggestion",
            Severity::Advisory => "advisory",
        }
    }
}

/// Ngữ vực của văn bản (Text Register / Domain)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Register {
    #[default]
    General,        // Phổ thông
    Administrative, // Hành chính, Pháp lý (Miễn trừ kiểm tra Bằng-Trắc)
    Technical,      // Kỹ thuật, Khoa học (Chấp nhận chuỗi thuật ngữ chuyên ngành)
    Literary,       // Văn học, Tùy bút
    Journalism,     // Báo chí chính luận
}

impl Register {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" | "administrative" | "legal" => Register::Administrative,
            "tech" | "technical" => Register::Technical,
            "lit" | "literary" | "essay" => Register::Literary,
            "news" | "journalism" => Register::Journalism,
            _ => Register::General,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Register::General => "general",
            Register::Administrative => "administrative",
            Register::Technical => "technical",
            Register::Literary => "literary",
            Register::Journalism => "journalism",
        }
    }
}

/// Nhãn từ loại theo chuẩn Universal Dependencies (UPOS) kết hợp Ngữ pháp chức năng.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[repr(u8)]
pub enum PosTag {
    Noun,   // Danh từ (học sinh, bàn là, báo cáo)
    Verb,   // Động từ (chạy, nghiên cứu, đòi hỏi, cho thấy)
    Adj,    // Tính từ (đẹp, tối ưu, nhanh)
    Adp,    // Giới từ (trong, ngoài, bằng, với, qua, theo, ở)
    Adv,    // Phó từ (rất, quá, đang, sẽ, không, chưa)
    Pron,   // Đại từ (tôi, nó, họ, chúng ta)
    Cconj,  // Liên từ đẳng lập (và, nhưng, hoặc, song)
    Sconj,  // Liên từ phụ thuộc (nếu, vì, mặc dù, tuy, để)
    Part,   // Trợ từ / Tình thái từ (thì, mà, nhé, nhỉ, là, chính)
    Num,    // Số từ (1, 10, một, hai)
    Punct,  // Dấu câu (., ,, !, ?)
    Sym,    // Ký hiệu ($, %, +)
    Other,  // Từ chưa phân loại / OOV
}

impl PosTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            PosTag::Noun => "NOUN",
            PosTag::Verb => "VERB",
            PosTag::Adj => "ADJ",
            PosTag::Adp => "ADP",
            PosTag::Adv => "ADV",
            PosTag::Pron => "PRON",
            PosTag::Cconj => "CCONJ",
            PosTag::Sconj => "SCONJ",
            PosTag::Part => "PART",
            PosTag::Num => "NUM",
            PosTag::Punct => "PUNCT",
            PosTag::Sym => "SYM",
            PosTag::Other => "OTHER",
        }
    }

    pub fn from_str_tag(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "NOUN" | "N" | "NC" | "PROPN" | "NNP" => PosTag::Noun,
            "VERB" | "V" | "AUX" => PosTag::Verb,
            "ADJ" | "A" => PosTag::Adj,
            "ADP" | "E" | "PREP" => PosTag::Adp,
            "ADV" | "R" => PosTag::Adv,
            "PRON" | "P" | "PRO" => PosTag::Pron,
            "CCONJ" | "C" => PosTag::Cconj,
            "SCONJ" | "SC" => PosTag::Sconj,
            "PART" | "T" | "PRT" => PosTag::Part,
            "NUM" | "M" => PosTag::Num,
            "PUNCT" | "CH" => PosTag::Punct,
            "SYM" => PosTag::Sym,
            _ => PosTag::Other,
        }
    }

    pub fn tag_index(&self) -> usize {
        match self {
            PosTag::Noun => 0,
            PosTag::Verb => 1,
            PosTag::Adj => 2,
            PosTag::Adp => 3,
            PosTag::Adv => 4,
            PosTag::Pron => 5,
            PosTag::Cconj => 6,
            PosTag::Sconj => 7,
            PosTag::Part => 8,
            PosTag::Num => 9,
            PosTag::Punct => 10,
            PosTag::Sym => 11,
            PosTag::Other => 12,
        }
    }
}

impl std::fmt::Display for PosTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Tọa độ và thông tin của một token (từ đơn hoặc từ ghép tiếng Việt).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenSpan {
    pub text: String,
    /// Offset ký tự bắt đầu (tính theo Unicode scalar value / code points)
    pub start: usize,
    /// Offset ký tự kết thúc (tính theo Unicode scalar value / code points)
    pub end: usize,
    pub is_compound: bool,
    /// Nhãn từ loại theo chuẩn Universal Dependencies
    pub pos: Option<PosTag>,
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
        pos: PosTag,
    ) -> Self {
        Self {
            text: text.into(),
            start,
            end,
            is_compound,
            pos: Some(pos),
        }
    }
}

/// Lệnh tự động thay thế trực tiếp (Auto-Fix) theo chuẩn Compiler & Linter (rustc/clippy/ruff)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFix {
    pub replacement: String,
    pub start: usize,
    pub end: usize,
}

/// Phát hiện vi phạm ngữ pháp / phong cách ngôn ngữ học.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub pattern_id: String,
    pub rule_id: String,
    pub category: String,
    pub severity: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f64,
    pub rationale: String,
    pub source: String,
    pub matched_span: String,
    /// Trường gợi ý chung (bảo toàn tương thích ngược 100% cho các API/CLI hiện hành)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    /// Lời khuyên định hướng tư duy (dành riêng cho người đọc tham khảo, không có lệnh sửa máy móc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion_note: Option<String>,
    /// Lệnh tự sửa trực tiếp cho máy tính (dành riêng cho 1-click apply của Frontend/Linter)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<AutoFix>,
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
        let pid = pattern_id.into();
        Self {
            rule_id: pid.clone(),
            pattern_id: pid,
            category: category.into(),
            severity: severity.into(),
            start,
            end,
            confidence,
            rationale: rationale.into(),
            source: source.into(),
            matched_span: matched_span.into(),
            suggestion: None,
            suggestion_note: None,
            fix: None,
        }
    }

    /// Gán gợi ý chung (tự động điền suggestion_note)
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        let s = suggestion.into();
        self.suggestion = Some(s.clone());
        self.suggestion_note = Some(s);
        self
    }

    /// Gán lời khuyên định hướng cho người đọc (fix = None)
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        let n = note.into();
        self.suggestion_note = Some(n.clone());
        self.suggestion = Some(n);
        self
    }

    /// Gán lệnh tự sửa trực tiếp cho máy tính (AutoFix 1-click)
    pub fn with_fix(mut self, start: usize, end: usize, replacement: impl Into<String>) -> Self {
        let rep = replacement.into();
        self.fix = Some(AutoFix {
            replacement: rep.clone(),
            start,
            end,
        });
        self.suggestion = Some(rep);
        self
    }
}
