//! Module impact and architectural risk scoring models.

use serde::{Deserialize, Serialize};
use crate::model::ModuleClassification;

/// Categorical impact / risk level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for ImpactLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImpactLevel::Low => write!(f, "LOW"),
            ImpactLevel::Medium => write!(f, "MEDIUM"),
            ImpactLevel::High => write!(f, "HIGH"),
            ImpactLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// An individual risk driver contributing to a module's impact score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDriver {
    pub name: String,
    pub description: String,
    pub points: u32,
}

/// Aggregate performance impact and architectural risk score for a single module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleImpactScore {
    pub module_name: String,
    pub classification: ModuleClassification,
    pub score: u32, // 0..100
    pub level: ImpactLevel,
    pub risk_drivers: Vec<RiskDriver>,
    pub plugins_count: usize,
    pub around_plugins_count: usize,
    pub hotpath_plugins_count: usize,
    pub observers_count: usize,
    pub hot_observers_count: usize,
    pub cron_jobs_count: usize,
    pub minutely_crons_count: usize,
    pub preferences_count: usize,
    pub core_preferences_count: usize,
    pub db_tables_count: usize,
    pub core_tables_altered_count: usize,
    pub ast_indicators_count: usize,
}

impl ModuleImpactScore {
    /// Provide a concise 1-line summary of top risk drivers.
    pub fn top_drivers_summary(&self) -> String {
        if self.risk_drivers.is_empty() {
            return "Clean / No significant risk indicators".to_string();
        }

        let mut sorted = self.risk_drivers.clone();
        sorted.sort_by_key(|b| std::cmp::Reverse(b.points));

        sorted
            .iter()
            .take(3)
            .map(|d| d.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ")
    }
}
