// FILE TỰ ĐỘNG SINH BỞI scripts/calc_prosody_baseline.py
// THAM SỐ THỰC CHỨNG TỪ 480.000+ TỪ VĂN PHONG CAO XUÂN HẠO & NGUYỄN HIẾN LÊ.
// KHÔNG SỬA TAY.

/// Baseline thống kê ngữ âm thực nghiệm của tiếng Việt tự nhiên chuẩn mực
#[derive(Debug, Clone, Copy)]
pub struct ProsodyBaseline {
    pub mean_sentence_len: f32,
    pub std_dev_sentence_len: f32,
    pub mean_trac_ratio: f32,
    pub std_dev_trac_ratio: f32,
    /// Ngưỡng chuỗi Trắc liên tiếp phân vị 99% (vượt quá mức này là bất thường thống kê p < 0.01)
    pub p99_consecutive_trac_threshold: usize,
}

/// Phân phối chuẩn của văn phong chính luận / văn học thuần Việt
pub static LITERARY_ESSAY_BASELINE: ProsodyBaseline = ProsodyBaseline {
    mean_sentence_len: 22.53f32,
    std_dev_sentence_len: 18.20f32,
    mean_trac_ratio: 0.5276f32,
    std_dev_trac_ratio: 0.1494f32,
    p99_consecutive_trac_threshold: 10,
};
