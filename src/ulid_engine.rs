use nu_protocol::{Record, Span, Value};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use ulid::Ulid;

/// Core ULID engine providing all ULID operations for the plugin
pub struct UlidEngine;

/// ULID parsing result containing structured components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UlidComponents {
    pub ulid: String,
    pub timestamp_ms: u64,
    pub randomness_hex: String,
    pub valid: bool,
}

/// ULID generation options
#[derive(Debug, Clone)]
pub struct UlidGenerationOptions {
    pub count: Option<usize>,
    pub timestamp_ms: Option<u64>,
    pub format: UlidOutputFormat,
}

/// Output format options for ULID operations
#[derive(Debug, Clone)]
pub enum UlidOutputFormat {
    String,
    Json,
    Binary,
}

impl Default for UlidGenerationOptions {
    fn default() -> Self {
        Self {
            count: None,
            timestamp_ms: None,
            format: UlidOutputFormat::String,
        }
    }
}

impl UlidEngine {
    /// Generate a single ULID
    pub fn generate() -> Result<Ulid, UlidError> {
        Ok(Ulid::new())
    }

    /// Generate a ULID with specific timestamp
    pub fn generate_with_timestamp(timestamp_ms: u64) -> Result<Ulid, UlidError> {
        let ulid = Ulid::from_parts(
            timestamp_ms,
            rand::random::<u128>() & 0xFFFFFFFFFFFFFFFFFFFF,
        );
        Ok(ulid)
    }

    /// Generate multiple ULIDs efficiently
    pub fn generate_bulk(count: usize) -> Result<Vec<Ulid>, UlidError> {
        if count == 0 {
            return Ok(Vec::new());
        }

        if count > 10_000 {
            return Err(UlidError::InvalidInput {
                message: "Bulk generation limited to 10,000 ULIDs per request for performance"
                    .to_string(),
            });
        }

        let mut result = Vec::with_capacity(count);
        for _ in 0..count {
            result.push(Ulid::new());
        }
        Ok(result)
    }

    /// Parse a ULID string into components
    pub fn parse(ulid_str: &str) -> Result<UlidComponents, UlidError> {
        match Ulid::from_str(ulid_str) {
            Ok(ulid) => {
                let components = UlidComponents {
                    ulid: ulid_str.to_string(),
                    timestamp_ms: ulid.timestamp_ms(),
                    randomness_hex: format!("{:x}", ulid.random()),
                    valid: true,
                };
                Ok(components)
            }
            Err(e) => Err(UlidError::InvalidFormat {
                input: ulid_str.to_string(),
                reason: format!("Parse error: {}", e),
            }),
        }
    }

    /// Validate a ULID string format
    pub fn validate(ulid_str: &str) -> bool {
        Ulid::from_str(ulid_str).is_ok()
    }

    /// Validate a ULID with detailed error information
    pub fn validate_detailed(ulid_str: &str) -> UlidValidationResult {
        let mut result = UlidValidationResult {
            valid: true,
            length: ulid_str.len(),
            charset_valid: true,
            timestamp_valid: true,
            errors: Vec::new(),
        };

        // Check length
        if ulid_str.len() != 26 {
            result.valid = false;
            result.errors.push(format!(
                "Invalid length: expected 26 characters, got {}",
                ulid_str.len()
            ));
        }

        // Check character set (Crockford Base32)
        let valid_chars = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
        for (i, c) in ulid_str.chars().enumerate() {
            if !valid_chars.contains(c) {
                result.valid = false;
                result.charset_valid = false;
                result.errors.push(format!(
                    "Invalid character '{}' at position {}. Valid characters: {}",
                    c, i, valid_chars
                ));
            }
        }

        // Attempt full parsing if basic checks pass
        if result.valid {
            match Ulid::from_str(ulid_str) {
                Ok(_) => {} // Valid ULID
                Err(e) => {
                    result.valid = false;
                    result.timestamp_valid = false;
                    result.errors.push(format!("Parse error: {}", e));
                }
            }
        }

        result
    }

