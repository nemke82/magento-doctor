//! crontab.xml parser for declared cron jobs.

use std::path::Path;
use mdoctor_core::CronJob;

pub fn parse_crontab_xml(file_path: &Path, module_name: &str) -> Vec<CronJob> {
    let mut jobs = Vec::new();

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return jobs,
    };

    let doc = match roxmltree::Document::parse(&content) {
        Ok(d) => d,
        Err(_) => return jobs,
    };

    for group_node in doc.descendants().filter(|n| n.has_tag_name("group")) {
        let group_id = group_node.attribute("id").unwrap_or("default").to_string();

        for job_node in group_node.children().filter(|n| n.has_tag_name("job")) {
            if let (Some(name), Some(instance), Some(method)) = (
                job_node.attribute("name"),
                job_node.attribute("instance"),
                job_node.attribute("method"),
            ) {
                let mut schedule = None;
                if let Some(sched_node) = job_node.children().find(|n| n.has_tag_name("schedule")) {
                    schedule = sched_node.text().map(|t| t.trim().to_string());
                }

                let interval_seconds = schedule.as_deref().and_then(estimate_interval_seconds);
                let line = doc.text_pos_at(job_node.range().start).row as usize;

                jobs.push(CronJob {
                    name: name.to_string(),
                    group: group_id.clone(),
                    instance: instance.to_string(),
                    method: method.to_string(),
                    schedule,
                    interval_seconds,
                    module: module_name.to_string(),
                    source_file: file_path.to_path_buf(),
                    line,
                });
            }
        }
    }

    jobs
}

/// Estimates scheduling interval in seconds for standard cron patterns.
pub fn estimate_interval_seconds(expr: &str) -> Option<u64> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        return None;
    }

    let min = parts[0];
    if min == "*" {
        return Some(60);
    }
    if let Some(step) = min.strip_prefix("*/") {
        if let Ok(m) = step.parse::<u64>() {
            return Some(m * 60);
        }
    }
    if min == "0" && parts[1] == "*" {
        return Some(3600); // Hourly
    }
    if min == "0" && parts[1] == "0" {
        return Some(86400); // Daily
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_crontab_xml_sample() {
        let sample = r#"<?xml version="1.0"?>
<config xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <group id="default">
        <job name="vendor_export_feed" instance="Vendor\Feed\Cron\Export" method="execute">
            <schedule>* * * * *</schedule>
        </job>
    </group>
</config>
"#;
        let temp = std::env::temp_dir().join("test_crontab.xml");
        std::fs::write(&temp, sample).unwrap();

        let jobs = parse_crontab_xml(&temp, "Vendor_Feed");
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "vendor_export_feed");
        assert_eq!(jobs[0].schedule.as_deref(), Some("* * * * *"));
        assert_eq!(jobs[0].interval_seconds, Some(60));

        let _ = std::fs::remove_file(temp);
    }
}
