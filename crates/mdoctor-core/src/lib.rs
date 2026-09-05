//! mdoctor_core: Fundamental types, normalized models, health scores, and findings.

pub mod finding;
pub mod health;
pub mod model;
pub mod safety;
pub mod snapshot;
pub mod version;

pub use finding::{Category, Confidence, Finding, Severity};
pub use health::HealthScore;
pub use model::*;
pub use safety::{SafetyLevel, ScanBudget};
pub use snapshot::DiagnosticSnapshot;
pub use version::{banner, CALVER_VERSION, CARGO_PKG_VERSION};
