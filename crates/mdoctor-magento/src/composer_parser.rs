//! Composer package and version inspector.

use std::collections::HashMap;
use std::path::Path;
use mdoctor_core::Edition;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct ComposerInfo {
    pub magento_version: Option<String>,
    pub edition: Edition,
    pub installed_packages: HashMap<String, String>, // package_name -> version
}

pub fn parse_composer_info(root: &Path) -> ComposerInfo {
    let mut info = ComposerInfo::default();

    // 1. Try reading vendor/composer/installed.json
    let installed_json_path = root.join("vendor/composer/installed.json");
    if let Ok(content) = std::fs::read_to_string(&installed_json_path) {
        if let Ok(val) = serde_json::from_str::<Value>(&content) {
            let packages_array = if let Some(arr) = val.get("packages").and_then(|p| p.as_array()) {
                arr
            } else if let Some(arr) = val.as_array() {
                arr
            } else {
                &vec![]
            };

            for pkg in packages_array {
                if let (Some(name), Some(version)) = (
                    pkg.get("name").and_then(|n| n.as_str()),
                    pkg.get("version").and_then(|v| v.as_str()),
                ) {
                    info.installed_packages
                        .insert(name.to_string(), version.to_string());
                }
            }
        }
    }

    // 2. Try reading composer.lock if installed.json had nothing or to supplement
    let lock_path = root.join("composer.lock");
    if let Ok(content) = std::fs::read_to_string(&lock_path) {
        if let Ok(val) = serde_json::from_str::<Value>(&content) {
            if let Some(packages) = val.get("packages").and_then(|p| p.as_array()) {
                for pkg in packages {
                    if let (Some(name), Some(version)) = (
                        pkg.get("name").and_then(|n| n.as_str()),
                        pkg.get("version").and_then(|v| v.as_str()),
                    ) {
                        info.installed_packages
                            .entry(name.to_string())
                            .or_insert_with(|| version.to_string());
                    }
                }
            }
        }
    }

    // 3. Determine edition and Magento version
    if let Some(v) = info.installed_packages.get("magento/product-enterprise-edition") {
        info.magento_version = Some(clean_version(v));
        info.edition = Edition::AdobeCommerce;
    } else if let Some(v) = info.installed_packages.get("magento/product-community-edition") {
        info.magento_version = Some(clean_version(v));
        info.edition = Edition::OpenSource;
    } else if let Some(v) = info.installed_packages.get("magento/framework") {
        info.magento_version = Some(clean_version(v));
        info.edition = Edition::OpenSource;
    } else {
        // Fallback: check root composer.json "require"
        let root_composer_path = root.join("composer.json");
        if let Ok(content) = std::fs::read_to_string(&root_composer_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                if let Some(req) = val.get("require").and_then(|r| r.as_object()) {
                    if let Some(v) = req.get("magento/product-enterprise-edition").and_then(|s| s.as_str()) {
                        info.magento_version = Some(clean_version(v));
                        info.edition = Edition::AdobeCommerce;
                    } else if let Some(v) = req.get("magento/product-community-edition").and_then(|s| s.as_str()) {
                        info.magento_version = Some(clean_version(v));
                        info.edition = Edition::OpenSource;
                    }
                }
            }
        }
    }

    info
}

fn clean_version(raw: &str) -> String {
    raw.trim_start_matches('v')
        .trim_start_matches('^')
        .trim_start_matches('~')
        .to_string()
}
