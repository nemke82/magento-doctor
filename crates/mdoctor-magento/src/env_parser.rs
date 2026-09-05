//! Parser for app/etc/env.php safely redacting all secrets.

use std::path::Path;
use mdoctor_core::{MagentoMode, SanitizedEnvConfig, SecretValue};
use regex::Regex;

/// Result of parsing app/etc/env.php.
#[derive(Debug, Default)]
pub struct ParsedEnv {
    pub mode: MagentoMode,
    pub config: SanitizedEnvConfig,
    pub raw_db_password: Option<String>, // Only used for live connection in mdoctor_db, never serialized or exposed!
}

/// Parse app/etc/env.php safely.
pub fn parse_env_php(env_file_path: &Path) -> ParsedEnv {
    let mut parsed = ParsedEnv::default();
    let content = match std::fs::read_to_string(env_file_path) {
        Ok(c) => c,
        Err(_) => return parsed,
    };

    // 1. Parse MAGE_MODE
    let mode_re = Regex::new(r#"['"]MAGE_MODE['"]\s*=>\s*['"]([a-zA-Z0-9_-]+)['"]"#).unwrap();
    if let Some(caps) = mode_re.captures(&content) {
        parsed.mode = match caps[1].to_lowercase().as_str() {
            "production" => MagentoMode::Production,
            "developer" => MagentoMode::Developer,
            "default" => MagentoMode::DefaultMode,
            "maintenance" => MagentoMode::Maintenance,
            _ => MagentoMode::Unknown,
        };
    }

    // 2. Parse Crypt key presence
    let crypt_re = Regex::new(r#"['"]crypt['"]\s*=>\s*\[[^\]]*['"]key['"]\s*=>\s*['"]([^'"]*)['"]"#).unwrap();
    if let Some(caps) = crypt_re.captures(&content) {
        let key_str = &caps[1];
        parsed.config.crypt_key_secret = if !key_str.trim().is_empty() {
            SecretValue::Present
        } else {
            SecretValue::Missing
        };
    }

    // 3. Parse Database connection
    let db_host_re = Regex::new(r#"['"]host['"]\s*=>\s*['"]([^'"]+)['"]"#).unwrap();
    if let Some(caps) = db_host_re.captures(&content) {
        parsed.config.db_host = Some(caps[1].to_string());
    }

    let db_name_re = Regex::new(r#"['"]dbname['"]\s*=>\s*['"]([^'"]+)['"]"#).unwrap();
    if let Some(caps) = db_name_re.captures(&content) {
        parsed.config.db_name = Some(caps[1].to_string());
    }

    let db_user_re = Regex::new(r#"['"]username['"]\s*=>\s*['"]([^'"]+)['"]"#).unwrap();
    if let Some(caps) = db_user_re.captures(&content) {
        parsed.config.db_user = Some(caps[1].to_string());
    }

    let db_pass_re = Regex::new(r#"['"]password['"]\s*=>\s*['"]([^'"]*)['"]"#).unwrap();
    if let Some(caps) = db_pass_re.captures(&content) {
        let pass = caps[1].to_string();
        if !pass.trim().is_empty() {
            parsed.config.db_password_secret = SecretValue::Present;
            parsed.raw_db_password = Some(pass);
        } else {
            parsed.config.db_password_secret = SecretValue::Missing;
        }
    }

    // 4. Redis session database
    if let Some(session_pos) = content.find("'session'").or_else(|| content.find("\"session\"")) {
        let session_slice = &content[session_pos..];
        // Stop before next top-level key or end
        let redis_session_db_re = Regex::new(r#"['"]database['"]\s*=>\s*['"]?([0-9]+)['"]?"#).unwrap();
        if let Some(caps) = redis_session_db_re.captures(session_slice) {
            parsed.config.redis_session_db = Some(caps[1].to_string());
        }
    }

    // 5. Redis cache & page cache database
    if let Some(cache_pos) = content.find("'cache'").or_else(|| content.find("\"cache\"")) {
        let cache_slice = &content[cache_pos..];

        let redis_cache_db_re = Regex::new(r#"['"]default['"]\s*=>\s*\[[\s\S]*?['"]database['"]\s*=>\s*['"]?([0-9]+)['"]?"#).unwrap();
        if let Some(caps) = redis_cache_db_re.captures(cache_slice) {
            parsed.config.redis_cache_db = Some(caps[1].to_string());
        }

        let redis_fpc_db_re = Regex::new(r#"['"]page_cache['"]\s*=>\s*\[[\s\S]*?['"]database['"]\s*=>\s*['"]?([0-9]+)['"]?"#).unwrap();
        if let Some(caps) = redis_fpc_db_re.captures(cache_slice) {
            parsed.config.redis_page_cache_db = Some(caps[1].to_string());
        }
    }

    parsed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_php_sample() {
        let sample = r#"<?php
return [
    'backend' => ['frontName' => 'admin'],
    'crypt' => ['key' => 'topsecretkey123'],
    'db' => [
        'connection' => [
            'default' => [
                'host' => '127.0.0.1',
                'dbname' => 'magento_test',
                'username' => 'db_user',
                'password' => 'super_secret_db_pass',
            ]
        ]
    ],
    'MAGE_MODE' => 'production',
    'session' => [
        'save' => 'redis',
        'redis' => [
            'host' => '127.0.0.1',
            'database' => '2'
        ]
    ],
    'cache' => [
        'frontend' => [
            'default' => ['backend_options' => ['database' => '0']],
            'page_cache' => ['backend_options' => ['database' => '1']]
        ]
    ]
];
"#;
        let temp = std::env::temp_dir().join("test_env.php");
        std::fs::write(&temp, sample).unwrap();

        let parsed = parse_env_php(&temp);
        assert_eq!(parsed.mode, MagentoMode::Production);
        assert_eq!(parsed.config.db_host.as_deref(), Some("127.0.0.1"));
        assert_eq!(parsed.config.db_name.as_deref(), Some("magento_test"));
        assert_eq!(parsed.config.db_user.as_deref(), Some("db_user"));
        assert_eq!(parsed.config.db_password_secret, SecretValue::Present);
        assert_eq!(parsed.config.crypt_key_secret, SecretValue::Present);
        assert_eq!(parsed.config.redis_session_db.as_deref(), Some("2"));
        assert_eq!(parsed.config.redis_cache_db.as_deref(), Some("0"));
        assert_eq!(parsed.config.redis_page_cache_db.as_deref(), Some("1"));

        let _ = std::fs::remove_file(temp);
    }
}
