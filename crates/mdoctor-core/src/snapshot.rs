//! Sanitized snapshot export and import for safe offline sharing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::finding::Finding;
use crate::health::HealthScore;
use crate::model::MagentoInstallation;
use crate::version::CALVER_VERSION;

/// Safe, sanitized diagnostic snapshot of a store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSnapshot {
    pub format_version: String,
    pub mdoctor_version: String,
    pub created_at: DateTime<Utc>,
    pub health_score: HealthScore,
    pub findings: Vec<Finding>,
    pub installation: MagentoInstallation,
}

impl DiagnosticSnapshot {
    pub fn new(installation: MagentoInstallation, findings: Vec<Finding>, health_score: HealthScore) -> Self {
        Self {
            format_version: "1.0".to_string(),
            mdoctor_version: CALVER_VERSION.to_string(),
            created_at: Utc::now(),
            health_score,
            findings,
            installation,
        }
    }

    /// Serialize snapshot to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse a snapshot from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
