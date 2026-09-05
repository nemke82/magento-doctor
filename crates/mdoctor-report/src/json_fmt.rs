//! JSON report generator.

use mdoctor_core::{Finding, HealthScore, MagentoInstallation, CALVER_VERSION};
use serde::Serialize;

#[derive(Serialize)]
pub struct JsonScanReport<'a> {
    pub mdoctor_version: &'static str,
    pub health_score: &'a HealthScore,
    pub installation: &'a MagentoInstallation,
    pub findings: &'a [Finding],
}

pub fn render_json_report(
    installation: &MagentoInstallation,
    findings: &[Finding],
    health: &HealthScore,
) -> Result<String, serde_json::Error> {
    let report = JsonScanReport {
        mdoctor_version: CALVER_VERSION,
        health_score: health,
        installation,
        findings,
    };
    serde_json::to_string_pretty(&report)
}
