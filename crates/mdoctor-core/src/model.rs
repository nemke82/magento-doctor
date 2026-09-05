//! Normalized installation model representing a complete Magento store state.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Secret value representation ensuring sensitive values are never logged or exported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SecretValue {
    Present,
    #[default]
    Missing,
}

impl SecretValue {
    pub fn from_opt_str(val: Option<&str>) -> Self {
        match val {
            Some(s) if !s.trim().is_empty() => SecretValue::Present,
            _ => SecretValue::Missing,
        }
    }

    pub fn is_present(&self) -> bool {
        matches!(self, SecretValue::Present)
    }
}

impl std::fmt::Display for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretValue::Present => write!(f, "[REDACTED: PRESENT]"),
            SecretValue::Missing => write!(f, "[NOT CONFIGURED]"),
        }
    }
}

/// Magento edition classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Edition {
    OpenSource,
    AdobeCommerce,
    AdobeCommerceCloud,
    #[default]
    Unknown,
}

impl std::fmt::Display for Edition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Edition::OpenSource => write!(f, "Magento Open Source"),
            Edition::AdobeCommerce => write!(f, "Adobe Commerce"),
            Edition::AdobeCommerceCloud => write!(f, "Adobe Commerce Cloud"),
            Edition::Unknown => write!(f, "Unknown Edition"),
        }
    }
}

/// Magento deployment mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MagentoMode {
    Production,
    Developer,
    DefaultMode,
    Maintenance,
    #[default]
    Unknown,
}

impl std::fmt::Display for MagentoMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MagentoMode::Production => write!(f, "production"),
            MagentoMode::Developer => write!(f, "developer"),
            MagentoMode::DefaultMode => write!(f, "default"),
            MagentoMode::Maintenance => write!(f, "maintenance"),
            MagentoMode::Unknown => write!(f, "unknown"),
        }
    }
}

/// Module origin/type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleClassification {
    Core,
    AdobeCommerce,
    BundledThirdParty,
    Marketplace,
    ComposerThirdParty,
    AppCodeCustom,
    Unknown,
}

impl std::fmt::Display for ModuleClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleClassification::Core => write!(f, "Magento core"),
            ModuleClassification::AdobeCommerce => write!(f, "Adobe Commerce"),
            ModuleClassification::BundledThirdParty => write!(f, "Bundled third party"),
            ModuleClassification::Marketplace => write!(f, "Marketplace package"),
            ModuleClassification::ComposerThirdParty => write!(f, "Composer third party"),
            ModuleClassification::AppCodeCustom => write!(f, "app/code custom"),
            ModuleClassification::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Integration footprint metrics for a single module.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleFootprint {
    pub plugins_count: usize,
    pub preferences_count: usize,
    pub observers_count: usize,
    pub cron_jobs_count: usize,
    pub db_tables_count: usize,
    pub db_columns_count: usize,
    pub db_indexes_count: usize,
    pub routes_count: usize,
    pub events_listened: Vec<String>,
}

/// Represents an installed Magento module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub vendor: String,
    pub package_name: Option<String>,
    pub version: Option<String>,
    pub sequence: Vec<String>,
    pub path: PathBuf,
    pub is_enabled: bool,
    pub classification: ModuleClassification,
    pub footprint: ModuleFootprint,
}

/// Plugin type in di.xml.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    Before,
    Around,
    After,
    Unknown,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::Before => write!(f, "before"),
            PluginType::Around => write!(f, "around"),
            PluginType::After => write!(f, "after"),
            PluginType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Declared DI plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub module: String,
    pub target_class: String,
    pub plugin_class: String,
    pub plugin_type: PluginType,
    pub sort_order: i32,
    pub is_disabled: bool,
    pub area: String,
    pub source_file: PathBuf,
    pub line: usize,
    /// Whether static analysis detected cost indicators in this plugin's implementation.
    pub cost_indicators: Vec<String>,
}

