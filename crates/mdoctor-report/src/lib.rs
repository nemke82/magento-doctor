//! mdoctor_report: Terminal, JSON, Markdown, and SARIF output renderers.

pub mod json_fmt;
pub mod markdown_fmt;
pub mod sarif_fmt;
pub mod terminal;

pub use json_fmt::render_json_report;
pub use markdown_fmt::render_markdown_report;
pub use sarif_fmt::render_sarif_report;
pub use terminal::render_terminal_report;
