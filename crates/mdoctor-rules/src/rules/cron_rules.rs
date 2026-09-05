//! Cron forensics rules (MD-CRON-001, MD-CRON-010).

use mdoctor_core::{Category, Confidence, Finding, MagentoInstallation, Severity};
use mdoctor_php::AstFinding;

/// Evaluates cron jobs, schedule backlogs, and overlap ratios.
pub fn evaluate_cron_rules(
    installation: &MagentoInstallation,
    ast_findings: &[AstFinding],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let cron_summary = &installation.database_metrics.cron_schedule;

    // 1. MD-CRON-001: Cron backlog / stuck running jobs
    if cron_summary.pending_rows > 100 || cron_summary.running_rows > 5 {
        let mut finding = Finding::new(
            "MD-CRON-001",
            "Cron backlog detected in cron_schedule",
            Severity::Critical,
            Confidence::High,
            Category::Cron,
        );

        finding.summary = format!(
            "{} total rows in cron_schedule. {} pending, {} running jobs.",
            cron_summary.total_rows, cron_summary.pending_rows, cron_summary.running_rows
        );

        finding.evidence.push(format!("Total rows: {}", cron_summary.total_rows));
        finding.evidence.push(format!("Pending jobs: {}", cron_summary.pending_rows));
        finding.evidence.push(format!("Jobs in running state: {}", cron_summary.running_rows));
        if let (Some(oldest), Some(secs)) = (&cron_summary.oldest_running_job, cron_summary.oldest_running_seconds) {
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            finding.evidence.push(format!("Oldest running job: {} — {}h {}m", oldest, hours, mins));
        }

        finding.impact = "Delays all scheduled operations including indexers, catalog price rules, and transactional emails. May cause table lock contention in MySQL.".to_string();
        finding.recommendation = "Investigate stuck running jobs, verify OS cron daemon is executing bin/magento cron:run every minute under the correct user, and clean orphaned records.".to_string();
        finding.verification_commands = vec![
            "mysql> SELECT job_code, status, executed_at FROM cron_schedule WHERE status = 'running';".to_string(),
            "bin/magento cron:run -vvv".to_string(),
        ];
        finding.related_tables.push("cron_schedule".to_string());

        findings.push(finding);
    }

    // 2. MD-CRON-010: Job schedule overlap storm / heavy cron job
    for job in &installation.cron_jobs {
        let interval = job.interval_seconds.unwrap_or(60);

        // Check if runtime stats exist for this job
        if let Some(stat) = cron_summary.job_stats.get(&job.name) {
            if stat.median_seconds > interval as f64 {
                let overlap_ratio = stat.median_seconds / (interval as f64);

                let mut finding = Finding::new(
                    "MD-CRON-010",
                    format!("Cron job '{}' overlap risk detected", job.name),
                    Severity::Critical,
                    Confidence::High,
                    Category::Cron,
                );

                finding.summary = format!(
                    "Cron job '{}' median runtime ({:.1}s) exceeds scheduling interval ({}s). Overlap ratio: {:.2}.",
                    job.name, stat.median_seconds, interval, overlap_ratio
                );

                finding.evidence.push(format!("Declared in: {}", job.source_file.display()));
                finding.evidence.push(format!("Schedule interval: {}s", interval));
                finding.evidence.push(format!("Median runtime: {:.1}s (P95: {:.1}s)", stat.median_seconds, stat.p95_seconds));
                finding.evidence.push(format!("Overlap ratio: {:.2} (HIGH probability of overlapping executions)", overlap_ratio));

                // Correlate with AST findings if this job class had costly operations!
                let matching_ast: Vec<_> = ast_findings
                    .iter()
                    .filter(|f| {
                        f.class_name
                            .as_deref()
                            .map(|c| job.instance.ends_with(c))
                            .unwrap_or(false)
                    })
                    .collect();

                if !matching_ast.is_empty() {
                    finding.evidence.push("AST static analysis detected costly operations inside this job:".to_string());
                    for ast in matching_ast {
                        finding.evidence.push(format!("  - {} at line {} ({})", ast.operation, ast.line_number, ast.call_signature));
                    }
                }

                finding.impact = "Concurrent executions will overlap, compounding MySQL and CPU load, exhausting PHP workers, and stalling Magento indexer cron jobs.".to_string();
                finding.recommendation = format!(
                    "Reduce frequency in etc/crontab.xml, prevent overlap with a lock provider, or optimize {}::{}()",
                    job.instance, job.method
                );
                finding.verification_commands = vec![
                    format!("grep -rn \"{}\" app/code/ vendor/", job.name),
                    format!("mysql> SELECT * FROM cron_schedule WHERE job_code = '{}' ORDER BY scheduled_at DESC LIMIT 10;", job.name),
                ];
                finding.related_modules.push(job.module.clone());
                finding.related_files.push(job.source_file.display().to_string());

                findings.push(finding);
            }
        }
    }

    findings
}