/// Declared DI preference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub for_class: String,
    pub type_class: String,
    pub module: String,
    pub area: String,
    pub source_file: PathBuf,
    pub line: usize,
}

/// Declared event observer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observer {
    pub name: String,
    pub event_name: String,
    pub instance: String,
    pub is_disabled: bool,
    pub is_shared: bool,
    pub area: String,
    pub module: String,
    pub source_file: PathBuf,
    pub line: usize,
}

/// Declared cron job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub name: String,
    pub group: String,
    pub instance: String,
    pub method: String,
    pub schedule: Option<String>,
    pub interval_seconds: Option<u64>,
    pub module: String,
    pub source_file: PathBuf,
    pub line: usize,
}

/// Declared indexer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indexer {
    pub id: String,
    pub view_id: String,
    pub action_class: String,
    pub title: String,
    pub description: String,
    pub module: String,
    pub is_scheduled: Option<bool>,
    pub status: Option<String>,
    pub updated_at: Option<String>,
}

/// Declared column definition in db_schema.xml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub unsigned: bool,
    pub identity: bool,
    pub comment: Option<String>,
}

/// Declared index definition in db_schema.xml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSchema {
    pub name: String,
    pub index_type: String,
    pub columns: Vec<String>,
}

/// Declared constraint definition (primary key, foreign key, unique).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintSchema {
    pub name: String,
    pub constraint_type: String,
    pub columns: Vec<String>,
    pub reference_table: Option<String>,
    pub reference_column: Option<String>,
    pub on_delete: Option<String>,
}

/// Declared table definition in db_schema.xml or actual table in DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub name: String,
    pub columns: HashMap<String, ColumnSchema>,
    pub indexes: HashMap<String, IndexSchema>,
    pub constraints: HashMap<String, ConstraintSchema>,
    pub engine: String,
    pub comment: Option<String>,
    pub owning_module: Option<String>,
}

/// Aggregated database schema.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseSchema {
    pub tables: HashMap<String, TableSchema>,
}

/// Table size metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSizeStat {
    pub table_name: String,
    pub row_count: u64,
    pub data_bytes: u64,
    pub index_bytes: u64,
    pub total_bytes: u64,
}

/// Runtime stats for a single cron job code in cron_schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRuntimeStat {
    pub job_code: String,
    pub total_executions: u64,
    pub p50_seconds: f64,
    pub p95_seconds: f64,
    pub median_seconds: f64,
    pub schedule_interval_seconds: Option<u64>,
    pub overlap_ratio: Option<f64>,
}

/// Cron schedule table summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CronScheduleSummary {
    pub total_rows: u64,
    pub pending_rows: u64,
    pub running_rows: u64,
    pub success_rows: u64,
    pub missed_rows: u64,
    pub error_rows: u64,
    pub oldest_running_job: Option<String>,
    pub oldest_running_seconds: Option<u64>,
    pub job_stats: HashMap<String, JobRuntimeStat>,
}

/// Runtime database metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub is_connected: bool,
    pub server_version: Option<String>,
    pub innodb_buffer_pool_bytes: Option<u64>,
    pub total_data_and_index_bytes: Option<u64>,
    pub table_sizes: Vec<TableSizeStat>,
    pub cron_schedule: CronScheduleSummary,
    pub changelog_sizes: HashMap<String, u64>,
}

/// Environment and host metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Environment {
    pub php_cli_version: Option<String>,
    pub php_web_version: Option<String>,
    pub composer_version: Option<String>,
    pub os_name: String,
    pub current_user: String,
    pub document_root: Option<PathBuf>,
    pub filesystem_owner: Option<String>,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub is_vendor_committed: bool,
}