    /// Extract timestamp from ULID
    pub fn extract_timestamp(ulid_str: &str) -> Result<u64, UlidError> {
        match Ulid::from_str(ulid_str) {
            Ok(ulid) => Ok(ulid.timestamp_ms()),
            Err(e) => Err(UlidError::InvalidFormat {
                input: ulid_str.to_string(),
                reason: format!("Cannot extract timestamp: {}", e),
            }),
        }
    }

    /// Extract randomness component from ULID
    pub fn extract_randomness(ulid_str: &str) -> Result<u128, UlidError> {
        match Ulid::from_str(ulid_str) {
            Ok(ulid) => Ok(ulid.random()),
            Err(e) => Err(UlidError::InvalidFormat {
                input: ulid_str.to_string(),
                reason: format!("Cannot extract randomness: {}", e),
            }),
        }
    }

    /// Convert ULID to Nushell Value based on format
    pub fn to_value(ulid: &Ulid, format: &UlidOutputFormat, span: Span) -> Value {
        match format {
            UlidOutputFormat::String => Value::string(ulid.to_string(), span),
            UlidOutputFormat::Json => {
                let mut record = Record::new();
                record.push("ulid", Value::string(ulid.to_string(), span));
                record.push("timestamp_ms", Value::int(ulid.timestamp_ms() as i64, span));
                record.push(
                    "randomness",
                    Value::string(format!("{:x}", ulid.random()), span),
                );
                Value::record(record, span)
            }
            UlidOutputFormat::Binary => {
                let bytes = ulid.to_bytes();
                Value::binary(bytes.to_vec(), span)
            }
        }
    }

    /// Convert UlidComponents to Nushell Value
    pub fn components_to_value(components: &UlidComponents, span: Span) -> Value {
        let mut record = Record::new();

        record.push("ulid", Value::string(components.ulid.clone(), span));

        let mut timestamp_record = Record::new();
        timestamp_record.push("ms", Value::int(components.timestamp_ms as i64, span));

        // Convert timestamp to ISO8601 format
        let timestamp_secs = components.timestamp_ms / 1000;
        let timestamp_nanos = (components.timestamp_ms % 1000) * 1_000_000;

        if let Some(datetime) =
            chrono::DateTime::from_timestamp(timestamp_secs as i64, timestamp_nanos as u32)
        {
            timestamp_record.push(
                "iso8601",
                Value::string(datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(), span),
            );
            timestamp_record.push("unix", Value::int(timestamp_secs as i64, span));
        }

        record.push("timestamp", Value::record(timestamp_record, span));

        let mut randomness_record = Record::new();
        randomness_record.push(
            "hex",
            Value::string(components.randomness_hex.clone(), span),
        );

        record.push("randomness", Value::record(randomness_record, span));

        record.push("valid", Value::bool(components.valid, span));

        Value::record(record, span)
    }

    /// Check if a ULID has security warnings
    pub fn has_security_concerns(usage_context: &str) -> bool {
        let unsafe_contexts = [
            "auth",
            "authentication",
            "token",
            "session",
            "password",
            "secret",
            "key",
            "security",
            "login",
            "credential",
        ];

        let context_lower = usage_context.to_lowercase();
        unsafe_contexts
            .iter()
            .any(|&unsafe_ctx| context_lower.contains(unsafe_ctx))
    }

    /// Get security advice for ULID usage
    pub fn get_security_advice() -> SecurityAdvice {
        SecurityAdvice {
            safe_use_cases: vec![
                "Database primary keys".to_string(),
                "Log correlation IDs".to_string(),
                "File/object naming".to_string(),
                "Sortable identifiers for analytics".to_string(),
                "General-purpose unique identifiers".to_string(),
            ],
            unsafe_use_cases: vec![
                "Authentication tokens".to_string(),
                "Session identifiers".to_string(),
                "Password reset tokens".to_string(),
                "API keys or secrets".to_string(),
                "Security-critical random values".to_string(),
            ],
            alternatives: vec![
                "Auth tokens: Use 256-bit cryptographically random strings".to_string(),
                "Sessions: Use UUID v4 or dedicated session token generators".to_string(),
                "API keys: Use proper key derivation functions".to_string(),
                "Secrets: Use dedicated secret management systems".to_string(),
            ],
            vulnerability_explanation: "ULIDs use monotonic generation within the same millisecond, making subsequent IDs predictable (previous + 1). This enables timing-based attacks in security contexts.".to_string(),
        }
    }
}

