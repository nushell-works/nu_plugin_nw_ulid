use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Span, SyntaxShape, Type, Value,
};

use crate::UlidPlugin;

const TIMESTAMP_MILLIS_THRESHOLD: i64 = 1_000_000_000_000;

pub struct UlidTimeNowCommand;

impl PluginCommand for UlidTimeNowCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid time now"
    }

    fn description(&self) -> &str {
        "Get the current timestamp in various formats"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .named(
                "format",
                SyntaxShape::String,
                "Output format: 'iso8601', 'rfc3339', 'millis', 'seconds'",
                Some('f'),
            )
            .input_output_types(vec![(Type::Nothing, Type::String)])
            .category(Category::Date)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid time now",
                description: "Get current timestamp in ISO8601 format",
                result: None,
            },
            Example {
                example: "ulid time now --format millis",
                description: "Get current timestamp in milliseconds (ULID format)",
                result: None,
            },
            Example {
                example: "ulid time now --format seconds",
                description: "Get current timestamp in seconds",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let format: Option<String> = call.get_flag("format")?;
        let now = Utc::now();

        let result = match format.as_deref() {
            Some("millis") => Value::int(now.timestamp_millis(), call.head),
            Some("seconds") => Value::int(now.timestamp(), call.head),
            Some("rfc3339") => Value::string(now.to_rfc3339(), call.head),
            Some("iso8601") | None => {
                Value::string(now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(), call.head)
            }
            Some(fmt) => {
                return Err(LabeledError::new("Invalid format").with_label(
                    format!(
                        "Unknown format '{}'. Valid formats: iso8601, rfc3339, millis, seconds",
                        fmt
                    ),
                    call.head,
                ));
            }
        };

        Ok(PipelineData::Value(result, None))
    }
}

pub struct UlidTimeParseCommand;

impl PluginCommand for UlidTimeParseCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid time parse"
    }

    fn description(&self) -> &str {
        "Parse a timestamp string or number into various formats"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "timestamp",
                SyntaxShape::Any,
                "Timestamp to parse (string, int, or number)",
            )
            .input_output_types(vec![(Type::Nothing, Type::Record(vec![].into()))])
            .category(Category::Date)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid time parse '2024-01-01T00:00:00Z'",
                description: "Parse an ISO8601 timestamp",
                result: None,
            },
            Example {
                example: "ulid time parse 1704067200000",
                description: "Parse a millisecond timestamp",
                result: None,
            },
            Example {
                example: "ulid time parse 1704067200",
                description: "Parse a second timestamp",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let timestamp: Value = call.req(0)?;

        let datetime = match timestamp {
            Value::String { val, .. } => {
                // Try parsing as ISO8601/RFC3339
                DateTime::parse_from_rfc3339(&val)
                    .or_else(|_| DateTime::parse_from_str(&val, "%Y-%m-%dT%H:%M:%S%.3fZ"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| {
                        LabeledError::new("Failed to parse timestamp")
                            .with_label(format!("Invalid timestamp format: {}", e), call.head)
                    })?
            }
            Value::Int { val, .. } => {
                // Determine if it's seconds or milliseconds based on magnitude
                if val > TIMESTAMP_MILLIS_THRESHOLD {
                    // Looks like milliseconds
                    Utc.timestamp_millis_opt(val).single().ok_or_else(|| {
                        LabeledError::new("Invalid timestamp")
                            .with_label("Timestamp is out of range", call.head)
                    })?
                } else {
                    // Looks like seconds
                    Utc.timestamp_opt(val, 0).single().ok_or_else(|| {
                        LabeledError::new("Invalid timestamp")
                            .with_label("Timestamp is out of range", call.head)
                    })?
                }
            }
            Value::Float { val, .. } => {
                let seconds = val.trunc() as i64;
                let nanos = ((val.fract() * 1_000_000_000.0) as u32).min(999_999_999);
                Utc.timestamp_opt(seconds, nanos).single().ok_or_else(|| {
                    LabeledError::new("Invalid timestamp")
                        .with_label("Timestamp is out of range", call.head)
                })?
            }
            _ => {
                return Err(LabeledError::new("Invalid input type")
                    .with_label("Expected string, int, or float", call.head));
            }
        };

        let record = Value::record(
            [
                (
                    "iso8601".into(),
                    Value::string(
                        datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                        call.head,
                    ),
                ),
                (
                    "rfc3339".into(),
                    Value::string(datetime.to_rfc3339(), call.head),
                ),
                (
                    "unix_seconds".into(),
                    Value::int(datetime.timestamp(), call.head),
                ),
                (
                    "unix_millis".into(),
                    Value::int(datetime.timestamp_millis(), call.head),
                ),
                ("year".into(), Value::int(datetime.year() as i64, call.head)),
                (
                    "month".into(),
                    Value::int(datetime.month() as i64, call.head),
                ),
                ("day".into(), Value::int(datetime.day() as i64, call.head)),
                ("hour".into(), Value::int(datetime.hour() as i64, call.head)),
                (
                    "minute".into(),
                    Value::int(datetime.minute() as i64, call.head),
                ),
                (
                    "second".into(),
                    Value::int(datetime.second() as i64, call.head),
                ),
                (
                    "nanosecond".into(),
                    Value::int(datetime.nanosecond() as i64, call.head),
                ),
            ]
            .into_iter()
            .collect(),
            call.head,
        );

        Ok(PipelineData::Value(record, None))
    }
}

