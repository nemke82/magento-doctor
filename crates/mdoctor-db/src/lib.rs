//! mdoctor_db: Database schema reconciliation, index rules, and MySQL live inspector.

pub mod fingerprint;
pub mod index_rules;
pub mod live_inspector;
pub mod schema_diff;

pub use fingerprint::fingerprint_query;
pub use index_rules::{find_redundant_indexes, RedundantIndex};
pub use live_inspector::inspect_live_database;
pub use schema_diff::{reconcile_schemas, SchemaDiffReport};
