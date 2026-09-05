//! Markdown report generator.

use mdoctor_core::{Finding, HealthScore, MagentoInstallation, Severity, CALVER_VERSION};

pub fn render_markdown_report(
    installation: &MagentoInstallation,
    findings: &[Finding],
    health: &HealthScore,
) -> String {
    let mut md = String::new();

    md.push_str(&format!("# Magento Doctor Report ({})\n\n", CALVER_VERSION));

    let edition_str = format!("{}", installation.edition);
    let version_str = installation.version.as_deref().unwrap_or("unknown");
    md.push_str(&format!("- **Edition & Version**: {} {}\n", edition_str, version_str));
    md.push_str(&format!("- **Deployment Mode**: {}\n", installation.mode));
    if let Some(php) = &installation.environment.php_cli_version {
        md.push_str(&format!("- **PHP CLI**: {}\n", php));
    }
    md.push_str(&format!(
        "- **Modules**: {} enabled / {} disabled ({} third-party)\n\n",
        installation.enabled_modules_count(),
        installation.disabled_modules_count(),
        installation.third_party_modules_count()
    ));

    md.push_str(&format!("## Overall Health: {} / 100\n\n", health.overall));
    md.push_str(&format!(
        "| Critical | Warning | Info |\n|:---:|:---:|:---:|\n| {} | {} | {} |\n\n",
        health.critical_count, health.warning_count, health.info_count
    ));

    md.push_str("## Primary Findings\n\n");
    if findings.is_empty() {
        md.push_str("✓ No critical findings or warnings identified.\n");
    } else {
        for finding in findings {
            let badge = match finding.severity {
                Severity::Critical => "🔴 **CRITICAL**",
                Severity::Warning => "🟡 **WARNING**",
                Severity::Info => "🔵 **INFO**",
            };

            md.push_str(&format!("### {} {}\n\n", badge, finding.title));
            md.push_str(&format!("- **Rule ID**: `{}`\n", finding.rule_id));
            md.push_str(&format!("- **Category**: {}\n", finding.category));
            md.push_str(&format!("- **Confidence**: {}\n\n", finding.confidence));
            md.push_str(&format!("{}\n\n", finding.summary));

            if !finding.evidence.is_empty() {
                md.push_str("**Evidence / Correlation:**\n");
                for ev in &finding.evidence {
                    md.push_str(&format!("- {}\n", ev));
                }
                md.push('\n');
            }

            if !finding.impact.is_empty() {
                md.push_str(&format!("**Impact**: {}\n\n", finding.impact));
            }

            if !finding.recommendation.is_empty() {
                md.push_str(&format!("**Recommendation**: {}\n\n", finding.recommendation));
            }
        }
    }

    md
}
