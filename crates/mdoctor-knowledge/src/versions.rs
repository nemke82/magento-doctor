//! Magento version requirements and service compatibility matrices.

#[derive(Debug, Clone)]
pub struct MagentoVersionSpec {
    pub version_prefix: &'static str,
    pub supported_php: &'static [&'static str],
    pub recommended_php: &'static str,
    pub supported_opensearch: &'static [&'static str],
    pub supported_mysql: &'static [&'static str],
    pub supported_mariadb: &'static [&'static str],
    pub supported_redis: &'static [&'static str],
    pub supported_valkey: &'static [&'static str],
}

pub const VERSION_MATRIX: &[MagentoVersionSpec] = &[
    MagentoVersionSpec {
        version_prefix: "2.4.4",
        supported_php: &["8.1"],
        recommended_php: "8.1",
        supported_opensearch: &["1.2", "2.5"],
        supported_mysql: &["8.0"],
        supported_mariadb: &["10.4", "10.6"],
        supported_redis: &["6.2", "7.0"],
        supported_valkey: &[],
    },
    MagentoVersionSpec {
        version_prefix: "2.4.5",
        supported_php: &["8.1", "8.2"],
        recommended_php: "8.1",
        supported_opensearch: &["1.2", "2.5"],
        supported_mysql: &["8.0"],
        supported_mariadb: &["10.4", "10.6"],
        supported_redis: &["6.2", "7.0"],
        supported_valkey: &[],
    },
    MagentoVersionSpec {
        version_prefix: "2.4.6",
        supported_php: &["8.1", "8.2"],
        recommended_php: "8.2",
        supported_opensearch: &["2.5", "2.12"],
        supported_mysql: &["8.0"],
        supported_mariadb: &["10.4", "10.6", "10.11"],
        supported_redis: &["7.0"],
        supported_valkey: &[],
    },
    MagentoVersionSpec {
        version_prefix: "2.4.7",
        supported_php: &["8.2", "8.3"],
        recommended_php: "8.3",
        supported_opensearch: &["2.12", "2.19"],
        supported_mysql: &["8.0"],
        supported_mariadb: &["10.6", "10.11"],
        supported_redis: &["7.2"],
        supported_valkey: &["7.2"],
    },
    MagentoVersionSpec {
        version_prefix: "2.4.8",
        supported_php: &["8.3", "8.4"],
        recommended_php: "8.3",
        supported_opensearch: &["2.19", "3.0", "3."],
        supported_mysql: &["8.0", "8.4", "9.0", "9.1"],
        supported_mariadb: &["10.11", "11.0", "11.1", "11.2", "11.4", "11."],
        supported_redis: &["7.2", "8.0"],
        supported_valkey: &["7.2", "8.0"],
    },
    MagentoVersionSpec {
        version_prefix: "2.4.9",
        supported_php: &["8.3", "8.4", "8.5"],
        recommended_php: "8.4",
        supported_opensearch: &["2.19", "3.0", "3."],
        supported_mysql: &["8.0", "8.4", "9.0", "9.1", "9.2"],
        supported_mariadb: &["10.11", "11.0", "11.1", "11.2", "11.4", "11."],
        supported_redis: &["7.2", "8.0"],
        supported_valkey: &["7.2", "8.0"],
    },
];

/// Find version specifications matching a Magento version string.
pub fn find_version_spec(magento_version: &str) -> Option<&'static MagentoVersionSpec> {
    VERSION_MATRIX
        .iter()
        .find(|spec| magento_version.starts_with(spec.version_prefix))
}

/// Check if a PHP version string is supported for a given Magento version.
pub fn is_php_supported(magento_version: &str, php_version: &str) -> Option<bool> {
    let spec = find_version_spec(magento_version)?;
    let mut clean = php_version.trim();
    for prefix in &["php-", "php:", "php", "v"] {
        clean = clean.strip_prefix(prefix).unwrap_or(clean);
    }
    let is_supp = spec
        .supported_php
        .iter()
        .any(|supp| clean.starts_with(supp));
    Some(is_supp)
}

