//! Dependency Injection rules (MD-DI-001, MD-DI-005).

use std::collections::HashMap;
use mdoctor_core::{Category, Confidence, Finding, MagentoInstallation, Severity};
use mdoctor_php::{AstFinding, OperationType};

pub fn evaluate_di_rules(
    installation: &MagentoInstallation,
    ast_findings: &[AstFinding],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    // 1. MD-DI-001: Core class replacement via preference
    let mut pref_targets: HashMap<String, Vec<&mdoctor_core::Preference>> = HashMap::new();
    for pref in &installation.preferences {
        pref_targets
            .entry(pref.for_class.clone())
            .or_default()
            .push(pref);
    }

    for (for_class, prefs) in pref_targets {
        // Skip preferences declared by Magento Core modules (e.g. Magento_Csp replacing Magento_Deploy)
        let custom_prefs: Vec<_> = prefs
            .into_iter()
            .filter(|p| !p.module.starts_with("Magento_") && !p.source_file.to_string_lossy().contains("/vendor/magento/"))
            .collect();

        if custom_prefs.is_empty() {
            continue;
        }

        let is_core_target = for_class.starts_with("Magento\\");
        let is_concrete = !for_class.ends_with("Interface") && !for_class.contains("\\Api\\");

        if is_core_target && is_concrete {
            let mut finding = Finding::new(
                "MD-DI-001",
                format!("Core concrete class replaced via preference: {}", for_class),
                Severity::Warning,
                Confidence::High,
                Category::DependencyInjection,
            );

            finding.summary = format!(
                "Preference replaces core concrete class '{}' with custom implementation.",
                for_class
            );

            for p in &custom_prefs {
                finding.evidence.push(format!(
                    "  - Replaced with '{}' by module '{}' ({}:{})",
                    p.type_class, p.module, p.source_file.display(), p.line
                ));
            }

            if custom_prefs.len() > 1 {
                finding.severity = Severity::Critical;
                finding.evidence.push(format!("CRITICAL: Multiple modules ({}) replace the exact same target class!", custom_prefs.len()));
            }

            finding.impact = "Replacing concrete classes breaks polymorphism, bypasses core bug fixes, and creates high risk of incompatibilities during Magento upgrades.".to_string();
            finding.recommendation = "Replace <preference> with plugins (before, around, after) or composition where possible.".to_string();
            for p in &custom_prefs {
                finding.related_modules.push(p.module.clone());
                finding.related_files.push(p.source_file.display().to_string());
            }

            findings.push(finding);
        }
    }

    // 2. MD-DI-005: Direct ObjectManager usage in source code
    for ast in ast_findings {
        if ast.operation == OperationType::ObjectManagerUsage {
            let mut finding = Finding::new(
                "MD-DI-005",
                "Direct ObjectManager usage detected",
                Severity::Warning,
                Confidence::High,
                Category::DependencyInjection,
            );

            finding.summary = format!(
                "Direct invocation of ObjectManager::getInstance() at line {}.",
                ast.line_number
            );

            finding.evidence.push(format!("Snippet: {}", ast.code_snippet));
            if let Some(cls) = &ast.class_name {
                finding.evidence.push(format!("Class: {}", cls));
            }
            if let Some(m) = &ast.method_name {
                finding.evidence.push(format!("Method: {}", m));
            }

            finding.impact = "Hides class dependencies, breaks dependency inversion, complicates unit testing, and violates Magento architecture guidelines.".to_string();
            finding.recommendation = "Inject the required dependency via constructor parameter instead of using ObjectManager::getInstance().".to_string();

            findings.push(finding);
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use mdoctor_core::Preference;

    #[test]
    fn test_core_preferences_are_ignored() {
        let mut installation = MagentoInstallation::new(PathBuf::from("/tmp"));
        installation.preferences.push(Preference {
            for_class: "Magento\\Deploy\\Package\\Processor\\PostProcessor\\Map".to_string(),
            type_class: "Magento\\Csp\\Model\\Deploy\\Package\\Processor\\PostProcessor\\Map".to_string(),
            module: "Magento_Csp".to_string(),
            area: "global".to_string(),
            source_file: PathBuf::from("vendor/magento/module-csp/etc/di.xml"),
            line: 131,
        });

        let findings = evaluate_di_rules(&installation, &[]);
        assert!(findings.is_empty(), "Core preferences must not trigger MD-DI-001");
    }

    #[test]
    fn test_custom_module_preference_reported() {
        let mut installation = MagentoInstallation::new(PathBuf::from("/tmp"));
        installation.preferences.push(Preference {
            for_class: "Magento\\Catalog\\Model\\Product".to_string(),
            type_class: "Custom\\Catalog\\Model\\Product".to_string(),
            module: "Custom_Catalog".to_string(),
            area: "global".to_string(),
            source_file: PathBuf::from("app/code/Custom/Catalog/etc/di.xml"),
            line: 25,
        });

        let findings = evaluate_di_rules(&installation, &[]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "MD-DI-001");
    }
}
