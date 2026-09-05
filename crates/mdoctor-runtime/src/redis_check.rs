//! Redis configuration and database collision inspection.

use mdoctor_core::SanitizedEnvConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedisIssue {
    SameDatabaseForSessionAndCache { session_db: String, cache_db: String },
    SameDatabaseForSessionAndPageCache { session_db: String, fpc_db: String },
    SameDatabaseForCacheAndPageCache { cache_db: String, fpc_db: String },
}

/// Analyze Redis configuration for collision bugs.
pub fn check_redis_config(config: &SanitizedEnvConfig) -> Vec<RedisIssue> {
    let mut issues = Vec::new();

    let session_db = config.redis_session_db.as_deref();
    let cache_db = config.redis_cache_db.as_deref();
    let fpc_db = config.redis_page_cache_db.as_deref();

    if let (Some(s), Some(c)) = (session_db, cache_db) {
        if s == c {
            issues.push(RedisIssue::SameDatabaseForSessionAndCache {
                session_db: s.to_string(),
                cache_db: c.to_string(),
            });
        }
    }

    if let (Some(s), Some(f)) = (session_db, fpc_db) {
        if s == f {
            issues.push(RedisIssue::SameDatabaseForSessionAndPageCache {
                session_db: s.to_string(),
                fpc_db: f.to_string(),
            });
        }
    }

    if let (Some(c), Some(f)) = (cache_db, fpc_db) {
        if c == f {
            issues.push(RedisIssue::SameDatabaseForCacheAndPageCache {
                cache_db: c.to_string(),
                fpc_db: f.to_string(),
            });
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_collision_detection() {
        let mut config = SanitizedEnvConfig::default();
        config.redis_session_db = Some("0".to_string());
        config.redis_cache_db = Some("0".to_string());
        config.redis_page_cache_db = Some("1".to_string());

        let issues = check_redis_config(&config);
        assert_eq!(issues.len(), 1);
        assert!(matches!(
            issues[0],
            RedisIssue::SameDatabaseForSessionAndCache { .. }
        ));
    }
}
