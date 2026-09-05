//! Module scanner, registration, and classification.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use mdoctor_core::{Module, ModuleClassification, ModuleFootprint};
use regex::Regex;
use walkdir::WalkDir;

/// Read enabled/disabled status of modules from app/etc/config.php.
pub fn parse_config_php(config_path: &Path) -> HashMap<String, bool> {
    let mut map = HashMap::new();
    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return map,
    };

    let re = Regex::new(r#"['"]([a-zA-Z0-9_]+)['"]\s*=>\s*([01])"#).unwrap();
    for cap in re.captures_iter(&content) {
        let name = cap[1].to_string();
        let is_enabled = &cap[2] == "1";
        map.insert(name, is_enabled);
    }
    map
}

/// Scan Magento root for modules in app/code and vendor directories.
pub fn discover_modules(
    root: &Path,
    config_statuses: &HashMap<String, bool>,
    installed_packages: &HashMap<String, String>,
) -> Vec<Module> {
    let mut modules = Vec::new();
    let mut discovered_names = std::collections::HashSet::new();

    // 1. Scan app/code
    let app_code = root.join("app/code");
    if app_code.exists() {
        for entry in WalkDir::new(&app_code)
            .min_depth(2)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() {
                let module_xml = entry.path().join("etc/module.xml");
                if module_xml.exists() {
                    if let Some(module) = parse_module_xml_file(&module_xml, entry.path(), true, config_statuses, installed_packages) {
                        discovered_names.insert(module.name.clone());
                        modules.push(module);
                    }
                }
            }
        }
    }

    // 2. Scan vendor
    let vendor = root.join("vendor");
    if vendor.exists() {
        for entry in WalkDir::new(&vendor)
            .min_depth(2)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() {
                let module_xml = entry.path().join("etc/module.xml");
                if module_xml.exists() {
                    if let Some(module) = parse_module_xml_file(&module_xml, entry.path(), false, config_statuses, installed_packages) {
                        if !discovered_names.contains(&module.name) {
                            discovered_names.insert(module.name.clone());
                            modules.push(module);
                        }
                    }
                }
            }
        }
    }

    // 3. For any modules declared in config.php that were not found on disk, add as placeholder
    for (name, is_enabled) in config_statuses {
        if !discovered_names.contains(name) {
            let classification = classify_module(name, false);
            let vendor_name = name.split('_').next().unwrap_or("Unknown").to_string();
            modules.push(Module {
                name: name.clone(),
                vendor: vendor_name,
                package_name: None,
                version: None,
                sequence: Vec::new(),
                path: PathBuf::new(),
                is_enabled: *is_enabled,
                classification,
                footprint: ModuleFootprint::default(),
            });
        }
    }

    modules.sort_by(|a, b| a.name.cmp(&b.name));
    modules
}

fn parse_module_xml_file(
    module_xml_path: &Path,
    module_dir: &Path,
    is_app_code: bool,
    config_statuses: &HashMap<String, bool>,
    installed_packages: &HashMap<String, String>,
) -> Option<Module> {
    let content = std::fs::read_to_string(module_xml_path).ok()?;
    let doc = roxmltree::Document::parse(&content).ok()?;

    let module_node = doc.descendants().find(|n| n.has_tag_name("module"))?;
    let name = module_node.attribute("name")?.to_string();
    let setup_version = module_node.attribute("setup_version").map(|s| s.to_string());

    let mut sequence = Vec::new();
    if let Some(seq_node) = module_node.children().find(|n| n.has_tag_name("sequence")) {
        for child in seq_node.children().filter(|n| n.has_tag_name("module")) {
            if let Some(dep_name) = child.attribute("name") {
                sequence.push(dep_name.to_string());
            }
        }
    }

    let is_enabled = config_statuses.get(&name).copied().unwrap_or(true);
    let classification = classify_module(&name, is_app_code);
    let vendor = name.split('_').next().unwrap_or("Unknown").to_string();

    // Check composer package name if present
    let composer_json_path = module_dir.join("composer.json");
    let mut package_name = None;
    let mut version = setup_version;
    if let Ok(c_content) = std::fs::read_to_string(&composer_json_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&c_content) {
            if let Some(p_name) = val.get("name").and_then(|n| n.as_str()) {
                package_name = Some(p_name.to_string());
                if let Some(pkg_ver) = installed_packages.get(p_name) {
                    version = Some(pkg_ver.clone());
                }
            }
        }
    }

    Some(Module {
        name,
        vendor,
        package_name,
        version,
        sequence,
        path: module_dir.to_path_buf(),
        is_enabled,
        classification,
        footprint: ModuleFootprint::default(),
    })
}

pub fn classify_module(module_name: &str, is_app_code: bool) -> ModuleClassification {
    if is_app_code {
        return ModuleClassification::AppCodeCustom;
    }

    if module_name.starts_with("Magento_Enterprise")
        || module_name.starts_with("Magento_Banner")
        || module_name.starts_with("Magento_Advanced")
        || module_name.starts_with("Magento_GiftCard")
    {
        return ModuleClassification::AdobeCommerce;
    }

    if module_name.starts_with("Magento_") {
        return ModuleClassification::Core;
    }

    let bundled_prefixes = [
        "Amazon_",
        "Dotdigitalgroup_",
        "Klarna_",
        "Yotpo_",
        "Temando_",
        "Vertex_",
        "PayPal_",
    ];
    for prefix in bundled_prefixes {
        if module_name.starts_with(prefix) {
            return ModuleClassification::BundledThirdParty;
        }
    }

    ModuleClassification::ComposerThirdParty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_classification() {
        assert_eq!(
            classify_module("Magento_Catalog", false),
            ModuleClassification::Core
        );
        assert_eq!(
            classify_module("Magento_GiftCard", false),
            ModuleClassification::AdobeCommerce
        );
        assert_eq!(
            classify_module("Klarna_Core", false),
            ModuleClassification::BundledThirdParty
        );
        assert_eq!(
            classify_module("Amasty_Shopby", false),
            ModuleClassification::ComposerThirdParty
        );
        assert_eq!(
            classify_module("Vendor_Custom", true),
            ModuleClassification::AppCodeCustom
        );
    }
}
