//! Finding, severity, and category definitions for diagnostic reports.

use serde::{Deserialize, Serialize};

/// Severity of a diagnostic finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// Confidence rating of a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Confidence::High => write!(f, "HIGH"),
            Confidence::Medium => write!(f, "MEDIUM"),
            Confidence::Low => write!(f, "LOW"),
        }
    }
}

/// Category taxonomy for Magento Doctor rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Category {
    Environment,
    MagentoCore,
    Modules,
    DependencyInjection,
    Plugins,
    Events,
    Cron,
    Indexers,
    Database,
    Queries,
    Cache,
    Search,
    Queues,
    Php,
    Filesystem,
    Security,
    Performance,
    Upgrade,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Environment => write!(f, "Environment"),
            Category::MagentoCore => write!(f, "Magento Core"),
            Category::Modules => write!(f, "Modules"),
            Category::DependencyInjection => write!(f, "Dependency Injection"),
            Category::Plugins => write!(f, "Plugins"),
            Category::Events => write!(f, "Events & Observers"),
            Category::Cron => write!(f, "Cron"),
            Category::Indexers => write!(f, "Indexers"),
            Category::Database => write!(f, "Database"),
            Category::Queries => write!(f, "Queries & Indexes"),
            Category::Cache => write!(f, "Cache & Redis"),
            Category::Search => write!(f, "Search Engine"),
            Category::Queues => write!(f, "Queues"),
            Category::Php => write!(f, "PHP Runtime"),
            Category::Filesystem => write!(f, "Filesystem"),
            Category::Security => write!(f, "Security"),
            Category::Performance => write!(f, "Performance"),
            Category::Upgrade => write!(f, "Upgrade Readiness"),
        }
    }
}

/// A structured, evidence-backed diagnostic finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Stable rule identifier, e.g. "MD-CRON-010" or "MD-PERF-021".
    pub rule_id: String,
    /// Human-readable title of the concern.
    pub title: String,
    /// Severity level (Critical, Warning, Info).
    pub severity: Severity,
    /// Confidence level (High, Medium, Low).
    pub confidence: Confidence,
    /// Category classification.
    pub category: Category,
    /// Concise summary.
    pub summary: String,
    /// Detailed factual evidence observed across code/schema/runtime.
    pub evidence: Vec<String>,
    /// Business, operational, or performance impact.
    pub impact: String,
    /// Recommended remediation steps.
    pub recommendation: String,
    /// Optional CLI or MySQL commands to verify manually.
    pub verification_commands: Vec<String>,
    /// Modules involved or responsible.
    pub related_modules: Vec<String>,
    /// Database tables involved.
    pub related_tables: Vec<String>,
    /// File paths and lines involved.
    pub related_files: Vec<String>,
}

impl Finding {
    pub fn new(
        rule_id: impl Into<String>,
        title: impl Into<String>,
        severity: Severity,
        confidence: Confidence,
        category: Category,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            title: title.into(),
            severity,
            confidence,
            category,
            summary: String::new(),
            evidence: Vec::new(),
            impact: String::new(),
            recommendation: String::new(),
            verification_commands: Vec::new(),
            related_modules: Vec::new(),
            related_tables: Vec::new(),
            related_files: Vec::new(),
        }
    }
}
