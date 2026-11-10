// FILE TỰ ĐỘNG SINH BỞI scripts/harvest_and_calc_multi_register.py
// MA TRẬN BASELINE ĐA NGỮ VỰC (MULTI-REGISTER PROSODY BASELINES)

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProsodyBaseline {
    pub mean_sentence_len: f32,
    pub std_dev_sentence_len: f32,
    pub mean_trac_ratio: f32,
    pub std_dev_trac_ratio: f32,
    pub p99_consecutive_trac_threshold: usize,
}

pub static LITERARY_ESSAY_BASELINE: ProsodyBaseline = ProsodyBaseline {
    mean_sentence_len: 22.53f32,
    std_dev_sentence_len: 18.20f32,
    mean_trac_ratio: 0.5276f32,
    std_dev_trac_ratio: 0.1494f32,
    p99_consecutive_trac_threshold: 10,
};

pub static JOURNALISM_BASELINE: ProsodyBaseline = ProsodyBaseline {
    mean_sentence_len: 20.18f32,
    std_dev_sentence_len: 10.25f32,
    mean_trac_ratio: 0.5128f32,
    std_dev_trac_ratio: 0.1306f32,
    p99_consecutive_trac_threshold: 9,
};

pub static ADMINISTRATIVE_BASELINE: ProsodyBaseline = ProsodyBaseline {
    mean_sentence_len: 29.73f32,
    std_dev_sentence_len: 4.35f32,
    mean_trac_ratio: 0.5778f32,
    std_dev_trac_ratio: 0.0874f32,
    p99_consecutive_trac_threshold: 8,
};

pub static TECHNICAL_BASELINE: ProsodyBaseline = ProsodyBaseline {
    mean_sentence_len: 26.56f32,
    std_dev_sentence_len: 1.42f32,
    mean_trac_ratio: 0.6517f32,
    std_dev_trac_ratio: 0.0897f32,
    p99_consecutive_trac_threshold: 9,
};

pub fn get_baseline_for_register(register: crate::types::Register) -> Option<&'static ProsodyBaseline> {
    match register {
        crate::types::Register::Literary => Some(&LITERARY_ESSAY_BASELINE),
        crate::types::Register::Journalism => Some(&JOURNALISM_BASELINE),
        crate::types::Register::Administrative => Some(&ADMINISTRATIVE_BASELINE),
        crate::types::Register::Technical => Some(&TECHNICAL_BASELINE),
        crate::types::Register::General => Some(&JOURNALISM_BASELINE),
    }
}
