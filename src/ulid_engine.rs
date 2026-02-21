//! Core ULID engine providing all ULID operations for the plugin.

use std::str::FromStr;

use nu_protocol::{Record, Span, Value};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Length of a ULID string in Crockford Base32 encoding.
pub const ULID_STRING_LENGTH: usize = 26;

/// Maximum number of ULIDs in a single bulk generation request.
pub const MAX_BULK_GENERATION: usize = 10_000;

/// Valid characters for Crockford Base32 encoding used by ULIDs.
pub const CROCKFORD_BASE32_CHARSET: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Nanoseconds per millisecond, used for timestamp conversions.
pub const NANOS_PER_MILLI: u64 = 1_000_000;

/// Bitmask for the 80-bit randomness component of a ULID.
const ULID_RANDOMNESS_MASK: u128 = 0xFFFF_FFFF_FFFF_FFFF_FFFF;

/// Core ULID engine providing all ULID operations for the plugin.
pub struct UlidEngine;

/// Parsed components of a ULID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UlidComponents {
    /// The original ULID string.
    pub ulid: String,
    /// Millisecond timestamp extracted from the ULID.
    pub timestamp_ms: u64,
    /// Hexadecimal representation of the randomness component.
    pub randomness_hex: String,
    /// Whether the ULID passed validation.
    pub valid: bool,
}

/// Output format options for ULID operations.
#[derive(Debug, Clone, PartialEq)]
pub enum UlidOutputFormat {
    /// Plain string representation.
    String,
    /// JSON record with structured fields.
    Json,
    /// Raw 16-byte binary representation.
    Binary,
}

impl UlidEngine {
    /// Generates a single ULID.
    pub fn generate() -> Result<Ulid, UlidError> {
        Ok(Ulid::new())
    }

    /// Generates a ULID with a specific timestamp.
    pub fn generate_with_timestamp(timestamp_ms: u64) -> Result<Ulid, UlidError> {
        let ulid = Ulid::from_parts(timestamp_ms, rand::random::<u128>() & ULID_RANDOMNESS_MASK);
        Ok(ulid)
    }

    /// Generates multiple ULIDs efficiently.
    pub fn generate_bulk(count: usize) -> Result<Vec<Ulid>, UlidError> {
        if count == 0 {
            return Ok(Vec::new());
        }

        if count > MAX_BULK_GENERATION {
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

    /// Parses a ULID string into components.
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

    /// Returns `true` if the string is a valid ULID.
    #[must_use]
    pub fn validate(ulid_str: &str) -> bool {
        Ulid::from_str(ulid_str).is_ok()
    }

    /// Validates a ULID with detailed error information.
    pub fn validate_detailed(ulid_str: &str) -> UlidValidationResult {
        let mut result = UlidValidationResult {
            valid: true,
            length: ulid_str.len(),
            charset_valid: true,
            timestamp_valid: true,
            errors: Vec::new(),
        };

        // Check length
        if ulid_str.len() != ULID_STRING_LENGTH {
            result.valid = false;
            result.errors.push(format!(
                "Invalid length: expected {} characters, got {}",
                ULID_STRING_LENGTH,
                ulid_str.len()
            ));
        }

        // Check character set (Crockford Base32)
        for (i, c) in ulid_str.chars().enumerate() {
            if !CROCKFORD_BASE32_CHARSET.contains(c) {
                result.valid = false;
                result.charset_valid = false;
                result.errors.push(format!(
                    "Invalid character '{}' at position {}. Valid characters: {}",
                    c, i, CROCKFORD_BASE32_CHARSET
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

    /// Extracts the timestamp from a ULID.
    pub fn extract_timestamp(ulid_str: &str) -> Result<u64, UlidError> {
        match Ulid::from_str(ulid_str) {
            Ok(ulid) => Ok(ulid.timestamp_ms()),
            Err(e) => Err(UlidError::InvalidFormat {
                input: ulid_str.to_string(),
                reason: format!("Cannot extract timestamp: {}", e),
            }),
        }
    }

    /// Extracts the randomness component from a ULID.
    pub fn extract_randomness(ulid_str: &str) -> Result<u128, UlidError> {
        match Ulid::from_str(ulid_str) {
            Ok(ulid) => Ok(ulid.random()),
            Err(e) => Err(UlidError::InvalidFormat {
                input: ulid_str.to_string(),
                reason: format!("Cannot extract randomness: {}", e),
            }),
        }
    }

    /// Converts a ULID to a Nushell `Value` based on the output format.
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

    /// Converts `UlidComponents` to a Nushell `Value`.
    pub fn components_to_value(components: &UlidComponents, span: Span) -> Value {
        let mut record = Record::new();

        record.push("ulid", Value::string(components.ulid.clone(), span));

        let mut timestamp_record = Record::new();
        timestamp_record.push("ms", Value::int(components.timestamp_ms as i64, span));

        // Convert timestamp to ISO8601 format
        let timestamp_secs = components.timestamp_ms / 1000;
        let timestamp_nanos = (components.timestamp_ms % 1000) * NANOS_PER_MILLI;

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
}

/// ULID validation result with detailed error information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UlidValidationResult {
    /// Whether the ULID is valid.
    pub valid: bool,
    /// Length of the input string.
    pub length: usize,
    /// Whether all characters are valid Crockford Base32.
    pub charset_valid: bool,
    /// Whether the timestamp component is valid.
    pub timestamp_valid: bool,
    /// Descriptions of any validation errors found.
    pub errors: Vec<String>,
}

/// Errors produced by ULID operations.
#[derive(Debug, Clone)]
pub enum UlidError {
    /// The input string is not a valid ULID.
    InvalidFormat {
        /// The input that failed validation.
        input: String,
        /// Human-readable reason for the failure.
        reason: String,
    },
    /// A general input validation error.
    InvalidInput {
        /// Description of the problem.
        message: String,
    },
    /// The timestamp exceeds the maximum representable value.
    TimestampOutOfRange {
        /// The provided timestamp.
        timestamp: u64,
        /// The maximum allowed timestamp.
        max_timestamp: u64,
    },
    /// ULID generation failed.
    GenerationError {
        /// Human-readable reason for the failure.
        reason: String,
    },
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