/// Sanitized configuration from env.php.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SanitizedEnvConfig {
    pub db_host: Option<String>,
    pub db_name: Option<String>,
    pub db_user: Option<String>,
    pub db_password_secret: SecretValue,
    pub crypt_key_secret: SecretValue,
    pub redis_cache_host: Option<String>,
    pub redis_cache_db: Option<String>,
    pub redis_page_cache_host: Option<String>,
    pub redis_page_cache_db: Option<String>,
    pub redis_session_host: Option<String>,
    pub redis_session_db: Option<String>,
    pub opensearch_host: Option<String>,
    pub opensearch_port: Option<u16>,
    pub rabbitmq_host: Option<String>,
}

/// Redis runtime status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RedisStatus {
    pub is_configured: bool,
    pub is_reachable: bool,
    pub version: Option<String>,
    pub used_memory_bytes: Option<u64>,
    pub maxmemory_bytes: Option<u64>,
    pub maxmemory_policy: Option<String>,
    pub evicted_keys: Option<u64>,
    pub connected_clients: Option<u64>,
}

/// OpenSearch runtime status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenSearchStatus {
    pub is_configured: bool,
    pub is_reachable: bool,
    pub version: Option<String>,
    pub cluster_name: Option<String>,
    pub status: Option<String>, // green, yellow, red
    pub number_of_nodes: Option<u32>,
    pub active_primary_shards: Option<u32>,
}

/// OPcache status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpcacheStatus {
    pub is_enabled: bool,
    pub memory_consumption_bytes: Option<u64>,
    pub max_accelerated_files: Option<u32>,
    pub validate_timestamps: Option<bool>,
}

/// Host & Runtime state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeState {
    pub redis_default: RedisStatus,
    pub redis_page_cache: RedisStatus,
    pub redis_session: RedisStatus,
    pub opensearch: OpenSearchStatus,
    pub opcache: OpcacheStatus,
    pub web_server: Option<String>,
}

/// Root normalized installation model for a Magento store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagentoInstallation {
    pub root: PathBuf,
    pub edition: Edition,
    pub version: Option<String>,
    pub mode: MagentoMode,
    pub environment: Environment,
    pub env_config: SanitizedEnvConfig,
    pub modules: Vec<Module>,
    pub plugins: Vec<Plugin>,
    pub preferences: Vec<Preference>,
    pub observers: Vec<Observer>,
    pub cron_jobs: Vec<CronJob>,
    pub indexers: Vec<Indexer>,
    pub declared_schema: DatabaseSchema,
    pub actual_schema: DatabaseSchema,
    pub database_metrics: DatabaseMetrics,
    pub runtime: RuntimeState,
}

impl MagentoInstallation {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            edition: Edition::Unknown,
            version: None,
            mode: MagentoMode::Unknown,
            environment: Environment::default(),
            env_config: SanitizedEnvConfig::default(),
            modules: Vec::new(),
            plugins: Vec::new(),
            preferences: Vec::new(),
            observers: Vec::new(),
            cron_jobs: Vec::new(),
            indexers: Vec::new(),
            declared_schema: DatabaseSchema::default(),
            actual_schema: DatabaseSchema::default(),
            database_metrics: DatabaseMetrics::default(),
            runtime: RuntimeState::default(),
        }
    }

    /// Count of enabled modules.
    pub fn enabled_modules_count(&self) -> usize {
        self.modules.iter().filter(|m| m.is_enabled).count()
    }

    /// Count of disabled modules.
    pub fn disabled_modules_count(&self) -> usize {
        self.modules.iter().filter(|m| !m.is_enabled).count()
    }

    /// Count of third-party modules (Composer or app/code).
    pub fn third_party_modules_count(&self) -> usize {
        self.modules
            .iter()
            .filter(|m| {
                matches!(
                    m.classification,
                    ModuleClassification::Marketplace
                        | ModuleClassification::ComposerThirdParty
                        | ModuleClassification::AppCodeCustom
                )
            })
            .count()
    }

    /// Find a module by name.
    pub fn find_module(&self, name: &str) -> Option<&Module> {
        self.modules.iter().find(|m| m.name == name)
    }
}
