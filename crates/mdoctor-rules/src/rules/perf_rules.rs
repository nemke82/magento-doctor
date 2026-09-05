//! Performance static analysis rules (MD-PERF-021, MD-PERF-015, MD-PERF-030).

use mdoctor_core::{Category, Confidence, Finding, Severity};
use mdoctor_php::{AstFinding, OperationType};

pub fn evaluate_perf_rules(ast_findings: &[AstFinding]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for ast in ast_findings {
        match ast.operation {
            OperationType::RepositoryLoad => {
                let mut finding = Finding::new(
                    "MD-PERF-021",
                    "N+1 Repository load inside loop",
                    Severity::Critical,
                    Confidence::High,
                    Category::Performance,
                );

                finding.summary = format!(
                    "Repository entity access ({}) executed inside loop at line {}.",
                    ast.call_signature, ast.line_number
                );

                finding.evidence.push(format!("Call: {}", ast.call_signature));
                finding.evidence.push(format!("Line: {}", ast.line_number));
                if let Some(c) = &ast.class_name {
                    finding.evidence.push(format!("Class: {}", c));
                }
                if let Some(m) = &ast.method_name {
                    finding.evidence.push(format!("Method: {}", m));
                }
                finding.evidence.push(format!("Code: {}", ast.code_snippet));

                finding.impact = "Generates N separate database queries where N is collection size. Degrades linearly or exponentially as catalog/order volume scales.".to_string();
                finding.recommendation = "Eager-load required attributes via collection or join, or load entities in bulk using SearchCriteriaBuilder with in-filters.".to_string();

                findings.push(finding);
            }
            OperationType::HttpRequest => {
                let mut finding = Finding::new(
                    "MD-PERF-015",
                    "Synchronous outbound HTTP request detected",
                    Severity::Warning,
                    Confidence::High,
                    Category::Performance,
                );

                finding.summary = format!(
                    "Synchronous HTTP call ({}) detected at line {}.",
                    ast.call_signature, ast.line_number
                );

                finding.evidence.push(format!("Call: {}", ast.call_signature));
                finding.evidence.push(format!("Line: {}", ast.line_number));
                if let Some(c) = &ast.class_name {
                    finding.evidence.push(format!("Class: {}", c));
                }
                if let Some(m) = &ast.method_name {
                    finding.evidence.push(format!("Method: {}", m));
                }
                finding.evidence.push(format!("In loop: {}", ast.in_loop));

                finding.impact = "Blocking external HTTP requests during user requests or cron executions can stall workers, cause gateway timeouts, or lock resources.".to_string();
                finding.recommendation = "Offload remote API calls to an asynchronous queue (RabbitMQ / DB queue consumer) or set strict HTTP connection timeouts.".to_string();

                findings.push(finding);
            }
            OperationType::LoggingInLoop => {
                let mut finding = Finding::new(
                    "MD-PERF-030",
                    "Logging inside loop",
                    Severity::Warning,
                    Confidence::Medium,
                    Category::Performance,
                );

                finding.summary = format!("Logging call ({}) inside loop at line {}.", ast.call_signature, ast.line_number);
                finding.evidence.push(format!("Call: {}", ast.call_signature));
                finding.evidence.push(format!("Line: {}", ast.line_number));
                finding.impact = "Frequent disk I/O from disk log flushes during large batch loops causes severe I/O degradation and disk filling.".to_string();
                finding.recommendation = "Buffer log messages and write once after loop termination, or log at debug level with appropriate log sampling.".to_string();

                findings.push(finding);
            }
            _ => {}
        }
    }

    findings
}
