//! Cache and Redis rules (MD-CACHE-001).

use mdoctor_core::{Category, Confidence, Finding, MagentoInstallation, Severity};
use mdoctor_runtime::{check_redis_config, RedisIssue};

pub fn evaluate_cache_rules(installation: &MagentoInstallation) -> Vec<Finding> {
    let mut findings = Vec::new();

    let redis_issues = check_redis_config(&installation.env_config);
    for issue in redis_issues {
        match issue {
            RedisIssue::SameDatabaseForSessionAndCache { session_db, cache_db } => {
                let mut finding = Finding::new(
                    "MD-CACHE-001",
                    "Redis session and cache share the same database number",
                    Severity::Critical,
                    Confidence::High,
                    Category::Cache,
                );

                finding.summary = format!(
                    "Redis session storage and default cache are configured to use the exact same database (DB {}).",
                    session_db
                );

                finding.evidence.push(format!("Session database: {}", session_db));
                finding.evidence.push(format!("Cache database: {}", cache_db));

                finding.impact = "Running 'bin/magento cache:flush' or cache eviction will destroy all active user sessions, logging out shoppers and clearing carts.".to_string();
                finding.recommendation = "Assign dedicated, distinct Redis database numbers (e.g. DB 0 for cache, DB 1 for FPC, DB 2 for sessions) or separate Redis instances in app/etc/env.php.".to_string();
                finding.verification_commands = vec![
                    "cat app/etc/env.php | grep -E '(database|session|cache)'".to_string(),
                ];

                findings.push(finding);
            }
            RedisIssue::SameDatabaseForSessionAndPageCache { session_db, fpc_db } => {
                let mut finding = Finding::new(
                    "MD-CACHE-001",
                    "Redis session and page cache share the same database number",
                    Severity::Critical,
                    Confidence::High,
                    Category::Cache,
                );

                finding.summary = format!(
                    "Redis session storage and page cache (FPC) share database {}.",
                    session_db
                );
                finding.evidence.push(format!("Session database: {}", session_db));
                finding.evidence.push(format!("Page cache database: {}", fpc_db));
                finding.impact = "Full page cache flushes will purge active customer sessions.".to_string();
                finding.recommendation = "Separate Redis database IDs in app/etc/env.php.".to_string();

                findings.push(finding);
            }
            RedisIssue::SameDatabaseForCacheAndPageCache { cache_db, fpc_db } => {
                let mut finding = Finding::new(
                    "MD-CACHE-002",
                    "Redis default cache and page cache share the same database number",
                    Severity::Warning,
                    Confidence::High,
                    Category::Cache,
                );

                finding.summary = format!(
                    "Default cache and page cache share database {}.",
                    cache_db
                );
                finding.evidence.push(format!("Default cache database: {}", cache_db));
                finding.evidence.push(format!("Page cache database: {}", fpc_db));
                finding.impact = "Flushing default cache unexpectedly purges full page cache, causing cold cache spikes.".to_string();
                finding.recommendation = "Separate database IDs in app/etc/env.php.".to_string();

                findings.push(finding);
            }
        }
    }

    // 2. MD-CACHE-003: Redis / Valkey version compatibility check
    if let (Some(m_ver), Some(cache_ver)) = (
        installation.version.as_deref(),
        installation.runtime.redis_default.version.as_deref(),
    ) {
        if let Some(false) = mdoctor_knowledge::versions::is_redis_or_valkey_supported(m_ver, cache_ver) {
            let mut finding = Finding::new(
                "MD-CACHE-003",
                format!("Unsupported Redis/Valkey version ({}) for Magento {}", cache_ver, m_ver),
                Severity::Warning,
                Confidence::High,
                Category::Cache,
            );

            finding.summary = format!(
                "Redis/Valkey server version '{}' is outside the supported compatibility matrix for Magento {}.",
                cache_ver, m_ver
            );
            finding.evidence.push(format!("Running version: {}", cache_ver));
            finding.evidence.push(format!("Magento version: {}", m_ver));
            finding.impact = "Unsupported cache servers can cause command protocol incompatibilities, unexpected eviction anomalies, or memory leaks.".to_string();
            finding.recommendation = format!(
                "Use a certified Redis or Valkey release (such as Redis 7.2/8.0 or Valkey 8.0) compatible with Magento {}.",
                m_ver
            );

            findings.push(finding);
        }
    }

    findings
}
