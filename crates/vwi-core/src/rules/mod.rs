pub mod admin_format;
pub mod artificial_passive;
pub mod aspect;
pub mod bureaucracy;
pub mod cliche;
pub mod concessive;
pub mod confusion;
#[cfg(feature = "context-lm")]
pub mod context_spellcheck;
pub mod engine;
pub mod lexical;
pub mod masking;
pub mod orthography;
pub mod phonotactics;
pub mod pleonasm;
pub mod pos_structure;
pub mod prosody;
pub mod real_word;
pub mod spellcheck;
pub mod syntax;
pub mod teencode;
pub mod tense;
pub mod topic_comment;
pub mod translationese;
pub mod trap_whitelist;

pub use admin_format::{
    evaluate_admin_format, R10_AGENCY_NAME_RULE_ID, R11_DECIMAL_RULE_ID,
    R12_QUOC_HIEU_RULE_ID, R13_REDUNDANT_WORD_RULE_ID, R1_DATE_PADDING_RULE_ID,
    R3_LEGAL_REF_RULE_ID, R4_SPECIAL_NOUN_RULE_ID, R5_SPECIAL_PLACE_RULE_ID,
    R7_MOTTO_RULE_ID, R8_DOC_NUMBER_RULE_ID, R9_PLACE_DATE_RULE_ID,
};
pub use pos_structure::{
    evaluate_adverb_placement, evaluate_classifier_mismatch, evaluate_list_structure_mismatch,
    evaluate_subject_verb_gap, ADVERB_PLACEMENT_RULE_ID, CLASSIFIER_MISMATCH_RULE_ID,
    LIST_STRUCTURE_MISMATCH_RULE_ID, SUBJECT_VERB_GAP_RULE_ID,
};
pub use real_word::{evaluate_real_word_errors, REAL_WORD_RULE_ID};
pub use spellcheck::{evaluate_spellcheck, SPELLCHECK_RULE_ID};
pub use teencode::{evaluate_teencode, TEENCODE_RULE_ID};
pub use tense::{evaluate_past_future_conflict, PAST_FUTURE_CONFLICT_RULE_ID};
pub use trap_whitelist::mask_trap_phrases;
pub use artificial_passive::{
    evaluate_artificial_passive, evaluate_artificial_passive_fast,
    evaluate_redundant_beneficiary_passive, ARTIFICIAL_PASSIVE_RULE_ID,
    REDUNDANT_BENEFICIARY_RULE_ID,
};
pub use aspect::{evaluate_aspect_mechanics, ASPECT_MECHANICS_RULE_ID};
pub use bureaucracy::{
    evaluate_bureaucratic_rules, BUREAUCRATIC_VERB_PADDING_RULE_ID, REDUNDANT_CONNECTORS_RULE_ID,
    VAGUE_INTENSIFIER_RULE_ID,
};
pub use cliche::{
    evaluate_ai_filler_words, evaluate_ai_opening_cliche, evaluate_ai_summary_tail,
    evaluate_discovered_ai_cliches, evaluate_formulaic_metaphor, AI_DISCOVERED_CLICHE_RULE_ID,
    AI_FILLER_WORDS_RULE_ID, AI_OPENING_RULE_ID, AI_SUMMARY_RULE_ID, FORMULAIC_METAPHOR_RULE_ID,
};
pub use concessive::{
    evaluate_concessive_inversion, evaluate_concessive_inversion_with_map,
    evaluate_factual_concessive_misuse, evaluate_hypothetical_concessive_misuse,
    CONCESSIVE_INVERSION_RULE_ID, FACTUAL_CONCESSIVE_RULE_ID, HYPOTHETICAL_CONCESSIVE_RULE_ID,
};
pub use confusion::{
    evaluate_adverb_order, evaluate_context_confusions, evaluate_duplicate_function_words,
    evaluate_voice_mismatch, ADVERB_ORDER_RULE_ID, CONTEXT_CONFUSION_RULE_ID,
    DUPLICATE_WORD_RULE_ID, VOICE_MISMATCH_RULE_ID,
};
#[cfg(feature = "context-lm")]
pub use context_spellcheck::{
    evaluate_context_spellcheck, evaluate_context_spellcheck_with, CONTEXT_SPELLCHECK_RULE_ID,
    DEFAULT_MARGIN, DEFAULT_MIN_ABS,
};
pub use engine::audit_linguistics;
pub use lexical::{
    evaluate_superlative_misuse, evaluate_suyt_nua_thi, SUPERLATIVE_CHI_IT_RULE_ID,
    SUYT_NUA_THI_RULE_ID,
};
pub use masking::create_masked_text_for_audit;
pub use orthography::{
    evaluate_orthography, OrthographyChecker, BRACKET_SPACE_INSIDE_RULE_ID,
    DECREE30_ADMIN_UNIT_LOWERCASE_RULE_ID, DECREE30_AGENCY_CAPITALIZATION_RULE_ID,
    ORTHOGRAPHY_TONE_INCONSISTENCY_RULE_ID, PUNCTUATION_NO_SPACE_AFTER_RULE_ID,
    PUNCTUATION_SPACE_BEFORE_RULE_ID, SENTENCE_CAPITALIZATION_RULE_ID,
};
pub use phonotactics::{PhonotacticEngine, TELEX_TYPO_ERROR_RULE_ID};
pub use pleonasm::{evaluate_excessive_su, evaluate_logical_pleonasm, EXCESSIVE_SU_RULE_ID, LOGICAL_PLEONASM_RULE_ID};
pub use prosody::{evaluate_prosody, PROSODIC_EXCESSIVE_TRAC_RULE_ID};
pub use syntax::{
    evaluate_he_nen, evaluate_missing_bay_nhieu, evaluate_missing_thi_after_neu,
    evaluate_redundant_cause, evaluate_redundant_conjunction_pairs, evaluate_redundant_reporting_la,
    evaluate_syntax_cliche, evaluate_syntax_parallel, evaluate_syntax_reporting,
    SYNTAX_CLICHE_RULE_ID, SYNTAX_HE_NEN_MISUSE_RULE_ID, SYNTAX_MISSING_BAY_NHIEU_RULE_ID,
    SYNTAX_MISSING_THI_AFTER_NEU_RULE_ID, SYNTAX_PARALLEL_RULE_ID, SYNTAX_REDUNDANT_CAUSE_RULE_ID,
    SYNTAX_REDUNDANT_CONJ_PAIRS_RULE_ID, SYNTAX_REDUNDANT_REPORTING_LA_RULE_ID,
    SYNTAX_REPORTING_RULE_ID,
};
pub use topic_comment::{
    evaluate_crippled_prepositional_topic, evaluate_preposition_conjunction_conflict,
    CRIPPLED_TOPIC_RULE_ID, PREP_CONJ_RULE_ID,
};
pub use translationese::{
    evaluate_dummy_it_translationese, evaluate_dummy_it_translationese_with_map,
    evaluate_nominalization_heavy, evaluate_temporal_framing, evaluate_translationese_cliches,
    evaluate_translationese_soleil, evaluate_wrong_bang, ADJECTIVAL_SUFFIX_FAKE_RULE_ID,
    DUMMY_IT_RULE_ID, NOMINALIZATION_HEAVY_RULE_ID, TEMPORAL_FRAMING_RULE_ID,
    TRANSLATIONESE_CLICHES_RULE_ID, TRANSLATIONESE_SOLEIL_RULE_ID, WRONG_BANG_RULE_ID,
};
