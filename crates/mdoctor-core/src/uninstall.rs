//! Module uninstall impact forensics and decommissioning safety analysis.

use serde::{Deserialize, Serialize};
use crate::model::*;

/// Safety assessment for uninstalling/disabling a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UninstallSafety {
    /// No dependents and no persistent DB footprint. Safe to disable.
    Safe,
    /// Leaves orphaned tables or injected columns behind. Review DB cleanup.
    Caution,
    /// Active modules depend on this module in <sequence>. Disabling will break Magento!
    Blocked,
}

impl std::fmt::Display for UninstallSafety {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UninstallSafety::Safe => write!(f, "SAFE"),
            UninstallSafety::Caution => write!(f, "CAUTION"),
            UninstallSafety::Blocked => write!(f, "BLOCKED"),
        }
    }
}

/// An installed module that depends on the target module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependentModule {
    pub name: String,
    pub classification: ModuleClassification,
    pub is_enabled: bool,
}

/// Details of database tables or columns affected by uninstalling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableImpact {
    pub table_name: String,
    pub is_custom_table: bool,
    pub affected_columns: Vec<String>,
    pub row_count: Option<u64>,
    pub total_bytes: Option<u64>,
}

/// Full forensic uninstall impact report for a module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallAnalysis {
    pub target_module: String,
    pub safety: UninstallSafety,
    pub dependents: Vec<DependentModule>,
    pub orphaned_tables: Vec<String>,
    pub altered_tables: Vec<TableImpact>,
    pub intercepted_classes: Vec<String>,
    pub cron_jobs_affected: Vec<String>,
    pub observers_affected: Vec<String>,
    pub decommission_steps: Vec<String>,
}

