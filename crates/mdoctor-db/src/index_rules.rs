//! Index optimization, redundancy detection, and composite candidate analysis.

use mdoctor_core::TableSchema;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedundantIndex {
    pub table_name: String,
    pub redundant_index_name: String,
    pub covering_index_name: String,
    pub redundant_columns: Vec<String>,
    pub covering_columns: Vec<String>,
}

/// Detects redundant left-prefix indexes on a table.
/// e.g. INDEX(status) is redundant when INDEX(status, created_at) exists.
pub fn find_redundant_indexes(table: &TableSchema) -> Vec<RedundantIndex> {
    let mut redundants = Vec::new();

    let indexes: Vec<_> = table.indexes.values().collect();
    for (i, idx_a) in indexes.iter().enumerate() {
        if idx_a.columns.is_empty() || idx_a.name.eq_ignore_ascii_case("PRIMARY") {
            continue;
        }

        for (j, idx_b) in indexes.iter().enumerate() {
            if i == j || idx_b.name.eq_ignore_ascii_case("PRIMARY") {
                continue;
            }

            // If idx_a columns form a strict prefix of idx_b columns
            if idx_b.columns.len() > idx_a.columns.len()
                && idx_b.columns.starts_with(&idx_a.columns)
            {
                redundants.push(RedundantIndex {
                    table_name: table.name.clone(),
                    redundant_index_name: idx_a.name.clone(),
                    covering_index_name: idx_b.name.clone(),
                    redundant_columns: idx_a.columns.clone(),
                    covering_columns: idx_b.columns.clone(),
                });
            }
        }
    }

    redundants
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdoctor_core::IndexSchema;
    use std::collections::HashMap;

    #[test]
    fn test_detect_redundant_left_prefix() {
        let mut table = TableSchema {
            name: "vendor_feed_queue".to_string(),
            columns: HashMap::new(),
            indexes: HashMap::new(),
            constraints: HashMap::new(),
            engine: "innodb".to_string(),
            comment: None,
            owning_module: None,
        };

        table.indexes.insert(
            "IDX_STATUS".to_string(),
            IndexSchema {
                name: "IDX_STATUS".to_string(),
                index_type: "btree".to_string(),
                columns: vec!["status".to_string()],
            },
        );

        table.indexes.insert(
            "IDX_STATUS_CREATED".to_string(),
            IndexSchema {
                name: "IDX_STATUS_CREATED".to_string(),
                index_type: "btree".to_string(),
                columns: vec!["status".to_string(), "created_at".to_string()],
            },
        );

        let redundants = find_redundant_indexes(&table);
        assert_eq!(redundants.len(), 1);
        assert_eq!(redundants[0].redundant_index_name, "IDX_STATUS");
        assert_eq!(redundants[0].covering_index_name, "IDX_STATUS_CREATED");
    }
}