/// ULID validation result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UlidValidationResult {
    pub valid: bool,
    pub length: usize,
    pub charset_valid: bool,
    pub timestamp_valid: bool,
    pub errors: Vec<String>,
}

/// Security advice structure
#[derive(Debug, Clone)]
pub struct SecurityAdvice {
    pub safe_use_cases: Vec<String>,
    pub unsafe_use_cases: Vec<String>,
    pub alternatives: Vec<String>,
    pub vulnerability_explanation: String,
}

/// ULID operation errors
#[derive(Debug, Clone)]
pub enum UlidError {
    InvalidFormat { input: String, reason: String },
    InvalidInput { message: String },
    TimestampOutOfRange { timestamp: u64, max_timestamp: u64 },
    GenerationError { reason: String },
}

impl std::fmt::Display for UlidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UlidError::InvalidFormat { input, reason } => {
                write!(f, "Invalid ULID format '{}': {}", input, reason)
            }
            UlidError::InvalidInput { message } => {
                write!(f, "Invalid input: {}", message)
            }
            UlidError::TimestampOutOfRange {
                timestamp,
                max_timestamp,
            } => {
                write!(
                    f,
                    "Timestamp {} is out of range (max: {})",
                    timestamp, max_timestamp
                )
            }
            UlidError::GenerationError { reason } => {
                write!(f, "ULID generation error: {}", reason)
            }
        }
    }
}

impl std::error::Error for UlidError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ulid_generation() {
        let ulid = UlidEngine::generate().unwrap();
        assert_eq!(ulid.to_string().len(), 26);
    }

    #[test]
    fn test_ulid_validation() {
        // Valid ULID
        assert!(UlidEngine::validate("01AN4Z07BY79KA1307SR9X4MV3"));

        // Invalid ULIDs
        assert!(!UlidEngine::validate("invalid"));
        assert!(!UlidEngine::validate("01AN4Z07BY79KA1307SR9X4MV")); // Too short
        assert!(!UlidEngine::validate("01AN4Z07BY79KA1307SR9X4MV34")); // Too long
    }

    #[test]
    fn test_ulid_parsing() {
        let ulid_str = "01AN4Z07BY79KA1307SR9X4MV3";
        let components = UlidEngine::parse(ulid_str).unwrap();

        assert_eq!(components.ulid, ulid_str);
        assert!(components.valid);
        // The actual timestamp for this ULID
        assert_eq!(components.timestamp_ms, 1465824320894);
    }

    #[test]
    fn test_bulk_generation() {
        let ulids = UlidEngine::generate_bulk(10).unwrap();
        assert_eq!(ulids.len(), 10);

        // All should be unique
        let unique_count = ulids
            .iter()
            .map(|u| u.to_string())
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(unique_count, 10);
    }

    #[test]
    fn test_security_context_detection() {
        assert!(UlidEngine::has_security_concerns("authentication_token"));
        assert!(UlidEngine::has_security_concerns("session_key"));
        assert!(UlidEngine::has_security_concerns("password_reset"));

        assert!(!UlidEngine::has_security_concerns("database_id"));
        assert!(!UlidEngine::has_security_concerns("log_correlation"));
        assert!(!UlidEngine::has_security_concerns("file_name"));
    }

    #[test]
    fn test_timestamp_extraction() {
        let ulid_str = "01AN4Z07BY79KA1307SR9X4MV3";
        let timestamp = UlidEngine::extract_timestamp(ulid_str).unwrap();
        assert_eq!(timestamp, 1465824320894);
    }

    #[test]
    fn test_bulk_generation_limit() {
        let result = UlidEngine::generate_bulk(10_001);
        assert!(result.is_err());

        if let Err(UlidError::InvalidInput { message }) = result {
            assert!(message.contains("10,000"));
        }
    }
}