/// Check if a MySQL version string is supported for a given Magento version.
pub fn is_mysql_supported(magento_version: &str, mysql_version: &str) -> Option<bool> {
    let spec = find_version_spec(magento_version)?;
    let mut clean = mysql_version.trim();
    for prefix in &["mysql-", "mysql:", "mysql", "v"] {
        clean = clean.strip_prefix(prefix).unwrap_or(clean);
    }
    let is_supp = spec
        .supported_mysql
        .iter()
        .any(|supp| clean.starts_with(supp));
    Some(is_supp)
}

/// Check if a MariaDB version string is supported for a given Magento version.
pub fn is_mariadb_supported(magento_version: &str, mariadb_version: &str) -> Option<bool> {
    let spec = find_version_spec(magento_version)?;
    let mut clean = mariadb_version.trim();
    for prefix in &["mariadb-", "mariadb:", "mariadb", "v"] {
        clean = clean.strip_prefix(prefix).unwrap_or(clean);
    }
    let is_supp = spec
        .supported_mariadb
        .iter()
        .any(|supp| clean.starts_with(supp));
    Some(is_supp)
}

/// Check if an OpenSearch version string is supported for a given Magento version.
pub fn is_opensearch_supported(magento_version: &str, opensearch_version: &str) -> Option<bool> {
    let spec = find_version_spec(magento_version)?;
    let mut clean = opensearch_version.trim();
    for prefix in &["opensearch-", "opensearch:", "opensearch", "v"] {
        clean = clean.strip_prefix(prefix).unwrap_or(clean);
    }
    let is_supp = spec
        .supported_opensearch
        .iter()
        .any(|supp| clean.starts_with(supp));
    Some(is_supp)
}

/// Check if a Redis or Valkey engine version string is supported for a given Magento version.
pub fn is_redis_or_valkey_supported(magento_version: &str, engine_version: &str) -> Option<bool> {
    let spec = find_version_spec(magento_version)?;
    let mut clean = engine_version.trim();
    for prefix in &["valkey-", "redis-", "valkey:", "redis:", "valkey", "redis", "v"] {
        clean = clean.strip_prefix(prefix).unwrap_or(clean);
    }
    let is_supp = spec.supported_redis.iter().any(|supp| clean.starts_with(supp))
        || spec.supported_valkey.iter().any(|supp| clean.starts_with(supp));
    Some(is_supp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_spec_247() {
        let spec = find_version_spec("2.4.7-p3").unwrap();
        assert_eq!(spec.recommended_php, "8.3");
        assert!(is_php_supported("2.4.7", "8.3.26").unwrap());
        assert!(!is_php_supported("2.4.7", "8.1.10").unwrap());
    }

    #[test]
    fn test_version_spec_249_modern_stack() {
        let spec = find_version_spec("2.4.9").unwrap();
        assert_eq!(spec.recommended_php, "8.4");

        // PHP 8.5
        assert!(is_php_supported("2.4.9", "8.5.0").unwrap());
        assert!(is_php_supported("2.4.9", "8.4.1").unwrap());

        // MariaDB 11
        assert!(is_mariadb_supported("2.4.9", "11.4.2-MariaDB").unwrap());
        assert!(is_mariadb_supported("2.4.9", "11.1.0").unwrap());

        // MySQL 8.4 LTS & 9.x latest
        assert!(is_mysql_supported("2.4.9", "8.4.1").unwrap());
        assert!(is_mysql_supported("2.4.9", "9.0.1").unwrap());

        // OpenSearch 3
        assert!(is_opensearch_supported("2.4.9", "3.0.0").unwrap());

        // Valkey 8 & Redis 8
        assert!(is_redis_or_valkey_supported("2.4.9", "8.0.0").unwrap());
        assert!(is_redis_or_valkey_supported("2.4.9", "valkey-8.0").unwrap());
        assert!(is_redis_or_valkey_supported("2.4.9", "7.2.5").unwrap());
    }
}
