//! Module impact and architectural risk calculation engine.

use mdoctor_core::{
    ImpactLevel, MagentoInstallation, Module, ModuleClassification, ModuleImpactScore,
    PluginType, RiskDriver,
};
use mdoctor_knowledge::hotpaths::{is_hot_event, is_hot_method, HotPathWeight};
use mdoctor_php::{AstFinding, OperationType};

/// Calculates the architectural risk and performance impact score for a specific module.
pub fn calculate_module_impact(
    module: &Module,
    installation: &MagentoInstallation,
    ast_findings: &[AstFinding],
) -> ModuleImpactScore {
    let mut risk_drivers = Vec::new();
    let mut raw_score: u32 = 0;

    // 1. Evaluate Plugins
    let mut plugins_count = 0;
    let mut around_plugins_count = 0;
    let mut hotpath_plugins_count = 0;

    for plugin in &installation.plugins {
        if plugin.module != module.name || plugin.is_disabled {
            continue;
        }
        plugins_count += 1;

        let is_around = plugin.plugin_type == PluginType::Around;
        if is_around {
            around_plugins_count += 1;
        }

        if let Some(weight) = is_hot_method(&plugin.target_class, None) {
            hotpath_plugins_count += 1;
            match weight {
                HotPathWeight::Critical => {
                    let pts = if is_around { 20 } else { 12 };
                    raw_score += pts;
                    risk_drivers.push(RiskDriver {
                        name: format!(
                            "{} plugin on critical hotpath: {}",
                            if is_around { "Around" } else { "Active" },
                            plugin.target_class
                        ),
                        description: format!(
                            "Plugin '{}' intercepts critical hotpath '{}'",
                            plugin.name, plugin.target_class
                        ),
                        points: pts,
                    });
                }
                HotPathWeight::High => {
                    let pts = if is_around { 15 } else { 8 };
                    raw_score += pts;
                    risk_drivers.push(RiskDriver {
                        name: format!(
                            "{} plugin on high-frequency method: {}",
                            if is_around { "Around" } else { "Active" },
                            plugin.target_class
                        ),
                        description: format!(
                            "Plugin '{}' intercepts frequent method '{}'",
                            plugin.name, plugin.target_class
                        ),
                        points: pts,
                    });
                }
                HotPathWeight::Medium => {
                    let pts = if is_around { 10 } else { 5 };
                    raw_score += pts;
                    risk_drivers.push(RiskDriver {
                        name: format!(
                            "{} plugin on hotpath: {}",
                            if is_around { "Around" } else { "Active" },
                            plugin.target_class
                        ),
                        description: format!(
                            "Plugin '{}' intercepts hotpath '{}'",
                            plugin.name, plugin.target_class
                        ),
                        points: pts,
                    });
                }
            }
        } else if is_around {
            raw_score += 6;
            risk_drivers.push(RiskDriver {
                name: format!("Around plugin: {}", plugin.target_class),
                description: format!("Around wrapper adds stack depth to '{}'", plugin.target_class),
                points: 6,
            });
        }
    }

    // 2. Evaluate AST Cost Indicators within this module's directory
    let mut ast_indicators_count = 0;
    for finding in ast_findings {
        let matches_module = finding
            .file_path
            .as_ref()
            .map(|p| p.starts_with(&module.path))
            .unwrap_or(false);

        if !matches_module {
            continue;
        }
        ast_indicators_count += 1;

        match finding.operation {
            OperationType::RepositoryLoad if finding.in_loop => {
                raw_score += 18;
                risk_drivers.push(RiskDriver {
                    name: "N+1 Repository Load inside Loop".to_string(),
                    description: format!(
                        "N+1 query in {}: {}",
                        finding.class_name.as_deref().unwrap_or("class"),
                        finding.call_signature
                    ),
                    points: 18,
                });
            }
            OperationType::HttpRequest => {
                raw_score += 16;
                risk_drivers.push(RiskDriver {
                    name: "Synchronous HTTP Request".to_string(),
                    description: format!(
                        "Outbound network request in {}: {}",
                        finding.class_name.as_deref().unwrap_or("class"),
                        finding.call_signature
                    ),
                    points: 16,
                });
            }
            OperationType::DirectSqlQuery => {
                raw_score += 12;
                risk_drivers.push(RiskDriver {
                    name: "Direct Raw SQL Execution".to_string(),
                    description: format!(
                        "Raw query in {}: {}",
                        finding.class_name.as_deref().unwrap_or("class"),
                        finding.call_signature
                    ),
                    points: 12,
                });
            }
            OperationType::DatabaseWrite => {
                raw_score += 8;
                risk_drivers.push(RiskDriver {
                    name: "Direct DB Mutation".to_string(),
                    description: format!(
                        "Save/delete call in {}: {}",
                        finding.class_name.as_deref().unwrap_or("class"),
                        finding.call_signature
                    ),
                    points: 8,
                });
            }
            OperationType::ObjectManagerUsage => {
                raw_score += 6;
                risk_drivers.push(RiskDriver {
                    name: "Direct ObjectManager anti-pattern".to_string(),
                    description: "ObjectManager::getInstance() invocation violates DI container".to_string(),
                    points: 6,
                });
            }
            _ => {}
        }
    }

    // 3. Evaluate Observers
    let mut observers_count = 0;
    let mut hot_observers_count = 0;

    for obs in &installation.observers {
        if obs.module != module.name || obs.is_disabled {
            continue;
        }
        observers_count += 1;

        if let Some(weight) = is_hot_event(&obs.event_name) {
            hot_observers_count += 1;
            let pts = match weight {
                HotPathWeight::Critical => 15,
                HotPathWeight::High => 10,
                HotPathWeight::Medium => 5,
            };
            raw_score += pts;
            risk_drivers.push(RiskDriver {
                name: format!("Observer on hot event: {}", obs.event_name),
                description: format!("Observer '{}' executes on event '{}'", obs.name, obs.event_name),
                points: pts,
            });
        }
    }

    // 4. Evaluate Cron Jobs
    let mut cron_jobs_count = 0;
    let mut minutely_crons_count = 0;

    for cron in &installation.cron_jobs {
        if cron.module != module.name {
            continue;
        }
        cron_jobs_count += 1;

        let is_minutely = cron.interval_seconds.map(|s| s <= 60).unwrap_or(false)
            || cron.schedule.as_deref() == Some("* * * * *");

        if is_minutely {
            minutely_crons_count += 1;
            raw_score += 14;
            risk_drivers.push(RiskDriver {
                name: format!("Minutely cron execution: {}", cron.name),
                description: format!(
                    "Cron '{}' executes every minute ({}), creating schedule lock risk",
                    cron.name,
                    cron.schedule.as_deref().unwrap_or("* * * * *")
                ),
                points: 14,
            });
        } else {
            raw_score += 3;
        }
    }

    // 5. Evaluate Preferences
    let mut preferences_count = 0;
    let mut core_preferences_count = 0;

    for pref in &installation.preferences {
        if pref.module != module.name {
            continue;
        }
        preferences_count += 1;

        let is_core = pref.for_class.starts_with("Magento\\");
        if is_core {
            core_preferences_count += 1;
            raw_score += 12;
            risk_drivers.push(RiskDriver {
                name: format!("Preference overriding core: {}", pref.for_class),
                description: format!(
                    "Preference redirects '{}' to '{}', breaking DI extensibility",
                    pref.for_class, pref.type_class
                ),
                points: 12,
            });
        } else {
            raw_score += 4;
        }
    }

    // 6. Database Schema Footprint
    let mut db_tables_count = 0;
    let core_tables_altered_count = 0;

    for (t_name, table) in &installation.declared_schema.tables {
        if table.owning_module.as_deref() == Some(&module.name) {
            db_tables_count += 1;
            raw_score += 5;
            risk_drivers.push(RiskDriver {
                name: format!("Custom DB table: {}", t_name),
                description: format!("Module owns and manages custom table '{}'", t_name),
                points: 5,
            });
        }
    }

    // Score clamping & impact level determination
    let score = raw_score.min(100);
    let level = match score {
        0..=20 => ImpactLevel::Low,
        21..=45 => ImpactLevel::Medium,
        46..=70 => ImpactLevel::High,
        _ => ImpactLevel::Critical,
    };

    ModuleImpactScore {
        module_name: module.name.clone(),
        classification: module.classification,
        score,
        level,
        risk_drivers,
        plugins_count,
        around_plugins_count,
        hotpath_plugins_count,
        observers_count,
        hot_observers_count,
        cron_jobs_count,
        minutely_crons_count,
        preferences_count,
        core_preferences_count,
        db_tables_count,
        core_tables_altered_count,
        ast_indicators_count,
    }
}

