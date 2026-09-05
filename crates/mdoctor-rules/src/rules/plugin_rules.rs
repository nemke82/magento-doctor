//! Plugin analysis rules (MD-PLG-001, MD-PLG-005).

use std::collections::HashMap;
use mdoctor_core::{Category, Confidence, Finding, MagentoInstallation, PluginType, Severity};
use mdoctor_knowledge::hotpaths::{is_hot_method, HotPathWeight};
use mdoctor_php::{AstFinding, OperationType};

pub fn evaluate_plugin_rules(
    installation: &MagentoInstallation,
    ast_findings: &[AstFinding],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    // 1. MD-PLG-001: Around plugin on hot path
    for plugin in &installation.plugins {
        if plugin.is_disabled {
            continue;
        }

        if plugin.plugin_type == PluginType::Around {
            if let Some(weight) = is_hot_method(&plugin.target_class, None) {
                // Find any AST findings in this plugin class
                let matching_ast: Vec<_> = ast_findings
                    .iter()
                    .filter(|f| {
                        f.class_name
                            .as_deref()
                            .map(|c| plugin.plugin_class.ends_with(c))
                            .unwrap_or(false)
                    })
                    .collect();

                let has_http = matching_ast.iter().any(|a| a.operation == OperationType::HttpRequest);
                let has_db_write = matching_ast.iter().any(|a| a.operation == OperationType::DatabaseWrite);

                let severity = if weight == HotPathWeight::Critical && (has_http || has_db_write) {
                    Severity::Critical
                } else {
                    Severity::Warning
                };

                let mut finding = Finding::new(
                    "MD-PLG-001",
                    format!("Around plugin on hot path: {}", plugin.target_class),
                    severity,
                    Confidence::High,
                    Category::Plugins,
                );

                finding.summary = format!(
                    "Plugin '{}' intercepts hot path '{}' using an around wrapper in module '{}'.",
                    plugin.name, plugin.target_class, plugin.module
                );

                finding.evidence.push(format!("Plugin class: {}", plugin.plugin_class));
                finding.evidence.push(format!("Intercepts: {}", plugin.target_class));
                finding.evidence.push(format!("Plugin type: around (sortOrder: {})", plugin.sort_order));
                finding.evidence.push(format!("Source: {}:{}", plugin.source_file.display(), plugin.line));

                if !matching_ast.is_empty() {
                    finding.evidence.push("AST static analysis detected costly operations inside this plugin:".to_string());
                    for ast in matching_ast {
                        finding.evidence.push(format!("  - {} at line {} ({})", ast.operation, ast.line_number, ast.call_signature));
                    }
                }

                finding.impact = "Around plugins introduce overhead by modifying call chains and prevent optimizations. If it fails to invoke proceed() or executes slow I/O, it directly degrades customer request latency.".to_string();
                finding.recommendation = "Refactor around plugin to 'before' or 'after' plugin, avoid synchronous network calls or heavy DB operations, and ensure $proceed() is called cleanly.".to_string();
                finding.related_modules.push(plugin.module.clone());
                finding.related_files.push(plugin.source_file.display().to_string());

                findings.push(finding);
            }
        }
    }

    // 2. MD-PLG-005: Duplicate sortOrder collisions
    let mut target_plugins: HashMap<(String, i32), Vec<&mdoctor_core::Plugin>> = HashMap::new();
    for plugin in &installation.plugins {
        if !plugin.is_disabled {
            target_plugins
                .entry((plugin.target_class.clone(), plugin.sort_order))
                .or_default()
                .push(plugin);
        }
    }

    for ((target_class, sort_order), group) in target_plugins {
        if group.len() > 1 && sort_order > 0 {
            let mut finding = Finding::new(
                "MD-PLG-005",
                format!("Plugin sortOrder conflict on '{}'", target_class),
                Severity::Warning,
                Confidence::High,
                Category::Plugins,
            );

            finding.summary = format!(
                "{} plugins intercept '{}' with identical sortOrder {}.",
                group.len(), target_class, sort_order
            );

            for p in &group {
                finding.evidence.push(format!(
                    "  - '{}' ({}) from module '{}' ({}:{})",
                    p.name, p.plugin_class, p.module, p.source_file.display(), p.line
                ));
            }

            finding.impact = "When sortOrder values match, execution sequence depends entirely on module load order, leading to non-deterministic behavior and intermittent bugs.".to_string();
            finding.recommendation = "Assign distinct sortOrder values in di.xml to guarantee predictable plugin execution sequence.".to_string();
            for p in &group {
                finding.related_modules.push(p.module.clone());
                finding.related_files.push(p.source_file.display().to_string());
            }

            findings.push(finding);
        }
    }

    findings
}
