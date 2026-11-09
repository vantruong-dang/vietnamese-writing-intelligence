pub mod artificial_passive;
pub mod aspect;
pub mod bureaucracy;
pub mod cliche;
pub mod concessive;
pub mod engine;
pub mod lexical;
pub mod pleonasm;
pub mod syntax;
pub mod topic_comment;
pub mod translationese;

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
    evaluate_ai_opening_cliche, evaluate_ai_summary_tail, evaluate_discovered_ai_cliches,
    evaluate_formulaic_metaphor, AI_DISCOVERED_CLICHE_RULE_ID, AI_OPENING_RULE_ID,
    AI_SUMMARY_RULE_ID, FORMULAIC_METAPHOR_RULE_ID,
};
pub use concessive::{
    evaluate_concessive_inversion, evaluate_concessive_inversion_with_map,
    evaluate_factual_concessive_misuse, evaluate_hypothetical_concessive_misuse,
    CONCESSIVE_INVERSION_RULE_ID, FACTUAL_CONCESSIVE_RULE_ID, HYPOTHETICAL_CONCESSIVE_RULE_ID,
};
pub use engine::audit_linguistics;
pub use lexical::{
    evaluate_superlative_misuse, evaluate_suyt_nua_thi, SUPERLATIVE_CHI_IT_RULE_ID,
    SUYT_NUA_THI_RULE_ID,
};
pub use pleonasm::{evaluate_logical_pleonasm, LOGICAL_PLEONASM_RULE_ID};
pub use syntax::{
    evaluate_syntax_cliche, evaluate_syntax_parallel, evaluate_syntax_reporting,
    SYNTAX_CLICHE_RULE_ID, SYNTAX_PARALLEL_RULE_ID, SYNTAX_REPORTING_RULE_ID,
};
pub use topic_comment::{
    evaluate_crippled_prepositional_topic, evaluate_preposition_conjunction_conflict,
    CRIPPLED_TOPIC_RULE_ID, PREP_CONJ_RULE_ID,
};
pub use translationese::{
    evaluate_dummy_it_translationese, evaluate_dummy_it_translationese_with_map,
    evaluate_nominalization_heavy, evaluate_temporal_framing, evaluate_translationese_soleil,
    ADJECTIVAL_SUFFIX_FAKE_RULE_ID, DUMMY_IT_RULE_ID, NOMINALIZATION_HEAVY_RULE_ID,
    TEMPORAL_FRAMING_RULE_ID, TRANSLATIONESE_SOLEIL_RULE_ID,
};
