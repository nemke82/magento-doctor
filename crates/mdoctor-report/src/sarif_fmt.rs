//! SARIF report generator for GitHub Code Scanning.

use mdoctor_core::{Finding, Severity, CALVER_VERSION};
use serde_json::json;

pub fn render_sarif_report(findings: &[Finding]) -> String {
    let rules: Vec<_> = findings
        .iter()
        .map(|f| {
            json!({
                "id": f.rule_id,
                "name": f.title,
                "shortDescription": {
                    "text": f.title
                },
                "fullDescription": {
                    "text": f.summary
                },
                "help": {
                    "text": format!("{}\n\nRecommendation:\n{}", f.impact, f.recommendation)
                }
            })
        })
        .collect();

    let results: Vec<_> = findings
        .iter()
        .map(|f| {
            let level = match f.severity {
                Severity::Critical => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
            };

            let file_uri = f.related_files.first().cloned().unwrap_or_else(|| "app/etc/env.php".to_string());

            json!({
                "ruleId": f.rule_id,
                "level": level,
                "message": {
                    "text": f.summary
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": file_uri
                        },
                        "region": {
                            "startLine": 1
                        }
                    }
                }]
            })
        })
        .collect();

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "Magento Doctor",
                    "semanticVersion": CALVER_VERSION,
                    "informationUri": "https://github.com/nemke82/magento-doctor",
                    "rules": rules
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
}
