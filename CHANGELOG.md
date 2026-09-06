# Changelog

All notable changes to **Magento Doctor** (`mdoctor`) are documented in this file.
This project follows [CalVer](https://calver.org) (`YYYY.MM.DD`) release versioning.

---

## [v2026.09.06] - 2026-09-06

### 🚀 New Features (v0.2 Capabilities)
- **Configuration Drift & Baseline Comparison**:
  - `mdoctor baseline create [--output <path>]`: Serializes complete store configuration, modules, declared schemas, and findings into a safe, sanitized baseline snapshot.
  - `mdoctor compare <baseline.json> [--format text|json|markdown|sarif]`: Detects store drift, version shifts, added/removed modules, schema changes, and new diagnostic regressions.
  - Deterministic CI exit codes: `2` on critical regressions, `1` on warnings, `0` on pass.
- **Module Architectural Risk & Impact Scoring**:
  - `mdoctor modules --impact` (and `mdoctor impact`): Scans custom and third-party extensions and ranks them by performance drag and risk (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`).
  - Evaluates hotpath interceptions, around plugin stack depth, minutely crons, core preferences, and AST cost indicators.
- **Module Uninstall Blast-Radius Forensics**:
  - `mdoctor module <Name> --uninstall-impact`: Evaluates sequence breaks (`[BLOCKED]`), orphaned custom database tables/columns (`[CAUTION]`), and generates an ordered safe removal checklist.
- **Mermaid.js Architecture Graph Generator**:
  - `mdoctor module <Name> --graph mermaid`: Visualizes extension touchpoints (sequence dependencies, plugins, observers, cron jobs, database tables) in standard Mermaid `flowchart TD` format.
- **GitHub Action PR Reviewer**:
  - Added official composite [action.yml](action.yml) for GitHub Actions CI/CD workflows.
  - Added sample pull-request workflow in `.github/workflows/pr-review.yml` for automated SARIF security and diagnostic code scanning.

### 🛠️ Improvements & Fixes (MVP Refinements)
- **Redis Multi-Instance Hostname Differentiation**:
  - Upgraded Redis parser to extract `host`, `server`, `port`, and `path`.
  - Same database numbers across different Redis hosts, ports, or sockets no longer trigger false-positive collision warnings.
- **Enhanced N+1 Repository Load Diagnostics (`MD-PERF-021`)**:
  - Attached exact file path (`File: path/to/Class.php:line`) to AST findings.
  - Added entity type detection (Category, Product, Order) and concrete copy-pasteable batch-loading recommendations (`CollectionFactory` / `SearchCriteriaBuilder`).
- **Core Magento Preference Filtering (`MD-DI-001`)**:
  - Filtered core Magento modules (`Magento_*`, `vendor/magento/*`) from preference anti-pattern warnings. Only custom and 3rd-party modules are evaluated.
  - Excluded vanilla core plugins from `MD-PLG-001` and `MD-PLG-005`.
- **Zero-Secret Exposure Guarantee**:
  - All database passwords, crypt keys, and Redis auth strings are strictly redacted (`SecretValue::Present`), preventing leakage in snapshots or terminal output.

---

## [v2026.08.26] - 2026-08-26

### Initial Release
- Initial release of Magento Doctor diagnostic engine.
- Zero-configuration discovery for Magento Open Source and Adobe Commerce.
- Tree-sitter PHP AST static analysis without PHP runtime dependencies.
- Declarative database schema reconciliation (`db_schema.xml` vs MySQL).
- Cron schedule forensics and overlap ratio analysis.
- Multi-format reporting: ANSI Terminal, JSON, Markdown, and SARIF.
