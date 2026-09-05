//! In-depth explanation engine for Magento Doctor rules.

pub struct RuleExplanation {
    pub rule_id: &'static str,
    pub title: &'static str,
    pub what: &'static str,
    pub why_affected: &'static str,
    pub detection_mechanism: &'static str,
    pub false_positives: &'static str,
    pub verification: &'static str,
    pub remediation: &'static str,
}

pub const RULE_EXPLANATIONS: &[RuleExplanation] = &[
    RuleExplanation {
        rule_id: "MD-CRON-001",
        title: "Cron Backlog in cron_schedule",
        what: "The cron_schedule table contains an unusually high number of pending or stuck running jobs.",
        why_affected: "Magento relies on its internal cron engine for vital asynchronous operations: reindexing changelogs (MView), updating catalog prices, sending transactional order emails, customer alerts, and clearing visitors. When cron falls behind, catalog updates stop reflecting on storefronts and database locks accumulate.",
        detection_mechanism: "Queries cron_schedule for rows in 'pending' status older than schedule threshold, or jobs in 'running' status with execution times exceeding 15 minutes.",
        false_positives: "Rare. A single massive one-off data migration script running via cron can temporarily display as stuck running.",
        verification: "Run in MySQL:\n  SELECT job_code, status, scheduled_at, executed_at FROM cron_schedule WHERE status = 'running' ORDER BY executed_at ASC;\nOr test manually:\n  bin/magento cron:run -vvv",
        remediation: "1. Terminate orphaned stuck PHP processes.\n2. Delete stale running rows from cron_schedule.\n3. Verify system crontab (* * * * * php /path/bin/magento cron:run) runs under the correct web/CLI user.",
    },
    RuleExplanation {
        rule_id: "MD-CRON-010",
        title: "Cron Job Overlap Storm",
        what: "A scheduled job's observed execution duration exceeds its scheduling interval (e.g. interval = 60s, median runtime = 74s).",
        why_affected: "When a job runs longer than its frequency interval, subsequent cron runs spawn new overlapping instances before previous instances finish. These concurrent instances fight for the same database locks and CPU cores, creating exponential load.",
        detection_mechanism: "Compares declared schedule interval in crontab.xml against median/P95 execution durations in cron_schedule, calculating overlap_ratio = runtime / interval. Correlates with PHP AST to identify heavy loops or HTTP calls.",
        false_positives: "Low. If historical runtime data is skewed by an isolated external API outage, runtime may be temporarily inflated.",
        verification: "Inspect historical executions:\n  SELECT job_code, status, executed_at, finished_at, TIMESTAMPDIFF(SECOND, executed_at, finished_at) AS dur\n  FROM cron_schedule WHERE job_code = 'YOUR_JOB' AND status = 'success'\n  ORDER BY executed_at DESC LIMIT 20;",
        remediation: "1. Decrease frequency in etc/crontab.xml (e.g. change '* * * * *' to '*/15 * * * *').\n2. Move heavy remote API integrations or exports to asynchronous queues.\n3. Implement execution locking in PHP to prevent concurrent execution.",
    },
    RuleExplanation {
        rule_id: "MD-PLG-001",
        title: "Around Plugin on Critical Hot Path",
        what: "An 'around' plugin intercepts a high-frequency Magento core method (such as QuoteManagement::submit or Product::getPrice).",
        why_affected: "Adobe's extension documentation explicitly discourages 'around' plugins where 'before' or 'after' plugins suffice. Around plugins construct additional call stacks and closure allocations. On hot paths, especially checkout or product loops, this multiplies response latency and introduces fatal recursion risks if $proceed is mishandled.",
        detection_mechanism: "Cross-checks di.xml plugin declarations against Magento Doctor's curated hot-path database and analyzes the plugin class AST for loops, database writes, or outbound network calls.",
        false_positives: "None on the presence of the around plugin. The actual impact depends on storefront traffic volume.",
        verification: "Check plugin declaration:\n  grep -rn \"QuoteManagement\" app/code/ vendor/ --include=di.xml",
        remediation: "Refactor the plugin into a 'before' or 'after' plugin. If arguments or return values must be altered, use before* or after* hooks.",
    },
    RuleExplanation {
        rule_id: "MD-PERF-021",
        title: "N+1 Repository Access in Collection Loop",
        what: "A repository load (e.g. $productRepository->getById()) is executed inside a foreach/while loop over a collection.",
        why_affected: "Calling getById() inside a loop causes Magento to execute a fresh database query (and multiple EAV joins) for every single item in the collection. For 1,000 items, this triggers 1,000 distinct SQL queries instead of 1 batched query.",
        detection_mechanism: "Tree-sitter PHP AST analysis walks the syntax tree, tracks loop depth, and flags member call expressions on repository instances within loop blocks.",
        false_positives: "Very low. In rare cases, a loop with a hardcoded limit of 1 or 2 iterations might trigger a warning.",
        verification: "Inspect the flagged source file and line number. Enable MySQL query logging or Magento DB profiler during that execution.",
        remediation: "Add required attributes directly to the parent collection using ->addAttributeToSelect(), or use SearchCriteriaBuilder with an 'in' filter to load all required entities in one round-trip.",
    },
    RuleExplanation {
        rule_id: "MD-DB-001",
        title: "Missing Declared Database Index",
        what: "A table index defined in a module's db_schema.xml is missing from the physical database table.",
        why_affected: "When an index is missing, MySQL must scan entire tables or use unindexed temporary filesorts for queries that developers expected to be indexed.",
        detection_mechanism: "Compares declared indexes in all active modules' db_schema.xml against information_schema.statistics.",
        false_positives: "None. If the index is in db_schema.xml and missing from MySQL, declarative schema was not applied.",
        verification: "Run in MySQL:\n  SHOW INDEX FROM <table>;",
        remediation: "Run 'bin/magento setup:db-declaration:generate-whitelist' followed by 'bin/magento setup:upgrade'.",
    },
    RuleExplanation {
        rule_id: "MD-CACHE-001",
        title: "Redis Session and Cache Database Collision",
        what: "Redis session storage and default application cache or full page cache are configured to use the exact same database number.",
        why_affected: "In Redis, FLUSHDB flushes the entire numbered database. If sessions and cache share DB 0, running 'bin/magento cache:flush' immediately purges all customer sessions, logging out shoppers and abandoning active carts.",
        detection_mechanism: "Inspects Redis database numbers configured under 'session' and 'cache' in app/etc/env.php.",
        false_positives: "None.",
        verification: "Inspect app/etc/env.php session and cache sections.",
        remediation: "Assign distinct database numbers in app/etc/env.php: e.g. database 0 for default cache, database 1 for page_cache, database 2 for session.",
    },
];

pub fn get_rule_explanation(rule_id: &str) -> Option<&'static RuleExplanation> {
    RULE_EXPLANATIONS.iter().find(|e| e.rule_id.eq_ignore_ascii_case(rule_id))
}
