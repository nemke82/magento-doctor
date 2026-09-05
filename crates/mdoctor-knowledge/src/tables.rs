//! Knowledge base of volatile and high-growth Magento database tables.

#[derive(Debug, Clone)]
pub struct TableKnowledge {
    pub name: &'static str,
    pub description: &'static str,
    pub warning_row_count: u64,
    pub critical_row_count: u64,
    pub owning_module: &'static str,
}

pub const VOLATILE_TABLES: &[TableKnowledge] = &[
    TableKnowledge {
        name: "cron_schedule",
        description: "Magento cron execution log and queue. Excessive rows cause cron runner locking and slow indexing.",
        warning_row_count: 50_000,
        critical_row_count: 500_000,
        owning_module: "Magento_Cron",
    },
    TableKnowledge {
        name: "customer_visitor",
        description: "Customer session visitor tracking table. Can grow into millions if log cleaning is unconfigured.",
        warning_row_count: 500_000,
        critical_row_count: 2_000_000,
        owning_module: "Magento_Customer",
    },
    TableKnowledge {
        name: "report_event",
        description: "Reports product view and compare events. Grows continuously unless cleaned.",
        warning_row_count: 1_000_000,
        critical_row_count: 10_000_000,
        owning_module: "Magento_Reports",
    },
    TableKnowledge {
        name: "queue_message_status",
        description: "Queue message statuses. Slow consumers or unacknowledged messages cause massive bloat.",
        warning_row_count: 500_000,
        critical_row_count: 3_000_000,
        owning_module: "Magento_MysqlMq",
    },
    TableKnowledge {
        name: "queue_message",
        description: "Asynchronous queue payload body storage.",
        warning_row_count: 200_000,
        critical_row_count: 1_000_000,
        owning_module: "Magento_MysqlMq",
    },
    TableKnowledge {
        name: "adminnotification_inbox",
        description: "Admin notification feeds. Unread or uncleared notifications accumulate indefinitely.",
        warning_row_count: 10_000,
        critical_row_count: 50_000,
        owning_module: "Magento_AdminNotification",
    },
];

/// Look up table knowledge by exact table name.
pub fn find_table_knowledge(name: &str) -> Option<&'static TableKnowledge> {
    VOLATILE_TABLES.iter().find(|t| t.name.eq_ignore_ascii_case(name))
}
