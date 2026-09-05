//! PHP runtime and version compatibility checks.

use mdoctor_core::Environment;
use mdoctor_knowledge::versions::is_php_supported;

#[derive(Debug, Clone)]
pub enum PhpRuntimeIssue {
    CliWebMismatch {
        cli_version: String,
        web_version: String,
    },
    UnsupportedPhpVersion {
        magento_version: String,
        php_version: String,
    },
}

pub fn check_php_runtime(
    env: &Environment,
    magento_version: Option<&str>,
) -> Vec<PhpRuntimeIssue> {
    let mut issues = Vec::new();

    // 1. Check CLI vs Web mismatch
    if let (Some(cli), Some(web)) = (&env.php_cli_version, &env.php_web_version) {
        if cli != web {
            issues.push(PhpRuntimeIssue::CliWebMismatch {
                cli_version: cli.clone(),
                web_version: web.clone(),
            });
        }
    }

    // 2. Check compatibility against Magento release requirements
    if let (Some(m_ver), Some(cli_ver)) = (magento_version, &env.php_cli_version) {
        if let Some(is_supp) = is_php_supported(m_ver, cli_ver) {
            if !is_supp {
                issues.push(PhpRuntimeIssue::UnsupportedPhpVersion {
                    magento_version: m_ver.to_string(),
                    php_version: cli_ver.clone(),
                });
            }
        }
    }

    issues
}
