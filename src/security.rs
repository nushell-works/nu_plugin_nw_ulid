use nu_protocol::{Record, Span, Value};

/// Security warning system for ULID usage
pub struct SecurityWarnings;

impl SecurityWarnings {
    /// Check if a context or usage description suggests security-sensitive use
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

    /// Get comprehensive security advice
    pub fn get_security_advice(span: Span) -> Value {
        let mut main_record = Record::new();

        // Header
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

        // Safe use cases
        let safe_cases = vec![
            "Database primary keys",
            "Log correlation IDs",
            "File and object naming",
            "Sortable identifiers for analytics",
            "General-purpose unique identifiers",
            "Event tracking and tracing",
            "Data pipeline identifiers",
        ];

        let safe_values: Vec<Value> = safe_cases
            .into_iter()
            .map(|case| Value::string(case, span))
            .collect();

        main_record.push("safe_use_cases", Value::list(safe_values, span));

        // Unsafe use cases
        let unsafe_cases = vec![
            "Authentication tokens",
            "Session identifiers",
            "Password reset tokens",
            "API keys or secrets",
            "Security-critical random values",
            "Cryptographic nonces",
            "CSRF tokens",
            "OAuth state parameters",
        ];

        let unsafe_values: Vec<Value> = unsafe_cases
            .into_iter()
            .map(|case| Value::string(case, span))
            .collect();

        main_record.push("unsafe_use_cases", Value::list(unsafe_values, span));

        // Vulnerability explanation
        main_record.push("vulnerability", Value::string(
            "When multiple ULIDs are generated within the same millisecond, the randomness component becomes a counter (incremented by 1). This creates predictable sequences that enable timing-based attacks.",
            span,
        ));

        // Attack example
        let mut attack_record = Record::new();
        attack_record.push(
            "scenario",
            Value::string("Generate two objects simultaneously", span),
        );
        attack_record.push(
            "time_t",
            Value::string("01AN4Z07BY + 79KA1307SR9X4MV3", span),
        );
        attack_record.push(
            "time_t_plus_1",
            Value::string("01AN4Z07BY + 79KA1307SR9X4MV4  (just incremented!)", span),
        );
        attack_record.push(
            "impact",
            Value::string("Second ULID = First ULID + 1 (predictable)", span),
        );

        main_record.push("attack_example", Value::record(attack_record, span));

        // Secure alternatives
        let alternatives = vec![
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

        let alt_values: Vec<Value> = alternatives
            .into_iter()
            .map(|(use_case, alternative)| {
                let mut alt_record = Record::new();
                alt_record.push("use_case", Value::string(use_case, span));
                alt_record.push("recommended", Value::string(alternative, span));
                Value::record(alt_record, span)
            })
            .collect();

        main_record.push("secure_alternatives", Value::list(alt_values, span));

        // Best practices
        let best_practices = vec![
            "Always assess whether your use case requires cryptographic security",
            "Document ULID usage context in your code and architecture",
            "Use ULIDs for identification, not authentication or authorization",
            "Prefer UUIDs or secure random generators for security-sensitive contexts",
            "Consider the trade-offs: sortability vs. cryptographic security",
            "Implement proper security reviews for identifier usage",
        ];

        let practice_values: Vec<Value> = best_practices
            .into_iter()
            .map(|practice| Value::string(practice, span))
            .collect();

        main_record.push("best_practices", Value::list(practice_values, span));

        // Additional resources
        main_record.push(
            "learn_more",
            Value::string("See ULID specification: https://github.com/ulid/spec", span),
        );

        Value::record(main_record, span)
    }

    /// Create a warning message for specific context
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

    /// Get security rating for a usage context
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

    /// Format security warning for command help text
    pub fn format_command_warning() -> String {
        "⚠️  WARNING: ULIDs are not suitable for security-sensitive contexts.\n\
         ✅  Safe: Database IDs, log correlation, file naming\n\
         ❌  Unsafe: Auth tokens, session IDs, API keys\n\
         📖  See: ulid security-advice"
            .to_string()
    }

    /// Check if we should show warnings for this operation
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

/// Security risk rating for ULID usage contexts
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityRating {
    Low,     // Safe for ULIDs
    Medium,  // Caution advised
    High,    // Not recommended for ULIDs
    Unknown, // Context unclear
}

impl SecurityRating {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityRating::Low => "Low",
            SecurityRating::Medium => "Medium",
            SecurityRating::High => "High",
            SecurityRating::Unknown => "Unknown",
        }
    }

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
}
