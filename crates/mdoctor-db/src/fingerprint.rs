//! SQL normalization and fingerprinting.

use regex::Regex;

/// Normalizes a raw SQL query into a parameterized fingerprint.
pub fn fingerprint_query(raw_sql: &str) -> String {
    let mut sql = raw_sql.trim().to_string();

    // Replace strings '...' or "..." with ?
    let str_re = Regex::new(r#"'[^']*'|"[^"]*""#).unwrap();
    sql = str_re.replace_all(&sql, "?").to_string();

    // Replace numeric literals with ?
    let num_re = Regex::new(r#"\b\d+\b"#).unwrap();
    sql = num_re.replace_all(&sql, "?").to_string();

    // Replace IN (?, ?, ?) with IN (?)
    let in_re = Regex::new(r#"(?i)\bin\s*\(\s*\?(?:\s*,\s*\?)*\s*\)"#).unwrap();
    sql = in_re.replace_all(&sql, "IN (?)").to_string();

    // Collapse multiple whitespace
    let ws_re = Regex::new(r#"\s+"#).unwrap();
    sql = ws_re.replace_all(&sql, " ").trim().to_string();

    sql
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint() {
        let q1 = "SELECT * FROM catalog_product_entity WHERE entity_id = 123";
        let q2 = "SELECT * FROM catalog_product_entity WHERE entity_id = 456";
        assert_eq!(fingerprint_query(q1), fingerprint_query(q2));
        assert_eq!(
            fingerprint_query(q1),
            "SELECT * FROM catalog_product_entity WHERE entity_id = ?"
        );

        let in_q = "SELECT * FROM table WHERE id IN (1, 2, 3)";
        assert_eq!(
            fingerprint_query(in_q),
            "SELECT * FROM table WHERE id IN (?)"
        );
    }
}
