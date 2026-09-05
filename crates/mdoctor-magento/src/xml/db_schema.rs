//! db_schema.xml parser for declarative database schema.

use std::collections::HashMap;
use std::path::Path;
use mdoctor_core::{ColumnSchema, ConstraintSchema, IndexSchema, TableSchema};

pub fn parse_db_schema_xml(file_path: &Path, module_name: &str) -> HashMap<String, TableSchema> {
    let mut tables = HashMap::new();

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return tables,
    };

    let doc = match roxmltree::Document::parse(&content) {
        Ok(d) => d,
        Err(_) => return tables,
    };

    for table_node in doc.descendants().filter(|n| n.has_tag_name("table")) {
        let table_name = match table_node.attribute("name") {
            Some(n) => n.to_string(),
            None => continue,
        };

        let engine = table_node.attribute("engine").unwrap_or("innodb").to_string();
        let comment = table_node.attribute("comment").map(|c| c.to_string());

        let mut columns = HashMap::new();
        let mut indexes = HashMap::new();
        let mut constraints = HashMap::new();

        for child in table_node.children() {
            if child.has_tag_name("column") {
                if let Some(col_name) = child.attribute("name") {
                    let data_type = child.attribute("type")
                        .or_else(|| child.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type")))
                        .unwrap_or("varchar")
                        .to_string();

                    let nullable = child
                        .attribute("nullable")
                        .map(|n| n == "true" || n == "1")
                        .unwrap_or(true);

                    let unsigned = child
                        .attribute("unsigned")
                        .map(|u| u == "true" || u == "1")
                        .unwrap_or(false);

                    let identity = child
                        .attribute("identity")
                        .map(|i| i == "true" || i == "1")
                        .unwrap_or(false);

                    let default = child.attribute("default").map(|d| d.to_string());
                    let col_comment = child.attribute("comment").map(|c| c.to_string());

                    columns.insert(
                        col_name.to_string(),
                        ColumnSchema {
                            name: col_name.to_string(),
                            data_type,
                            nullable,
                            default,
                            unsigned,
                            identity,
                            comment: col_comment,
                        },
                    );
                }
            } else if child.has_tag_name("index") {
                let ref_id = child.attribute("referenceId")
                    .or_else(|| child.attribute("indexName"))
                    .unwrap_or("INDEX")
                    .to_string();

                let index_type = child.attribute("indexType").unwrap_or("btree").to_string();

                let mut col_names = Vec::new();
                for col_child in child.children().filter(|c| c.has_tag_name("column")) {
                    if let Some(name) = col_child.attribute("name") {
                        col_names.push(name.to_string());
                    }
                }

                indexes.insert(
                    ref_id.clone(),
                    IndexSchema {
                        name: ref_id,
                        index_type,
                        columns: col_names,
                    },
                );
            } else if child.has_tag_name("constraint") {
                let ref_id = child.attribute("referenceId").unwrap_or("PRIMARY").to_string();
                let constraint_type = child.attribute("type")
                    .or_else(|| child.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type")))
                    .unwrap_or("primary")
                    .to_string();

                let ref_table = child.attribute("referenceTable").map(|t| t.to_string());
                let on_delete = child.attribute("onDelete").map(|o| o.to_string());

                let mut col_names = Vec::new();
                for col_child in child.children().filter(|c| c.has_tag_name("column")) {
                    if let Some(name) = col_child.attribute("name") {
                        col_names.push(name.to_string());
                    }
                }

                constraints.insert(
                    ref_id.clone(),
                    ConstraintSchema {
                        name: ref_id,
                        constraint_type,
                        columns: col_names,
                        reference_table: ref_table,
                        reference_column: None,
                        on_delete,
                    },
                );
            }
        }

        tables.insert(
            table_name.clone(),
            TableSchema {
                name: table_name,
                columns,
                indexes,
                constraints,
                engine,
                comment,
                owning_module: Some(module_name.to_string()),
            },
        );
    }

    tables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_db_schema_xml_sample() {
        let sample = r#"<?xml version="1.0"?>
<schema xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <table name="vendor_feed_queue" resource="default" engine="innodb" comment="Feed Queue">
        <column xsi:type="int" name="entity_id" unsigned="true" nullable="false" identity="true" comment="Entity ID"/>
        <column xsi:type="varchar" name="status" nullable="false" default="pending" comment="Status"/>
        <column xsi:type="timestamp" name="created_at" nullable="false" default="CURRENT_TIMESTAMP" comment="Created At"/>
        <constraint xsi:type="primary" referenceId="PRIMARY">
            <column name="entity_id"/>
        </constraint>
        <index referenceId="IDX_STATUS_CREATED" indexType="btree">
            <column name="status"/>
            <column name="created_at"/>
        </index>
    </table>
</schema>
"#;
        let temp = std::env::temp_dir().join("test_db_schema.xml");
        std::fs::write(&temp, sample).unwrap();

        let tables = parse_db_schema_xml(&temp, "Vendor_Feed");
        assert!(tables.contains_key("vendor_feed_queue"));
        let t = &tables["vendor_feed_queue"];
        assert_eq!(t.columns.len(), 3);
        assert_eq!(t.indexes.len(), 1);
        let idx = &t.indexes["IDX_STATUS_CREATED"];
        assert_eq!(idx.columns, vec!["status", "created_at"]);

        let _ = std::fs::remove_file(temp);
    }
}
