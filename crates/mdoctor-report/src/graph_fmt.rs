//! Mermaid.js architecture graph generator for Magento modules.

use mdoctor_core::{MagentoInstallation, Module};
use std::collections::HashMap;

/// Sanitizes a string for use as a Mermaid node ID.
fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// Generates a valid Mermaid flowchart diagram visualizing the module's architecture and touchpoints.
pub fn render_mermaid_graph(module: &Module, installation: &MagentoInstallation) -> String {
    let mut out = String::new();

    out.push_str("flowchart TD\n");
    out.push_str("    %% Styling definitions\n");
    out.push_str("    classDef moduleNode fill:#2d3748,stroke:#1a202c,stroke-width:3px,color:#fff;\n");
    out.push_str("    classDef classNode fill:#2b6cb0,stroke:#2c5282,stroke-width:2px,color:#fff;\n");
    out.push_str("    classDef eventNode fill:#6b46c1,stroke:#553c9a,stroke-width:2px,color:#fff;\n");
    out.push_str("    classDef cronNode fill:#d69e2e,stroke:#b7791f,stroke-width:2px,color:#000;\n");
    out.push_str("    classDef dbNode fill:#2f855a,stroke:#22543d,stroke-width:2px,color:#fff;\n");
    out.push_str("    classDef depNode fill:#4a5568,stroke:#2d3748,stroke-width:1px,color:#fff;\n\n");

    let mod_id = sanitize_id(&module.name);
    let version_str = module.version.as_deref().unwrap_or("1.0.0");
    out.push_str(&format!(
        "    {}[\"<b>{}</b><br/>({})<br/>v{}\"]:::moduleNode\n\n",
        mod_id, module.name, module.classification, version_str
    ));

    // 1. Dependencies (Sequences)
    if !module.sequence.is_empty() {
        out.push_str("    subgraph Dependencies [Sequence Dependencies]\n");
        for dep in &module.sequence {
            let dep_id = format!("DEP_{}", sanitize_id(dep));
            out.push_str(&format!("        {}[\"{}\"]:::depNode\n", dep_id, dep));
            out.push_str(&format!("        {} -.->|depends on| {}\n", mod_id, dep_id));
        }
        out.push_str("    end\n\n");
    }

    // 2. Plugins & Interceptions
    let module_plugins: Vec<_> = installation
        .plugins
        .iter()
        .filter(|p| p.module == module.name)
        .collect();

    if !module_plugins.is_empty() {
        out.push_str("    subgraph Interceptions [Plugins & Intercepted Classes]\n");
        let mut seen_classes = HashMap::new();
        for p in &module_plugins {
            let cls_id = format!("CLS_{}", sanitize_id(&p.target_class));
            if !seen_classes.contains_key(&cls_id) {
                seen_classes.insert(cls_id.clone(), p.target_class.clone());
                out.push_str(&format!("        {}[\"{}\"]:::classNode\n", cls_id, p.target_class));
            }
            out.push_str(&format!(
                "        {} -->|{} plugin| {}\n",
                mod_id, p.plugin_type, cls_id
            ));
        }
        out.push_str("    end\n\n");
    }

    // 3. Observers & Events
    let module_observers: Vec<_> = installation
        .observers
        .iter()
        .filter(|o| o.module == module.name)
        .collect();

    if !module_observers.is_empty() {
        out.push_str("    subgraph Events [Event Listeners]\n");
        for obs in &module_observers {
            let evt_id = format!("EVT_{}", sanitize_id(&obs.event_name));
            out.push_str(&format!("        {}[\"{}\"]:::eventNode\n", evt_id, obs.event_name));
            out.push_str(&format!("        {} -->|observes| {}\n", mod_id, evt_id));
        }
        out.push_str("    end\n\n");
    }

    // 4. Cron Jobs
    let module_crons: Vec<_> = installation
        .cron_jobs
        .iter()
        .filter(|c| c.module == module.name)
        .collect();

    if !module_crons.is_empty() {
        out.push_str("    subgraph ScheduledTasks [Cron Jobs]\n");
        for cron in &module_crons {
            let cron_id = format!("CRON_{}", sanitize_id(&cron.name));
            let sched = cron.schedule.as_deref().unwrap_or("none");
            out.push_str(&format!(
                "        {}[\"{}<br/>Schedule: {}\"]:::cronNode\n",
                cron_id, cron.name, sched
            ));
            out.push_str(&format!("        {} -->|runs| {}\n", mod_id, cron_id));
        }
        out.push_str("    end\n\n");
    }

    // 5. Database Tables
    let module_tables: Vec<_> = installation
        .declared_schema
        .tables
        .iter()
        .filter(|(_, t)| t.owning_module.as_deref() == Some(&module.name))
        .collect();

    if !module_tables.is_empty() {
        out.push_str("    subgraph Database [Database Tables]\n");
        for (t_name, _) in &module_tables {
            let tbl_id = format!("TBL_{}", sanitize_id(t_name));
            out.push_str(&format!("        {}[(\"{}\")]:::dbNode\n", tbl_id, t_name));
            out.push_str(&format!("        {} -->|owns| {}\n", mod_id, tbl_id));
        }
        out.push_str("    end\n\n");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdoctor_core::{ModuleClassification, ModuleFootprint, Plugin, PluginType};
    use std::path::PathBuf;

    #[test]
    fn test_render_mermaid_graph_syntax() {
        let mut inst = MagentoInstallation::new(PathBuf::from("/var/www/html"));
        let module = Module {
            name: "Vendor_CatalogExtra".to_string(),
            vendor: "Vendor".to_string(),
            package_name: None,
            version: Some("2.1.0".to_string()),
            sequence: vec!["Magento_Catalog".to_string()],
            path: PathBuf::new(),
            is_enabled: true,
            classification: ModuleClassification::AppCodeCustom,
            footprint: ModuleFootprint::default(),
        };

        inst.plugins.push(Plugin {
            name: "vendor_extra_plugin".to_string(),
            module: "Vendor_CatalogExtra".to_string(),
            target_class: "Magento\\Catalog\\Model\\Product".to_string(),
            plugin_class: "Vendor\\CatalogExtra\\Plugin\\ProductPlugin".to_string(),
            plugin_type: PluginType::After,
            sort_order: 10,
            is_disabled: false,
            area: "frontend".to_string(),
            source_file: PathBuf::new(),
            line: 1,
            cost_indicators: vec![],
        });

        let graph = render_mermaid_graph(&module, &inst);
        assert!(graph.starts_with("flowchart TD\n"));
        assert!(graph.contains("Vendor_CatalogExtra"));
        assert!(graph.contains("Magento_Catalog"));
        assert!(graph.contains("Magento\\Catalog\\Model\\Product"));
    }
}
