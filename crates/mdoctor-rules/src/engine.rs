//! Central correlation engine executing all cross-analysis rules.

use std::path::Path;
use mdoctor_core::{Finding, MagentoInstallation, Severity};
use mdoctor_php::{AstFinding, PhpAstAnalyzer};
use walkdir::WalkDir;

use crate::rules::cache_rules::evaluate_cache_rules;
use crate::rules::cron_rules::evaluate_cron_rules;
use crate::rules::db_rules::evaluate_db_rules;
use crate::rules::di_rules::evaluate_di_rules;
use crate::rules::env_rules::evaluate_env_rules;
use crate::rules::perf_rules::evaluate_perf_rules;
use crate::rules::plugin_rules::evaluate_plugin_rules;

pub struct CrossAnalysisEngine;

impl CrossAnalysisEngine {
    /// Runs full cross-analysis against the normalized installation model.
    pub fn analyze(installation: &MagentoInstallation) -> Vec<Finding> {
        // 1. Run PHP AST analysis across app/code and custom vendor modules
        let ast_findings = scan_php_sources(&installation.root);

        // 2. Evaluate all rules
        let mut findings = Vec::new();

        findings.extend(evaluate_cron_rules(installation, &ast_findings));
        findings.extend(evaluate_plugin_rules(installation, &ast_findings));
        findings.extend(evaluate_di_rules(installation, &ast_findings));
        findings.extend(evaluate_perf_rules(&ast_findings));
        findings.extend(evaluate_db_rules(installation));
        findings.extend(evaluate_cache_rules(installation));
        findings.extend(evaluate_env_rules(installation));

        // 3. Sort findings: Critical first, then Warning, then Info
        findings.sort_by(|a, b| {
            let sev_order = match (a.severity, b.severity) {
                (Severity::Critical, Severity::Critical) => std::cmp::Ordering::Equal,
                (Severity::Critical, _) => std::cmp::Ordering::Less,
                (_, Severity::Critical) => std::cmp::Ordering::Greater,
                (Severity::Warning, Severity::Warning) => std::cmp::Ordering::Equal,
                (Severity::Warning, _) => std::cmp::Ordering::Less,
                (_, Severity::Warning) => std::cmp::Ordering::Greater,
                (Severity::Info, Severity::Info) => std::cmp::Ordering::Equal,
            };

            if sev_order == std::cmp::Ordering::Equal {
                a.rule_id.cmp(&b.rule_id)
            } else {
                sev_order
            }
        });

        findings
    }
}

/// Scans PHP files in app/code and relevant vendor directories.
fn scan_php_sources(root: &Path) -> Vec<AstFinding> {
    let mut findings = Vec::new();
    let mut analyzer = PhpAstAnalyzer::new();

    let scan_dirs = [root.join("app/code"), root.join("vendor")];

    for dir in scan_dirs {
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(dir)
            .max_depth(8)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "php" {
                        let path_str = entry.path().to_string_lossy();
                        // Ignore test files to prevent false alarms
                        if path_str.contains("/Test/") || path_str.contains("/tests/") {
                            continue;
                        }

                        if let Ok(ast_f) = analyzer.analyze_file(entry.path()) {
                            findings.extend(ast_f);
                        }
                    }
                }
            }
        }
    }

    findings
}
