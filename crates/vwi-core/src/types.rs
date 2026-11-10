use serde::{Deserialize, Serialize};

/// Mức độ nghiêm trọng của vi phạm ngôn ngữ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Suggestion,
    Minor,
    Major,
    Critical,
}

impl Severity {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => Severity::Critical,
            "major" => Severity::Major,
            "minor" => Severity::Minor,
            _ => Severity::Suggestion,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::Major => "major",
            Severity::Minor => "minor",
            Severity::Suggestion => "suggestion",
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
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
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}
