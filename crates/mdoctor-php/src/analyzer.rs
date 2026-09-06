//! Tree-sitter powered PHP AST visitor for Magento static analysis.

use std::path::Path;
use tree_sitter::{Node, Parser};
use crate::cost::{AstFinding, OperationType};

pub struct PhpAstAnalyzer {
    parser: Parser,
}

impl Default for PhpAstAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PhpAstAnalyzer {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_php::LANGUAGE_PHP;
        parser
            .set_language(&language.into())
            .expect("Error loading PHP grammar");
        Self { parser }
    }

    /// Analyze PHP source code directly.
    pub fn analyze_source(&mut self, source: &str) -> Vec<AstFinding> {
        let tree = match self.parser.parse(source, None) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut findings = Vec::new();
        let root = tree.root_node();
        let mut context = TraversalContext {
            source: source.as_bytes(),
            current_class: None,
            current_method: None,
            loop_depth: 0,
        };

        walk_node(root, &mut context, &mut findings);
        findings
    }

    /// Analyze a PHP file on disk.
    pub fn analyze_file(&mut self, path: &Path) -> Result<Vec<AstFinding>, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let mut findings = self.analyze_source(&content);
        for f in &mut findings {
            f.file_path = Some(path.to_path_buf());
        }
        Ok(findings)
    }
}

struct TraversalContext<'a> {
    source: &'a [u8],
    current_class: Option<String>,
    current_method: Option<String>,
    loop_depth: usize,
}

impl<'a> TraversalContext<'a> {
    fn node_text(&self, node: &Node) -> String {
        node.utf8_text(self.source).unwrap_or("").to_string()
    }
}

fn walk_node(node: Node, ctx: &mut TraversalContext, findings: &mut Vec<AstFinding>) {
    let kind = node.kind();

    let mut is_loop = false;
    let mut class_pushed = false;
    let mut method_pushed = false;

    match kind {
        "class_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                ctx.current_class = Some(ctx.node_text(&name_node));
                class_pushed = true;
            }
        }
        "method_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                ctx.current_method = Some(ctx.node_text(&name_node));
                method_pushed = true;
            }
        }
        "foreach_statement" | "for_statement" | "while_statement" | "do_statement" => {
            ctx.loop_depth += 1;
            is_loop = true;
        }
        "member_call_expression" => {
            check_member_call(&node, ctx, findings);
        }
        "scoped_call_expression" => {
            check_scoped_call(&node, ctx, findings);
        }
        "function_call_expression" => {
            check_function_call(&node, ctx, findings);
        }
        _ => {}
    }

    // Traverse children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_node(child, ctx, findings);
    }

    // Cleanup state on exit
    if is_loop {
        ctx.loop_depth = ctx.loop_depth.saturating_sub(1);
    }
    if class_pushed {
        ctx.current_class = None;
    }
    if method_pushed {
        ctx.current_method = None;
    }
}

fn check_member_call(node: &Node, ctx: &TraversalContext, findings: &mut Vec<AstFinding>) {
    let name_node = match node.child_by_field_name("name") {
        Some(n) => n,
        None => return,
    };
    let obj_node = match node.child_by_field_name("object") {
        Some(o) => o,
        None => return,
    };

    let method_name = ctx.node_text(&name_node);
    let obj_text = ctx.node_text(&obj_node);
    let line_number = node.start_position().row + 1;
    let full_call = ctx.node_text(node);

    let in_loop = ctx.loop_depth > 0;

    // 1. Repository load in loop (MD-PERF-021)
    if in_loop
        && (method_name == "getById" || method_name == "get" || method_name == "getBySku")
        && (obj_text.to_lowercase().contains("repo") || obj_text.to_lowercase().contains("repository"))
    {
        findings.push(AstFinding {
            operation: OperationType::RepositoryLoad,
            class_name: ctx.current_class.clone(),
            method_name: ctx.current_method.clone(),
            file_path: None,
            call_signature: format!("{}->{}()", obj_text, method_name),
            in_loop: true,
            line_number,
            code_snippet: full_call.clone(),
        });
    }

    // 2. Collection or entity load inside loop
    if in_loop && (method_name == "load" || method_name == "getItems") {
        findings.push(AstFinding {
            operation: OperationType::CollectionLoad,
            class_name: ctx.current_class.clone(),
            method_name: ctx.current_method.clone(),
            file_path: None,
            call_signature: format!("{}->{}()", obj_text, method_name),
            in_loop: true,
            line_number,
            code_snippet: full_call.clone(),
        });
    }

    // 3. Database write inside loop
    if in_loop && (method_name == "save" || method_name == "delete") {
        findings.push(AstFinding {
            operation: OperationType::DatabaseWrite,
            class_name: ctx.current_class.clone(),
            method_name: ctx.current_method.clone(),
            file_path: None,
            call_signature: format!("{}->{}()", obj_text, method_name),
            in_loop: true,
            line_number,
            code_snippet: full_call.clone(),
        });
    }

    // 4. Synchronous HTTP client calls
    if (method_name == "request" || method_name == "get" || method_name == "post" || method_name == "send")
        && (obj_text.to_lowercase().contains("client")
            || obj_text.to_lowercase().contains("http")
            || obj_text.to_lowercase().contains("guzzle"))
    {
        findings.push(AstFinding {
            operation: OperationType::HttpRequest,
            class_name: ctx.current_class.clone(),
            method_name: ctx.current_method.clone(),
            file_path: None,
            call_signature: format!("{}->{}()", obj_text, method_name),
            in_loop,
            line_number,
            code_snippet: full_call.clone(),
        });
    }

    // 5. Logging inside loop
    if in_loop
        && (method_name == "info" || method_name == "debug" || method_name == "log" || method_name == "error")
        && (obj_text.to_lowercase().contains("logger") || obj_text.to_lowercase().contains("log"))
    {
        findings.push(AstFinding {
            operation: OperationType::LoggingInLoop,
            class_name: ctx.current_class.clone(),
            method_name: ctx.current_method.clone(),
            file_path: None,
            call_signature: format!("{}->{}()", obj_text, method_name),
            in_loop: true,
            line_number,
            code_snippet: full_call.clone(),
        });
    }

    // 6. Direct SQL query execution
    if (method_name == "query" || method_name == "rawQuery")
        && (obj_text.to_lowercase().contains("conn") || obj_text.to_lowercase().contains("adapter"))
    {
        findings.push(AstFinding {
            operation: OperationType::DirectSqlQuery,
            class_name: ctx.current_class.clone(),
            method_name: ctx.current_method.clone(),
            file_path: None,
            call_signature: format!("{}->{}()", obj_text, method_name),
            in_loop,
            line_number,
            code_snippet: full_call,
        });
    }
}