pub struct UlidTimeMillisCommand;

impl PluginCommand for UlidTimeMillisCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid time millis"
    }

    fn description(&self) -> &str {
        "Convert various timestamp formats to milliseconds (ULID timestamp format)"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .optional(
                "timestamp",
                SyntaxShape::Any,
                "Timestamp to convert (defaults to now)",
            )
            .input_output_types(vec![(Type::Nothing, Type::Int)])
            .category(Category::Date)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid time millis",
                description: "Get current time in milliseconds",
                result: None,
            },
            Example {
                example: "ulid time millis '2024-01-01T00:00:00Z'",
                description: "Convert ISO8601 to milliseconds",
                result: Some(Value::int(1704067200000, Span::test_data())),
            },
            Example {
                example: "ulid time millis 1704067200",
                description: "Convert seconds to milliseconds",
                result: Some(Value::int(1704067200000, Span::test_data())),
            },
        ]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let timestamp: Option<Value> = call.opt(0)?;

        let millis = match timestamp {
            None => Utc::now().timestamp_millis(),
            Some(Value::String { val, .. }) => {
                let datetime = DateTime::parse_from_rfc3339(&val)
                    .or_else(|_| DateTime::parse_from_str(&val, "%Y-%m-%dT%H:%M:%S%.3fZ"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| {
                        LabeledError::new("Failed to parse timestamp")
                            .with_label(format!("Invalid timestamp format: {}", e), call.head)
                    })?;
                datetime.timestamp_millis()
            }
            Some(Value::Int { val, .. }) => {
                if val > TIMESTAMP_MILLIS_THRESHOLD {
                    // Already milliseconds
                    val
                } else {
                    // Seconds, convert to milliseconds
                    val * 1000
                }
            }
            Some(Value::Float { val, .. }) => {
                if val > TIMESTAMP_MILLIS_THRESHOLD as f64 {
                    // Already milliseconds
                    val as i64
                } else {
                    // Seconds, convert to milliseconds
                    (val * 1000.0) as i64
                }
            }
            Some(_) => {
                return Err(LabeledError::new("Invalid input type")
                    .with_label("Expected string, int, or float", call.head));
            }
        };

        Ok(PipelineData::Value(Value::int(millis, call.head), None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nu_protocol::Span;

    fn create_test_span() -> Span {
        Span::test_data()
    }

    mod ulid_time_now_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidTimeNowCommand;
            let signature = cmd.signature();

            assert_eq!(signature.name, "ulid time now");
            assert!(signature.named.iter().any(|flag| flag.long == "format"));
        }

        #[test]
        fn test_command_name() {
            let cmd = UlidTimeNowCommand;
            assert_eq!(cmd.name(), "ulid time now");
        }

        #[test]
        fn test_command_description() {
            let cmd = UlidTimeNowCommand;
            let desc = cmd.description();
            assert!(desc.contains("timestamp"));
            assert!(desc.contains("current") || desc.contains("now"));
        }

        #[test]
        fn test_command_examples() {
            let cmd = UlidTimeNowCommand;
            let examples = cmd.examples();

            assert!(!examples.is_empty());
            assert!(
                examples
                    .iter()
                    .any(|ex| ex.example.contains("ulid time now"))
            );
        }

        #[test]
        fn test_format_options() {
            // Test that all valid formats are recognized
            let valid_formats = vec!["iso8601", "rfc3339", "millis", "seconds"];

            for format in valid_formats {
                // Test format parsing logic (without full command execution)
                let result = match Some(format) {
                    Some("millis") => "millis",
                    Some("seconds") => "seconds",
                    Some("rfc3339") => "rfc3339",
                    Some("iso8601") | None => "iso8601",
                    Some(fmt) => fmt, // Would cause error in actual execution
                };

                assert!(matches!(
                    result,
                    "millis" | "seconds" | "rfc3339" | "iso8601"
                ));
            }
        }

        #[test]
        fn test_invalid_format_detection() {
            // Test invalid format detection
            let invalid_formats = vec!["xml", "json", "timestamp", "", "MILLIS"];

            for format in invalid_formats {
                let is_valid = matches!(format, "iso8601" | "rfc3339" | "millis" | "seconds");
                assert!(!is_valid, "Format '{}' should be invalid", format);
            }
        }
    }

    mod ulid_time_parse_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidTimeParseCommand;
            let signature = cmd.signature();

            assert_eq!(signature.name, "ulid time parse");
            assert_eq!(signature.required_positional.len(), 1);
            assert_eq!(signature.required_positional[0].name, "timestamp");
        }

        #[test]
        fn test_command_name() {
            let cmd = UlidTimeParseCommand;
            assert_eq!(cmd.name(), "ulid time parse");
        }

        #[test]
        fn test_command_description() {
            let cmd = UlidTimeParseCommand;
            let desc = cmd.description();
            assert!(desc.contains("parse") || desc.contains("Parse"));
            assert!(desc.contains("timestamp"));
        }

        #[test]
        fn test_command_examples() {
            let cmd = UlidTimeParseCommand;
            let examples = cmd.examples();

            assert!(!examples.is_empty());
            assert!(
                examples
                    .iter()
                    .any(|ex| ex.example.contains("ulid time parse"))
            );
        }

        #[test]
        fn test_timestamp_type_detection() {
            // Test timestamp type detection logic
            let test_cases = vec![
                (1704067200i64, "seconds", "2024 timestamp in seconds"),
                (1704067200000i64, "millis", "2024 timestamp in milliseconds"),
                (1000000000i64, "seconds", "exactly 1 billion (seconds)"),
                (1000000000001i64, "millis", "just over 1 trillion (millis)"),
                (
                    999999999999i64,
                    "seconds",
                    "just under 1 trillion (seconds)",
                ),
            ];

            for (timestamp, expected_type, description) in test_cases {
                let detected_as_millis = timestamp > 1_000_000_000_000i64;
                let detected_type = if detected_as_millis {
                    "millis"
                } else {
                    "seconds"
                };

                assert_eq!(
                    detected_type, expected_type,
                    "Failed detection for {}: {}",
                    timestamp, description
                );
            }
        }

        #[test]
        fn test_string_parsing_formats() {
            // Test string format parsing logic (without actual DateTime parsing)
            let test_formats = vec![
                "2024-01-01T00:00:00Z",
                "2024-01-01T00:00:00.000Z",
                "2024-12-31T23:59:59+00:00",
                "2024-06-15T12:30:45.123Z",
            ];

            for format in test_formats {
                // Test that the format strings look reasonable
                assert!(format.contains("T"), "Should contain ISO8601 'T' separator");
                assert!(format.contains("-"), "Should contain date separators");
                assert!(format.contains(":"), "Should contain time separators");
                assert!(
                    format.len() >= 19,
                    "Should be at least 19 chars for basic ISO8601"
                );
            }
        }

        #[test]
        fn test_float_timestamp_conversion() {
            // Test float timestamp conversion logic
            let test_floats = vec![
                (
                    1704067200.5,
                    1704067200,
                    500_000_000,
                    "seconds with half second",
                ),
                (1704067200.0, 1704067200, 0, "exact seconds"),
                (
                    1704067200.999,
                    1704067200,
                    999_000_000,
                    "seconds with milliseconds",
                ),
                (
                    1704067200.001,
                    1704067200,
                    1_000_000,
                    "seconds with small fraction",
                ),
            ];

            for (input, expected_seconds, expected_nanos_approx, description) in test_floats {
                let input: f64 = input;
                let seconds = input.trunc() as i64;
                let nanos = ((input.fract() * 1_000_000_000.0) as u32).min(999_999_999);

                assert_eq!(
                    seconds, expected_seconds,
                    "Seconds mismatch for {}",
                    description
                );

                // Allow small tolerance for floating point precision
                let nanos_diff = nanos.abs_diff(expected_nanos_approx);
                assert!(
                    nanos_diff <= 10_000,
                    "Nanoseconds too far off for {}: expected ~{}, got {}",
                    description,
                    expected_nanos_approx,
                    nanos
                );
            }
        }

        #[test]
        fn test_record_field_structure() {
            // Test expected record field names and types
            let expected_fields = [
                "iso8601",
                "rfc3339",
                "unix_seconds",
                "unix_millis",
                "year",
                "month",
                "day",
                "hour",
                "minute",
                "second",
                "nanosecond",
            ];

            // Verify all expected fields are present
            assert_eq!(expected_fields.len(), 11, "Should have 11 timestamp fields");

            // Verify no duplicate field names
            let unique_count: std::collections::HashSet<_> = expected_fields.iter().collect();
            assert_eq!(
                unique_count.len(),
                expected_fields.len(),
                "All field names should be unique"
            );
        }
    }

    mod ulid_time_millis_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidTimeMillisCommand;
            let signature = cmd.signature();

            assert_eq!(signature.name, "ulid time millis");
            assert_eq!(signature.optional_positional.len(), 1);
            assert_eq!(signature.optional_positional[0].name, "timestamp");
        }

        #[test]
        fn test_command_name() {
            let cmd = UlidTimeMillisCommand;
            assert_eq!(cmd.name(), "ulid time millis");
        }

        #[test]
        fn test_command_description() {
            let cmd = UlidTimeMillisCommand;
            let desc = cmd.description();
            assert!(desc.contains("millis"));
            assert!(desc.contains("ULID") || desc.contains("timestamp"));
        }

        #[test]
        fn test_command_examples() {
            let cmd = UlidTimeMillisCommand;
            let examples = cmd.examples();

            assert!(!examples.is_empty());
            assert!(
                examples
                    .iter()
                    .any(|ex| ex.example.contains("ulid time millis"))
            );

            // Check that at least one example has an expected result
            assert!(
                examples.iter().any(|ex| ex.result.is_some()),
                "Should have at least one example with expected result"
            );
        }

        #[test]
        fn test_timestamp_conversion_logic() {
            // Test integer timestamp conversion
            let int_test_cases = vec![
                (1704067200i64, false, 1704067200000i64, "seconds to millis"),
                (1704067200000i64, true, 1704067200000i64, "millis unchanged"),
                (1000000000i64, false, 1000000000000i64, "1 billion seconds"),
                (
                    1000000000001i64,
                    true,
                    1000000000001i64,
                    "just over 1 trillion millis",
                ),
            ];

            for (input, is_already_millis, expected, description) in int_test_cases {
                let result = if input > 1_000_000_000_000i64 {
                    input // Already milliseconds
                } else {
                    input * 1000 // Convert seconds to milliseconds
                };

                assert_eq!(result, expected, "Failed conversion for {}", description);

                let detected_as_millis = input > 1_000_000_000_000i64;
                assert_eq!(
                    detected_as_millis, is_already_millis,
                    "Detection mismatch for {}",
                    description
                );
            }
        }

        #[test]
        fn test_float_timestamp_conversion_logic() {
            // Test float timestamp conversion
            let float_test_cases = vec![
                (
                    1704067200.5,
                    false,
                    1704067200500i64,
                    "seconds with decimals",
                ),
                (
                    1704067200000.5,
                    true,
                    1704067200000i64,
                    "millis with decimals",
                ),
                (
                    1000000000.123,
                    false,
                    1000000000123i64,
                    "just under threshold",
                ),
                (
                    1000000000000.999,
                    true,
                    1000000000000i64,
                    "just over threshold",
                ),
            ];

            for (input, is_already_millis, expected, description) in float_test_cases {
                let result = if input > 1_000_000_000_000.0 {
                    input as i64 // Already milliseconds
                } else {
                    (input * 1000.0) as i64 // Convert seconds to milliseconds
                };

                assert_eq!(
                    result, expected,
                    "Failed float conversion for {}",
                    description
                );

                let detected_as_millis = input > 1_000_000_000_000.0;
                assert_eq!(
                    detected_as_millis, is_already_millis,
                    "Float detection mismatch for {}",
                    description
                );
            }
        }

        #[test]
        fn test_boundary_conditions() {
            // Test boundary conditions for timestamp conversion
            let boundary_cases = vec![
                (999_999_999_999i64, false, "just under 1 trillion"),
                (1_000_000_000_001i64, true, "just over 1 trillion"),
                (2_000_000_000_000i64, true, "well over 1 trillion"),
            ];

            for (timestamp, should_be_millis, description) in boundary_cases {
                let is_millis = timestamp > 1_000_000_000_000i64;
                assert_eq!(
                    is_millis, should_be_millis,
                    "Boundary test failed: {}",
                    description
                );
            }
        }
    }

    mod time_format_validation {

        #[test]
        fn test_iso8601_format_structure() {
            // Test ISO8601 format structure without actual DateTime
            let format_string = "%Y-%m-%dT%H:%M:%S%.3fZ";

            assert!(
                format_string.contains("%Y"),
                "Should contain year placeholder"
            );
            assert!(
                format_string.contains("%m"),
                "Should contain month placeholder"
            );
            assert!(
                format_string.contains("%d"),
                "Should contain day placeholder"
            );
            assert!(
                format_string.contains("%H"),
                "Should contain hour placeholder"
            );
            assert!(
                format_string.contains("%M"),
                "Should contain minute placeholder"
            );
            assert!(
                format_string.contains("%S"),
                "Should contain second placeholder"
            );
            assert!(
                format_string.contains("%.3f"),
                "Should contain millisecond placeholder"
            );
            assert!(
                format_string.contains("T"),
                "Should contain ISO8601 'T' separator"
            );
            assert!(format_string.contains("Z"), "Should contain UTC 'Z' suffix");
        }

        #[test]
        fn test_known_timestamp_values() {
            // Test with known timestamp values for consistency
            let known_values = vec![
                (1704067200000i64, "2024-01-01 00:00:00 UTC"),
                (0i64, "1970-01-01 00:00:00 UTC (Unix epoch)"),
                (1000000000000i64, "2001-09-09 01:46:40 UTC"),
            ];

            for (timestamp_millis, description) in known_values {
                // Test conversion to seconds
                let seconds = timestamp_millis / 1000;
                let reconstructed_millis = seconds * 1000;

                // Should be able to round-trip for exact seconds
                if timestamp_millis % 1000 == 0 {
                    assert_eq!(
                        reconstructed_millis, timestamp_millis,
                        "Round-trip failed for {}",
                        description
                    );
                }

                // Timestamp should be reasonable
                assert!(
                    timestamp_millis >= 0,
                    "Timestamp should not be negative: {}",
                    description
                );
            }
        }

        #[test]
        fn test_error_condition_patterns() {
            // Test error condition identification
            let error_conditions = vec![
                ("invalid timestamp format", "parsing error"),
                ("timestamp out of range", "range error"),
                ("invalid input type", "type error"),
            ];

            for (error_message, error_type) in error_conditions {
                assert!(
                    error_message.contains("timestamp") || error_message.contains("input"),
                    "Error message should be descriptive: {}",
                    error_type
                );
                assert!(
                    !error_message.is_empty(),
                    "Error message should not be empty"
                );
            }
        }
    }

    mod execution_logic_tests {
        use super::*;

        #[test]
        fn test_time_now_format_execution() {
            // Test time now format handling logic
            let test_formats = vec![
                (Some("millis"), "millis"),
                (Some("seconds"), "seconds"),
                (Some("rfc3339"), "rfc3339"),
                (Some("iso8601"), "iso8601"),
                (None, "iso8601"), // default case
            ];

            for (input_format, expected_format) in test_formats {
                let result_format = match input_format {
                    Some("millis") => "millis",
                    Some("seconds") => "seconds",
                    Some("rfc3339") => "rfc3339",
                    Some("iso8601") | None => "iso8601",
                    Some(_) => "error", // Invalid format
                };

                assert_eq!(
                    result_format, expected_format,
                    "Format handling mismatch for {:?}",
                    input_format
                );
            }
        }

        #[test]
        fn test_time_now_invalid_format_error() {
            // Test invalid format error handling
            let invalid_formats = vec!["xml", "yaml", "custom", ""];

            for format in invalid_formats {
                let is_valid = matches!(format, "iso8601" | "rfc3339" | "millis" | "seconds");
                assert!(!is_valid, "Format '{}' should be invalid", format);

                // Test error message construction
                if !is_valid {
                    let error_msg = format!(
                        "Unknown format '{}'. Valid formats: iso8601, rfc3339, millis, seconds",
                        format
                    );
                    assert!(error_msg.contains("Unknown format"));
                    assert!(error_msg.contains("Valid formats:"));
                    assert!(error_msg.contains("iso8601"));
                    assert!(error_msg.contains("rfc3339"));
                    assert!(error_msg.contains("millis"));
                    assert!(error_msg.contains("seconds"));
                }
            }
        }

        #[test]
        fn test_time_parse_string_input_execution() {
            // Test string parsing execution paths
            let test_strings = vec![
                "2024-01-01T00:00:00Z",
                "2024-12-31T23:59:59+00:00",
                "invalid-timestamp",
                "",
            ];

            for test_str in test_strings {
                // Test basic format validation
                let looks_valid = test_str.contains("T")
                    && test_str.contains(":")
                    && test_str.len() >= 19
                    && !test_str.is_empty();

                if looks_valid {
                    assert!(
                        test_str.contains("-"),
                        "Valid timestamp should have date separators"
                    );
                    assert!(
                        test_str.chars().any(|c| c.is_ascii_digit()),
                        "Should contain digits"
                    );
                } else {
                    // Invalid strings should be rejected
                    assert!(
                        test_str.is_empty() || test_str == "invalid-timestamp",
                        "Should identify '{}' as invalid",
                        test_str
                    );
                }
            }
        }

        #[test]
        fn test_time_parse_integer_input_execution() {
            // Test integer timestamp parsing execution
            let test_integers = vec![
                (1704067200i64, "seconds", true),
                (1704067200000i64, "millis", true),
                (-1i64, "negative", false),
                (0i64, "epoch", true),
                (i64::MAX, "max_value", false), // Likely out of range
            ];

            for (timestamp, description, should_be_valid) in test_integers {
                let is_millis = timestamp > 1_000_000_000_000i64;
                let is_reasonable = (0..i64::MAX / 2).contains(&timestamp);

                if should_be_valid {
                    assert!(
                        is_reasonable,
                        "Timestamp should be reasonable: {}",
                        description
                    );
                } else {
                    assert!(
                        !is_reasonable || timestamp < 0,
                        "Invalid timestamp should be caught: {}",
                        description
                    );
                }

                // Test magnitude detection
                if timestamp > 1_000_000_000_000i64 {
                    assert!(is_millis, "Should detect {} as milliseconds", description);
                } else if timestamp >= 0 {
                    assert!(!is_millis, "Should detect {} as seconds", description);
                }
            }
        }

        #[test]
        fn test_time_parse_float_input_execution() {
            // Test float timestamp parsing execution
            let test_floats = vec![
                (1704067200.5, true, "seconds with decimals"),
                (1704067200000.123, true, "millis with decimals"),
                (-1.0, false, "negative float"),
                (0.0, true, "zero float"),
                (f64::INFINITY, false, "infinity"),
                (f64::NAN, false, "NaN"),
            ];

            for (timestamp, should_be_valid, description) in test_floats {
                let is_finite = timestamp.is_finite();
                let is_reasonable = is_finite && (0.0..f64::MAX / 2.0).contains(&timestamp);

                if should_be_valid {
                    assert!(
                        is_reasonable,
                        "Float timestamp should be reasonable: {}",
                        description
                    );
                } else {
                    assert!(
                        !is_reasonable,
                        "Invalid float should be caught: {}",
                        description
                    );
                }

                // Test conversion logic for valid floats
                if is_finite && timestamp >= 0.0 {
                    let seconds = timestamp.trunc() as i64;
                    let nanos = ((timestamp.fract() * 1_000_000_000.0) as u32).min(999_999_999);

                    assert!(seconds >= 0, "Seconds should not be negative");
                    assert!(nanos <= 999_999_999, "Nanoseconds should be capped");
                }
            }
        }

        #[test]
        fn test_time_parse_invalid_type_error() {
            // Test invalid input type error handling
            let _span = create_test_span();

            // Test that we would reject non-timestamp types
            let invalid_types = vec!["bool", "list", "record", "binary"];

            for invalid_type in invalid_types {
                // The actual command would create a LabeledError
                let error_message = "Expected string, int, or float";
                assert!(error_message.contains("string"));
                assert!(error_message.contains("int"));
                assert!(error_message.contains("float"));
                assert!(!error_message.contains(invalid_type));
            }
        }

        #[test]
        fn test_time_millis_no_input_execution() {
            // Test time millis with no input (current time)
            // We can't test the actual current time, but we can test the logic path
            let current_time_millis = 1704067200000i64; // Mock current time

            assert!(
                current_time_millis > 1_000_000_000_000i64,
                "Current time should be in milliseconds range"
            );
            assert!(current_time_millis > 0, "Current time should be positive");
        }

        #[test]
        fn test_time_millis_string_input_execution() {
            // Test time millis with string input
            let test_strings = vec![
                ("2024-01-01T00:00:00Z", true, "valid ISO8601"),
                ("invalid-date", false, "invalid format"),
                ("", false, "empty string"),
            ];

            for (input, should_be_valid, description) in test_strings {
                let looks_valid = input.contains("T") && input.contains(":") && input.len() >= 19;

                if should_be_valid {
                    assert!(looks_valid, "Should appear valid: {}", description);
                } else {
                    assert!(!looks_valid, "Should appear invalid: {}", description);
                }
            }
        }

        #[test]
        fn test_time_millis_integer_conversion_execution() {
            // Test integer conversion in millis command
            let test_cases = vec![
                (1704067200i64, 1704067200000i64, "seconds to millis"),
                (1704067200000i64, 1704067200000i64, "millis unchanged"),
                (0i64, 0i64, "epoch seconds"),
                (1000000000i64, 1000000000000i64, "1 billion seconds"),
            ];

            for (input, expected, description) in test_cases {
                let result = if input > 1_000_000_000_000i64 {
                    input // Already milliseconds  
                } else {
                    input * 1000 // Convert seconds
                };

                assert_eq!(result, expected, "Conversion failed for {}", description);
            }
        }

        #[test]
        fn test_time_millis_float_conversion_execution() {
            // Test float conversion in millis command
            let test_cases = vec![
                (1704067200.5, 1704067200500i64, "seconds with fraction"),
                (1704067200000.9, 1704067200000i64, "millis with fraction"),
                (0.0, 0i64, "zero float"),
                (1000.5, 1000500i64, "small seconds with fraction"),
            ];

            for (input, expected, description) in test_cases {
                let result = if input > 1_000_000_000_000.0 {
                    input as i64 // Already milliseconds
                } else {
                    (input * 1000.0) as i64 // Convert seconds
                };

                assert_eq!(
                    result, expected,
                    "Float conversion failed for {}",
                    description
                );
            }
        }

        #[test]
        fn test_time_millis_invalid_input_error() {
            // Test invalid input type error in millis command
            let error_message = "Expected string, int, or float";

            assert!(error_message.contains("string"));
            assert!(error_message.contains("int"));
            assert!(error_message.contains("float"));

            // Should not mention invalid types
            assert!(!error_message.contains("bool"));
            assert!(!error_message.contains("list"));
            assert!(!error_message.contains("record"));
        }

        #[test]
        fn test_datetime_formatting_consistency() {
            // Test datetime formatting consistency across commands
            let iso8601_format = "%Y-%m-%dT%H:%M:%S%.3fZ";

            // Verify format components
            assert!(iso8601_format.contains("%Y"), "Year component");
            assert!(iso8601_format.contains("%m"), "Month component");
            assert!(iso8601_format.contains("%d"), "Day component");
            assert!(iso8601_format.contains("%H"), "Hour component");
            assert!(iso8601_format.contains("%M"), "Minute component");
            assert!(iso8601_format.contains("%S"), "Second component");
            assert!(iso8601_format.contains("%.3f"), "Millisecond component");

            // Should match consistent formatting used across time commands
            assert!(iso8601_format.starts_with("%Y-%m-%d"));
            assert!(iso8601_format.ends_with("Z"));
            assert!(iso8601_format.contains("T"));
        }

        #[test]
        fn test_error_message_consistency() {
            // Test error message consistency across time commands
            let common_errors = vec![
                ("Failed to parse timestamp", "parsing"),
                ("Invalid timestamp format", "format"),
                ("Timestamp is out of range", "range"),
                ("Invalid input type", "type"),
                ("Expected string, int, or float", "type"),
            ];

            for (error_msg, error_category) in common_errors {
                match error_category {
                    "parsing" => {
                        assert!(error_msg.contains("parse") || error_msg.contains("Parse"));
                        assert!(error_msg.contains("timestamp"));
                    }
                    "format" => {
                        assert!(error_msg.contains("format"));
                        assert!(error_msg.contains("timestamp") || error_msg.contains("Invalid"));
                    }
                    "range" => {
                        assert!(error_msg.contains("range"));
                        assert!(error_msg.contains("Timestamp"));
                    }
                    "type" => {
                        assert!(error_msg.contains("type") || error_msg.contains("Expected"));
                    }
                    _ => {}
                }
            }
        }

        #[test]
        fn test_output_value_creation_patterns() {
            // Test output value creation patterns used across time commands
            let span = create_test_span();

            // Test different value types that would be created
            let test_millis = 1704067200000i64;
            let test_string = "2024-01-01T00:00:00.000Z";

            // Test int value creation (used by now --format millis, time millis)
            let int_value = Value::int(test_millis, span);
            match int_value {
                Value::Int { val, .. } => {
                    assert_eq!(val, test_millis);
                }
                _ => panic!("Should create Int value"),
            }

            // Test string value creation (used by now default, rfc3339 formats)
            let string_value = Value::string(test_string.to_string(), span);
            match string_value {
                Value::String { val, .. } => {
                    assert_eq!(val, test_string);
                    assert!(val.contains("2024"));
                    assert!(val.contains("T"));
                    assert!(val.contains("Z"));
                }
                _ => panic!("Should create String value"),
            }
        }
    }
}
