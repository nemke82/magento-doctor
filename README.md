# Magento Doctor (`mdoctor`)

> **Deep diagnostics, static analysis and performance forensics for Magento 2.**

[![CI](https://github.com/nemke82/magento-doctor/actions/workflows/ci.yml/badge.svg)](https://github.com/nemke82/magento-doctor/actions/workflows/ci.yml)
[![Version](https://img.shields.io/badge/version-v2026.08.26-blue.svg)](https://github.com/nemke82/magento-doctor/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Magento Doctor is much more than a health checker. Rather than returning 150 disconnected checklist warnings, Magento Doctor understands how a Magento 2 installation is assembled and correlates code, configuration, database schema, runtime state, logs, cron/indexer behavior, and third-party modules into an engineer-grade diagnosis.

---

## Design Principle: Collect → Model → Correlate → Diagnose

```
Collectors
    ↓
Normalized Installation Model (MagentoInstallation)
    ↓
Cross-Analysis Correlation Engine
    ↓
Evidence-First Findings & Recommendations
```

Every analyzer works against a normalized internal representation of the store rather than raw grep queries. This enables Magento Doctor to answer complex questions such as:

- *Does this third-party module define an around plugin on a checkout hot path that makes synchronous HTTP requests?*
- *Does a vendor cron job run every minute while having a median runtime of 74 seconds, creating an overlap storm?*
- *Does this model execute an N+1 repository load inside a collection loop?*
- *Which declarative schema tables/indexes are missing or redundant in the live database?*

---

## Quick Start

```bash
cd /var/www/html/magento
mdoctor scan
```

### Example Diagnosis Output

```text
Magento Doctor v2026.08.26

Magento Open Source 2.4.7-p3
Mode: production
PHP: 8.3.26
Database: MariaDB 10.11
Search: OpenSearch 2.19
Redis: 7.2
Modules: 312 enabled / 17 disabled
Third-party modules: 84

Overall Health: 71 / 100

CRITICAL   2
WARNING    4
INFO      12

Primary concerns

[CRITICAL] Cron backlog and overlap storm detected (MD-CRON-010)
  18,431 rows in cron_schedule
  1,922 jobs are pending beyond their expected execution window.
  87 jobs are stuck in "running".
  Oldest running job: vendor_export_feed — 3h 17m

  Correlation:
    Vendor_Feed defines vendor_export_feed in etc/crontab.xml.
    Schedule interval: 60 seconds.
    Median runtime observed: 74 seconds.
    Overlap ratio: 1.23 (HIGH probability of overlapping executions)

  Impact:
    Jobs overlap faster than they complete, creating CPU/DB load and delaying core indexers.

  Recommendation:
    Reduce frequency, prevent overlap, move remote calls to queue,
    or optimize Vendor\Feed\Cron\Export::execute().

[CRITICAL] Missing useful database index (MD-DB-001)
  Table: vendor_feed_queue
  Candidate: INDEX(status, created_at)
  Confidence: HIGH

[WARNING] Around plugin on checkout hot path (MD-PLG-001)
  Vendor_Payment
  plugin: Vendor\Payment\Plugin\QuoteManagement
  intercepts: Magento\Quote\Model\QuoteManagement::submit()
  plugin type: around
  plugin performs:
    - repository load
    - synchronous HTTP request
  Risk: HIGH
```

---

## Commands

| Command | Description |
|---|---|
| `mdoctor scan` | Run full comprehensive scan (`--offline`, `--deep`, `--format [text\|json\|markdown\|sarif]`) |
| `mdoctor doctor` | Quick operational health check |
| `mdoctor modules` | Module inventory, categorization, and integration footprint |
| `mdoctor module <Vendor_Module>` | Deep inspection of specific module footprint, plugins, and dependencies |
| `mdoctor cron` | Cron forensics, schedule analysis, and overlap ratios |
| `mdoctor indexers` | Indexer and MView changelog table status and backlog |
| `mdoctor db` | Declarative schema reconciliation, missing/redundant indexes, table sizes |
| `mdoctor explain <RULE_ID>` | In-depth explanation, impact, and manual verification commands for a rule |
| `mdoctor snapshot create` | Export sanitized installation snapshot safe for GitHub/support issues |
| `mdoctor snapshot analyze <file>` | Analyze an exported snapshot offline |
| `mdoctor baseline create` | Capture environment baseline for drift detection |
| `mdoctor compare <baseline.json>` | Compare current installation with a previous baseline |
| `mdoctor why slow` | Targeted bottleneck triage (database, cron, hot-path plugins, Redis) |

---

## Safety & Confidentiality

- **Zero Exposure of Secrets**: `env.php` passwords, crypt keys, Redis passwords, RabbitMQ credentials, and AWS tokens are strictly sanitized in memory (`SecretValue::Present` / `SecretValue::Missing`) and never printed or saved.
- **Safe by Default**: `mdoctor scan` executes only non-intrusive operations. Intrusive operations (`EXPLAIN ANALYZE`) require explicit opt-in flags.
- **Offline & Air-Gapped**: Runs 100% locally with zero telemetry and no cloud dependencies.

---

## Architecture

Built in Rust as a modular multi-crate workspace:

- `mdoctor-core`: Normalized installation model, health scoring, findings, and safety levels.
- `mdoctor-knowledge`: Magento version compatibility matrices, hot paths, and known table footprints.
- `mdoctor-php`: Tree-sitter PHP AST static analysis (N+1 in loops, ObjectManager, sync HTTP).
- `mdoctor-magento`: Magento discovery, XML parsers (`di.xml`, `events.xml`, `crontab.xml`, `db_schema.xml`).
- `mdoctor-db`: MySQL introspection, declarative schema reconciliation, query normalization, index optimization.
- `mdoctor-runtime`: Environment forensics (PHP CLI vs Web, OPcache, Redis, OpenSearch, filesystem).
- `mdoctor-rules`: Cross-analysis correlation engine implementing `MD-*` rules.
- `mdoctor-report`: ANSI terminal rendering, JSON, Markdown, and GitHub Code Scanning SARIF formatters.
- `mdoctor-cli`: Clap-based CLI binary (`mdoctor`).

---

## License

MIT © 2026 nemke82 and Magento Doctor Contributors.
