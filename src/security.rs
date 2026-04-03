//! Security warning and rating system for ULID usage contexts.

use nu_protocol::{Record, Span, Value};

/// Security warning system for ULID usage.
pub struct SecurityWarnings;

impl SecurityWarnings {
    /// Gets comprehensive security advice as a Nushell record value.
    pub fn get_security_advice(span: Span) -> Value {
        let mut main_record = Record::new();

        main_record.push(
            "title",
            Value::string("🚨 ULID Security Considerations", span),
        );
        main_record.push(
            "warning",
            Value::string(
                "ULIDs have important security limitations due to monotonic generation patterns",
                span,
            ),
        );
        main_record.push("safe_use_cases", build_use_case_list(SAFE_USE_CASES, span));
        main_record.push(
            "unsafe_use_cases",
            build_use_case_list(UNSAFE_USE_CASES, span),
        );
        main_record.push("vulnerability", Value::string(
            "When multiple ULIDs are generated within the same millisecond, the randomness component becomes a counter (incremented by 1). This creates predictable sequences that enable timing-based attacks.",
            span,
        ));
        main_record.push("attack_example", build_attack_example(span));
        main_record.push("secure_alternatives", build_secure_alternatives(span));
        main_record.push("best_practices", build_use_case_list(BEST_PRACTICES, span));
        main_record.push(
            "learn_more",
            Value::string("See ULID specification: https://github.com/ulid/spec", span),
        );

        Value::record(main_record, span)
    }
}

const SAFE_USE_CASES: &[&str] = &[
    "Database primary keys",
    "Log correlation IDs",
    "File and object naming",
    "Sortable identifiers for analytics",
    "General-purpose unique identifiers",
    "Event tracking and tracing",
    "Data pipeline identifiers",
];

const UNSAFE_USE_CASES: &[&str] = &[
    "Authentication tokens",
    "Session identifiers",
    "Password reset tokens",
    "API keys or secrets",
    "Security-critical random values",
    "Cryptographic nonces",
    "CSRF tokens",
    "OAuth state parameters",
];

const BEST_PRACTICES: &[&str] = &[
    "Always assess whether your use case requires cryptographic security",
    "Document ULID usage context in your code and architecture",
    "Use ULIDs for identification, not authentication or authorization",
    "Prefer UUIDs or secure random generators for security-sensitive contexts",
    "Consider the trade-offs: sortability vs. cryptographic security",
    "Implement proper security reviews for identifier usage",
];

fn build_use_case_list(items: &[&str], span: Span) -> Value {
    let values: Vec<Value> = items
        .iter()
        .map(|item| Value::string(*item, span))
        .collect();
    Value::list(values, span)
}

fn build_attack_example(span: Span) -> Value {
    let mut record = Record::new();
    record.push(
        "scenario",
        Value::string("Generate two objects simultaneously", span),
    );
    record.push(
        "time_t",
        Value::string("01AN4Z07BY + 79KA1307SR9X4MV3", span),
    );
    record.push(
        "time_t_plus_1",
        Value::string("01AN4Z07BY + 79KA1307SR9X4MV4  (just incremented!)", span),
    );
    record.push(
        "impact",
        Value::string("Second ULID = First ULID + 1 (predictable)", span),
    );
    Value::record(record, span)
}

fn build_secure_alternatives(span: Span) -> Value {
    let alternatives = [
        (
            "Authentication tokens",
            "256-bit cryptographically random strings",
        ),
        (
            "Session IDs",
            "UUID v4 or dedicated session token generators",
        ),
        (
            "API keys",
            "Proper key derivation functions (PBKDF2, scrypt, Argon2)",
        ),
        (
            "CSRF tokens",
            "Cryptographically secure random byte generators",
        ),
        (
            "Password reset tokens",
            "Secure random generators with expiration",
        ),
    ];

    let values: Vec<Value> = alternatives
        .iter()
        .map(|(use_case, alternative)| {
            let mut alt_record = Record::new();
            alt_record.push("use_case", Value::string(*use_case, span));
            alt_record.push("recommended", Value::string(*alternative, span));
            Value::record(alt_record, span)
        })
        .collect();

    Value::list(values, span)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_use_case_list() {
        let span = Span::test_data();
        let result = build_use_case_list(SAFE_USE_CASES, span);
        match result {
            Value::List { vals, .. } => {
                assert_eq!(vals.len(), SAFE_USE_CASES.len());
                assert_eq!(vals[0].as_str().unwrap(), "Database primary keys");
            }
            _ => panic!("Expected list value"),
        }
    }

    #[test]
    fn test_build_attack_example() {
        let span = Span::test_data();
        let result = build_attack_example(span);
        match result {
            Value::Record { val, .. } => {
                assert!(val.get("scenario").is_some());
                assert!(val.get("time_t").is_some());
                assert!(val.get("time_t_plus_1").is_some());
                assert!(val.get("impact").is_some());
            }
            _ => panic!("Expected record value"),
        }
    }

    #[test]
    fn test_build_secure_alternatives() {
        let span = Span::test_data();
        let result = build_secure_alternatives(span);
        match result {
            Value::List { vals, .. } => {
                assert_eq!(vals.len(), 5);
                // Each item should be a record with use_case and recommended
                match &vals[0] {
                    Value::Record { val, .. } => {
                        assert!(val.get("use_case").is_some());
                        assert!(val.get("recommended").is_some());
                    }
                    _ => panic!("Expected record in alternatives list"),
                }
            }
            _ => panic!("Expected list value"),
        }
    }

    #[test]
    fn test_get_security_advice_structure() {
        let span = Span::test_data();
        let result = SecurityWarnings::get_security_advice(span);
        match result {
            Value::Record { val, .. } => {
                assert!(val.get("title").is_some());
                assert!(val.get("warning").is_some());
                assert!(val.get("safe_use_cases").is_some());
                assert!(val.get("unsafe_use_cases").is_some());
                assert!(val.get("vulnerability").is_some());
                assert!(val.get("attack_example").is_some());
                assert!(val.get("secure_alternatives").is_some());
                assert!(val.get("best_practices").is_some());
                assert!(val.get("learn_more").is_some());
            }
            _ => panic!("Expected record value"),
        }
    }
}
