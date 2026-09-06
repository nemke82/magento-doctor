//! Configuration and diagnostic drift detection comparing two store snapshots.

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::finding::{Finding, Severity};
use crate::model::*;
use crate::snapshot::DiagnosticSnapshot;

/// Type of change observed in an entity between baseline and current state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftChangeType {
    Added,
    Removed,
    Modified { details: String },
}

/// Drift observed in high-level store metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetadataDrift {
    pub edition_changed: Option<(Edition, Edition)>,
    pub version_changed: Option<(Option<String>, Option<String>)>,
    pub mode_changed: Option<(MagentoMode, MagentoMode)>,
}

impl MetadataDrift {
    pub fn is_empty(&self) -> bool {
        self.edition_changed.is_none()
            && self.version_changed.is_none()
            && self.mode_changed.is_none()
    }
}

/// Module drift entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDrift {
    pub name: String,
    pub change_type: DriftChangeType,
}

/// Plugin drift entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDrift {
    pub name: String,
    pub target_class: String,
    pub module: String,
    pub change_type: DriftChangeType,
}

/// Preference drift entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceDrift {
    pub for_class: String,
    pub type_class: String,
    pub module: String,
    pub change_type: DriftChangeType,
}

/// Cron job drift entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronDrift {
    pub job_code: String,
    pub module: String,
    pub change_type: DriftChangeType,
}

/// Environment / env.php drift entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvDrift {
    pub key: String,
    pub old_value: String,
    pub new_value: String,
}

/// Database schema drift entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaDrift {
    pub added_tables: Vec<String>,
    pub dropped_tables: Vec<String>,
    pub added_columns: Vec<String>,
    pub dropped_columns: Vec<String>,
    pub added_indexes: Vec<String>,
    pub dropped_indexes: Vec<String>,
}

impl SchemaDrift {
    pub fn is_empty(&self) -> bool {
        self.added_tables.is_empty()
            && self.dropped_tables.is_empty()
            && self.added_columns.is_empty()
            && self.dropped_columns.is_empty()
            && self.added_indexes.is_empty()
            && self.dropped_indexes.is_empty()
    }
}

/// Findings drift comparing diagnostics between two runs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FindingsDrift {
    pub new_findings: Vec<Finding>,
    pub resolved_findings: Vec<Finding>,
    pub persistent_findings: Vec<Finding>,
}

/// Health score drift.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthDrift {
    pub baseline_overall: u32,
    pub current_overall: u32,
    pub delta_overall: i32,
    pub baseline_critical: usize,
    pub current_critical: usize,
    pub delta_critical: i64,
    pub baseline_warning: usize,
    pub current_warning: usize,
    pub delta_warning: i64,
}

/// Complete configuration and diagnostic drift report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub baseline_time: DateTime<Utc>,
    pub current_time: DateTime<Utc>,
    pub metadata_drift: MetadataDrift,
    pub modules_drift: Vec<ModuleDrift>,
    pub plugins_drift: Vec<PluginDrift>,
    pub preferences_drift: Vec<PreferenceDrift>,
    pub cron_drift: Vec<CronDrift>,
    pub env_drift: Vec<EnvDrift>,
    pub schema_drift: SchemaDrift,
    pub findings_drift: FindingsDrift,
    pub health_drift: HealthDrift,
    pub has_regressions: bool,
    pub max_regression_severity: Option<Severity>,
}

