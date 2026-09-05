# Magento Doctor (`mdoctor`)

<p align="center">
  <strong>Deep diagnostics, static analysis and performance forensics for Magento 2.</strong>
</p>

<p align="center">
  <a href="https://github.com/nemke82/magento-doctor/actions/workflows/ci.yml"><img src="https://github.com/nemke82/magento-doctor/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/nemke82/magento-doctor/releases"><img src="https://img.shields.io/badge/release-v2026.08.26-blue.svg" alt="Release" /></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT" /></a>
  <img src="https://img.shields.io/badge/Rust-1.75+-orange.svg" alt="Rust 1.75+" />
  <img src="https://img.shields.io/badge/Magento-2.4.4_--_2.4.9-orange.svg" alt="Magento 2.4.4 - 2.4.9" />
</p>

---

Magento Doctor is much more than a health checker. Rather than dumping 150 disconnected warnings, Magento Doctor understands how a Magento installation is assembled and correlates code, configuration, database schema, runtime state, logs, cron/indexer behavior, and third-party modules into an **engineer-grade diagnosis**.

```text
Collectors
    ↓
Normalized Installation Model (MagentoInstallation)
    ↓
Cross-Analysis Correlation Engine
    ↓
Evidence-First Diagnosis & Recommendations
```

Every analyzer operates against a normalized internal representation of the store rather than grep heuristics. This allows Magento Doctor to correlate facts across different subsystems to answer critical questions:

- *Does this third-party module define an around plugin on a checkout hot path that makes synchronous HTTP calls?*
- *Does a vendor cron job run every minute while having an observed median runtime of 74 seconds, creating an overlap storm?*
- *Does a collection loop execute N+1 repository loads (`$repo->getById()`) inside a loop?*
- *Which declarative schema tables or indexes are missing or redundant in the physical database?*

---

## Example Output

```text
Magento Doctor v2026.08.26

Magento Open Source 2.4.7-p3
Mode: production
PHP: 8.3.26
Database Host: 127.0.0.1
Modules: 312 enabled / 17 disabled
Third-party modules: 84

Overall Health: 71 / 100

CRITICAL   3
WARNING   14
INFO      22

Primary concerns

[CRITICAL] Cron backlog and overlap storm detected (MD-CRON-010)
  18,431 rows in cron_schedule
  1,922 jobs are pending beyond their expected execution window.
  87 jobs are stuck in "running".
  Oldest running job: vendor_export_feed — 3h 17m

  Evidence / Correlation:
    Vendor_Feed defines vendor_export_feed in etc/crontab.xml.
    Schedule interval: 60 seconds.
    Median runtime observed: 74 seconds.
    Overlap ratio: 1.23 (HIGH probability of overlapping executions)
    AST static analysis detected costly operations inside this job:
      - Repository Load at line 21 ($this->productRepository->getById())
      - Synchronous HTTP Request at line 28 (curl_exec())

  Impact:
    Jobs overlap faster than they complete, compounding MySQL load,
    exhausting PHP workers, and delaying core Magento indexer cron jobs.

  Recommendation:
    Reduce frequency in etc/crontab.xml, prevent overlap, move remote calls
    to queue, or optimize Vendor\Feed\Cron\Export::execute().

[CRITICAL] Around plugin on checkout hot path (MD-PLG-001)
  Plugin 'vendor_payment_quote_around' intercepts hot path 'Magento\Quote\Model\QuoteManagement'
  using an around wrapper in module 'Vendor_Payment'.

  Evidence / Correlation:
    Plugin class: Vendor\Payment\Plugin\QuoteManagement
    Intercepts: Magento\Quote\Model\QuoteManagement::submit()
    Plugin type: around (sortOrder: 10)
    Source: app/code/Vendor/Payment/etc/di.xml:4
    AST static analysis detected costly operations inside this plugin:
      - Synchronous HTTP Request at line 13 ($this->client->request())

  Impact:
    Blocking external HTTP requests during order placement directly degrades
    checkout conversion and locks customer quotes.

  Recommendation:
    Refactor around plugin to 'before' or 'after' plugin, avoid synchronous network calls,
    and ensure $proceed() is called cleanly.
```

