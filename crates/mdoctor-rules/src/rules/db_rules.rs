//! Database rules (MD-DB-001, MD-DB-002, MD-DB-005, MD-DB-010).

use mdoctor_core::{Category, Confidence, Finding, MagentoInstallation, Severity};
use mdoctor_db::{find_redundant_indexes, reconcile_schemas};
use mdoctor_knowledge::tables::find_table_knowledge;

pub fn evaluate_db_rules(installation: &MagentoInstallation) -> Vec<Finding> {
    let mut findings = Vec::new();

    // 1. Schema reconciliation: Missing declared tables & indexes
    let diff = reconcile_schemas(&installation.declared_schema, &installation.actual_schema);

    for (table, index) in diff.missing_indexes {
        let mut finding = Finding::new(
            "MD-DB-001",
            format!("Missing declared database index: {}.{}", table, index),
            Severity::Warning,
            Confidence::High,
            Category::Database,
        );

        finding.summary = format!(
            "Index '{}' is declared in module db_schema.xml but missing from actual database table '{}'.",
            index, table
        );

        finding.evidence.push(format!("Table: {}", table));
        finding.evidence.push(format!("Missing Index: {}", index));
        finding.impact = "Queries relying on this declared index will perform full table scans or filesorts, degrading database performance.".to_string();
        finding.recommendation = "Run 'bin/magento setup:db-declaration:generate-whitelist' and 'bin/magento setup:upgrade' to apply declarative schema.".to_string();
        finding.verification_commands = vec![
            format!("mysql> SHOW INDEX FROM {};", table),
        ];
        finding.related_tables.push(table);

        findings.push(finding);
    }

    // 2. MD-DB-002: Redundant left-prefix indexes
    for table in installation.actual_schema.tables.values() {
        let redundants = find_redundant_indexes(table);
        for red in redundants {
            let mut finding = Finding::new(
                "MD-DB-002",
                format!("Redundant left-prefix index on table '{}'", red.table_name),
                Severity::Info,
                Confidence::High,
                Category::Queries,
            );

            finding.summary = format!(
                "Index '{}' ({:?}) is redundant because index '{}' ({:?}) already covers its prefix.",
                red.redundant_index_name, red.redundant_columns, red.covering_index_name, red.covering_columns
            );

            finding.evidence.push(format!("Table: {}", red.table_name));
            finding.evidence.push(format!("Redundant index: {}", red.redundant_index_name));
            finding.evidence.push(format!("Covered by index: {}", red.covering_index_name));

            finding.impact = "Redundant indexes consume disk space, pollute the InnoDB buffer pool, and slow down INSERT/UPDATE/DELETE operations with zero query benefit.".to_string();
            finding.recommendation = format!(
                "Verify no historical query relies exclusively on {} and drop the redundant index.",
                red.redundant_index_name
            );
            finding.verification_commands = vec![
                format!("mysql> SHOW INDEX FROM {};", red.table_name),
            ];
            finding.related_tables.push(red.table_name);

            findings.push(finding);
        }
    }

    // 3. MD-DB-005: Orphan database tables
    for orphan in diff.orphan_tables {
        let mut finding = Finding::new(
            "MD-DB-005",
            format!("Possible orphan database table: {}", orphan),
            Severity::Info,
            Confidence::Medium,
            Category::Database,
        );

        finding.summary = format!(
            "Table '{}' exists in database but is not declared in any enabled module db_schema.xml.",
            orphan
        );

        finding.evidence.push(format!("Table name: {}", orphan));
        finding.impact = "Orphan tables from uninstalled third-party modules consume database storage, bloat backups, and complicate schema maintenance.".to_string();
        finding.recommendation = "Investigate if this table belongs to a removed extension before considering cleanup. Never drop tables without verification.".to_string();
        finding.related_tables.push(orphan);

        findings.push(finding);
    }

    // 4. MD-DB-010: Volatile table size check
    for size_stat in &installation.database_metrics.table_sizes {
        if let Some(known) = find_table_knowledge(&size_stat.table_name) {
            if size_stat.row_count > known.critical_row_count {
                let mut finding = Finding::new(
                    "MD-DB-010",
                    format!("Critical table growth: {}", size_stat.table_name),
                    Severity::Critical,
                    Confidence::High,
                    Category::Database,
                );

                let size_mb = size_stat.total_bytes / (1024 * 1024);
                finding.summary = format!(
                    "Table '{}' has {} rows ({} MB), exceeding critical threshold of {}.",
                    size_stat.table_name, size_stat.row_count, size_mb, known.critical_row_count
                );

                finding.evidence.push(format!("Row count: {}", size_stat.row_count));
                finding.evidence.push(format!("Data + Index size: {} MB", size_mb));
                finding.evidence.push(format!("Table purpose: {}", known.description));

                finding.impact = "Excessive table size increases buffer pool pressure, slows queries, and lengthens maintenance backups.".to_string();
                finding.recommendation = format!(
                    "Configure log cleaning / cron maintenance or run cleaning scripts for {}.",
                    known.owning_module
                );
                finding.related_tables.push(size_stat.table_name.clone());

                findings.push(finding);
            }
        }
    }

    findings
}