fn check_scoped_call(node: &Node, ctx: &TraversalContext, findings: &mut Vec<AstFinding>) {
    let scope_node = match node.child_by_field_name("scope") {
        Some(s) => s,
        None => return,
    };
    let name_node = match node.child_by_field_name("name") {
        Some(n) => n,
        None => return,
    };

    let scope_text = ctx.node_text(&scope_node);
    let method_name = ctx.node_text(&name_node);
    let line_number = node.start_position().row + 1;
    let full_call = ctx.node_text(node);

    // 1. Direct ObjectManager usage (MD-DI-005)
    if scope_text.ends_with("ObjectManager") && method_name == "getInstance" {
        findings.push(AstFinding {
            operation: OperationType::ObjectManagerUsage,
            class_name: ctx.current_class.clone(),
            method_name: ctx.current_method.clone(),
            file_path: None,
            call_signature: format!("{}::{}()", scope_text, method_name),
            in_loop: ctx.loop_depth > 0,
            line_number,
            code_snippet: full_call.clone(),
        });
    }

    // 2. Session initialization
    if scope_text.contains("Session") && (method_name == "start" || method_name == "writeClose") {
        findings.push(AstFinding {
            operation: OperationType::SessionAccess,
            class_name: ctx.current_class.clone(),
            method_name: ctx.current_method.clone(),
            file_path: None,
            call_signature: format!("{}::{}()", scope_text, method_name),
            in_loop: ctx.loop_depth > 0,
            line_number,
            code_snippet: full_call,
        });
    }
}

fn check_function_call(node: &Node, ctx: &TraversalContext, findings: &mut Vec<AstFinding>) {
    let fn_node = match node.child_by_field_name("function") {
        Some(f) => f,
        None => return,
    };

    let fn_name = ctx.node_text(&fn_node);
    let line_number = node.start_position().row + 1;
    let full_call = ctx.node_text(node);

    // Synchronous curl execution
    if fn_name == "curl_exec" {
        findings.push(AstFinding {
            operation: OperationType::HttpRequest,
            class_name: ctx.current_class.clone(),
            method_name: ctx.current_method.clone(),
            file_path: None,
            call_signature: "curl_exec()".to_string(),
            in_loop: ctx.loop_depth > 0,
            line_number,
            code_snippet: full_call,
        });
    } else if fn_name == "file_get_contents"
        && (full_call.contains("http://") || full_call.contains("https://"))
    {
        findings.push(AstFinding {
            operation: OperationType::HttpRequest,
            class_name: ctx.current_class.clone(),
            method_name: ctx.current_method.clone(),
            file_path: None,
            call_signature: "file_get_contents(http...)".to_string(),
            in_loop: ctx.loop_depth > 0,
            line_number,
            code_snippet: full_call,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_n_plus_one_repository_in_loop() {
        let code = r#"<?php
namespace Vendor\Feed\Cron;

class Export {
    public function execute() {
        foreach ($products as $product) {
            $loaded = $this->productRepository->getById($product->getId());
        }
    }
}
"#;
        let mut analyzer = PhpAstAnalyzer::new();
        let findings = analyzer.analyze_source(code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].operation, OperationType::RepositoryLoad);
        assert!(findings[0].in_loop);
        assert_eq!(findings[0].line_number, 7);
        assert_eq!(findings[0].class_name.as_deref(), Some("Export"));
        assert_eq!(findings[0].method_name.as_deref(), Some("execute"));
    }

    #[test]
    fn test_detect_object_manager_usage() {
        let code = r#"<?php
class BadClass {
    public function run() {
        $om = \Magento\Framework\App\ObjectManager::getInstance();
    }
}
"#;
        let mut analyzer = PhpAstAnalyzer::new();
        let findings = analyzer.analyze_source(code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].operation, OperationType::ObjectManagerUsage);
        assert_eq!(findings[0].line_number, 4);
    }

    #[test]
    fn test_detect_sync_http_request() {
        let code = r#"<?php
class PaymentPlugin {
    public function aroundSubmit($subject, callable $proceed) {
        $ch = curl_init("https://api.gateway.com");
        $res = curl_exec($ch);
        return $proceed();
    }
}
"#;
        let mut analyzer = PhpAstAnalyzer::new();
        let findings = analyzer.analyze_source(code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].operation, OperationType::HttpRequest);
        assert_eq!(findings[0].line_number, 5);
    }
}
