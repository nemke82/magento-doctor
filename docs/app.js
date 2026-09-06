/**
 * Magento Doctor (mdoctor) Showcase Script
 * Pure Vanilla JavaScript - zero external dependencies
 */

// Terminal commands data
const TERMINAL_OUTPUTS = {
  scan: {
    command: "mdoctor scan --offline",
    output: `
<span class="ansi-header">Magento Doctor v2026.09.06</span>

<span class="ansi-meta">Magento Open Source 2.4.7-p3 | Mode: production | PHP: 8.3.26
Host: 127.0.0.1 | Modules: 312 enabled / 17 disabled | 3rd-Party: 84</span>

<div class="ansi-health">Overall Health Score: <span class="ansi-crit">71 / 100</span> (Needs Attention)</div>
<span class="ansi-crit">CRITICAL: 3</span>  <span class="ansi-warn">WARNING: 2</span>  <span class="ansi-info">INFO: 5</span>

<div class="term-finding-card">
  <div class="ansi-crit">[CRITICAL] Cron backlog and overlap storm detected (MD-CRON-010)</div>
  <div class="ansi-dim">18,431 rows in cron_schedule | 1,922 jobs pending | 87 stuck in "running"</div>
  <div style="margin-top: 6px;">Oldest running: <span class="ansi-warn">vendor_export_feed (3h 17m)</span></div>
  <div style="margin-top: 6px;" class="ansi-evidence">
    Evidence: Schedule interval is 60s, but observed median runtime is 74s (Overlap: 1.23).
    AST Analysis detected costly operations inside this job:
      - Repository load in loop at line 21 ($productRepo->getById())
      - Synchronous HTTP call at line 28 (curl_exec())
  </div>
  <div style="margin-top: 6px; color: #a7f3d0;">
    Recommendation: Reduce frequency in etc/crontab.xml or offload remote sync to asynchronous message queue.
  </div>
</div>

<div class="term-finding-card">
  <div class="ansi-crit">[CRITICAL] Around plugin on checkout hot path (MD-PLG-001)</div>
  <div class="ansi-dim">Plugin 'vendor_payment_quote_around' wraps Magento\\Quote\\Model\\QuoteManagement::submit()</div>
  <div style="margin-top: 6px;" class="ansi-evidence">
    Declared in: app/code/Vendor/Payment/etc/di.xml:4 (sortOrder: 10)
    AST Analysis: Contains blocking synchronous HTTP network call at line 13.
  </div>
  <div style="margin-top: 6px; color: #a7f3d0;">
    Recommendation: Refactor to 'before' or 'after' plugin, or decouple external API call from quote submission.
  </div>
</div>

<div class="term-finding-card">
  <div class="ansi-crit">[CRITICAL] Unsupported PHP version for Magento release (MD-ENV-001)</div>
  <div class="ansi-dim">Running PHP: 8.4.1 | Magento: 2.4.7-p3</div>
  <div style="margin-top: 6px; color: #a7f3d0;">
    Recommendation: Align runtime with official certified PHP 8.2 or 8.3 for Magento 2.4.7.
  </div>
</div>

<div class="term-finding-card warning">
  <div class="ansi-warn">[WARNING] Missing declared database index (MD-DB-001)</div>
  <div class="ansi-dim">Index 'VENDOR_FEED_STATUS' in etc/db_schema.xml is missing from physical table 'vendor_feed'.</div>
</div>
`
  },
  doctor: {
    command: "mdoctor doctor",
    output: `
<span class="ansi-header">v2026.09.06 - Operational Health Check</span>

Overall Health: <span class="ansi-crit">71 / 100</span>
<span class="ansi-crit">Critical: 3</span>  <span class="ansi-warn">Warning: 2</span>  <span class="ansi-info">5</span>

<span class="ansi-crit">CRITICAL CONCERNS:</span>
  • <span class="ansi-crit">[MD-CRON-010]</span> <b>Cron backlog and overlap storm detected</b>
    Reduce frequency in etc/crontab.xml, prevent overlap, or move remote calls to queue.
  • <span class="ansi-crit">[MD-PLG-001]</span> <b>Around plugin on checkout hot path</b>
    Refactor around plugin to 'before' or 'after' plugin and avoid synchronous network calls.
  • <span class="ansi-crit">[MD-ENV-001]</span> <b>Unsupported PHP version for Magento release</b>
    Upgrade or downgrade PHP to a version officially supported in Adobe's system requirements matrix.
`
  },
  cron: {
    command: "mdoctor cron",
    output: `
<span class="ansi-header">Magento Cron Forensics & Overlap Analysis</span>

Cron Schedule Status:
  Total Rows: 18,431 | Pending: 1,922 | Running: 87 | Success: 16,104 | Missed: 318
  Oldest Running Job: vendor_export_feed (running for 11,820s)

┌──────────────────────┬─────────────┬───────────┬───────────┬───────────────┬────────────────┐
│ Job Code             │ Interval    │ Median    │ P95       │ Overlap Ratio │ Risk           │
├──────────────────────┼─────────────┼───────────┼───────────┼───────────────┼────────────────┤
│ vendor_export_feed   │ 60s (* * *) │ 74.2s     │ 142.1s    │ <span class="ansi-crit">1.23</span>          │ <span class="ansi-crit">STORM RISK</span>     │
│ catalog_reindex_all  │ 300s (*/5)  │ 42.1s     │ 88.0s     │ 0.14          │ SAFE           │
│ sales_send_order_... │ 60s (* * *) │ 1.8s      │ 4.5s      │ 0.03          │ SAFE           │
│ customer_visitor_cln │ 86400s (0 0)│ 194.0s    │ 240.0s    │ 0.002         │ SAFE           │
└──────────────────────┴─────────────┴───────────┴───────────────┴───────────────┴────────────────┘

Correlated Root Cause:
  vendor_export_feed spawns a new instance every minute, but runs for 74s on average.
  At any moment, 2+ concurrent instances lock the same rows and saturate DB connection pools.
`
  },
  module: {
    command: "mdoctor module Vendor_Feed",
    output: `
<span class="ansi-header">Module Deep Forensics: Vendor_Feed</span>

Identification:
  Vendor:           Vendor
  Module Name:      Vendor_Feed
  Classification:   <span class="ansi-info">app/code custom</span>
  Status:           <span class="ansi-success">enabled</span>
  Physical Path:    fixtures/magento-2.4.7/app/code/Vendor/Feed

Declared Sequence Dependencies:
  • Magento_Catalog

Integration Footprint:
  Plugins:          0
  Preferences:      0
  Observers:        0
  Cron Jobs:        1 (vendor_export_feed)
  DB Tables:        1 (vendor_feed)
  DB Columns:       3
  DB Indexes:       2

Static Analysis Code Issues:
  • <span class="ansi-crit">N+1 Repository Load in Loop</span> at Cron/Export.php:21
    $this->productRepository->getById($id) inside foreach ($feedItems as $item)
  • <span class="ansi-warn">Synchronous Outbound HTTP</span> at Cron/Export.php:28
    curl_exec() executed synchronously inside cron worker
  • <span class="ansi-warn">Excessive Loop Logging</span> at Cron/Export.php:32
    $this->logger->info() invoked 1,000+ times inside collection loop
`
  },
  db: {
    command: "mdoctor db",
    output: `
<span class="ansi-header">Declarative Database Schema Reconciliation</span>

Missing Declared Indexes (in db_schema.xml, missing in MySQL):
  • <span class="ansi-crit">vendor_feed.VENDOR_FEED_STATUS</span>: Full table scans will occur on queries filtering by status.

Redundant Left-Prefix Indexes:
  • <span class="ansi-info">sales_order.IDX_SALES_ORDER_INC_ID</span> covers prefix of (increment_id, store_id).
    Redundant index consumes 320MB of InnoDB buffer pool with zero query benefit.

Volatile Table Growth:
  • <span class="ansi-warn">cron_schedule</span>: 18,431 rows (14 MB) — Backlog accumulation detected.
  • <span class="ansi-warn">customer_visitor</span>: 412,900 rows (89 MB) — Needs log cleaning.
`
  },
  explain: {
    command: "mdoctor explain MD-PERF-021",
    output: `
<span class="ansi-header">Rule MD-PERF-021: N+1 Repository Access in Collection Loop</span>

<span class="ansi-info">WHAT WAS DETECTED:</span>
  A repository load ($productRepository->getById()) is executed inside a foreach/while
  loop over a collection.

<span class="ansi-crit">WHY THIS IMPACTS PERFORMANCE:</span>
  Calling getById() inside a loop causes Magento to execute a fresh database query
  (and multiple EAV joins) for every single item in the collection. For 1,000 items,
  this triggers 1,000 distinct SQL queries instead of 1 batched query.

<span class="ansi-warn">DETECTION MECHANISM:</span>
  Tree-sitter PHP AST analysis walks the syntax tree, tracks loop depth, and flags
  member call expressions on repository instances within loop blocks.

<span class="ansi-success">HOW TO VERIFY:</span>
  Inspect the flagged source file and line number.
  Enable MySQL query logging or the Magento DB profiler during execution.

<span class="ansi-header">REMEDIATION:</span>
  1. Add required attributes directly to the parent collection using:
     $collection->addAttributeToSelect(['sku', 'price', 'status']);
  2. Or use SearchCriteriaBuilder with an 'in' filter to batch load all required entities:
     $searchCriteria = $builder->addFilter('entity_id', $ids, 'in')->create();
`
  },
  why: {
    command: "mdoctor why slow",
    output: `
<span class="ansi-header">Forensic Bottleneck Triage: Why is Storefront / Cron Slow?</span>

Correlated Culprits (ranked by impact):

<span class="ansi-crit">1. Checkout Conversion Degradation</span>
   • Cause: Around plugin 'vendor_payment_quote_around' wraps QuoteManagement::submit().
   • Cost: Executes blocking synchronous curl_exec() to third-party gateway during checkout lock.
   • Location: app/code/Vendor/Payment/Plugin/QuoteManagement.php:13

<span class="ansi-crit">2. Cron Saturation & Database Lock Storm</span>
   • Cause: Job 'vendor_export_feed' has frequency 60s, but median runtime is 74s (overlap: 1.23).
   • Cost: Constant overlapping PHP workers, CPU thrashing, and table locks on cron_schedule.
   • Subsystem: Cron scheduler / MySQL connection pool

<span class="ansi-warn">3. N+1 Catalog Queries During Export</span>
   • Cause: $productRepo->getById() invoked in loop across collection items.
   • Cost: 1,000 individual EAV round-trips over localhost MySQL socket.
`
  },
  compare: {
    command: "mdoctor compare mdoctor-baseline.json",
    output: `
<span class="ansi-header">=== MAGENTO DOCTOR CONFIGURATION DRIFT REPORT ===</span>

Baseline: 2026-09-01 08:30:00 UTC  ->  Current: 2026-09-06 11:45:12 UTC
Health Score: 85 -> 78 (<span class="ansi-crit">-7</span>)
Findings Diff: Critical (+1)  Warning (+1)

<span class="ansi-crit">⚠️  REGRESSIONS DETECTED SINCE BASELINE:</span>
  + [<span class="ansi-crit">CRITICAL</span>] [<b>MD-PLG-001</b>] Around plugin on checkout hot path
    Plugin 'vendor_checkout_wrap' intercepts QuoteManagement::submit using an around wrapper.
    Fix: Refactor around plugin to 'before' or 'after' plugin, avoid synchronous network calls.

<span class="ansi-success">🎉 RESOLVED ISSUES (FIXED SINCE BASELINE):</span>
  - [MD-ENV-001] Unsupported PHP version for Magento release

<span class="ansi-info">Module Changes:</span>
┌────────────────────┬──────────┬─────────────────────────────┐
│ Module             │ Change   │ Details                     │
├────────────────────┼──────────┼─────────────────────────────┤
│ Vendor_NewCheckout │ <span class="ansi-success">ADDED</span>    │                             │
│ Vendor_OldBanner   │ <span class="ansi-crit">REMOVED</span>  │                             │
│ Vendor_PaymentGate │ <span class="ansi-warn">MODIFIED</span> │ version: 1.0.2 -> 1.1.0     │
└────────────────────┴──────────┴─────────────────────────────┘

<span class="ansi-crit">VERDICT: FAIL - Regressions or significant health score drop detected.</span>
`
  },
  impact: {
    command: "mdoctor modules --impact",
    output: `
<span class="ansi-header">=== MODULE ARCHITECTURAL RISK & PERFORMANCE IMPACT RANKINGS ===</span>

┌────────────────────┬─────────────────┬──────────────┬────────┬───────────────────────────────────┐
│ Module             │ Classification  │ Impact Level │ Score  │ Primary Risk Drivers              │
├────────────────────┼─────────────────┼──────────────┼────────┼───────────────────────────────────┤
│ Vendor_Checkout    │ app/code custom │ <span class="ansi-crit">CRITICAL</span>     │ 78/100 │ 2 Around Plugins on Hotpath,      │
│                    │                 │              │        │ 1 Minutely Cron, Direct SQL Query │
│ Vendor_Feed        │ app/code custom │ <span class="ansi-warn">HIGH</span>         │ 53/100 │ N+1 Repository Load inside Loop,  │
│                    │                 │              │        │ Synchronous HTTP Request          │
│ Vendor_Payment     │ Composer pkg    │ <span class="ansi-warn">MEDIUM</span>       │ 36/100 │ Around plugin on QuoteManagement, │
│                    │                 │              │        │ Synchronous HTTP Request          │
│ Vendor_StoreLoc    │ Composer pkg    │ <span class="ansi-success">LOW</span>          │ 12/100 │ Declares custom DB table          │
└────────────────────┴─────────────────┴──────────────┴────────┴───────────────────────────────────┘

Evaluated 4 modules: 1 Critical, 1 High, 1 Medium, 1 Low impact.
`
  }
};

