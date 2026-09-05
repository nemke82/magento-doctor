//! mdoctor_rules: Cross-analysis correlation engine and rule implementations.

pub mod engine;
pub mod explanations;
pub mod rules;

pub use engine::CrossAnalysisEngine;
pub use explanations::{get_rule_explanation, RuleExplanation};
