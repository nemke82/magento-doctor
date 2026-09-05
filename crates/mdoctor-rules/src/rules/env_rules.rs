//! Environment and Security rules (MD-ENV-001, MD-ENV-002, MD-SEC-001, MD-SEC-002).

use mdoctor_core::{Category, Confidence, Finding, MagentoInstallation, MagentoMode, Severity};
use mdoctor_runtime::{check_php_runtime, inspect_filesystem, PhpRuntimeIssue};

pub fn evaluate_env_rules(installation: &MagentoInstallation) -> Vec<Finding> {
    let mut findings = Vec::new();

    // 1. PHP Runtime checks
    let php_issues = check_php_runtime(&installation.environment, installation.version.as_deref());
    for issue in php_issues {
        match issue {
            PhpRuntimeIssue::CliWebMismatch { cli_version, web_version } => {
                let mut finding = Finding::new(
                    "MD-ENV-002",
                    "PHP version mismatch between CLI and Web server",
                    Severity::Warning,
                    Confidence::High,
                    Category::Environment,
                );

                finding.summary = format!(
                    "CLI PHP version ({}) differs from Web PHP version ({}).",
                    cli_version, web_version
                );

                finding.evidence.push(format!("CLI PHP: {}", cli_version));
                finding.evidence.push(format!("Web PHP: {}", web_version));

                finding.impact = "Magento cron jobs run under CLI PHP while web requests run under Web PHP. Discrepancies cause syntax errors, serialization failures, or fatal errors in background workers.".to_string();
                finding.recommendation = "Align CLI PHP binary path in crontab with the Web PHP version.".to_string();
                finding.verification_commands = vec![
                    "which php".to_string(),
                    "php -v".to_string(),
                ];

                findings.push(finding);
            }
            PhpRuntimeIssue::UnsupportedPhpVersion { magento_version, php_version } => {
                let mut finding = Finding::new(
                    "MD-ENV-001",
                    "Unsupported PHP version for Magento release",
                    Severity::Critical,
                    Confidence::High,
                    Category::Environment,
                );

                finding.summary = format!(
                    "PHP {} is not officially supported for Magento {}.",
                    php_version, magento_version
                );

                finding.evidence.push(format!("Running PHP: {}", php_version));
                finding.evidence.push(format!("Magento Version: {}", magento_version));

                finding.impact = "Running on unsupported PHP versions risks crashes, memory leaks, unhandled exceptions, and security vulnerabilities.".to_string();
                finding.recommendation = "Upgrade or downgrade PHP to a version officially supported in Adobe's system requirements matrix.".to_string();

                findings.push(finding);
            }
        }
    }

    // 2. MD-SEC-001: Deployment mode check
    if installation.mode == MagentoMode::Developer {
        let mut finding = Finding::new(
            "MD-SEC-001",
            "Store is running in 'developer' mode",
            Severity::Warning,
            Confidence::High,
            Category::Security,
        );

        finding.summary = "MAGE_MODE is set to 'developer' instead of 'production'.".to_string();
        finding.evidence.push("MAGE_MODE: developer".to_string());
        finding.impact = "Developer mode drastically slows storefront response times by compiling assets on-the-fly and exposes stack traces to visitors.".to_string();
        finding.recommendation = "Switch deployment mode to production: 'bin/magento deploy:mode:set production'".to_string();
        finding.verification_commands = vec![
            "bin/magento deploy:mode:show".to_string(),
        ];

        findings.push(finding);
    }

    // 3. MD-SEC-002: World-writable filesystem checks
    let fs_results = inspect_filesystem(&installation.root);
    for res in fs_results {
        if res.is_world_writable {
            let mut finding = Finding::new(
                "MD-SEC-002",
                format!("World-writable directory detected: {}", res.path),
                Severity::Critical,
                Confidence::High,
                Category::Security,
            );

            finding.summary = format!("Path '{}' has world-writable permissions (e.g. 0777).", res.path);
            finding.evidence.push(format!("Path: {}", res.path));
            finding.impact = "World-writable directories allow other local OS users or compromised processes to modify code or inject malicious scripts.".to_string();
            finding.recommendation = format!("Restrict permissions on '{}' to 0755 or 0770: 'chmod 750 {}'", res.path, res.path);
            finding.verification_commands = vec![
                format!("ls -ld {}", res.path),
            ];

            findings.push(finding);
        }
    }

    findings
}
