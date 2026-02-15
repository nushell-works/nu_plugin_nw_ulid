//! Security warning and rating system for ULID usage contexts.

use nu_protocol::{Record, Span, Value};

/// Security warning system for ULID usage.
pub struct SecurityWarnings;

impl SecurityWarnings {
    /// Checks if a context or usage description suggests security-sensitive use.
    #[must_use]
    pub fn is_security_sensitive_context(context: &str) -> bool {
        let sensitive_keywords = [
            "auth",
            "authentication",
            "authorize",
            "authorization",
            "token",
            "session",
            "password",
            "secret",
            "key",
            "credential",
            "login",
            "signin",
            "signup",
            "security",
            "secure",
            "api_key",
            "apikey",
            "access_token",
            "refresh_token",
            "jwt",
            "oauth",
            "saml",
            "oidc",
            "reset",
            "recovery",
            "verification",
            "confirm",
            "nonce",
            "csrf",
            "xsrf",
            "challenge",
        ];

        let context_lower = context.to_lowercase();
        sensitive_keywords
            .iter()
            .any(|&keyword| context_lower.contains(keyword))
    }

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

    /// Creates a warning message for a specific context.
    pub fn create_context_warning(context: &str, span: Span) -> Value {
        let mut record = Record::new();

        record.push(
            "warning",
            Value::string("⚠️  Potential security concern detected", span),
        );

        record.push("context", Value::string(context, span));

        record.push(
            "message",
            Value::string(
                format!(
                    "The context '{}' suggests security-sensitive usage. ULIDs may not be appropriate for authentication, session management, or cryptographic purposes.",
                    context
                ),
                span,
            ),
        );

        record.push(
            "recommendation",
            Value::string(
                "Consider using cryptographically secure random tokens instead. Run 'ulid security-advice' for detailed guidance.",
                span,
            ),
        );

        Value::record(record, span)
    }

    /// Gets the security rating for a usage context.
    #[must_use]
    pub fn get_security_rating(context: &str) -> SecurityRating {
        let context_lower = context.to_lowercase();

        // High risk contexts
        let high_risk = [
            "auth",
            "authentication",
            "token",
            "session",
            "password",
            "secret",
            "key",
            "login",
            "api_key",
            "jwt",
            "oauth",
        ];

        // Medium risk contexts
        let medium_risk = [
            "user", "account", "profile", "admin", "security", "reset", "verify", "confirm",
            "access",
        ];

        // Low risk contexts
        let low_risk = [
            "database",
            "db",
            "record",
            "log",
            "file",
            "object",
            "trace",
            "correlation",
            "analytics",
            "monitoring",
        ];

        if high_risk.iter().any(|&risk| context_lower.contains(risk)) {
            SecurityRating::High
        } else if medium_risk.iter().any(|&risk| context_lower.contains(risk)) {
            SecurityRating::Medium
        } else if low_risk.iter().any(|&risk| context_lower.contains(risk)) {
            SecurityRating::Low
        } else {
            SecurityRating::Unknown
        }
    }

    /// Formats the security warning for command help text.
    pub fn format_command_warning() -> String {
        "⚠️  WARNING: ULIDs are not suitable for security-sensitive contexts.\n\
         ✅  Safe: Database IDs, log correlation, file naming\n\
         ❌  Unsafe: Auth tokens, session IDs, API keys\n\
         📖  See: ulid security-advice"
            .to_string()
    }

    /// Checks whether warnings should be shown for this operation.
    #[must_use]
    pub fn should_warn_for_operation(operation: &str, context: Option<&str>) -> bool {
        match context {
            Some(ctx) => Self::is_security_sensitive_context(ctx),
            None => {
                // Warn for bulk generation or operations that suggest production use
                operation.contains("bulk")
                    || operation.contains("batch")
                    || operation.contains("generate") && operation.contains("count")
            }
        }
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

/// Security risk rating for ULID usage contexts.
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityRating {
    /// Safe for ULIDs.
    Low,
    /// Caution advised.
    Medium,
    /// Not recommended for ULIDs.
    High,
    /// Context unclear.
    Unknown,
}

impl SecurityRating {
    /// Returns the rating as a human-readable string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityRating::Low => "Low",
            SecurityRating::Medium => "Medium",
            SecurityRating::High => "High",
            SecurityRating::Unknown => "Unknown",
        }
    }

