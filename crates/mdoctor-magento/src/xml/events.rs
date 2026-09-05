//! events.xml parser for observers.

use std::path::Path;
use mdoctor_core::Observer;

pub fn parse_events_xml(file_path: &Path, module_name: &str, area: &str) -> Vec<Observer> {
    let mut observers = Vec::new();

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return observers,
    };

    let doc = match roxmltree::Document::parse(&content) {
        Ok(d) => d,
        Err(_) => return observers,
    };

    for event_node in doc.descendants().filter(|n| n.has_tag_name("event")) {
        let event_name = match event_node.attribute("name") {
            Some(n) => n,
            None => continue,
        };

        for obs_node in event_node.children().filter(|n| n.has_tag_name("observer")) {
            if let (Some(name), Some(instance)) = (obs_node.attribute("name"), obs_node.attribute("instance")) {
                let disabled = obs_node
                    .attribute("disabled")
                    .map(|d| d == "true" || d == "1")
                    .unwrap_or(false);
                let shared = obs_node
                    .attribute("shared")
                    .map(|s| s != "false" && s != "0")
                    .unwrap_or(true);

                let line = doc.text_pos_at(obs_node.range().start).row as usize;
                observers.push(Observer {
                    name: name.to_string(),
                    event_name: event_name.to_string(),
                    instance: instance.to_string(),
                    is_disabled: disabled,
                    is_shared: shared,
                    area: area.to_string(),
                    module: module_name.to_string(),
                    source_file: file_path.to_path_buf(),
                    line,
                });
            }
        }
    }

    observers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_events_xml_sample() {
        let sample = r#"<?xml version="1.0"?>
<config xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <event name="sales_order_place_after">
        <observer name="vendor_order_sync" instance="Vendor\ERP\Observer\OrderSync" />
    </event>
</config>
"#;
        let temp = std::env::temp_dir().join("test_events.xml");
        std::fs::write(&temp, sample).unwrap();

        let observers = parse_events_xml(&temp, "Vendor_ERP", "global");
        assert_eq!(observers.len(), 1);
        assert_eq!(observers[0].name, "vendor_order_sync");
        assert_eq!(observers[0].event_name, "sales_order_place_after");
        assert_eq!(observers[0].instance, "Vendor\\ERP\\Observer\\OrderSync");

        let _ = std::fs::remove_file(temp);
    }
}