---

## Installation

### Option 1: Download Standalone Binary (Recommended)

Pre-compiled standalone binaries are available for Linux, macOS, and Windows. No PHP, Composer, or runtime dependencies required:

```bash
# Linux (x86_64)
curl -sSL https://github.com/nemke82/magento-doctor/releases/latest/download/mdoctor-linux-amd64 -o /usr/local/bin/mdoctor
chmod +x /usr/local/bin/mdoctor

# Linux (musl static binary)
curl -sSL https://github.com/nemke82/magento-doctor/releases/latest/download/mdoctor-linux-amd64-musl -o /usr/local/bin/mdoctor
chmod +x /usr/local/bin/mdoctor

# macOS (Apple Silicon arm64)
curl -sSL https://github.com/nemke82/magento-doctor/releases/latest/download/mdoctor-macos-arm64 -o /usr/local/bin/mdoctor
chmod +x /usr/local/bin/mdoctor
```

Verify installation:
```bash
mdoctor --version
# Output: mdoctor v2026.08.26
```

### Option 2: Cargo Install

```bash
cargo install --git https://github.com/nemke82/magento-doctor mdoctor-cli
```

### Option 3: Build From Source

```bash
git clone https://github.com/nemke82/magento-doctor.git
cd magento-doctor
cargo build --release
sudo cp target/release/mdoctor /usr/local/bin/
```

---

## Zero-Configuration Discovery

Magento Doctor automatically discovers the root directory by searching upward from your current working directory for `app/etc/env.php`, `config.php`, `composer.json`, and `bin/magento`:

```bash
cd /var/www/html/magento
mdoctor scan
```

You can also specify an explicit directory via `--root` or the `MAGENTO_ROOT` environment variable:
```bash
mdoctor --root /var/www/magento scan
# or
MAGENTO_ROOT=/var/www/magento mdoctor scan
```

---

## Commands & Usage

| Command | Description |
|---|---|
| `mdoctor scan` | Run full comprehensive scan across code, configuration, database schema, and cron |
| `mdoctor doctor` | Fast operational health check highlighting critical blockages |
| `mdoctor modules` | Module inventory, classification, and integration footprint metrics |
| `mdoctor module <Vendor_Module>` | Deep inspection of a specific module's plugins, observers, crons, and schema |
| `mdoctor cron` | Cron forensics, schedule frequency, overlap risk, and backlog analysis |
| `mdoctor indexers` | Indexer and MView changelog status, backlog, and schedule mode |
| `mdoctor db` | Declarative schema reconciliation, missing/redundant indexes, table sizes |
| `mdoctor explain <RULE_ID>` | In-depth engineering explanation with manual verification commands |
| `mdoctor snapshot create` | Export sanitized store snapshot safe for sharing in GitHub issues |
| `mdoctor snapshot analyze <file>` | Analyze an exported snapshot offline |
| `mdoctor why slow` | Targeted bottleneck triage across database, cron, hot-path plugins, and Redis |

---

### Deep Scans & CI Reporting

```bash
# Offline mode (CI / Source repos without live MySQL)
mdoctor scan --offline

# Deep scan with extended budget for larger stores
mdoctor scan --deep --budget 120

# Machine-readable JSON output
mdoctor scan --format json

# Markdown report
mdoctor scan --format markdown

# GitHub Code Scanning SARIF format
mdoctor scan --format sarif > results.sarif
```

### Investigating Rules (`mdoctor explain`)

Every diagnosis is backed by factual evidence and transparent heuristics. View the complete engineering explanation and manual verification SQL queries:

```bash
mdoctor explain MD-PERF-021
```

