//! mdoctor_php: Tree-sitter static analyzer for PHP codebases.

pub mod analyzer;
pub mod cost;

pub use analyzer::PhpAstAnalyzer;
pub use cost::{AstFinding, OperationType};
