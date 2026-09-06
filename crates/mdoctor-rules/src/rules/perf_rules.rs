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

                let loc_str = if let Some(path) = &ast.file_path {
                    format!("in {} at line {}", path.display(), ast.line_number)
                } else {
                    format!("at line {}", ast.line_number)
                };

                finding.summary = format!(
                    "Repository entity access ({}) executed inside loop {}.",
                    ast.call_signature, loc_str
                );

                finding.evidence.push(format!("Call: {}", ast.call_signature));
                if let Some(path) = &ast.file_path {
                    finding.evidence.push(format!("File: {}:{}", path.display(), ast.line_number));
                    finding.related_files.push(path.display().to_string());
                } else {
                    finding.evidence.push(format!("Line: {}", ast.line_number));
                }
                if let Some(c) = &ast.class_name {
                    finding.evidence.push(format!("Class: {}", c));
                }
                if let Some(m) = &ast.method_name {
                    finding.evidence.push(format!("Method: {}", m));
                }
                finding.evidence.push(format!("Code: {}", ast.code_snippet));

                let snippet_lower = ast.code_snippet.to_lowercase();
                let call_lower = ast.call_signature.to_lowercase();

                let (entity_type, code_example) = if snippet_lower.contains("category") || call_lower.contains("category") {
                    (
                        "Category",
                        "// 1. Collect category IDs before entering the loop:\n\
                         $categoryIds = array_unique(array_filter($allCategoryIds));\n\n\
                         // 2. Batch-load in a single query via CollectionFactory:\n\
                         $collection = $this->categoryCollectionFactory->create()\n\
                             ->addAttributeToSelect(['name', 'url_key', 'is_active'])\n\
                             ->addAttributeToFilter('entity_id', ['in' => $categoryIds]);\n\n\
                         // 3. Pre-index by ID for O(1) in-memory resolution:\n\
                         $categoriesById = [];\n\
                         foreach ($collection as $cat) {\n\
                             $categoriesById[$cat->getId()] = $cat;\n\
                         }\n\
                         // Inside loop: $category = $categoriesById[$categoryId] ?? null;"
                    )
                } else if snippet_lower.contains("product") || call_lower.contains("product") {
                    (
                        "Product",
                        "// 1. Collect product IDs before loop:\n\
                         $productIds = array_unique(array_filter($ids));\n\n\
                         // 2. Batch-load in 1 query via CollectionFactory:\n\
                         $collection = $this->productCollectionFactory->create()\n\
                             ->addAttributeToSelect(['name', 'sku', 'price', 'status'])\n\
                             ->addIdFilter($productIds);\n\n\
                         // 3. Pre-map into lookup array for O(1) loop access:\n\
                         $productsById = [];\n\
                         foreach ($collection as $prod) {\n\
                             $productsById[$prod->getId()] = $prod;\n\
                         }\n\
                         // Inside loop: $product = $productsById[$id] ?? null;"
                    )
                } else if snippet_lower.contains("order") || call_lower.contains("order") {
                    (
                        "Order",
                        "// 1. Collect order IDs before loop:\n\
                         $orderIds = array_unique(array_filter($ids));\n\n\
                         // 2. Batch-load using SearchCriteriaBuilder:\n\
                         $criteria = $this->searchCriteriaBuilder\n\
                             ->addFilter('entity_id', $orderIds, 'in')\n\
                             ->create();\n\
                         $orders = $this->orderRepository->getList($criteria)->getItems();\n\n\
                         // 3. Map orders by ID:\n\
                         $ordersById = [];\n\
                         foreach ($orders as $order) { $ordersById[$order->getEntityId()] = $order; }"
                    )
                } else {
                    (
                        "Entity",
                        "// 1. Pre-collect target entity IDs before the loop:\n\
                         $entityIds = array_unique(array_filter($ids));\n\n\
                         // 2. Batch-load outside the loop via SearchCriteriaBuilder:\n\
                         $criteria = $this->searchCriteriaBuilder\n\
                             ->addFilter('entity_id', $entityIds, 'in')\n\
                             ->create();\n\
                         $items = $this->repository->getList($criteria)->getItems();\n\n\
                         // 3. Index entities into an associative array for O(1) in-memory lookup:\n\
                         $itemsById = [];\n\
                         foreach ($items as $item) { $itemsById[$item->getId()] = $item; }"
                    )
                };

                finding.evidence.push(format!("Detected Entity Type: {}", entity_type));
                finding.impact = format!(
                    "Generates N separate database queries inside the loop (where N = loop iteration count). Degrades linearly or exponentially as {} volume grows, blocking PHP-FPM workers and saturating MySQL thread pools.",
                    entity_type
                );
                finding.recommendation = format!(
                    "Refactor to batch-load all {} entities before entering the loop to achieve O(1) lookup:\n{}",
                    entity_type, code_example
                );

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

                let loc_str = if let Some(path) = &ast.file_path {
                    format!("in {} at line {}", path.display(), ast.line_number)
                } else {
                    format!("at line {}", ast.line_number)
                };

                finding.summary = format!(
                    "Synchronous HTTP call ({}) detected {}.",
                    ast.call_signature, loc_str
                );

                finding.evidence.push(format!("Call: {}", ast.call_signature));
                if let Some(path) = &ast.file_path {
                    finding.evidence.push(format!("File: {}:{}", path.display(), ast.line_number));
                    finding.related_files.push(path.display().to_string());
                } else {
                    finding.evidence.push(format!("Line: {}", ast.line_number));
                }
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

                let loc_str = if let Some(path) = &ast.file_path {
                    format!("in {} at line {}", path.display(), ast.line_number)
                } else {
                    format!("at line {}", ast.line_number)
                };

                finding.summary = format!("Logging call ({}) inside loop {}.", ast.call_signature, loc_str);
                finding.evidence.push(format!("Call: {}", ast.call_signature));
                if let Some(path) = &ast.file_path {
                    finding.evidence.push(format!("File: {}:{}", path.display(), ast.line_number));
                    finding.related_files.push(path.display().to_string());
                } else {
                    finding.evidence.push(format!("Line: {}", ast.line_number));
                }
                finding.impact = "Frequent disk I/O from disk log flushes during large batch loops causes severe I/O degradation and disk filling.".to_string();
                finding.recommendation = "Buffer log messages and write once after loop termination, or log at debug level with appropriate log sampling.".to_string();

                findings.push(finding);
            }
            _ => {}
        }
    }

    findings
}
