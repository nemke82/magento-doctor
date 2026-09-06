use std::path::PathBuf;
use mdoctor_core::{DiagnosticSnapshot, Edition, HealthScore, Severity};
use mdoctor_magento::collect_installation;
use mdoctor_rules::CrossAnalysisEngine;

#[test]
fn test_fixture_scan_end_to_end() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.join("../../fixtures/magento-2.4.7");

    assert!(root.exists(), "Fixture directory must exist at {:?}", root);

    // 1. Collect installation
    let installation = collect_installation(&root);
    assert_eq!(installation.edition, Edition::OpenSource);
    assert_eq!(installation.version.as_deref(), Some("2.4.7-p3"));
    assert_eq!(installation.enabled_modules_count(), 4);
    assert_eq!(installation.disabled_modules_count(), 1);

    // Verify module footprint
    let feed_module = installation.find_module("Vendor_Feed").expect("Vendor_Feed must exist");
    assert_eq!(feed_module.footprint.cron_jobs_count, 1);
    assert_eq!(feed_module.footprint.db_tables_count, 1);

    let payment_module = installation.find_module("Vendor_Payment").expect("Vendor_Payment must exist");
    assert_eq!(payment_module.footprint.plugins_count, 1);

    // 2. Run CrossAnalysisEngine
    let findings = CrossAnalysisEngine::analyze(&installation);
    assert!(!findings.is_empty());

    // Check MD-PERF-021 (N+1 in loop)
    let n_plus_one = findings.iter().find(|f| f.rule_id == "MD-PERF-021");
    assert!(n_plus_one.is_some(), "Expected MD-PERF-021 finding for repository in loop");
    let n1 = n_plus_one.unwrap();
    assert_eq!(n1.severity, Severity::Critical);
    assert!(n1.evidence.iter().any(|e| e.contains("getById")));

    // Check MD-PLG-001 (Around plugin on hot path)
    let around_hot = findings.iter().find(|f| f.rule_id == "MD-PLG-001");
    assert!(around_hot.is_some(), "Expected MD-PLG-001 finding for around plugin on QuoteManagement");
    let plg = around_hot.unwrap();
    assert!(plg.evidence.iter().any(|e| e.contains("Vendor\\Payment\\Plugin\\QuoteManagement")));

    // Check MD-PERF-015 (Synchronous HTTP call)
    let http_call = findings.iter().find(|f| f.rule_id == "MD-PERF-015");
    assert!(http_call.is_some(), "Expected MD-PERF-015 finding for HTTP client call");

    // 3. Health Score calculation
    let health = HealthScore::calculate(&findings);
    assert!(health.overall < 100);
    assert!(health.critical_count >= 2);

    // 4. Test Snapshot serialization and roundtrip
    let snapshot = DiagnosticSnapshot::new(installation.clone(), findings.clone(), health);
    let json = snapshot.to_json().expect("Snapshot should serialize to JSON");
    assert!(!json.contains("secret_mock_password_never_print"), "Secrets must never be in snapshot!");
    assert!(!json.contains("mock_crypt_key_do_not_expose_12345"), "Crypt key must never be in snapshot!");

    let restored = DiagnosticSnapshot::from_json(&json).expect("Snapshot should deserialize cleanly");
    assert_eq!(restored.installation.version, Some("2.4.7-p3".to_string()));
    assert_eq!(restored.findings.len(), findings.len());

    // 5. Test v0.2 Drift Comparison
    let drift = mdoctor_core::compare_installations(&snapshot, &restored);
    assert!(!drift.has_regressions);
    assert_eq!(drift.health_drift.delta_overall, 0);
    assert!(drift.findings_drift.new_findings.is_empty());
    assert!(drift.modules_drift.is_empty());

    // 6. Test v0.2 Module Impact Scoring
    let ast_findings = mdoctor_rules::scan_php_sources(&installation);
    let impacts = mdoctor_rules::calculate_all_modules_impact(&installation, &ast_findings);
    assert_eq!(impacts.len(), 2, "Expected 2 custom/third-party modules scored");
    let feed_impact = impacts.iter().find(|i| i.module_name == "Vendor_Feed").expect("Vendor_Feed impact");
    assert_eq!(feed_impact.level, mdoctor_core::ImpactLevel::High);
    assert!(feed_impact.score >= 50);

    let payment_impact = impacts.iter().find(|i| i.module_name == "Vendor_Payment").expect("Vendor_Payment impact");
    assert_eq!(payment_impact.level, mdoctor_core::ImpactLevel::Medium);
    assert_eq!(payment_impact.hotpath_plugins_count, 1);

    // 7. Test v0.2 Module Uninstall Impact Forensics
    let feed_uninstall = mdoctor_core::calculate_uninstall_impact(&installation, "Vendor_Feed").expect("Vendor_Feed analysis");
    assert_eq!(feed_uninstall.safety, mdoctor_core::UninstallSafety::Caution);
    assert_eq!(feed_uninstall.orphaned_tables.len(), 1);
    assert_eq!(feed_uninstall.orphaned_tables[0], "vendor_feed_queue");

    let catalog_uninstall = mdoctor_core::calculate_uninstall_impact(&installation, "Magento_Catalog").expect("Magento_Catalog analysis");
    assert_eq!(catalog_uninstall.safety, mdoctor_core::UninstallSafety::Blocked);
    assert!(catalog_uninstall.dependents.iter().any(|d| d.name == "Vendor_Feed"));

    // 8. Test v0.2 Mermaid Graph Generation
    let graph = mdoctor_report::render_mermaid_graph(feed_module, &installation);
    assert!(graph.contains("flowchart TD"));
    assert!(graph.contains("Vendor_Feed"));
    assert!(graph.contains("vendor_export_feed"));
    assert!(graph.contains("vendor_feed_queue"));
}