// Rules catalog data
const RULES_DATA = [
  {
    id: "MD-CRON-010",
    category: "cron",
    severity: "critical",
    title: "Cron Job Overlap Storm",
    summary: "Observed execution duration exceeds scheduling frequency, spawning concurrent fighting workers.",
    mechanism: "Crontab schedule expression vs median/P95 runtime in cron_schedule.",
    remediation: "Decrease frequency, implement mutex execution locking, or offload to queue."
  },
  {
    id: "MD-CRON-001",
    category: "cron",
    severity: "critical",
    title: "Cron Backlog in cron_schedule",
    summary: "High volume of pending or stuck running jobs delaying indexers and emails.",
    mechanism: "Queries cron_schedule for stale pending jobs or running jobs older than 15m.",
    remediation: "Terminate orphaned PHP CLI workers and clean stale running rows."
  },
  {
    id: "MD-PLG-001",
    category: "plugins",
    severity: "critical",
    title: "Around Plugin on Critical Hot Path",
    summary: "Interception of QuoteManagement::submit or Product::getPrice using around plugin.",
    mechanism: "di.xml cross-referenced with hot-path DB and PHP AST cost indicators.",
    remediation: "Refactor into before/after plugin without modifying core execution flow."
  },
  {
    id: "MD-PLG-002",
    category: "plugins",
    severity: "warning",
    title: "Duplicate Plugin Sort Order Collision",
    summary: "Multiple plugins on the same target method share identical sortOrder.",
    mechanism: "Collects plugins per target class/method and flags sortOrder collisions.",
    remediation: "Explicitly sequence plugin execution via distinct sortOrder numbers."
  },
  {
    id: "MD-PERF-021",
    category: "perf",
    severity: "critical",
    title: "N+1 Repository Access in Collection Loop",
    summary: "$productRepo->getById() executed inside loop over collection items.",
    mechanism: "Tree-sitter PHP AST loop depth traversal and method call pattern matching.",
    remediation: "Use addAttributeToSelect() on collection or batch load with SearchCriteriaBuilder."
  },
  {
    id: "MD-PERF-015",
    category: "perf",
    severity: "warning",
    title: "Synchronous HTTP Request on Critical Path",
    summary: "curl_exec or Guzzle sync calls during checkout, cart total, or cron loops.",
    mechanism: "PHP AST identification of HTTP client methods in classes or plugins.",
    remediation: "Move remote API synchronizations to RabbitMQ or DB queues."
  },
  {
    id: "MD-PERF-030",
    category: "perf",
    severity: "info",
    title: "Excessive File Logging in Loop",
    summary: "Calling $logger->info() inside high-iteration loops causing disk I/O bottlenecks.",
    mechanism: "PHP AST identification of logger calls within loop bodies.",
    remediation: "Batch logs outside the loop or aggregate summary messages."
  },
  {
    id: "MD-DB-001",
    category: "db",
    severity: "warning",
    title: "Missing Declared Database Index",
    summary: "Index specified in db_schema.xml missing from physical MySQL/MariaDB table.",
    mechanism: "Reconciliation of db_schema.xml against information_schema.statistics.",
    remediation: "Run setup:db-declaration:generate-whitelist and setup:upgrade."
  },
  {
    id: "MD-DB-002",
    category: "db",
    severity: "info",
    title: "Redundant Left-Prefix Index",
    summary: "Secondary index is an exact left prefix of another composite index.",
    mechanism: "Permutation analysis of composite index column sequences in information_schema.",
    remediation: "Drop redundant prefix index to reclaim buffer pool memory and speed writes."
  },
  {
    id: "MD-DB-004",
    category: "db",
    severity: "critical",
    title: "Unsupported MySQL or MariaDB Version",
    summary: "Live database server version is not certified for this Magento release.",
    mechanism: "Live @@version compared against certified MySQL & MariaDB compatibility matrix.",
    remediation: "Migrate to certified MySQL 8.0/8.4 LTS/9.x or MariaDB 10.6/10.11/11.x."
  },
  {
    id: "MD-DB-005",
    category: "db",
    severity: "info",
    title: "Orphan Database Table",
    summary: "Table exists in MySQL but is not declared in any active module's db_schema.xml.",
    mechanism: "Schema reconciliation against all enabled modules' schema declarations.",
    remediation: "Verify if table belongs to an uninstalled legacy extension before dropping."
  },
  {
    id: "MD-CACHE-001",
    category: "cache",
    severity: "critical",
    title: "Redis Session & Cache Database Collision",
    summary: "Session storage and default/FPC cache configured with the same DB number.",
    mechanism: "Inspection of redis database numbers in app/etc/env.php.",
    remediation: "Assign distinct DB numbers (e.g. DB 0 cache, DB 1 FPC, DB 2 session)."
  },
  {
    id: "MD-CACHE-003",
    category: "cache",
    severity: "warning",
    title: "Unsupported Redis or Valkey Version",
    summary: "Configured cache server version is outside the certified compatibility matrix.",
    mechanism: "INFO server comparison with supported Redis/Valkey matrix.",
    remediation: "Align with supported Redis 7.0/7.2/8.0 or Valkey 7.2/8.0."
  },
  {
    id: "MD-ENV-001",
    category: "env",
    severity: "critical",
    title: "Unsupported PHP Version",
    summary: "Running PHP version is not certified for the detected Magento release.",
    mechanism: "php -v check against official Adobe compatibility matrix.",
    remediation: "Upgrade or downgrade PHP to match Adobe system requirements."
  },
  {
    id: "MD-ENV-002",
    category: "env",
    severity: "warning",
    title: "PHP CLI vs Web Server Mismatch",
    summary: "CLI PHP version differs from web server PHP (PHP-FPM/mod_php) version.",
    mechanism: "Compares crontab/CLI binary version against $_SERVER environment.",
    remediation: "Align crontab PHP path with Web server PHP runtime."
  },
  {
    id: "MD-ENV-003",
    category: "env",
    severity: "critical",
    title: "Unsupported OpenSearch Version",
    summary: "Configured OpenSearch version is outside the certified compatibility matrix.",
    mechanism: "Query cluster root endpoint GET / version.number against matrix.",
    remediation: "Deploy certified OpenSearch 2.12, 2.19, or 3.0."
  },
  {
    id: "MD-SEC-001",
    category: "sec",
    severity: "warning",
    title: "Store Running in Developer Mode",
    summary: "MAGE_MODE set to developer in production, slowing requests and leaking stacks.",
    mechanism: "Inspection of MAGE_MODE in app/etc/env.php.",
    remediation: "Execute 'bin/magento deploy:mode:set production'."
  },
  {
    id: "MD-SEC-002",
    category: "sec",
    severity: "critical",
    title: "World-Writable Directory Permissions",
    summary: "Root, app/etc, or pub directories configured with permissions 0777.",
    mechanism: "Filesystem mode bitwise mask check on sensitive Magento directory trees.",
    remediation: "Restore safe permissions: chmod 750 / chmod 755."
  },
  {
    id: "MD-DI-005",
    category: "di",
    severity: "warning",
    title: "Direct ObjectManager Usage",
    summary: "Class calls ObjectManager::getInstance() directly instead of constructor injection.",
    mechanism: "Tree-sitter PHP AST detection of direct ObjectManager call expressions.",
    remediation: "Inject required dependencies via class constructor."
  }
];