/// Calculate the uninstall impact of disabling or removing a module.
pub fn calculate_uninstall_impact(
    installation: &MagentoInstallation,
    target_module_name: &str,
) -> Option<UninstallAnalysis> {
    let target_mod = installation.find_module(target_module_name)?;

    // 1. Identify dependents: modules declaring target_module in their <sequence>
    let mut dependents = Vec::new();
    for m in &installation.modules {
        if m.name != target_module_name && m.sequence.contains(&target_module_name.to_string()) {
            dependents.push(DependentModule {
                name: m.name.clone(),
                classification: m.classification,
                is_enabled: m.is_enabled,
            });
        }
    }
    dependents.sort_by(|a, b| a.name.cmp(&b.name));

    // 2. Identify orphaned custom tables owned by this module
    let mut orphaned_tables = Vec::new();
    let mut altered_tables = Vec::new();

    for (t_name, table) in &installation.declared_schema.tables {
        let is_owned = table.owning_module.as_deref() == Some(target_module_name);
        if is_owned {
            orphaned_tables.push(t_name.clone());

            // Check if live metrics has size stats
            let stat = installation
                .database_metrics
                .table_sizes
                .iter()
                .find(|s| s.table_name.eq_ignore_ascii_case(t_name));

            altered_tables.push(TableImpact {
                table_name: t_name.clone(),
                is_custom_table: true,
                affected_columns: table.columns.keys().cloned().collect(),
                row_count: stat.map(|s| s.row_count),
                total_bytes: stat.map(|s| s.total_bytes),
            });
        }
    }
    orphaned_tables.sort();

    // 3. Intercepted classes (plugins + preferences)
    let mut intercepted = Vec::new();
    for p in &installation.plugins {
        if p.module == target_module_name && !intercepted.contains(&p.target_class) {
            intercepted.push(p.target_class.clone());
        }
    }
    for pref in &installation.preferences {
        if pref.module == target_module_name && !intercepted.contains(&pref.for_class) {
            intercepted.push(pref.for_class.clone());
        }
    }
    intercepted.sort();

    // 4. Affected cron jobs
    let mut cron_jobs_affected = Vec::new();
    for c in &installation.cron_jobs {
        if c.module == target_module_name {
            cron_jobs_affected.push(c.name.clone());
        }
    }

    // 5. Affected observers
    let mut observers_affected = Vec::new();
    for o in &installation.observers {
        if o.module == target_module_name {
            observers_affected.push(format!("{} -> {}", o.event_name, o.name));
        }
    }

    // 6. Assess safety
    let has_active_dependents = dependents.iter().any(|d| d.is_enabled);
    let safety = if has_active_dependents {
        UninstallSafety::Blocked
    } else if !orphaned_tables.is_empty() || !altered_tables.is_empty() {
        UninstallSafety::Caution
    } else {
        UninstallSafety::Safe
    };

    // 7. Decommissioning plan
    let mut steps = Vec::new();
    if has_active_dependents {
        let active_deps: Vec<_> = dependents
            .iter()
            .filter(|d| d.is_enabled)
            .map(|d| d.name.as_str())
            .collect();
        steps.push(format!(
            "BLOCKED: You must first disable or refactor active dependent modules: {}",
            active_deps.join(", ")
        ));
        steps.push(format!(
            "Execute: bin/magento module:disable {}",
            active_deps.join(" ")
        ));
    }

    steps.push(format!(
        "Disable module: bin/magento module:disable {}",
        target_mod.name
    ));
    steps.push("Compile DI and upgrade schema: bin/magento setup:upgrade && bin/magento setup:di:compile".to_string());

    if !orphaned_tables.is_empty() {
        steps.push(format!(
            "Database Cleanup: Module declares {} custom table(s) ({}) that will remain in MySQL.",
            orphaned_tables.len(),
            orphaned_tables.join(", ")
        ));
        steps.push("If removing permanently, back up and drop custom tables using a data patch or DB migration.".to_string());
    }

    if let Some(pkg) = &target_mod.package_name {
        steps.push(format!("Remove Composer package: composer remove {}", pkg));
    }

    Some(UninstallAnalysis {
        target_module: target_mod.name.clone(),
        safety,
        dependents,
        orphaned_tables,
        altered_tables,
        intercepted_classes: intercepted,
        cron_jobs_affected,
        observers_affected,
        decommission_steps: steps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_test_installation() -> MagentoInstallation {
        let mut inst = MagentoInstallation::new(PathBuf::from("/var/www/html"));

        let mod_a = Module {
            name: "Vendor_Base".to_string(),
            vendor: "Vendor".to_string(),
            package_name: Some("vendor/module-base".to_string()),
            version: Some("1.0.0".to_string()),
            sequence: vec![],
            path: PathBuf::new(),
            is_enabled: true,
            classification: ModuleClassification::ComposerThirdParty,
            footprint: ModuleFootprint::default(),
        };

        let mod_b = Module {
            name: "Vendor_Extension".to_string(),
            vendor: "Vendor".to_string(),
            package_name: Some("vendor/module-extension".to_string()),
            version: Some("1.0.0".to_string()),
            sequence: vec!["Vendor_Base".to_string()],
            path: PathBuf::new(),
            is_enabled: true,
            classification: ModuleClassification::ComposerThirdParty,
            footprint: ModuleFootprint::default(),
        };

        inst.modules.push(mod_a);
        inst.modules.push(mod_b);
        inst
    }

    #[test]
    fn test_uninstall_blocked_by_active_sequence_dependent() {
        let inst = make_test_installation();
        let analysis = calculate_uninstall_impact(&inst, "Vendor_Base").expect("Found");
        assert_eq!(analysis.safety, UninstallSafety::Blocked);
        assert_eq!(analysis.dependents.len(), 1);
        assert_eq!(analysis.dependents[0].name, "Vendor_Extension");
    }

    #[test]
    fn test_uninstall_safe_for_leaf_module() {
        let inst = make_test_installation();
        let analysis = calculate_uninstall_impact(&inst, "Vendor_Extension").expect("Found");
        assert_eq!(analysis.safety, UninstallSafety::Safe);
        assert!(analysis.dependents.is_empty());
    }
}
