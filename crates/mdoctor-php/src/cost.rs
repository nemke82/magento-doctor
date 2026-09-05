//! Cost indicators and operation classifications for PHP static analysis.

use serde::{Deserialize, Serialize};

/// High-cost operation identified in PHP AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    /// N+1 style repository entity loading: e.g. $repo->getById()
    RepositoryLoad,
    /// Collection query loading: e.g. $col->load(), create()
    CollectionLoad,
    /// Synchronous outbound network request: e.g. curl_exec, Guzzle, Laminas
    HttpRequest,
    /// Direct service locator anti-pattern: ObjectManager::getInstance()
    ObjectManagerUsage,
    /// Database mutation: e.g. $model->save(), $resource->delete()
    DatabaseWrite,
    /// Session initialization that breaks full-page caching: e.g. SessionManager::start()
    SessionAccess,
    /// Logging inside loops: e.g. $logger->info()
    LoggingInLoop,
    /// Direct SQL execution: e.g. $connection->query()
    DirectSqlQuery,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::RepositoryLoad => write!(f, "Repository Load"),
            OperationType::CollectionLoad => write!(f, "Collection Load"),
            OperationType::HttpRequest => write!(f, "Synchronous HTTP Request"),
            OperationType::ObjectManagerUsage => write!(f, "Direct ObjectManager Usage"),
            OperationType::DatabaseWrite => write!(f, "Database Write"),
            OperationType::SessionAccess => write!(f, "Session Access"),
            OperationType::LoggingInLoop => write!(f, "Logging Inside Loop"),
            OperationType::DirectSqlQuery => write!(f, "Direct SQL Query"),
        }
    }
}

/// A specific occurrence of a costly operation in a PHP source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstFinding {
    pub operation: OperationType,
    pub class_name: Option<String>,
    pub method_name: Option<String>,
    pub call_signature: String,
    pub in_loop: bool,
    pub line_number: usize,
    pub code_snippet: String,
}