// Installation Snippets
const INSTALL_SNIPPETS = {
  linux: `# Download pre-compiled standalone binary (Linux x86_64 glibc)
curl -sSL https://github.com/nemke82/magento-doctor/releases/latest/download/mdoctor-linux-amd64 -o /usr/local/bin/mdoctor
chmod +x /usr/local/bin/mdoctor

# Verify installation
mdoctor --version`,
  musl: `# Download fully static binary (Alpine Linux / Containers / Minimal OS)
curl -sSL https://github.com/nemke82/magento-doctor/releases/latest/download/mdoctor-linux-amd64-musl -o /usr/local/bin/mdoctor
chmod +x /usr/local/bin/mdoctor

# Run anywhere with zero dynamic library dependencies
mdoctor --version`,
  macos: `# Download Apple Silicon binary (macOS ARM64 / M1, M2, M3, M4)
curl -sSL https://github.com/nemke82/magento-doctor/releases/latest/download/mdoctor-macos-arm64 -o /usr/local/bin/mdoctor
chmod +x /usr/local/bin/mdoctor

# Verify installation
mdoctor --version`,
  windows: `# Download Windows x64 executable via PowerShell
Invoke-WebRequest -Uri "https://github.com/nemke82/magento-doctor/releases/latest/download/mdoctor-windows-amd64.exe" -OutFile "mdoctor.exe"

# Run scan
.\\mdoctor.exe scan`,
  cargo: `# Build directly from source with Rust toolchain (1.75+)
git clone https://github.com/nemke82/magento-doctor.git
cd magento-doctor
cargo build --release --bin mdoctor

# Executable is ready at target/release/mdoctor
./target/release/mdoctor --version`
};

