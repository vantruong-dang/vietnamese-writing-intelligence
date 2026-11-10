#![allow(clippy::approx_constant)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::doc_lazy_continuation)]

pub mod auditor;
pub mod report;
pub mod rules;
pub mod segmentation;
pub mod types;
pub mod unicode_utils;

pub use auditor::{Auditor, AuditorConfig};
pub use report::AuditReport;
pub use rules::{
    audit_linguistics, evaluate_ai_opening_cliche, evaluate_ai_summary_tail,
    evaluate_artificial_passive, evaluate_artificial_passive_fast, evaluate_aspect_mechanics,
    evaluate_bureaucratic_rules, evaluate_concessive_inversion,
    evaluate_crippled_prepositional_topic, evaluate_dummy_it_translationese,
    evaluate_factual_concessive_misuse, evaluate_formulaic_metaphor,
    evaluate_hypothetical_concessive_misuse, evaluate_logical_pleonasm,
    evaluate_nominalization_heavy, evaluate_preposition_conjunction_conflict,
    evaluate_redundant_beneficiary_passive, evaluate_superlative_misuse, evaluate_suyt_nua_thi,
    evaluate_syntax_cliche, evaluate_syntax_parallel, evaluate_syntax_reporting,
    evaluate_temporal_framing, evaluate_translationese_soleil, ADJECTIVAL_SUFFIX_FAKE_RULE_ID,
    AI_OPENING_RULE_ID, AI_SUMMARY_RULE_ID, ARTIFICIAL_PASSIVE_RULE_ID, ASPECT_MECHANICS_RULE_ID,
    CONCESSIVE_INVERSION_RULE_ID, CRIPPLED_TOPIC_RULE_ID, DUMMY_IT_RULE_ID,
    FACTUAL_CONCESSIVE_RULE_ID, FORMULAIC_METAPHOR_RULE_ID, HYPOTHETICAL_CONCESSIVE_RULE_ID,
    LOGICAL_PLEONASM_RULE_ID, NOMINALIZATION_HEAVY_RULE_ID, PREP_CONJ_RULE_ID,
    REDUNDANT_BENEFICIARY_RULE_ID, SUPERLATIVE_CHI_IT_RULE_ID, SUYT_NUA_THI_RULE_ID,
    SYNTAX_CLICHE_RULE_ID, SYNTAX_PARALLEL_RULE_ID, SYNTAX_REPORTING_RULE_ID,
    TEMPORAL_FRAMING_RULE_ID, TRANSLATIONESE_SOLEIL_RULE_ID,
};
pub use segmentation::{split_sentences, tokenize_viterbi, DaachorseSegmenter, SentenceSpan, EMBEDDED_DICTIONARY};
pub use types::{AutoFix, Finding, Severity, TokenSpan};
pub use unicode_utils::{normalize_nfc, CharOffsetMap};
