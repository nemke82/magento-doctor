//! Declarative schema diff and reconciliation.

use std::collections::HashSet;
use mdoctor_core::DatabaseSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDiffReport {
    pub missing_tables: Vec<String>,
    pub orphan_tables: Vec<String>,
    pub missing_columns: Vec<(String, String)>, // (table, column)
    pub missing_indexes: Vec<(String, String)>, // (table, index)
}

/// Reconciles declared schema from db_schema.xml against actual live schema.
pub fn reconcile_schemas(declared: &DatabaseSchema, actual: &DatabaseSchema) -> SchemaDiffReport {
    let mut missing_tables = Vec::new();
    let mut orphan_tables = Vec::new();
    let mut missing_columns = Vec::new();
    let mut missing_indexes = Vec::new();

    let declared_table_names: HashSet<&String> = declared.tables.keys().collect();
    let actual_table_names: HashSet<&String> = actual.tables.keys().collect();

    // Tables declared but missing in actual DB
    for declared_name in &declared_table_names {
        if !actual_table_names.contains(declared_name) {
            missing_tables.push((*declared_name).clone());
        }
    }

    // Tables in DB but not declared by any module
    for actual_name in &actual_table_names {
        if !declared_table_names.contains(actual_name) {
            orphan_tables.push((*actual_name).clone());
        }
    }

    // Check columns and indexes for tables present in both
    for (t_name, declared_table) in &declared.tables {
        if let Some(actual_table) = actual.tables.get(t_name) {
            for col_name in declared_table.columns.keys() {
                if !actual_table.columns.contains_key(col_name) {
                    missing_columns.push((t_name.clone(), col_name.clone()));
                }
            }

            for idx_name in declared_table.indexes.keys() {
                if !actual_table.indexes.contains_key(idx_name) {
                    missing_indexes.push((t_name.clone(), idx_name.clone()));
                }
            }
        }
    }

    missing_tables.sort();
    orphan_tables.sort();
    missing_columns.sort();
    missing_indexes.sort();

    SchemaDiffReport {
        missing_tables,
        orphan_tables,
        missing_columns,
        missing_indexes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdoctor_core::TableSchema;
    use std::collections::HashMap;

    #[test]
    fn test_reconcile_schemas() {
        let mut declared = DatabaseSchema::default();
        let mut t1 = TableSchema {
            name: "t1".to_string(),
            columns: HashMap::new(),
            indexes: HashMap::new(),
            constraints: HashMap::new(),
            engine: "innodb".to_string(),
            comment: None,
            owning_module: Some("Vendor_Module".to_string()),
        };
        t1.columns.insert(
            "col1".to_string(),
            mdoctor_core::ColumnSchema {
                name: "col1".to_string(),
                data_type: "int".to_string(),
                nullable: false,
                default: None,
                unsigned: true,
                identity: true,
                comment: None,
            },
        );
        declared.tables.insert("t1".to_string(), t1);

        let mut actual = DatabaseSchema::default();
        let t2 = TableSchema {
            name: "old_orphan_table".to_string(),
            columns: HashMap::new(),
            indexes: HashMap::new(),
            constraints: HashMap::new(),
            engine: "innodb".to_string(),
            comment: None,
            owning_module: None,
        };
        actual.tables.insert("old_orphan_table".to_string(), t2);

        let diff = reconcile_schemas(&declared, &actual);
        assert_eq!(diff.missing_tables, vec!["t1"]);
        assert_eq!(diff.orphan_tables, vec!["old_orphan_table"]);
    }
}