// Initialize DOM interactions
document.addEventListener("DOMContentLoaded", () => {
  initThemeToggle();
  initTerminal();
  initRuleExplorer();
  initInstallTabs();
  initCopyButtons();
});

/**
 * Theme switcher (Light / Dark mode, defaults to Light)
 */
function initThemeToggle() {
  const toggleBtn = document.getElementById("theme-toggle");
  if (!toggleBtn) return;

  function updateAriaLabel(theme) {
    if (theme === "dark") {
      toggleBtn.setAttribute("aria-label", "Switch to light theme");
      toggleBtn.setAttribute("title", "Switch to light theme");
    } else {
      toggleBtn.setAttribute("aria-label", "Switch to dark theme");
      toggleBtn.setAttribute("title", "Switch to dark theme");
    }
  }

  // Current theme from html attribute or localStorage (default light)
  let currentTheme = document.documentElement.getAttribute("data-theme") || "light";
  updateAriaLabel(currentTheme);

  toggleBtn.addEventListener("click", () => {
    const newTheme = currentTheme === "light" ? "dark" : "light";
    document.documentElement.setAttribute("data-theme", newTheme);
    try {
      localStorage.setItem("mdoctor_theme", newTheme);
    } catch (e) {
      console.warn("Unable to save theme preference:", e);
    }
    currentTheme = newTheme;
    updateAriaLabel(currentTheme);
  });
}