    /// Returns actionable advice for the given rating level.
    #[must_use]
    pub fn get_advice(&self) -> &'static str {
        match self {
            SecurityRating::Low => "ULIDs are appropriate for this use case",
            SecurityRating::Medium => {
                "Consider security implications; ULIDs may be acceptable with caution"
            }
            SecurityRating::High => {
                "ULIDs are NOT recommended; use cryptographically secure alternatives"
            }
            SecurityRating::Unknown => "Assess security requirements before using ULIDs",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_context_detection() {
        // High risk contexts
        assert!(SecurityWarnings::is_security_sensitive_context(
            "authentication_token"
        ));
        assert!(SecurityWarnings::is_security_sensitive_context(
            "session_key"
        ));
        assert!(SecurityWarnings::is_security_sensitive_context(
            "password_reset"
        ));
        assert!(SecurityWarnings::is_security_sensitive_context(
            "api_key_generation"
        ));
        assert!(SecurityWarnings::is_security_sensitive_context(
            "JWT_secret"
        ));

        // Safe contexts
        assert!(!SecurityWarnings::is_security_sensitive_context(
            "database_id"
        ));
        assert!(!SecurityWarnings::is_security_sensitive_context(
            "log_correlation"
        ));
        assert!(!SecurityWarnings::is_security_sensitive_context(
            "file_naming"
        ));
        assert!(!SecurityWarnings::is_security_sensitive_context(
            "analytics_tracking"
        ));
    }

    #[test]
    fn test_security_rating() {
        assert_eq!(
            SecurityWarnings::get_security_rating("auth_token"),
            SecurityRating::High
        );
        assert_eq!(
            SecurityWarnings::get_security_rating("user_profile"),
            SecurityRating::Medium
        );
        assert_eq!(
            SecurityWarnings::get_security_rating("database_record"),
            SecurityRating::Low
        );
        assert_eq!(
            SecurityWarnings::get_security_rating("random_stuff"),
            SecurityRating::Unknown
        );
    }

    #[test]
    fn test_operation_warning_logic() {
        assert!(SecurityWarnings::should_warn_for_operation(
            "bulk_generate",
            None
        ));
        assert!(SecurityWarnings::should_warn_for_operation(
            "generate_with_count",
            None
        ));
        assert!(SecurityWarnings::should_warn_for_operation(
            "generate",
            Some("auth_token")
        ));
        assert!(!SecurityWarnings::should_warn_for_operation(
            "validate", None
        ));
        assert!(!SecurityWarnings::should_warn_for_operation("parse", None));
    }

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

    #[test]
    fn test_create_context_warning() {
        let span = Span::test_data();
        let result = SecurityWarnings::create_context_warning("auth_token", span);
        match result {
            Value::Record { val, .. } => {
                assert!(val.get("warning").is_some());
                assert!(val.get("context").is_some());
                assert_eq!(val.get("context").unwrap().as_str().unwrap(), "auth_token");
                assert!(val.get("message").is_some());
                assert!(val.get("recommendation").is_some());
            }
            _ => panic!("Expected record value"),
        }
    }

    #[test]
    fn test_format_command_warning() {
        let warning = SecurityWarnings::format_command_warning();
        assert!(warning.contains("WARNING"));
        assert!(warning.contains("Safe"));
        assert!(warning.contains("Unsafe"));
    }

    #[test]
    fn test_security_rating_as_str() {
        assert_eq!(SecurityRating::Low.as_str(), "Low");
        assert_eq!(SecurityRating::Medium.as_str(), "Medium");
        assert_eq!(SecurityRating::High.as_str(), "High");
        assert_eq!(SecurityRating::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_security_rating_get_advice() {
        assert!(SecurityRating::Low.get_advice().contains("appropriate"));
        assert!(SecurityRating::Medium.get_advice().contains("caution"));
        assert!(
            SecurityRating::High
                .get_advice()
                .contains("NOT recommended")
        );
        assert!(SecurityRating::Unknown.get_advice().contains("Assess"));
    }
}
