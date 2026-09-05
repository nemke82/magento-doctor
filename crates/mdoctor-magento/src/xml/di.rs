//! di.xml parser for plugins and preferences.

use std::path::Path;
use mdoctor_core::{Plugin, PluginType, Preference};

pub fn parse_di_xml(file_path: &Path, module_name: &str, area: &str) -> (Vec<Plugin>, Vec<Preference>) {
    let mut plugins = Vec::new();
    let mut preferences = Vec::new();

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return (plugins, preferences),
    };

    let doc = match roxmltree::Document::parse(&content) {
        Ok(d) => d,
        Err(_) => return (plugins, preferences),
    };

    for node in doc.descendants() {
        if node.has_tag_name("preference") {
            if let (Some(for_class), Some(type_class)) = (node.attribute("for"), node.attribute("type")) {
                let line = doc.text_pos_at(node.range().start).row as usize;
                preferences.push(Preference {
                    for_class: for_class.to_string(),
                    type_class: type_class.to_string(),
                    module: module_name.to_string(),
                    area: area.to_string(),
                    source_file: file_path.to_path_buf(),
                    line,
                });
            }
        } else if node.has_tag_name("plugin") {
            if let (Some(plugin_name), Some(plugin_type_str)) = (node.attribute("name"), node.attribute("type")) {
                let parent = node.parent();
                let target_class = parent
                    .filter(|p| p.has_tag_name("type"))
                    .and_then(|p| p.attribute("name"))
                    .unwrap_or("")
                    .to_string();

                let sort_order = node
                    .attribute("sortOrder")
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);

                let is_disabled = node
                    .attribute("disabled")
                    .map(|d| d == "true" || d == "1")
                    .unwrap_or(false);

                // Guess plugin type or default to Unknown until inspected
                let p_type = if plugin_name.to_lowercase().contains("around") {
                    PluginType::Around
                } else if plugin_name.to_lowercase().contains("before") {
                    PluginType::Before
                } else if plugin_name.to_lowercase().contains("after") {
                    PluginType::After
                } else {
                    PluginType::Around // Default to Around if unspecified or inspect later
                };

                let line = doc.text_pos_at(node.range().start).row as usize;
                plugins.push(Plugin {
                    name: plugin_name.to_string(),
                    module: module_name.to_string(),
                    target_class,
                    plugin_class: plugin_type_str.to_string(),
                    plugin_type: p_type,
                    sort_order,
                    is_disabled,
                    area: area.to_string(),
                    source_file: file_path.to_path_buf(),
                    line,
                    cost_indicators: Vec::new(),
                });
            }
        }
    }

    (plugins, preferences)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_di_xml_sample() {
        let sample = r#"<?xml version="1.0"?>
<config xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <preference for="Magento\Catalog\Model\Product" type="Vendor\Catalog\Model\Product" />
    <type name="Magento\Quote\Model\QuoteManagement">
        <plugin name="vendor_quote_around" type="Vendor\Payment\Plugin\QuoteManagement" sortOrder="10" />
    </type>
</config>
"#;
        let temp = std::env::temp_dir().join("test_di.xml");
        std::fs::write(&temp, sample).unwrap();

        let (plugins, preferences) = parse_di_xml(&temp, "Vendor_Payment", "global");
        assert_eq!(preferences.len(), 1);
        assert_eq!(preferences[0].for_class, "Magento\\Catalog\\Model\\Product");
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].target_class, "Magento\\Quote\\Model\\QuoteManagement");
        assert_eq!(plugins[0].plugin_class, "Vendor\\Payment\\Plugin\\QuoteManagement");
        assert_eq!(plugins[0].sort_order, 10);

        let _ = std::fs::remove_file(temp);
    }
}