/**
 * Terminal simulator tab switcher
 */
function initTerminal() {
  const tabs = document.querySelectorAll(".term-tab");
  const promptCmd = document.querySelector("#terminal-command-text");
  const outputArea = document.querySelector("#terminal-output-body");

  tabs.forEach(tab => {
    tab.addEventListener("click", () => {
      tabs.forEach(t => t.classList.remove("active"));
      tab.classList.add("active");

      const key = tab.getAttribute("data-tab");
      const data = TERMINAL_OUTPUTS[key];
      if (data) {
        promptCmd.textContent = data.command;
        outputArea.innerHTML = data.output;
      }
    });
  });

  // Default to scan
  if (outputArea && promptCmd) {
    promptCmd.textContent = TERMINAL_OUTPUTS.scan.command;
    outputArea.innerHTML = TERMINAL_OUTPUTS.scan.output;
  }
}

/**
 * Rule catalog filter & search
 */
function initRuleExplorer() {
  const container = document.querySelector("#rules-container");
  const pills = document.querySelectorAll(".category-pill");
  const searchInput = document.querySelector("#rule-search-input");

  let currentCategory = "all";
  let currentSearch = "";

  function renderRules() {
    if (!container) return;

    const filtered = RULES_DATA.filter(rule => {
      const matchCat = currentCategory === "all" || rule.category === currentCategory;
      const matchSearch = currentSearch === "" || 
        rule.id.toLowerCase().includes(currentSearch) ||
        rule.title.toLowerCase().includes(currentSearch) ||
        rule.summary.toLowerCase().includes(currentSearch);
      return matchCat && matchSearch;
    });

    if (filtered.length === 0) {
      container.innerHTML = `
        <div style="grid-column: 1/-1; text-align: center; padding: 40px; color: var(--text-muted);">
          No matching rules found for "<b>${escapeHtml(currentSearch)}</b>" in this category.
        </div>
      `;
      return;
    }

    container.innerHTML = filtered.map(rule => `
      <div class="rule-item-card">
        <div class="rule-header-row">
          <span class="rule-id-badge">${rule.id}</span>
          <span class="sev-badge ${rule.severity}">${rule.severity}</span>
        </div>
        <h4 class="rule-title">${escapeHtml(rule.title)}</h4>
        <p class="rule-desc">${escapeHtml(rule.summary)}</p>
        <div class="rule-meta-box">
          <span class="rule-meta-label">Detection Mechanism</span>
          <div style="color: var(--text-secondary);">${escapeHtml(rule.mechanism)}</div>
        </div>
        <div class="rule-meta-box" style="margin-top: 8px; border-left-color: var(--emerald);">
          <span class="rule-meta-label">Remediation</span>
          <div style="color: #a7f3d0;">${escapeHtml(rule.remediation)}</div>
        </div>
      </div>
    `).join("");
  }

  pills.forEach(pill => {
    pill.addEventListener("click", () => {
      pills.forEach(p => p.classList.remove("active"));
      pill.classList.add("active");
      currentCategory = pill.getAttribute("data-cat");
      renderRules();
    });
  });

  if (searchInput) {
    searchInput.addEventListener("input", (e) => {
      currentSearch = e.target.value.trim().toLowerCase();
      renderRules();
    });
  }

  renderRules();
}