```text
N+1 Repository Access in Collection Loop [MD-PERF-021]

WHAT IS THIS?
A repository load (e.g. $productRepository->getById()) is executed inside a foreach/while loop over a collection.

WHY DOES IT MATTER?
Calling getById() inside a loop causes Magento to execute a fresh database query (and multiple EAV joins) for every single item in the collection. For 1,000 items, this triggers 1,000 distinct SQL queries instead of 1 batched query.

HOW DETECTION WORKS
Tree-sitter PHP AST analysis walks the syntax tree, tracks loop depth, and flags member call expressions on repository instances within loop blocks.

POTENTIAL FALSE POSITIVES
Very low. In rare cases, a loop with a hardcoded limit of 1 or 2 iterations might trigger a warning.

MANUAL VERIFICATION
Inspect the flagged source file and line number. Enable MySQL query logging or Magento DB profiler during that execution.

REMEDIATION
Add required attributes directly to the parent collection using ->addAttributeToSelect(), or use SearchCriteriaBuilder with an 'in' filter to load all required entities in one round-trip.
```

---

## Diagnostic Snapshots

Need to share diagnostic state with team members, hosting support, or attached to a GitHub issue? `mdoctor snapshot create` exports a complete, sanitized representation of the store:

```bash
mdoctor snapshot create --output customer_audit.mdoctor
```

You can then inspect the snapshot on any machine without access to the customer's server:
```bash
mdoctor snapshot analyze customer_audit.mdoctor
```

---

## Safety & Confidentiality Guarantees

- **Zero Exposure of Secrets**: Database passwords, crypt keys, Redis passwords, RabbitMQ credentials, and authorization headers are never logged, printed, or saved. The normalized model maps them strictly as `SecretValue::Present` or `SecretValue::Missing`.
- **Safe Read-Only Operations**: All default collectors run non-locking, read-only queries with strict timeouts. Intrusive operations (`EXPLAIN ANALYZE`) require explicit opt-in.
- **100% Local & Air-Gapped**: Zero telemetry, zero external network calls, zero AI API dependencies. Your code and database metadata never leave your infrastructure.

---

## Rule Catalog Taxonomy

| Category | Prefix | Focus Area |
|---|---|---|
| **Environment** | `MD-ENV-*` | PHP version compatibility, CLI vs Web version mismatches |
| **Magento Core**| `MD-MAG-*` | Core integrity, deployment mode |
| **Modules**     | `MD-MOD-*` | Inconsistent modules, sequence circularities |
| **Dependency Injection** | `MD-DI-*` | Core concrete class replacement, direct `ObjectManager` |
| **Plugins**     | `MD-PLG-*` | Around plugins on hot paths, duplicate `sortOrder` |
| **Events**      | `MD-EVT-*` | Heavy observers on critical storefront events |
| **Cron**        | `MD-CRON-*`| Schedule backlog, overlap storms, stuck running jobs |
| **Indexers**    | `MD-IDX-*` | Realtime vs scheduled indexer modes, changelog backlog |
| **Database**    | `MD-DB-*`  | Missing declared indexes, redundant left-prefix indexes, orphan tables, volatile bloat |
| **Cache**       | `MD-CACHE-*`| Redis database collisions (session vs cache sharing DB IDs) |
| **Performance** | `MD-PERF-*`| N+1 repository loops, synchronous HTTP calls, loop logging |
| **Security**    | `MD-SEC-*` | Developer mode in production, world-writable directories |

---

## Supported Versions

Magento Doctor ships with built-in version matrices and hot-path databases covering:
- **Magento Open Source & Adobe Commerce**: `2.4.4`, `2.4.5`, `2.4.6`, `2.4.7`, `2.4.8`, and `2.4.9`
- **PHP**: `8.1`, `8.2`, `8.3`, and `8.4`
- **Database**: MariaDB `10.4`, `10.6`, `10.11` & MySQL `8.0`, `8.4`
- **Search**: OpenSearch `1.2`, `2.5`, `2.12`, `2.19`
- **Cache**: Redis / Valkey `6.2`, `7.0`, `7.2`

---

## License

MIT © 2026 nemke82 and Magento Doctor Contributors.
