//! Filesystem health and permission checks.

use std::path::Path;

#[derive(Debug, Clone)]
pub struct FsCheckResult {
    pub path: String,
    pub exists: bool,
    pub is_writable: bool,
    pub is_world_writable: bool,
}

/// Checks required writable Magento paths.
pub fn inspect_filesystem(root: &Path) -> Vec<FsCheckResult> {
    let required_paths = [
        "var",
        "generated",
        "pub/static",
        "pub/media",
        "app/etc",
    ];

    let mut results = Vec::new();
    for rel in required_paths {
        let p = root.join(rel);
        let exists = p.exists();
        let is_writable = if exists {
            // Check writability by attempting to read metadata or check permissions
            std::fs::metadata(&p)
                .map(|m| !m.permissions().readonly())
                .unwrap_or(false)
        } else {
            false
        };

        // On Unix, check if world-writable
        #[cfg(unix)]
        let is_world_writable = if exists {
            use std::os::unix::fs::PermissionsExt;
            std::fs::metadata(&p)
                .map(|m| (m.permissions().mode() & 0o002) != 0)
                .unwrap_or(false)
        } else {
            false
        };
        #[cfg(not(unix))]
        let is_world_writable = false;

        results.push(FsCheckResult {
            path: rel.to_string(),
            exists,
            is_writable,
            is_world_writable,
        });
    }

    results
}
