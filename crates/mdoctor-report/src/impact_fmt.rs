//! Formatter for module performance impact and architectural risk rankings.

use colored::*;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use mdoctor_core::{ImpactLevel, ModuleImpactScore};

/// Renders an ANSI table of module impact rankings with risk driver summaries.
pub fn render_impact_table(impacts: &[ModuleImpactScore]) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "\n{}\n\n",
        "=== MODULE ARCHITECTURAL RISK & PERFORMANCE IMPACT RANKINGS ==="
            .bold()
            .cyan()
    ));

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Module").fg(Color::Cyan),
            Cell::new("Classification").fg(Color::Cyan),
            Cell::new("Impact Level").fg(Color::Cyan),
            Cell::new("Score").fg(Color::Cyan),
            Cell::new("Primary Risk Drivers").fg(Color::Cyan),
        ]);

    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;

    for impact in impacts {
        let (level_cell, score_cell) = match impact.level {
            ImpactLevel::Critical => {
                critical_count += 1;
                (
                    Cell::new("CRITICAL").fg(Color::Magenta),
                    Cell::new(format!("{}/100", impact.score)).fg(Color::Magenta),
                )
            }
            ImpactLevel::High => {
                high_count += 1;
                (
                    Cell::new("HIGH").fg(Color::Red),
                    Cell::new(format!("{}/100", impact.score)).fg(Color::Red),
                )
            }
            ImpactLevel::Medium => {
                medium_count += 1;
                (
                    Cell::new("MEDIUM").fg(Color::Yellow),
                    Cell::new(format!("{}/100", impact.score)).fg(Color::Yellow),
                )
            }
            ImpactLevel::Low => {
                low_count += 1;
                (
                    Cell::new("LOW").fg(Color::Green),
                    Cell::new(format!("{}/100", impact.score)).fg(Color::Green),
                )
            }
        };

        table.add_row(vec![
            Cell::new(&impact.module_name),
            Cell::new(format!("{}", impact.classification)),
            level_cell,
            score_cell,
            Cell::new(impact.top_drivers_summary()),
        ]);
    }

    out.push_str(&format!("{}\n\n", table));

    out.push_str(&format!(
        "Evaluated {} modules: {} Critical, {} High, {} Medium, {} Low impact.\n",
        impacts.len(),
        critical_count.to_string().magenta().bold(),
        high_count.to_string().red().bold(),
        medium_count.to_string().yellow().bold(),
        low_count.to_string().green().bold(),
    ));

    out
}