/// Calculate and rank impact scores across all non-core modules in the installation.
pub fn calculate_all_modules_impact(
    installation: &MagentoInstallation,
    ast_findings: &[AstFinding],
) -> Vec<ModuleImpactScore> {
    let mut scores = Vec::new();

    for module in &installation.modules {
        if !module.is_enabled {
            continue;
        }

        // Only evaluate custom or third-party modules
        match module.classification {
            ModuleClassification::Core | ModuleClassification::AdobeCommerce => continue,
            _ => {}
        }

        scores.push(calculate_module_impact(module, installation, ast_findings));
    }

    // Rank descending: highest impact first
    scores.sort_by_key(|b| std::cmp::Reverse(b.score));
    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdoctor_core::{ModuleFootprint, Plugin};
    use std::path::PathBuf;

    #[test]
    fn test_clean_module_low_impact() {
        let inst = MagentoInstallation::new(PathBuf::from("/var/www/html"));
        let module = Module {
            name: "Vendor_Simple".to_string(),
            vendor: "Vendor".to_string(),
            package_name: None,
            version: Some("1.0.0".to_string()),
            sequence: vec![],
            path: PathBuf::new(),
            is_enabled: true,
            classification: ModuleClassification::ComposerThirdParty,
            footprint: ModuleFootprint::default(),
        };

        let impact = calculate_module_impact(&module, &inst, &[]);
        assert_eq!(impact.level, ImpactLevel::Low);
        assert_eq!(impact.score, 0);
    }

    #[test]
    fn test_hotpath_around_plugin_increases_impact() {
        let mut inst = MagentoInstallation::new(PathBuf::from("/var/www/html"));
        let module = Module {
            name: "Vendor_SlowCheckout".to_string(),
            vendor: "Vendor".to_string(),
            package_name: None,
            version: Some("1.0.0".to_string()),
            sequence: vec![],
            path: PathBuf::new(),
            is_enabled: true,
            classification: ModuleClassification::ComposerThirdParty,
            footprint: ModuleFootprint::default(),
        };

        inst.plugins.push(Plugin {
            name: "vendor_slow_quote".to_string(),
            module: "Vendor_SlowCheckout".to_string(),
            target_class: "Magento\\Quote\\Model\\QuoteManagement".to_string(),
            plugin_class: "Vendor\\SlowCheckout\\Plugin\\QuotePlugin".to_string(),
            plugin_type: PluginType::Around,
            sort_order: 10,
            is_disabled: false,
            area: "global".to_string(),
            source_file: PathBuf::new(),
            line: 1,
            cost_indicators: vec![],
        });

        let impact = calculate_module_impact(&module, &inst, &[]);
        assert_eq!(impact.hotpath_plugins_count, 1);
        assert_eq!(impact.around_plugins_count, 1);
        assert!(impact.score >= 20);
        assert!(!impact.risk_drivers.is_empty());
    }
}