/// Compare a baseline diagnostic snapshot against a current snapshot.
pub fn compare_installations(
    baseline: &DiagnosticSnapshot,
    current: &DiagnosticSnapshot,
) -> DriftReport {
    let mut meta_drift = MetadataDrift::default();
    if baseline.installation.edition != current.installation.edition {
        meta_drift.edition_changed = Some((
            baseline.installation.edition,
            current.installation.edition,
        ));
    }
    if baseline.installation.version != current.installation.version {
        meta_drift.version_changed = Some((
            baseline.installation.version.clone(),
            current.installation.version.clone(),
        ));
    }
    if baseline.installation.mode != current.installation.mode {
        meta_drift.mode_changed = Some((
            baseline.installation.mode,
            current.installation.mode,
        ));
    }

    // 1. Modules comparison
    let mut modules_drift = Vec::new();
    let b_modules: HashMap<&str, &Module> = baseline
        .installation
        .modules
        .iter()
        .map(|m| (m.name.as_str(), m))
        .collect();
    let c_modules: HashMap<&str, &Module> = current
        .installation
        .modules
        .iter()
        .map(|m| (m.name.as_str(), m))
        .collect();

    for (name, c_mod) in &c_modules {
        match b_modules.get(name) {
            None => {
                modules_drift.push(ModuleDrift {
                    name: name.to_string(),
                    change_type: DriftChangeType::Added,
                });
            }
            Some(b_mod) => {
                let mut changes = Vec::new();
                if b_mod.version != c_mod.version {
                    changes.push(format!(
                        "version: {} -> {}",
                        b_mod.version.as_deref().unwrap_or("unknown"),
                        c_mod.version.as_deref().unwrap_or("unknown")
                    ));
                }
                if b_mod.is_enabled != c_mod.is_enabled {
                    changes.push(format!(
                        "status: {} -> {}",
                        if b_mod.is_enabled { "enabled" } else { "disabled" },
                        if c_mod.is_enabled { "enabled" } else { "disabled" }
                    ));
                }
                if !changes.is_empty() {
                    modules_drift.push(ModuleDrift {
                        name: name.to_string(),
                        change_type: DriftChangeType::Modified {
                            details: changes.join(", "),
                        },
                    });
                }
            }
        }
    }
    for name in b_modules.keys() {
        if !c_modules.contains_key(name) {
            modules_drift.push(ModuleDrift {
                name: name.to_string(),
                change_type: DriftChangeType::Removed,
            });
        }
    }
    modules_drift.sort_by(|a, b| a.name.cmp(&b.name));

    // 2. Plugins comparison
    let mut plugins_drift = Vec::new();
    let b_plugins: HashMap<String, &Plugin> = baseline
        .installation
        .plugins
        .iter()
        .map(|p| (format!("{}:{}", p.target_class, p.name), p))
        .collect();
    let c_plugins: HashMap<String, &Plugin> = current
        .installation
        .plugins
        .iter()
        .map(|p| (format!("{}:{}", p.target_class, p.name), p))
        .collect();

    for (key, c_p) in &c_plugins {
        match b_plugins.get(key) {
            None => {
                plugins_drift.push(PluginDrift {
                    name: c_p.name.clone(),
                    target_class: c_p.target_class.clone(),
                    module: c_p.module.clone(),
                    change_type: DriftChangeType::Added,
                });
            }
            Some(b_p) => {
                let mut changes = Vec::new();
                if b_p.plugin_type != c_p.plugin_type {
                    changes.push(format!("type: {} -> {}", b_p.plugin_type, c_p.plugin_type));
                }
                if b_p.is_disabled != c_p.is_disabled {
                    changes.push(format!(
                        "disabled: {} -> {}",
                        b_p.is_disabled, c_p.is_disabled
                    ));
                }
                if b_p.cost_indicators != c_p.cost_indicators {
                    changes.push(format!(
                        "cost indicators: [{}] -> [{}]",
                        b_p.cost_indicators.join(", "),
                        c_p.cost_indicators.join(", ")
                    ));
                }
                if !changes.is_empty() {
                    plugins_drift.push(PluginDrift {
                        name: c_p.name.clone(),
                        target_class: c_p.target_class.clone(),
                        module: c_p.module.clone(),
                        change_type: DriftChangeType::Modified {
                            details: changes.join(", "),
                        },
                    });
                }
            }
        }
    }
    for (key, b_p) in &b_plugins {
        if !c_plugins.contains_key(key) {
            plugins_drift.push(PluginDrift {
                name: b_p.name.clone(),
                target_class: b_p.target_class.clone(),
                module: b_p.module.clone(),
                change_type: DriftChangeType::Removed,
            });
        }
    }

    // 3. Preferences comparison
    let mut preferences_drift = Vec::new();
    let b_prefs: HashMap<&str, &Preference> = baseline
        .installation
        .preferences
        .iter()
        .map(|p| (p.for_class.as_str(), p))
        .collect();
    let c_prefs: HashMap<&str, &Preference> = current
        .installation
        .preferences
        .iter()
        .map(|p| (p.for_class.as_str(), p))
        .collect();

    for (for_class, c_pref) in &c_prefs {
        match b_prefs.get(for_class) {
            None => {
                preferences_drift.push(PreferenceDrift {
                    for_class: for_class.to_string(),
                    type_class: c_pref.type_class.clone(),
                    module: c_pref.module.clone(),
                    change_type: DriftChangeType::Added,
                });
            }
            Some(b_pref) => {
                if b_pref.type_class != c_pref.type_class {
                    preferences_drift.push(PreferenceDrift {
                        for_class: for_class.to_string(),
                        type_class: c_pref.type_class.clone(),
                        module: c_pref.module.clone(),
                        change_type: DriftChangeType::Modified {
                            details: format!(
                                "type: {} -> {}",
                                b_pref.type_class, c_pref.type_class
                            ),
                        },
                    });
                }
            }
        }
    }
    for (for_class, b_pref) in &b_prefs {
        if !c_prefs.contains_key(for_class) {
            preferences_drift.push(PreferenceDrift {
                for_class: for_class.to_string(),
                type_class: b_pref.type_class.clone(),
                module: b_pref.module.clone(),
                change_type: DriftChangeType::Removed,
            });
        }
    }

    // 4. Cron comparison
    let mut cron_drift = Vec::new();
    let b_crons: HashMap<&str, &CronJob> = baseline
        .installation
        .cron_jobs
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    let c_crons: HashMap<&str, &CronJob> = current
        .installation
        .cron_jobs
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();

    for (name, c_cron) in &c_crons {
        match b_crons.get(name) {
            None => {
                cron_drift.push(CronDrift {
                    job_code: name.to_string(),
                    module: c_cron.module.clone(),
                    change_type: DriftChangeType::Added,
                });
            }
            Some(b_cron) => {
                if b_cron.schedule != c_cron.schedule {
                    cron_drift.push(CronDrift {
                        job_code: name.to_string(),
                        module: c_cron.module.clone(),
                        change_type: DriftChangeType::Modified {
                            details: format!(
                                "schedule: {} -> {}",
                                b_cron.schedule.as_deref().unwrap_or("none"),
                                c_cron.schedule.as_deref().unwrap_or("none")
                            ),
                        },
                    });
                }
            }
        }
    }
    for (name, b_cron) in &b_crons {
        if !c_crons.contains_key(name) {
            cron_drift.push(CronDrift {
                job_code: name.to_string(),
                module: b_cron.module.clone(),
                change_type: DriftChangeType::Removed,
            });
        }
    }

    // 5. Env drift (sanitized env config)
    let mut env_drift = Vec::new();
    let b_env = &baseline.installation.env_config;
    let c_env = &current.installation.env_config;

    let check_env_pair = |key: &str, old_val: Option<&String>, new_val: Option<&String>, out: &mut Vec<EnvDrift>| {
        if old_val != new_val {
            out.push(EnvDrift {
                key: key.to_string(),
                old_value: old_val.map(|s| s.as_str()).unwrap_or("(none)").to_string(),
                new_value: new_val.map(|s| s.as_str()).unwrap_or("(none)").to_string(),
            });
        }
    };
    check_env_pair("db_host", b_env.db_host.as_ref(), c_env.db_host.as_ref(), &mut env_drift);
    check_env_pair("db_name", b_env.db_name.as_ref(), c_env.db_name.as_ref(), &mut env_drift);
    check_env_pair("db_user", b_env.db_user.as_ref(), c_env.db_user.as_ref(), &mut env_drift);
    check_env_pair("redis_cache_host", b_env.redis_cache_host.as_ref(), c_env.redis_cache_host.as_ref(), &mut env_drift);
    check_env_pair("redis_cache_db", b_env.redis_cache_db.as_ref(), c_env.redis_cache_db.as_ref(), &mut env_drift);
    check_env_pair("redis_page_cache_host", b_env.redis_page_cache_host.as_ref(), c_env.redis_page_cache_host.as_ref(), &mut env_drift);
    check_env_pair("redis_page_cache_db", b_env.redis_page_cache_db.as_ref(), c_env.redis_page_cache_db.as_ref(), &mut env_drift);
    check_env_pair("redis_session_host", b_env.redis_session_host.as_ref(), c_env.redis_session_host.as_ref(), &mut env_drift);
    check_env_pair("redis_session_db", b_env.redis_session_db.as_ref(), c_env.redis_session_db.as_ref(), &mut env_drift);
    check_env_pair("opensearch_host", b_env.opensearch_host.as_ref(), c_env.opensearch_host.as_ref(), &mut env_drift);
    check_env_pair("rabbitmq_host", b_env.rabbitmq_host.as_ref(), c_env.rabbitmq_host.as_ref(), &mut env_drift);

    // 6. Schema drift
    let mut schema_drift = SchemaDrift::default();
    let b_tables = &baseline.installation.declared_schema.tables;
    let c_tables = &current.installation.declared_schema.tables;

    for (t_name, c_table) in c_tables {
        match b_tables.get(t_name) {
            None => schema_drift.added_tables.push(t_name.clone()),
            Some(b_table) => {
                for col in c_table.columns.keys() {
                    if !b_table.columns.contains_key(col) {
                        schema_drift.added_columns.push(format!("{}.{}", t_name, col));
                    }
                }
                for idx in c_table.indexes.keys() {
                    if !b_table.indexes.contains_key(idx) {
                        schema_drift.added_indexes.push(format!("{}.{}", t_name, idx));
                    }
                }
            }
        }
    }
    for (t_name, b_table) in b_tables {
        match c_tables.get(t_name) {
            None => schema_drift.dropped_tables.push(t_name.clone()),
            Some(c_table) => {
                for col in b_table.columns.keys() {
                    if !c_table.columns.contains_key(col) {
                        schema_drift.dropped_columns.push(format!("{}.{}", t_name, col));
                    }
                }
                for idx in b_table.indexes.keys() {
                    if !c_table.indexes.contains_key(idx) {
                        schema_drift.dropped_indexes.push(format!("{}.{}", t_name, idx));
                    }
                }
            }
        }
    }

    // 7. Findings drift
    let mut findings_drift = FindingsDrift::default();
    let b_finding_keys: HashSet<(String, String)> = baseline
        .findings
        .iter()
        .map(|f| (f.rule_id.clone(), f.title.clone()))
        .collect();
    let c_finding_keys: HashSet<(String, String)> = current
        .findings
        .iter()
        .map(|f| (f.rule_id.clone(), f.title.clone()))
        .collect();

    for f in &current.findings {
        let key = (f.rule_id.clone(), f.title.clone());
        if !b_finding_keys.contains(&key) {
            findings_drift.new_findings.push(f.clone());
        } else {
            findings_drift.persistent_findings.push(f.clone());
        }
    }
    for f in &baseline.findings {
        let key = (f.rule_id.clone(), f.title.clone());
        if !c_finding_keys.contains(&key) {
            findings_drift.resolved_findings.push(f.clone());
        }
    }

    // 8. Health drift
    let delta_overall = (current.health_score.overall as i32) - (baseline.health_score.overall as i32);
    let delta_critical = (current.health_score.critical_count as i64) - (baseline.health_score.critical_count as i64);
    let delta_warning = (current.health_score.warning_count as i64) - (baseline.health_score.warning_count as i64);

    let health_drift = HealthDrift {
        baseline_overall: baseline.health_score.overall,
        current_overall: current.health_score.overall,
        delta_overall,
        baseline_critical: baseline.health_score.critical_count,
        current_critical: current.health_score.critical_count,
        delta_critical,
        baseline_warning: baseline.health_score.warning_count,
        current_warning: current.health_score.warning_count,
        delta_warning,
    };

    // Determine regressions
    let mut has_regressions = false;
    let mut max_regression_severity = None;

    if findings_drift.new_findings.iter().any(|f| f.severity == Severity::Critical) {
        has_regressions = true;
        max_regression_severity = Some(Severity::Critical);
    } else if findings_drift.new_findings.iter().any(|f| f.severity == Severity::Warning) || delta_overall < -5 {
        has_regressions = true;
        max_regression_severity = Some(Severity::Warning);
    }

    DriftReport {
        baseline_time: baseline.created_at,
        current_time: current.created_at,
        metadata_drift: meta_drift,
        modules_drift,
        plugins_drift,
        preferences_drift,
        cron_drift,
        env_drift,
        schema_drift,
        findings_drift,
        health_drift,
        has_regressions,
        max_regression_severity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{Category, Confidence};
    use crate::health::HealthScore;
    use std::path::PathBuf;

    fn make_test_snapshot(version: &str, module_names: &[&str], findings: Vec<Finding>) -> DiagnosticSnapshot {
        let mut inst = MagentoInstallation::new(PathBuf::from("/var/www/html"));
        inst.version = Some(version.to_string());
        inst.edition = Edition::OpenSource;
        inst.mode = MagentoMode::Production;

        for name in module_names {
            inst.modules.push(Module {
                name: name.to_string(),
                vendor: "Test".to_string(),
                package_name: None,
                version: Some("1.0.0".to_string()),
                sequence: vec![],
                path: PathBuf::new(),
                is_enabled: true,
                classification: ModuleClassification::AppCodeCustom,
                footprint: ModuleFootprint::default(),
            });
        }

        let health = HealthScore::calculate(&findings);
        DiagnosticSnapshot::new(inst, findings, health)
    }

    #[test]
    fn test_drift_modules_added_and_removed() {
        let b = make_test_snapshot("2.4.7", &["Vendor_ModA", "Vendor_ModB"], vec![]);
        let c = make_test_snapshot("2.4.7", &["Vendor_ModB", "Vendor_ModC"], vec![]);

        let report = compare_installations(&b, &c);
        assert_eq!(report.modules_drift.len(), 2);

        let added = report.modules_drift.iter().find(|m| m.name == "Vendor_ModC").unwrap();
        assert_eq!(added.change_type, DriftChangeType::Added);

        let removed = report.modules_drift.iter().find(|m| m.name == "Vendor_ModA").unwrap();
        assert_eq!(removed.change_type, DriftChangeType::Removed);
    }

    #[test]
    fn test_drift_findings_and_regression() {
        let b = make_test_snapshot("2.4.7", &["Vendor_ModA"], vec![]);
        let critical_finding = Finding::new(
            "MD-SEC-001",
            "Critical Security Issue",
            Severity::Critical,
            Confidence::High,
            Category::Security,
        );
        let c = make_test_snapshot("2.4.7", &["Vendor_ModA"], vec![critical_finding]);

        let report = compare_installations(&b, &c);
        assert!(report.has_regressions);
        assert_eq!(report.max_regression_severity, Some(Severity::Critical));
        assert_eq!(report.findings_drift.new_findings.len(), 1);
        assert!(report.health_drift.delta_overall < 0);
    }
}
