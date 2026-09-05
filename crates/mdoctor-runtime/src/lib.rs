//! mdoctor_runtime: Environment, PHP, Redis, and filesystem inspection.

pub mod filesystem;
pub mod php_check;
pub mod redis_check;

pub use filesystem::{inspect_filesystem, FsCheckResult};
pub use php_check::{check_php_runtime, PhpRuntimeIssue};
pub use redis_check::{check_redis_config, RedisIssue};
