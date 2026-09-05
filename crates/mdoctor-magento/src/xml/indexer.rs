//! indexer.xml parser.

use std::path::Path;
use mdoctor_core::Indexer;

pub fn parse_indexer_xml(file_path: &Path, module_name: &str) -> Vec<Indexer> {
    let mut indexers = Vec::new();

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return indexers,
    };

    let doc = match roxmltree::Document::parse(&content) {
        Ok(d) => d,
        Err(_) => return indexers,
    };

    for node in doc.descendants().filter(|n| n.has_tag_name("indexer")) {
        let id = match node.attribute("id") {
            Some(i) => i.to_string(),
            None => continue,
        };

        let view_id = node.attribute("view_id").unwrap_or(&id).to_string();
        let class = node.attribute("class").unwrap_or("").to_string();

        let mut title = id.clone();
        let mut description = String::new();

        if let Some(t_node) = node.children().find(|n| n.has_tag_name("title")) {
            if let Some(t) = t_node.text() {
                title = t.trim().to_string();
            }
        }
        if let Some(d_node) = node.children().find(|n| n.has_tag_name("description")) {
            if let Some(d) = d_node.text() {
                description = d.trim().to_string();
            }
        }

        indexers.push(Indexer {
            id,
            view_id,
            action_class: class,
            title,
            description,
            module: module_name.to_string(),
            is_scheduled: None,
            status: None,
            updated_at: None,
        });
    }

    indexers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_indexer_xml_sample() {
        let sample = r#"<?xml version="1.0"?>
<config xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <indexer id="catalog_product_price" view_id="catalog_product_price" class="Magento\Catalog\Model\Indexer\Product\Price">
        <title translate="true">Product Price</title>
        <description translate="true">Indexes product prices</description>
    </indexer>
</config>
"#;
        let temp = std::env::temp_dir().join("test_indexer.xml");
        std::fs::write(&temp, sample).unwrap();

        let indexers = parse_indexer_xml(&temp, "Magento_Catalog");
        assert_eq!(indexers.len(), 1);
        assert_eq!(indexers[0].id, "catalog_product_price");
        assert_eq!(indexers[0].title, "Product Price");

        let _ = std::fs::remove_file(temp);
    }
}
