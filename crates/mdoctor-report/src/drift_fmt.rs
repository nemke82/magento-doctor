//! Renderers for configuration and diagnostic drift reports.

use colored::*;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use mdoctor_core::{DriftChangeType, DriftReport, Severity};

/// Render a terminal-friendly ANSI drift comparison report.
pub fn render_drift_terminal(drift: &DriftReport) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "\n{}\n\n",
        "=== MAGENTO DOCTOR CONFIGURATION DRIFT REPORT ==="
            .bold()
            .cyan()
    ));

    out.push_str(&format!(
        "Baseline: {}  ->  Current: {}\n",
        drift.baseline_time.format("%Y-%m-%d %H:%M:%S UTC").to_string().yellow(),
        drift.current_time.format("%Y-%m-%d %H:%M:%S UTC").to_string().cyan()
    ));

    // Health score diff
    let delta_str = if drift.health_drift.delta_overall >= 0 {
        format!("+{}", drift.health_drift.delta_overall).green().bold()
    } else {
        format!("{}", drift.health_drift.delta_overall).red().bold()
    };

    out.push_str(&format!(
        "Health Score: {} -> {} ({})\n",
        drift.health_drift.baseline_overall,
        drift.health_drift.current_overall,
        delta_str
    ));

    out.push_str(&format!(
        "Findings Diff: Critical ({:+})  Warning ({:+})\n\n",
        drift.health_drift.delta_critical,
        drift.health_drift.delta_warning
    ));

    // Regressions Alert
    if drift.has_regressions {
        out.push_str(&format!(
            "{}\n",
            "⚠️  REGRESSIONS DETECTED SINCE BASELINE:"
                .red()
                .bold()
                .underline()
        ));
        for f in &drift.findings_drift.new_findings {
            let sev = match f.severity {
                Severity::Critical => "CRITICAL".red().bold(),
                Severity::Warning => "WARNING".yellow().bold(),
                Severity::Info => "INFO".blue().bold(),
            };
            out.push_str(&format!("  + [{}] [{}] {}\n", sev, f.rule_id.bold(), f.title));
            out.push_str(&format!("    Fix: {}\n", f.recommendation.cyan()));
        }
        out.push('\n');
    } else {
        out.push_str(&format!(
            "{}\n\n",
            "✓ No diagnostic regressions detected since baseline.".green().bold()
        ));
    }

    // Resolved issues
    if !drift.findings_drift.resolved_findings.is_empty() {
        out.push_str(&format!(
            "{}\n",
            "🎉 RESOLVED ISSUES (FIXED SINCE BASELINE):"
                .green()
                .bold()
                .underline()
        ));
        for f in &drift.findings_drift.resolved_findings {
            out.push_str(&format!("  - [{}] {}\n", f.rule_id.green(), f.title));
        }
        out.push('\n');
    }

    // Module drift table
    if !drift.modules_drift.is_empty() {
        out.push_str(&format!("{}\n", "Module Changes:".bold().underline()));
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Module").fg(Color::Cyan),
                Cell::new("Change").fg(Color::Cyan),
                Cell::new("Details").fg(Color::Cyan),
            ]);

        for m in &drift.modules_drift {
            let (change_cell, detail_str) = match &m.change_type {
                DriftChangeType::Added => (Cell::new("ADDED").fg(Color::Green), String::new()),
                DriftChangeType::Removed => (Cell::new("REMOVED").fg(Color::Red), String::new()),
                DriftChangeType::Modified { details } => (
                    Cell::new("MODIFIED").fg(Color::Yellow),
                    details.clone(),
                ),
            };
            table.add_row(vec![
                Cell::new(&m.name),
                change_cell,
                Cell::new(detail_str),
            ]);
        }
        out.push_str(&format!("{}\n\n", table));
    }

    // Plugins drift
    if !drift.plugins_drift.is_empty() {
        out.push_str(&format!("{}\n", "Plugin Interception Changes:".bold().underline()));
        for p in &drift.plugins_drift {
            match &p.change_type {
                DriftChangeType::Added => {
                    out.push_str(&format!("  + Added plugin '{}' on '{}' ({})\n", p.name.green(), p.target_class, p.module));
                }
                DriftChangeType::Removed => {
                    out.push_str(&format!("  - Removed plugin '{}' on '{}' ({})\n", p.name.red(), p.target_class, p.module));
                }
                DriftChangeType::Modified { details } => {
                    out.push_str(&format!("  ~ Modified plugin '{}' on '{}': {}\n", p.name.yellow(), p.target_class, details));
                }
            }
        }
        out.push('\n');
    }

    // Cron drift
    if !drift.cron_drift.is_empty() {
        out.push_str(&format!("{}\n", "Cron Job Changes:".bold().underline()));
        for c in &drift.cron_drift {
            match &c.change_type {
                DriftChangeType::Added => {
                    out.push_str(&format!("  + Added cron job '{}' ({})\n", c.job_code.green(), c.module));
                }
                DriftChangeType::Removed => {
                    out.push_str(&format!("  - Removed cron job '{}' ({})\n", c.job_code.red(), c.module));
                }
                DriftChangeType::Modified { details } => {
                    out.push_str(&format!("  ~ Modified cron job '{}': {}\n", c.job_code.yellow(), details));
                }
            }
        }
        out.push('\n');
    }

    // Schema drift
    if !drift.schema_drift.is_empty() {
        out.push_str(&format!("{}\n", "Database Schema Changes:".bold().underline()));
        for t in &drift.schema_drift.added_tables {
            out.push_str(&format!("  + Declared Table Added: {}\n", t.green()));
        }
        for t in &drift.schema_drift.dropped_tables {
            out.push_str(&format!("  - Declared Table Dropped: {}\n", t.red()));
        }
        for c in &drift.schema_drift.added_columns {
            out.push_str(&format!("  + Column Added: {}\n", c.green()));
        }
        for c in &drift.schema_drift.dropped_columns {
            out.push_str(&format!("  - Column Dropped: {}\n", c.red()));
        }
        out.push('\n');
    }

    // Environment drift
    if !drift.env_drift.is_empty() {
        out.push_str(&format!("{}\n", "Environment Config Changes (env.php):".bold().underline()));
        for env in &drift.env_drift {
            out.push_str(&format!("  ~ {}: {} -> {}\n", env.key.yellow(), env.old_value, env.new_value.bold()));
        }
        out.push('\n');
    }

    // Final Verdict
    if drift.has_regressions {
        out.push_str(&format!(
            "{}\n",
            "VERDICT: FAIL - Regressions or significant health score drop detected."
                .red()
                .bold()
        ));
    } else {
        out.push_str(&format!(
            "{}\n",
            "VERDICT: PASS - Installation matches or improves upon baseline health."
                .green()
                .bold()
        ));
    }

    out
}

