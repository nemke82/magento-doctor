//! Upward search and validation of Magento 2 installations.

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("Could not find a valid Magento installation at '{0}' or parent directories")]
    NotFound(PathBuf),
    #[error("Incomplete Magento installation: missing {0}")]
    Incomplete(String),
}

/// Discovers the Magento root directory.
/// If `custom_root` is provided, validates that directory.
/// Otherwise, checks `MAGENTO_ROOT` env var, then searches upwards from `start_dir`.
pub fn discover_magento_root(
    custom_root: Option<&Path>,
    start_dir: Option<&Path>,
) -> Result<PathBuf, DiscoveryError> {
    if let Some(root) = custom_root {
        if is_valid_magento_root(root) {
            return Ok(root.to_path_buf());
        } else {
            return Err(DiscoveryError::NotFound(root.to_path_buf()));
        }
    }

    if let Ok(env_root) = std::env::var("MAGENTO_ROOT") {
        let path = PathBuf::from(env_root);
        if is_valid_magento_root(&path) {
            return Ok(path);
        }
    }

    let start = match start_dir {
        Some(d) => d.to_path_buf(),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let mut current = start.canonicalize().unwrap_or(start);
    loop {
        if is_valid_magento_root(&current) {
            return Ok(current);
        }
        if !current.pop() {
            break;
        }
    }

    let fallback = start_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    Err(DiscoveryError::NotFound(fallback))
}

/// Checks if a directory appears to be a Magento 2 root installation.
pub fn is_valid_magento_root(path: &Path) -> bool {
    let has_bin_magento = path.join("bin/magento").exists();
    let has_app_etc = path.join("app/etc").exists();
    let has_composer = path.join("composer.json").exists();

    (has_bin_magento || has_composer) && has_app_etc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_magento_root_check() {
        let temp = std::env::temp_dir().join("test_m2_root");
        let _ = std::fs::create_dir_all(temp.join("app/etc"));
        let _ = std::fs::create_dir_all(temp.join("bin"));
        let _ = std::fs::write(temp.join("bin/magento"), "#!/usr/bin/env php");

        assert!(is_valid_magento_root(&temp));

        let non_m2 = std::env::temp_dir().join("not_m2");
        assert!(!is_valid_magento_root(&non_m2));

        let _ = std::fs::remove_dir_all(temp);
    }
}
