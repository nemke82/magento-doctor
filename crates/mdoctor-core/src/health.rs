//! Health scoring engine for Magento Doctor.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::finding::{Category, Finding, Severity};

/// Detailed health score with category breakdowns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    /// Overall score from 0 to 100.
    pub overall: u32,
    /// Category scores from 0 to 100.
    pub categories: HashMap<Category, u32>,
    /// Summary counts of findings.
    pub critical_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
}

impl HealthScore {
    /// Compute a weighted health score based on findings.
    /// Critical findings deduct 15 points (clamped), Warnings deduct 5 points.
    pub fn calculate(findings: &[Finding]) -> Self {
        let mut critical_count = 0;
        let mut warning_count = 0;
        let mut info_count = 0;

        let mut category_penalties: HashMap<Category, u32> = HashMap::new();

        for finding in findings {
            match finding.severity {
                Severity::Critical => {
                    critical_count += 1;
                    *category_penalties.entry(finding.category).or_insert(0) += 20;
                }
                Severity::Warning => {
                    warning_count += 1;
                    *category_penalties.entry(finding.category).or_insert(0) += 8;
                }
                Severity::Info => {
                    info_count += 1;
                    *category_penalties.entry(finding.category).or_insert(0) += 1;
                }
            }
        }

        // Overall score: start at 100, deduct based on total severities
        let total_deduction = (critical_count * 15) + (warning_count * 4);
        let overall = 100u32.saturating_sub(total_deduction as u32);

        // Subsystem categories
        let all_categories = [
            Category::MagentoCore,
            Category::Database,
            Category::Cron,
            Category::Indexers,
            Category::Modules,
            Category::Cache,
            Category::Search,
            Category::Php,
            Category::Filesystem,
            Category::Security,
        ];

        let mut categories = HashMap::new();
        for cat in all_categories {
            let penalty = category_penalties.get(&cat).copied().unwrap_or(0);
            categories.insert(cat, 100u32.saturating_sub(penalty));
        }

        Self {
            overall,
            categories,
            critical_count,
            warning_count,
            info_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Confidence;

    #[test]
    fn test_perfect_health_score() {
        let findings = vec![];
        let score = HealthScore::calculate(&findings);
        assert_eq!(score.overall, 100);
        assert_eq!(score.critical_count, 0);
        assert_eq!(score.warning_count, 0);
    }

    #[test]
    fn test_deductions() {
        let findings = vec![
            Finding::new("MD-CRON-001", "Cron backlog", Severity::Critical, Confidence::High, Category::Cron),
            Finding::new("MD-PLG-001", "Around plugin", Severity::Warning, Confidence::High, Category::Plugins),
        ];
        let score = HealthScore::calculate(&findings);
        assert_eq!(score.critical_count, 1);
        assert_eq!(score.warning_count, 1);
        assert_eq!(score.overall, 100 - 15 - 4);
        assert_eq!(score.categories.get(&Category::Cron), Some(&80));
    }
}