/// Serialize drift report to JSON.
pub fn render_drift_json(drift: &DriftReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(drift)
}

/// Render a Markdown-formatted drift comparison report suitable for GitHub PR comments.
pub fn render_drift_markdown(drift: &DriftReport) -> String {
    let mut out = String::new();

    out.push_str("## 🩺 Magento Doctor Drift Report\n\n");

    let status_badge = if drift.has_regressions {
        "🔴 **Action Required: Regressions Detected**"
    } else {
        "🟢 **All Checks Passing: No Regressions**"
    };

    out.push_str(&format!("Status: {}\n\n", status_badge));

    let delta_sign = if drift.health_drift.delta_overall >= 0 { "+" } else { "" };
    out.push_str(&format!(
        "| Metric | Baseline | Current | Delta |\n|---|---|---|---|\n| **Overall Health** | {} | {} | {}{} |\n| **Critical Issues** | {} | {} | {:+} |\n| **Warnings** | {} | {} | {:+} |\n\n",
        drift.health_drift.baseline_overall,
        drift.health_drift.current_overall,
        delta_sign,
        drift.health_drift.delta_overall,
        drift.health_drift.baseline_critical,
        drift.health_drift.current_critical,
        drift.health_drift.delta_critical,
        drift.health_drift.baseline_warning,
        drift.health_drift.current_warning,
        drift.health_drift.delta_warning,
    ));

    if !drift.findings_drift.new_findings.is_empty() {
        out.push_str("### ⚠️ Newly Introduced Findings\n\n");
        for f in &drift.findings_drift.new_findings {
            out.push_str(&format!(
                "- **[{}] `{}`**: {}\n  - *Recommendation*: {}\n",
                f.severity, f.rule_id, f.title, f.recommendation
            ));
        }
        out.push('\n');
    }

    if !drift.findings_drift.resolved_findings.is_empty() {
        out.push_str("### 🎉 Resolved Issues\n\n");
        for f in &drift.findings_drift.resolved_findings {
            out.push_str(&format!("- ~~[{}] `{}`: {}~~\n", f.severity, f.rule_id, f.title));
        }
        out.push('\n');
    }

    if !drift.modules_drift.is_empty() {
        out.push_str("### 📦 Module Changes\n\n");
        for m in &drift.modules_drift {
            match &m.change_type {
                DriftChangeType::Added => out.push_str(&format!("- ➕ **{}** (Added)\n", m.name)),
                DriftChangeType::Removed => out.push_str(&format!("- ➖ **{}** (Removed)\n", m.name)),
                DriftChangeType::Modified { details } => {
                    out.push_str(&format!("- 🔄 **{}**: {}\n", m.name, details))
                }
            }
        }
        out.push('\n');
    }

    out
}