/**
 * Installation tab switcher
 */
function initInstallTabs() {
  const tabs = document.querySelectorAll(".install-tab-btn");
  const codeBox = document.querySelector("#install-code-box");

  tabs.forEach(tab => {
    tab.addEventListener("click", () => {
      tabs.forEach(t => t.classList.remove("active"));
      tab.classList.add("active");

      const os = tab.getAttribute("data-os");
      if (codeBox && INSTALL_SNIPPETS[os]) {
        codeBox.textContent = INSTALL_SNIPPETS[os];
      }
    });
  });

  if (codeBox) {
    codeBox.textContent = INSTALL_SNIPPETS.linux;
  }
}

/**
 * Universal copy buttons
 */
function initCopyButtons() {
  document.querySelectorAll("[data-copy-target]").forEach(btn => {
    btn.addEventListener("click", () => {
      const targetId = btn.getAttribute("data-copy-target");
      const targetEl = document.getElementById(targetId);
      if (targetEl) {
        const text = targetEl.innerText || targetEl.textContent;
        navigator.clipboard.writeText(text).then(() => {
          const original = btn.innerHTML;
          btn.innerHTML = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"></polyline></svg> Copied!`;
          btn.style.background = "var(--emerald)";
          btn.style.color = "#06080d";
          setTimeout(() => {
            btn.innerHTML = original;
            btn.style.background = "";
            btn.style.color = "";
          }, 2000);
        });
      }
    });
  });
}

function escapeHtml(str) {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}
