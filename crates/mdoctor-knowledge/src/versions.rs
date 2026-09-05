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
    },
    MagentoVersionSpec {
        version_prefix: "2.4.5",
        supported_php: &["8.1", "8.2"],
        recommended_php: "8.1",
        supported_opensearch: &["1.2", "2.5"],
        supported_mysql: &["8.0"],
        supported_mariadb: &["10.4", "10.6"],
        supported_redis: &["6.2", "7.0"],
    },
    MagentoVersionSpec {
        version_prefix: "2.4.6",
        supported_php: &["8.1", "8.2"],
        recommended_php: "8.2",
        supported_opensearch: &["2.5", "2.12"],
        supported_mysql: &["8.0"],
        supported_mariadb: &["10.4", "10.6", "10.11"],
        supported_redis: &["7.0"],
    },
    MagentoVersionSpec {
        version_prefix: "2.4.7",
        supported_php: &["8.2", "8.3"],
        recommended_php: "8.3",
        supported_opensearch: &["2.12", "2.19"],
        supported_mysql: &["8.0"],
        supported_mariadb: &["10.6", "10.11"],
        supported_redis: &["7.2"],
    },
    MagentoVersionSpec {
        version_prefix: "2.4.8",
        supported_php: &["8.3", "8.4"],
        recommended_php: "8.3",
        supported_opensearch: &["2.19", "3.0"],
        supported_mysql: &["8.0", "8.4"],
        supported_mariadb: &["10.11", "11.4"],
        supported_redis: &["7.2", "8.0"],
    },
    MagentoVersionSpec {
        version_prefix: "2.4.9",
        supported_php: &["8.3", "8.4", "8.5"],
        recommended_php: "8.4",
        supported_opensearch: &["2.19", "3.0"],
        supported_mysql: &["8.0", "8.4", "9.0"],
        supported_mariadb: &["10.11", "11.0", "11.4"],
        supported_redis: &["7.2", "8.0"],
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
    let clean_php = php_version.trim();
    let is_supp = spec
        .supported_php
        .iter()
        .any(|supp| clean_php.starts_with(supp));
    Some(is_supp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_spec() {
        let spec = find_version_spec("2.4.7-p3").unwrap();
        assert_eq!(spec.recommended_php, "8.3");
        assert!(is_php_supported("2.4.7", "8.3.26").unwrap());
        assert!(!is_php_supported("2.4.7", "8.1.10").unwrap());
    }
}
