//! Safety levels and execution budget constraints for Magento Doctor.

use serde::{Deserialize, Serialize};

/// The execution cost and intrusiveness of an analyzer or command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SafetyLevel {
    /// Zero external system impact. Fast, in-memory static analysis only.
    Safe,
    /// Negligible impact. Reading metadata, small config files, non-locking queries.
    Low,
    /// Moderate impact. Inspection of larger tables or process states with safeguards.
    Moderate,
    /// Resource-intensive. Extended queries, large file scanning, or deep profiling.
    Expensive,
    /// Potentially alters system state or runs executable queries in MySQL (e.g. EXPLAIN ANALYZE).
    Intrusive,
}

impl std::fmt::Display for SafetyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafetyLevel::Safe => write!(f, "SAFE"),
            SafetyLevel::Low => write!(f, "LOW"),
            SafetyLevel::Moderate => write!(f, "MODERATE"),
            SafetyLevel::Expensive => write!(f, "EXPENSIVE"),
            SafetyLevel::Intrusive => write!(f, "INTRUSIVE"),
        }
    }
}

/// Execution budget configuration to prevent runaway operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanBudget {
    /// Maximum allowed total duration in seconds.
    pub max_seconds: u64,
    /// Maximum allowed database query duration in seconds.
    pub max_db_seconds: u64,
    /// Whether EXPLAIN statements are permitted.
    pub allow_explain: bool,
    /// Whether EXPLAIN ANALYZE (which actually executes the query) is permitted.
    pub allow_explain_analyze: bool,
    /// Maximum allowed safety level for this run.
    pub max_safety_level: SafetyLevel,
}

impl Default for ScanBudget {
    fn default() -> Self {
        Self {
            max_seconds: 60,
            max_db_seconds: 15,
            allow_explain: true,
            allow_explain_analyze: false,
            max_safety_level: SafetyLevel::Low,
        }
    }
}

impl ScanBudget {
    /// Create a budget allowing moderate operations (e.g. for --deep scans).
    pub fn deep() -> Self {
        Self {
            max_seconds: 180,
            max_db_seconds: 45,
            allow_explain: true,
            allow_explain_analyze: false,
            max_safety_level: SafetyLevel::Moderate,
        }
    }

    /// Check if an operation with a given safety level is permitted under this budget.
    pub fn is_allowed(&self, level: SafetyLevel) -> bool {
        level <= self.max_safety_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_ordering() {
        assert!(SafetyLevel::Safe < SafetyLevel::Low);
        assert!(SafetyLevel::Low < SafetyLevel::Moderate);
        assert!(SafetyLevel::Moderate < SafetyLevel::Expensive);
        assert!(SafetyLevel::Expensive < SafetyLevel::Intrusive);
    }

    #[test]
    fn test_budget_permissions() {
        let default_budget = ScanBudget::default();
        assert!(default_budget.is_allowed(SafetyLevel::Safe));
        assert!(default_budget.is_allowed(SafetyLevel::Low));
        assert!(!default_budget.is_allowed(SafetyLevel::Moderate));
        assert!(!default_budget.is_allowed(SafetyLevel::Intrusive));

        let deep_budget = ScanBudget::deep();
        assert!(deep_budget.is_allowed(SafetyLevel::Moderate));
        assert!(!deep_budget.is_allowed(SafetyLevel::Expensive));
    }
}
