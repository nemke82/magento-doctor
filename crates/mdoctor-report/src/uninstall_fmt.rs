//! Formatter for module uninstallation blast-radius and forensics.

use colored::*;
use mdoctor_core::{UninstallAnalysis, UninstallSafety};

/// Renders a terminal-friendly ANSI uninstall impact and safety analysis report.
pub fn render_uninstall_terminal(analysis: &UninstallAnalysis) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "\n{}\n\n",
        "=== MODULE UNINSTALL IMPACT & FORENSICS ==="
            .bold()
            .cyan()
    ));

    let (badge, explanation) = match analysis.safety {
        UninstallSafety::Blocked => (
            "[BLOCKED]".red().bold(),
            "Do NOT disable this module immediately! Active modules depend on it."
                .red()
                .bold(),
        ),
        UninstallSafety::Caution => (
            "[CAUTION]".yellow().bold(),
            "Safe to disable from runtime, but leaves orphaned database tables or columns."
                .yellow(),
        ),
        UninstallSafety::Safe => (
            "[SAFE]".green().bold(),
            "Safe to disable and remove. No active dependents or persistent schema changes."
                .green(),
        ),
    };

    out.push_str(&format!("Module: {}\n", analysis.target_module.bold()));
    out.push_str(&format!("Uninstall Safety: {} - {}\n\n", badge, explanation));

    // 1. Dependents
    if !analysis.dependents.is_empty() {
        out.push_str(&format!(
            "{}\n",
            "⚠️  ACTIVE DEPENDENT MODULES (<sequence> BREAKS):"
                .red()
                .bold()
                .underline()
        ));
        out.push_str("Disabling this module without disabling the following will cause `bin/magento setup:di:compile` to fail:\n");
        for dep in &analysis.dependents {
            let status = if dep.is_enabled {
                "ENABLED".green().bold()
            } else {
                "DISABLED".dimmed()
            };
            out.push_str(&format!("  • {:<35} ({}, {})\n", dep.name.bold(), dep.classification, status));
        }
        out.push('\n');
    }

    // 2. Database Footprint
    if !analysis.orphaned_tables.is_empty() {
        out.push_str(&format!(
            "{}\n",
            "🗄️  ORPHANED DATABASE TABLES:"
                .yellow()
                .bold()
                .underline()
        ));
        out.push_str("The following custom database tables were declared by this module and will remain in MySQL:\n");
        for table in &analysis.altered_tables {
            let size_str = if let (Some(rows), Some(bytes)) = (table.row_count, table.total_bytes) {
                let mb = bytes / (1024 * 1024);
                format!("({} rows, {} MB)", rows, mb)
            } else {
                format!("({} columns)", table.affected_columns.len())
            };
            out.push_str(&format!("  • {:<35} {}\n", table.table_name.yellow(), size_str));
        }
        out.push('\n');
    }

    // 3. Intercepted classes
    if !analysis.intercepted_classes.is_empty() {
        out.push_str(&format!(
            "{}\n",
            "🔌 INTERCEPTED CORE LOGIC (PLUGINS & PREFERENCES):"
                .bold()
                .underline()
        ));
        out.push_str("The following core/third-party classes will revert to their base behavior when disabled:\n");
        for cls in &analysis.intercepted_classes {
            out.push_str(&format!("  • {}\n", cls.cyan()));
        }
        out.push('\n');
    }

    // 4. Cron jobs & observers
    if !analysis.cron_jobs_affected.is_empty() || !analysis.observers_affected.is_empty() {
        out.push_str(&format!(
            "{}\n",
            "⏱️  SCHEDULED & EVENT HOOKS DEACTIVATED:"
                .bold()
                .underline()
        ));
        for cron in &analysis.cron_jobs_affected {
            out.push_str(&format!("  • Cron: {}\n", cron));
        }
        for obs in &analysis.observers_affected {
            out.push_str(&format!("  • Observer: {}\n", obs));
        }
        out.push('\n');
    }

    // 5. Recommended Decommissioning Steps
    out.push_str(&format!(
        "{}\n",
        "📋 RECOMMENDED SAFE REMOVAL PLAN:"
            .green()
            .bold()
            .underline()
    ));
    for (i, step) in analysis.decommission_steps.iter().enumerate() {
        out.push_str(&format!("  {}. {}\n", i + 1, step));
    }
    out.push('\n');

    out
}
