use nu_protocol::{ShellError, Span, Value};

/// Convert UlidError to Nushell ShellError with user-friendly messages
pub fn ulid_error_to_shell_error(error: crate::UlidError, span: Span) -> ShellError {
    match error {
        crate::UlidError::InvalidFormat { input, reason } => ShellError::GenericError {
            error: "Invalid ULID format".to_string(),
            msg: format!("The input '{}' is not a valid ULID", input),
            span: Some(span),
            help: Some(format!(
                "{}\n\nValid ULID format: 26 characters using Crockford Base32\nExample: 01AN4Z07BY79KA1307SR9X4MV3",
                reason
            )),
            inner: Vec::new(),
        },
        crate::UlidError::InvalidInput { message } => ShellError::GenericError {
            error: "Invalid input".to_string(),
            msg: message.clone(),
            span: Some(span),
            help: Some("Check the command parameters and try again".to_string()),
            inner: Vec::new(),
        },
        crate::UlidError::TimestampOutOfRange {
            timestamp,
            max_timestamp,
        } => ShellError::GenericError {
            error: "Timestamp out of range".to_string(),
            msg: format!("Timestamp {} exceeds maximum allowed value", timestamp),
            span: Some(span),
            help: Some(format!(
                "Maximum timestamp: {} (year 10889)\nUse a timestamp between 0 and {}",
                max_timestamp, max_timestamp
            )),
            inner: Vec::new(),
        },
        crate::UlidError::GenerationError { reason } => ShellError::GenericError {
            error: "ULID generation failed".to_string(),
            msg: reason.clone(),
            span: Some(span),
            help: Some(
                "This may be due to system randomness issues or resource constraints".to_string(),
            ),
            inner: Vec::new(),
        },
    }
}

/// Create a security warning for inappropriate ULID usage
pub fn create_security_warning(context: &str, span: Span) -> ShellError {
    ShellError::GenericError {
        error: "⚠️  ULID Security Warning".to_string(),
        msg: format!("Using ULIDs for '{}' may not be secure", context),
        span: Some(span),
        help: Some(
            "ULIDs are not suitable for security-sensitive contexts like authentication tokens.\n\n\
            ✅ Safe uses: Database IDs, log correlation, file naming\n\
            ❌ Unsafe uses: Auth tokens, session IDs, API keys\n\n\
            For security contexts, use cryptographically random tokens instead.\n\
            Run 'ulid security-advice' for detailed guidance.".to_string()
        ),
        inner: Vec::new(),
    }
}

/// Validate command parameters with helpful error messages
pub fn validate_positive_integer(
    value: i64,
    param_name: &str,
    span: Span,
) -> Result<usize, Box<ShellError>> {
    if value < 0 {
        return Err(Box::new(ShellError::GenericError {
            error: "Invalid parameter".to_string(),
            msg: format!("Parameter '{}' must be positive", param_name),
            span: Some(span),
            help: Some(format!("Got: {}, expected: positive integer", value)),
            inner: Vec::new(),
        }));
    }

    if value > 10_000 {
        return Err(Box::new(ShellError::GenericError {
            error: "Parameter too large".to_string(),
            msg: format!("Parameter '{}' exceeds maximum allowed value", param_name),
            span: Some(span),
            help: Some("Maximum allowed: 10,000 for performance reasons".to_string()),
            inner: Vec::new(),
        }));
    }

    Ok(value as usize)
}

/// Validate ULID string with helpful error messages
pub fn validate_ulid_string(ulid_str: &str, span: Span) -> Result<(), Box<ShellError>> {
    if ulid_str.is_empty() {
        return Err(Box::new(ShellError::GenericError {
            error: "Empty ULID".to_string(),
            msg: "ULID string cannot be empty".to_string(),
            span: Some(span),
            help: Some("Provide a valid ULID string (26 characters)".to_string()),
            inner: Vec::new(),
        }));
    }

    if ulid_str.len() != 26 {
        return Err(Box::new(ShellError::GenericError {
            error: "Invalid ULID length".to_string(),
            msg: format!("ULID must be exactly 26 characters, got {}", ulid_str.len()),
            span: Some(span),
            help: Some("Valid ULID example: 01AN4Z07BY79KA1307SR9X4MV3".to_string()),
            inner: Vec::new(),
        }));
    }

    // Check character set
    let valid_chars = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    for (i, c) in ulid_str.chars().enumerate() {
        if !valid_chars.contains(c) {
            return Err(Box::new(ShellError::GenericError {
                error: "Invalid ULID character".to_string(),
                msg: format!("Invalid character '{}' at position {}", c, i),
                span: Some(span),
                help: Some(format!("Valid characters: {}", valid_chars)),
                inner: Vec::new(),
            }));
        }
    }

    Ok(())
}

/// Create error for unsupported operations
pub fn unsupported_operation_error(operation: &str, reason: &str, span: Span) -> ShellError {
    ShellError::GenericError {
        error: "Unsupported operation".to_string(),
        msg: format!("Operation '{}' is not supported", operation),
        span: Some(span),
        help: Some(format!(
            "{}\n\nSee 'ulid --help' for supported operations",
            reason
        )),
        inner: Vec::new(),
    }
}

/// Create informational message value
pub fn create_info_value(title: &str, message: &str, span: Span) -> Value {
    use nu_protocol::Record;

    let mut record = Record::new();
    record.push(
        "info",
        Value::String {
            val: title.to_string(),
            internal_span: span,
        },
    );
    record.push(
        "message",
        Value::String {
            val: message.to_string(),
            internal_span: span,
        },
    );

    Value::Record {
        val: record.into(),
        internal_span: span,
    }
}

/// Create success message with data
pub fn create_success_result<T>(data: T, message: &str, span: Span) -> Value
where
    T: Into<Value>,
{
    use nu_protocol::Record;

    let mut record = Record::new();
    record.push(
        "success",
        Value::Bool {
            val: true,
            internal_span: span,
        },
    );
    record.push(
        "message",
        Value::String {
            val: message.to_string(),
            internal_span: span,
        },
    );
    record.push("data", data.into());

    Value::Record {
        val: record.into(),
        internal_span: span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_positive_integer() {
        let span = Span::test_data();

        // Valid cases
        assert_eq!(validate_positive_integer(5, "count", span).unwrap(), 5);
        assert_eq!(
            validate_positive_integer(1000, "count", span).unwrap(),
            1000
        );

        // Invalid cases
        assert!(validate_positive_integer(-1, "count", span).is_err());
        assert!(validate_positive_integer(10_001, "count", span).is_err());
    }

    #[test]
    fn test_validate_ulid_string() {
        let span = Span::test_data();

        // Valid ULID
        assert!(validate_ulid_string("01AN4Z07BY79KA1307SR9X4MV3", span).is_ok());

        // Invalid cases
        assert!(validate_ulid_string("", span).is_err());
        assert!(validate_ulid_string("too_short", span).is_err());
        assert!(validate_ulid_string("01AN4Z07BY79KA1307SR9X4MV34", span).is_err()); // Too long
        assert!(validate_ulid_string("01AN4Z07BY79KA1307SR9X4MVI", span).is_err());
        // Invalid char 'I'
    }
}
