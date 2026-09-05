//! Hot path database for Magento 2 performance forensics.

/// Critical hot path definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotPathWeight {
    Critical, // e.g. QuoteManagement::submit, Quote::collectTotals
    High,     // e.g. Product::getPrice, ProductRepository::getById
    Medium,   // e.g. Layout rendering, Layered navigation
}

/// Hot path entry representing a method or class frequently executed in requests.
#[derive(Debug, Clone)]
pub struct HotMethod {
    pub target_class: &'static str,
    pub target_method: Option<&'static str>,
    pub weight: HotPathWeight,
    pub description: &'static str,
}

/// Built-in list of Magento 2 hot methods.
pub const KNOWN_HOT_METHODS: &[HotMethod] = &[
    HotMethod {
        target_class: "Magento\\Quote\\Model\\QuoteManagement",
        target_method: Some("submit"),
        weight: HotPathWeight::Critical,
        description: "Order placement execution; delays directly hurt checkout conversion",
    },
    HotMethod {
        target_class: "Magento\\Quote\\Model\\Quote",
        target_method: Some("collectTotals"),
        weight: HotPathWeight::Critical,
        description: "Totals calculation executed on cart changes, shipping updates, and checkout",
    },
    HotMethod {
        target_class: "Magento\\Catalog\\Model\\Product",
        target_method: Some("getPrice"),
        weight: HotPathWeight::High,
        description: "Called repeatedly in category loops and product listings",
    },
    HotMethod {
        target_class: "Magento\\Catalog\\Model\\Product\\Type\\Price",
        target_method: Some("getPrice"),
        weight: HotPathWeight::High,
        description: "Product price calculation engine",
    },
    HotMethod {
        target_class: "Magento\\Catalog\\Api\\ProductRepositoryInterface",
        target_method: Some("getById"),
        weight: HotPathWeight::High,
        description: "Direct product load; dangerous when invoked in collection loops",
    },
    HotMethod {
        target_class: "Magento\\Customer\\Api\\CustomerRepositoryInterface",
        target_method: Some("getById"),
        weight: HotPathWeight::High,
        description: "Customer profile lookup",
    },
    HotMethod {
        target_class: "Magento\\Framework\\View\\Layout",
        target_method: Some("generateElements"),
        weight: HotPathWeight::Medium,
        description: "Layout element tree construction",
    },
    HotMethod {
        target_class: "Magento\\Framework\\View\\TemplateEngine\\Php",
        target_method: Some("fetchView"),
        weight: HotPathWeight::Medium,
        description: "Template PHTML rendering loop",
    },
];

/// Known hot events.
#[derive(Debug, Clone)]
pub struct HotEvent {
    pub name: &'static str,
    pub weight: HotPathWeight,
    pub description: &'static str,
}

pub const KNOWN_HOT_EVENTS: &[HotEvent] = &[
    HotEvent {
        name: "sales_order_place_after",
        weight: HotPathWeight::Critical,
        description: "Dispatched immediately upon order placement; synchronous delays stall customers",
    },
    HotEvent {
        name: "checkout_submit_all_after",
        weight: HotPathWeight::Critical,
        description: "Order and quote conversion completed",
    },
    HotEvent {
        name: "catalog_product_load_after",
        weight: HotPathWeight::High,
        description: "Dispatched every time an individual product model is loaded",
    },
    HotEvent {
        name: "controller_action_predispatch",
        weight: HotPathWeight::High,
        description: "Executed before every HTTP storefront controller action",
    },
    HotEvent {
        name: "controller_action_layout_render_before",
        weight: HotPathWeight::Medium,
        description: "Executed right before layout HTML generation; session access breaks FPC",
    },
];

/// Check if a given class & method is considered a Magento hot path.
pub fn is_hot_method(class_name: &str, method_name: Option<&str>) -> Option<HotPathWeight> {
    let normalized_class = class_name.trim_start_matches('\\');
    for hot in KNOWN_HOT_METHODS {
        if hot.target_class.trim_start_matches('\\').eq_ignore_ascii_case(normalized_class) {
            if let (Some(target_m), Some(m)) = (hot.target_method, method_name) {
                if target_m.eq_ignore_ascii_case(m) {
                    return Some(hot.weight);
                }
            } else if method_name.is_none() || hot.target_method.is_none() {
                return Some(hot.weight);
            }
        }
    }
    None
}

/// Check if an event name is a known hot event.
pub fn is_hot_event(event_name: &str) -> Option<HotPathWeight> {
    for hot in KNOWN_HOT_EVENTS {
        if hot.name.eq_ignore_ascii_case(event_name) {
            return Some(hot.weight);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_method_lookup() {
        assert_eq!(
            is_hot_method("Magento\\Quote\\Model\\QuoteManagement", Some("submit")),
            Some(HotPathWeight::Critical)
        );
        assert_eq!(
            is_hot_method("\\Magento\\Catalog\\Model\\Product", Some("getPrice")),
            Some(HotPathWeight::High)
        );
        assert_eq!(
            is_hot_method("Vendor\\Custom\\Model", Some("doSomething")),
            None
        );
    }

    #[test]
    fn test_hot_event_lookup() {
        assert_eq!(
            is_hot_event("sales_order_place_after"),
            Some(HotPathWeight::Critical)
        );
        assert_eq!(is_hot_event("unknown_event"), None);
    }
}
