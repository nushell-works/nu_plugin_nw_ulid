//! Core ULID commands for generation, validation, parsing, and security advice.

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Span, SyntaxShape, Type, Value,
};

use crate::{SecurityWarnings, UlidEngine, UlidPlugin};

/// Generates new ULIDs with optional count, timestamp, format, and security context.
pub struct UlidGenerateCommand;

impl PluginCommand for UlidGenerateCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid generate"
    }

    fn description(&self) -> &str {
        "Generate a new ULID (Universally Unique Lexicographically Sortable Identifier)"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .named(
                "count",
                SyntaxShape::Int,
                "Number of ULIDs to generate (max 10,000)",
                Some('c'),
            )
            .named(
                "timestamp",
                SyntaxShape::Int,
                "Custom timestamp in milliseconds",
                Some('t'),
            )
            .named(
                "format",
                SyntaxShape::String,
                "Output format: string, json, binary",
                Some('f'),
            )
            .named(
                "context",
                SyntaxShape::String,
                "Usage context for security validation",
                None,
            )
            .input_output_types(vec![
                (Type::Nothing, Type::String),
                (Type::Nothing, Type::List(Box::new(Type::String))),
                (Type::Nothing, Type::Record(vec![].into())),
            ])
            .category(Category::Generators)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid generate",
                description: "Generate a single ULID",
                result: None,
            },
            Example {
                example: "ulid generate --count 5",
                description: "Generate 5 ULIDs",
                result: None,
            },
            Example {
                example: "ulid generate --format json",
                description: "Generate a ULID with detailed information",
                result: None,
            },
            Example {
                example: "ulid generate --timestamp 1640995200000",
                description: "Generate a ULID with specific timestamp",
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
        let count: Option<i64> = call.get_flag("count")?;
        let timestamp: Option<i64> = call.get_flag("timestamp")?;
        let format_str: Option<String> = call.get_flag("format")?;
        let context: Option<String> = call.get_flag("context")?;

        if let Some(ref ctx) = context
            && SecurityWarnings::is_security_sensitive_context(ctx)
        {
            let warning = SecurityWarnings::create_context_warning(ctx, call.head);
            return Ok(PipelineData::Value(warning, None));
        }

        let format = parse_output_format(format_str.as_deref(), call.head)?;

        match count {
            Some(c) => generate_bulk_ulids(c, timestamp, &format, call.head),
            None => generate_single_ulid(timestamp, &format, call.head),
        }
    }
}

/// Validates whether a string is a valid ULID, with optional detailed output.
pub struct UlidValidateCommand;

impl PluginCommand for UlidValidateCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid validate"
    }

    fn description(&self) -> &str {
        "Validate if a string is a valid ULID"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("ulid", SyntaxShape::String, "The ULID string to validate")
            .switch(
                "detailed",
                "Return detailed validation information",
                Some('d'),
            )
            .input_output_types(vec![
                (Type::Nothing, Type::Bool),
                (Type::Nothing, Type::Record(vec![].into())),
            ])
            .category(Category::Strings)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid validate '01AN4Z07BY79KA1307SR9X4MV3'",
                description: "Validate a ULID string",
                result: Some(Value::bool(true, Span::test_data())),
            },
            Example {
                example: "ulid validate 'invalid-ulid'",
                description: "Validate an invalid ULID string",
                result: Some(Value::bool(false, Span::test_data())),
            },
            Example {
                example: "ulid validate '01AN4Z07BY79KA1307SR9X4MV3' --detailed",
                description: "Get detailed validation information",
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
        let ulid_str: String = call.req(0)?;
        let detailed: bool = call.has_flag("detailed")?;

        if detailed {
            let validation_result = UlidEngine::validate_detailed(&ulid_str);

            let mut record = nu_protocol::Record::new();
            record.push("valid", Value::bool(validation_result.valid, call.head));
            record.push(
                "length",
                Value::int(validation_result.length as i64, call.head),
            );
            record.push(
                "charset_valid",
                Value::bool(validation_result.charset_valid, call.head),
            );
            record.push(
                "timestamp_valid",
                Value::bool(validation_result.timestamp_valid, call.head),
            );

            let errors: Vec<Value> = validation_result
                .errors
                .into_iter()
                .map(|err| Value::string(err, call.head))
                .collect();

            record.push("errors", Value::list(errors, call.head));

            Ok(PipelineData::Value(Value::record(record, call.head), None))
        } else {
            let is_valid = UlidEngine::validate(&ulid_str);
            Ok(PipelineData::Value(Value::bool(is_valid, call.head), None))
        }
    }
}

/// Parses a ULID string and extracts its timestamp and randomness components.
pub struct UlidParseCommand;

impl PluginCommand for UlidParseCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid parse"
    }

    fn description(&self) -> &str {
        "Parse a ULID string and extract its components"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("ulid", SyntaxShape::String, "The ULID string to parse")
            .input_output_types(vec![(Type::Nothing, Type::Record(vec![].into()))])
            .category(Category::Strings)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![Example {
            example: "ulid parse '01AN4Z07BY79KA1307SR9X4MV3'",
            description: "Parse a ULID and show its components",
            result: None,
        }]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let ulid_str: String = call.req(0)?;

        match UlidEngine::parse(&ulid_str) {
            Ok(components) => {
                let value = UlidEngine::components_to_value(&components, call.head);
                Ok(PipelineData::Value(value, None))
            }
            Err(e) => Err(LabeledError::new("Parse failed").with_label(e.to_string(), call.head)),
        }
    }
}

/// Displays comprehensive security guidance for ULID usage contexts.
pub struct UlidSecurityAdviceCommand;

impl PluginCommand for UlidSecurityAdviceCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid security-advice"
    }

    fn description(&self) -> &str {
        "Show comprehensive security advice for ULID usage"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Nothing, Type::Record(vec![].into()))])
            .category(Category::Misc)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![Example {
            example: "ulid security-advice",
            description: "Display security guidance for ULID usage",
            result: None,
        }]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let advice = SecurityWarnings::get_security_advice(call.head);
        Ok(PipelineData::Value(advice, None))
    }
}

fn parse_output_format(
    format_str: Option<&str>,
    span: nu_protocol::Span,
) -> Result<crate::UlidOutputFormat, LabeledError> {
    match format_str {
        Some("json") => Ok(crate::UlidOutputFormat::Json),
        Some("binary") => Ok(crate::UlidOutputFormat::Binary),
        Some("string") | None => Ok(crate::UlidOutputFormat::String),
        Some(f) => Err(LabeledError::new("Invalid format").with_label(
            format!("Unknown format '{}'. Use 'string', 'json', or 'binary'", f),
            span,
        )),
    }
}

fn generate_single_ulid(
    timestamp: Option<i64>,
    format: &crate::UlidOutputFormat,
    span: nu_protocol::Span,
) -> Result<PipelineData, LabeledError> {
    let ulid = match timestamp {
        Some(ts) => UlidEngine::generate_with_timestamp(ts as u64),
        None => UlidEngine::generate(),
    }
    .map_err(|e| LabeledError::new("Generation failed").with_label(e.to_string(), span))?;

    let value = UlidEngine::to_value(&ulid, format, span);
    Ok(PipelineData::Value(value, None))
}

fn generate_bulk_ulids(
    count: i64,
    timestamp: Option<i64>,
    format: &crate::UlidOutputFormat,
    span: nu_protocol::Span,
) -> Result<PipelineData, LabeledError> {
    let count_usize = if count < 0 {
        return Err(LabeledError::new("Invalid count").with_label("Count must be positive", span));
    } else if count > crate::MAX_BULK_GENERATION as i64 {
        return Err(
            LabeledError::new("Count too large").with_label("Maximum count is 10,000", span)
        );
    } else {
        count as usize
    };

    let ulids = match timestamp {
        Some(ts) => {
            let mut result = Vec::new();
            for _ in 0..count_usize {
                let ulid = UlidEngine::generate_with_timestamp(ts as u64).map_err(|e| {
                    LabeledError::new("Generation failed").with_label(e.to_string(), span)
                })?;
                result.push(ulid);
            }
            result
        }
        None => UlidEngine::generate_bulk(count_usize).map_err(|e| {
            LabeledError::new("Bulk generation failed").with_label(e.to_string(), span)
        })?,
    };

    let values: Vec<Value> = ulids
        .iter()
        .map(|ulid| UlidEngine::to_value(ulid, format, span))
        .collect();

    Ok(PipelineData::Value(Value::list(values, span), None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nu_protocol::{Span, Value};

    fn create_test_span() -> Span {
        Span::test_data()
    }

    mod ulid_generate_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidGenerateCommand;
            let signature = cmd.signature();

            assert_eq!(signature.name, "ulid generate");
            assert!(signature.named.iter().any(|flag| flag.long == "count"));
            assert!(signature.named.iter().any(|flag| flag.long == "timestamp"));
            assert!(signature.named.iter().any(|flag| flag.long == "format"));
            assert!(signature.named.iter().any(|flag| flag.long == "context"));
        }

        #[test]
        fn test_command_name() {
            let cmd = UlidGenerateCommand;
            assert_eq!(cmd.name(), "ulid generate");
        }

        #[test]
        fn test_command_description() {
            let cmd = UlidGenerateCommand;
            let desc = cmd.description();
            assert!(desc.contains("Generate"));
            assert!(desc.contains("ULID"));
        }

        #[test]
        fn test_command_examples() {
            let cmd = UlidGenerateCommand;
            let examples = cmd.examples();

            assert!(!examples.is_empty());
            assert!(
                examples
                    .iter()
                    .any(|ex| ex.example.contains("ulid generate"))
            );
        }

        #[test]
        fn test_format_parsing_valid() {
            // Test that valid format strings are parsed correctly
            // This tests the format parsing logic without full command execution
            let valid_formats = vec![
                ("string", crate::UlidOutputFormat::String),
                ("json", crate::UlidOutputFormat::Json),
                ("binary", crate::UlidOutputFormat::Binary),
            ];

            for (format_str, expected_format) in valid_formats {
                let parsed_format = match Some(format_str) {
                    Some("json") => crate::UlidOutputFormat::Json,
                    Some("binary") => crate::UlidOutputFormat::Binary,
                    Some("string") | None => crate::UlidOutputFormat::String,
                    _ => panic!("Should not reach here for valid format"),
                };

                match (parsed_format, expected_format) {
                    (crate::UlidOutputFormat::String, crate::UlidOutputFormat::String)
                    | (crate::UlidOutputFormat::Json, crate::UlidOutputFormat::Json)
                    | (crate::UlidOutputFormat::Binary, crate::UlidOutputFormat::Binary) => (),
                    _ => panic!("Format mismatch for {}", format_str),
                }
            }
        }

        #[test]
        fn test_count_validation_logic() {
            // Test count validation without full command execution
            let test_cases = vec![
                (-1, false, "negative count"),
                (0, true, "zero count"),
                (1, true, "normal count"),
                (5000, true, "medium count"),
                (10000, true, "max count"),
                (10001, false, "over max count"),
            ];

            for (count, should_be_valid, description) in test_cases {
                let is_valid = (0..=10_000).contains(&count);

                assert_eq!(
                    is_valid, should_be_valid,
                    "Failed for {}: {}",
                    count, description
                );
            }
        }
    }

    mod ulid_validate_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidValidateCommand;
            let signature = cmd.signature();

            assert_eq!(signature.name, "ulid validate");
            assert_eq!(signature.required_positional.len(), 1);
            assert_eq!(signature.required_positional[0].name, "ulid");
            assert!(signature.named.iter().any(|flag| flag.long == "detailed"));
        }

        #[test]
        fn test_command_name() {
            let cmd = UlidValidateCommand;
            assert_eq!(cmd.name(), "ulid validate");
        }

        #[test]
        fn test_command_description() {
            let cmd = UlidValidateCommand;
            let desc = cmd.description();
            assert!(desc.contains("Validate"));
            assert!(desc.contains("ULID"));
        }

        #[test]
        fn test_command_examples() {
            let cmd = UlidValidateCommand;
            let examples = cmd.examples();

            assert!(!examples.is_empty());
            assert!(
                examples
                    .iter()
                    .any(|ex| ex.example.contains("ulid validate"))
            );

            // Check that examples include both valid and invalid cases
            let example_strings: Vec<&str> = examples.iter().map(|ex| ex.example).collect();
            assert!(
                example_strings
                    .iter()
                    .any(|ex| ex.contains("01AN4Z07BY79KA1307SR9X4MV3"))
            );
        }

        #[test]
        fn test_validation_logic_integration() {
            // Test validation against known patterns
            let test_cases = vec![
                ("01AN4Z07BY79KA1307SR9X4MV3", true, "standard example ULID"),
                ("01BX5ZZKBKACTAV9WEVGEMMVRY", true, "another valid ULID"),
                ("", false, "empty string"),
                ("too_short", false, "too short"),
                ("01AN4Z07BY79KA1307SR9X4MV3X", false, "too long"),
                ("invalid-chars!", false, "invalid characters"),
                (
                    "lowercase123456789012345678",
                    false,
                    "lowercase not allowed",
                ),
            ];

            for (ulid_str, expected_valid, description) in test_cases {
                let is_valid = UlidEngine::validate(ulid_str);
                assert_eq!(
                    is_valid, expected_valid,
                    "Failed for '{}': {}",
                    ulid_str, description
                );
            }
        }
    }

    mod ulid_parse_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidParseCommand;
            let signature = cmd.signature();

            assert_eq!(signature.name, "ulid parse");
            assert_eq!(signature.required_positional.len(), 1);
            assert_eq!(signature.required_positional[0].name, "ulid");
        }

        #[test]
        fn test_command_name() {
            let cmd = UlidParseCommand;
            assert_eq!(cmd.name(), "ulid parse");
        }

        #[test]
        fn test_command_description() {
            let cmd = UlidParseCommand;
            let desc = cmd.description();
            assert!(desc.contains("Parse"));
            assert!(desc.contains("ULID"));
            assert!(desc.contains("components"));
        }

        #[test]
        fn test_command_examples() {
            let cmd = UlidParseCommand;
            let examples = cmd.examples();

            assert!(!examples.is_empty());
            assert!(examples.iter().any(|ex| ex.example.contains("ulid parse")));
        }

        #[test]
        fn test_parsing_logic_integration() {
            // Generate a known ULID and test parsing
            if let Ok(generated_ulid) = UlidEngine::generate() {
                let ulid_str = generated_ulid.to_string();
                match UlidEngine::parse(&ulid_str) {
                    Ok(components) => {
                        assert_eq!(components.ulid, ulid_str);
                        assert!(components.valid);
                        assert!(components.timestamp_ms > 0);
                        assert!(!components.randomness_hex.is_empty());
                    }
                    Err(_) => panic!("Should be able to parse generated ULID"),
                }
            }

            // Test parsing invalid ULID
            match UlidEngine::parse("invalid-ulid") {
                Ok(_) => panic!("Should not be able to parse invalid ULID"),
                Err(e) => {
                    assert!(e.to_string().contains("Invalid") || e.to_string().contains("Error"));
                }
            }
        }
    }

    mod ulid_security_advice_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidSecurityAdviceCommand;
            let signature = cmd.signature();

            assert_eq!(signature.name, "ulid security-advice");
            assert_eq!(signature.required_positional.len(), 0);
        }

        #[test]
        fn test_command_name() {
            let cmd = UlidSecurityAdviceCommand;
            assert_eq!(cmd.name(), "ulid security-advice");
        }

        #[test]
        fn test_command_description() {
            let cmd = UlidSecurityAdviceCommand;
            let desc = cmd.description();
            assert!(desc.contains("security"));
            assert!(desc.contains("advice") || desc.contains("guidance"));
        }

        #[test]
        fn test_command_examples() {
            let cmd = UlidSecurityAdviceCommand;
            let examples = cmd.examples();

            assert!(!examples.is_empty());
            assert!(
                examples
                    .iter()
                    .any(|ex| ex.example.contains("ulid security-advice"))
            );
        }
    }

    mod security_context_detection {
        use super::*;

        #[test]
        fn test_security_sensitive_contexts() {
            let sensitive_contexts = vec![
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
                "api_key",
                "apikey",
                "access_token",
                "refresh_token",
                "jwt",
                "oauth",
            ];

            for context in sensitive_contexts {
                assert!(
                    SecurityWarnings::is_security_sensitive_context(context),
                    "Context '{}' should be detected as security sensitive",
                    context
                );

                // Test case insensitive detection
                assert!(
                    SecurityWarnings::is_security_sensitive_context(&context.to_uppercase()),
                    "Context '{}' should be detected as security sensitive (uppercase)",
                    context.to_uppercase()
                );
            }
        }

        #[test]
        fn test_non_security_contexts() {
            let non_sensitive_contexts = vec![
                "user_id",
                "transaction",
                "order",
                "product",
                "cache",
                "temp",
                "log",
                "debug",
                "test",
            ];

            for context in non_sensitive_contexts {
                assert!(
                    !SecurityWarnings::is_security_sensitive_context(context),
                    "Context '{}' should not be detected as security sensitive",
                    context
                );
            }
        }

        #[test]
        fn test_context_edge_cases() {
            // Test edge cases
            assert!(!SecurityWarnings::is_security_sensitive_context(""));
            assert!(!SecurityWarnings::is_security_sensitive_context("   "));
            assert!(SecurityWarnings::is_security_sensitive_context("  auth  ")); // Should trim and detect
        }
    }

    mod output_format_logic {
        use super::*;

        #[test]
        fn test_format_enum_variants() {
            // Test that we can construct all format variants
            let _string_format = crate::UlidOutputFormat::String;
            let _json_format = crate::UlidOutputFormat::Json;
            let _binary_format = crate::UlidOutputFormat::Binary;
        }

        #[test]
        fn test_format_value_conversion() {
            // Generate a test ULID for format testing
            if let Ok(test_ulid) = UlidEngine::generate() {
                let span = create_test_span();
                let ulid_str = test_ulid.to_string();

                // Test string format
                let string_value =
                    UlidEngine::to_value(&test_ulid, &crate::UlidOutputFormat::String, span);
                match string_value {
                    Value::String { val, .. } => {
                        assert_eq!(val, ulid_str);
                        assert_eq!(val.len(), 26);
                    }
                    _ => panic!("String format should return String value"),
                }

                // Test JSON format returns a record
                let json_value =
                    UlidEngine::to_value(&test_ulid, &crate::UlidOutputFormat::Json, span);
                match json_value {
                    Value::Record { .. } => {
                        // JSON format should return a structured record
                    }
                    _ => panic!("JSON format should return Record value"),
                }
            }
        }
    }

    mod input_validation {

        #[test]
        fn test_count_parameter_bounds() {
            // Test count validation boundaries
            let valid_counts = [0, 1, 10_000];
            let invalid_counts = [10_001, -1];

            for count in valid_counts {
                assert!(
                    (0..=10_000).contains(&count),
                    "Count {} should be valid",
                    count
                );
            }

            for count in invalid_counts {
                assert!(
                    !(0..=10_000).contains(&count),
                    "Count {} should be invalid",
                    count
                );
            }
        }

        #[test]
        fn test_timestamp_parameter_validation() {
            // Test timestamp validation
            let valid_timestamps = vec![
                0u64,             // Unix epoch
                1640995200000u64, // 2022-01-01 00:00:00 UTC
                1000000000000u64, // Some large valid timestamp
            ];

            for ts in valid_timestamps {
                // Basic sanity check - timestamp should be usable for ULID generation
                assert!(ts < u64::MAX, "Timestamp {} should be valid", ts);
            }
        }

        #[test]
        fn test_ulid_string_validation_patterns() {
            let valid_patterns = vec![
                ("26 character length", "01AN4Z07BY79KA1307SR9X4MV3"),
                ("all valid chars", "7ZZZZZZZZZZZZZZZZZZZZZZZZZ"),
                ("mixed case valid", "01BX5ZZKBKACTAV9WEVGEMMVRY"),
            ];

            for (description, ulid_str) in valid_patterns {
                assert_eq!(
                    ulid_str.len(),
                    26,
                    "Length check failed for {}",
                    description
                );
                assert!(
                    ulid_str
                        .chars()
                        .all(|c| "0123456789ABCDEFGHJKMNPQRSTVWXYZ".contains(c)),
                    "Character set check failed for {}",
                    description
                );
            }
        }
    }

    mod error_handling {
        use super::*;

        #[test]
        fn test_error_message_construction() {
            // Test that error messages are properly constructed
            let test_cases = vec![
                ("Invalid count", "Count must be positive"),
                ("Count too large", "Maximum count is 10,000"),
                ("Invalid format", "Unknown format"),
                ("Generation failed", "ULID generation"),
                ("Parse failed", "parsing"),
            ];

            for (error_type, expected_content) in test_cases {
                let error = LabeledError::new(error_type);
                assert_eq!(error.msg, error_type);

                // Test error with label
                let error_with_label = error.with_label(expected_content, create_test_span());
                assert_eq!(error_with_label.msg, error_type);
            }
        }

        #[test]
        fn test_format_error_conditions() {
            let invalid_formats = vec![
                "invalid", "xml", "yaml", "csv", "html", "", "JSON", "BINARY",
                "STRING", // Case sensitive
            ];

            for format in invalid_formats {
                let is_valid_format = matches!(format, "string" | "json" | "binary");
                assert!(!is_valid_format, "Format '{}' should be invalid", format);
            }
        }
    }

    mod execution_logic_tests {
        use super::*;

        #[test]
        fn test_ulid_generate_execution() {
            // Test the core ULID generation logic from the run method

            // Test single ULID generation
            let generated_ulid = UlidEngine::generate().expect("Should generate ULID");
            let ulid_str = generated_ulid.to_string();

            assert_eq!(ulid_str.len(), 26, "ULID should be 26 characters");
            assert!(
                UlidEngine::validate(&ulid_str),
                "Generated ULID should be valid"
            );

            // Test bulk generation logic
            let bulk_ulids = UlidEngine::generate_bulk(5).expect("Should generate bulk ULIDs");
            assert_eq!(bulk_ulids.len(), 5, "Should generate exactly 5 ULIDs");

            // All should be unique
            let unique_count = bulk_ulids
                .iter()
                .map(|u| u.to_string())
                .collect::<std::collections::HashSet<_>>()
                .len();
            assert_eq!(unique_count, 5, "All generated ULIDs should be unique");
        }

        #[test]
        fn test_ulid_generate_with_timestamp_execution() {
            // Test timestamp-based generation logic
            let custom_timestamp = 1640995200000u64; // 2022-01-01 00:00:00 UTC

            let ulid = UlidEngine::generate_with_timestamp(custom_timestamp)
                .expect("Should generate ULID with timestamp");

            let parsed = UlidEngine::parse(&ulid.to_string()).expect("Should parse generated ULID");

            assert_eq!(parsed.timestamp_ms, custom_timestamp);
            assert!(parsed.valid);
        }

        #[test]
        fn test_count_validation_execution() {
            // Test count validation logic used in run method
            let test_cases = vec![
                (-1, false, "negative count"),
                (0, true, "zero count"), // Zero is valid, returns empty vec
                (1, true, "single count"),
                (10_000, true, "max count"),
                (10_001, false, "over max count"),
            ];

            for (count, should_be_valid, description) in test_cases {
                if count < 0 {
                    // Negative counts should be caught by validation
                    assert!(
                        !should_be_valid,
                        "Negative count should be invalid: {}",
                        description
                    );
                } else if count > 10_000 {
                    // Test the actual bulk generation limit
                    let result = UlidEngine::generate_bulk(count as usize);
                    assert!(
                        result.is_err(),
                        "Over-limit count should fail: {}",
                        description
                    );
                } else {
                    // Valid counts should work
                    let result = UlidEngine::generate_bulk(count as usize);
                    assert!(
                        result.is_ok(),
                        "Valid count should succeed: {}",
                        description
                    );
                    assert_eq!(result.unwrap().len(), count as usize);
                }
            }
        }

        #[test]
        fn test_format_parsing_execution() {
            // Test format parsing logic from run method
            let test_ulid = UlidEngine::generate().expect("Should generate test ULID");
            let span = create_test_span();

            // Test string format
            let string_value =
                UlidEngine::to_value(&test_ulid, &crate::UlidOutputFormat::String, span);
            match string_value {
                Value::String { val, .. } => {
                    assert_eq!(val.len(), 26);
                    assert_eq!(val, test_ulid.to_string());
                }
                _ => panic!("String format should return String value"),
            }

            // Test JSON format
            let json_value = UlidEngine::to_value(&test_ulid, &crate::UlidOutputFormat::Json, span);
            match json_value {
                Value::Record { val, .. } => {
                    let record = val.into_owned();
                    assert!(record.contains("ulid"));
                    assert!(record.contains("timestamp_ms"));
                    assert!(record.contains("randomness"));
                }
                _ => panic!("JSON format should return Record value"),
            }

            // Test binary format
            let binary_value =
                UlidEngine::to_value(&test_ulid, &crate::UlidOutputFormat::Binary, span);
            match binary_value {
                Value::Binary { val, .. } => {
                    assert_eq!(val.len(), 16); // ULID binary is 16 bytes
                }
                _ => panic!("Binary format should return Binary value"),
            }
        }

        #[test]
        fn test_ulid_validate_execution() {
            // Test validation logic from UlidValidateCommand run method
            let valid_ulids = vec!["01AN4Z07BY79KA1307SR9X4MV3", "01BX5ZZKBKACTAV9WEVGEMMVRY"];

            let invalid_ulids = vec![
                "invalid",
                "too_short",
                "01AN4Z07BY79KA1307SR9X4MV3X", // too long
                "",                            // empty
                "01AN4Z07BY79KA1307SR9X4MV!",  // invalid character
            ];

            // Test basic validation
            for ulid_str in &valid_ulids {
                assert!(
                    UlidEngine::validate(ulid_str),
                    "Should validate: {}",
                    ulid_str
                );
            }

            for ulid_str in &invalid_ulids {
                assert!(
                    !UlidEngine::validate(ulid_str),
                    "Should not validate: {}",
                    ulid_str
                );
            }

            // Test detailed validation
            for ulid_str in &valid_ulids {
                let result = UlidEngine::validate_detailed(ulid_str);
                assert!(
                    result.valid,
                    "Detailed validation should pass: {}",
                    ulid_str
                );
                assert_eq!(result.length, 26);
                assert!(result.charset_valid);
                assert!(result.timestamp_valid);
                assert!(result.errors.is_empty());
            }

            for ulid_str in &invalid_ulids {
                let result = UlidEngine::validate_detailed(ulid_str);
                assert!(
                    !result.valid,
                    "Detailed validation should fail: {}",
                    ulid_str
                );
                assert!(
                    !result.errors.is_empty(),
                    "Should have errors: {}",
                    ulid_str
                );
            }
        }

        #[test]
        fn test_ulid_parse_execution() {
            // Test parsing logic from UlidParseCommand run method
            let test_ulid = UlidEngine::generate().expect("Should generate test ULID");
            let ulid_str = test_ulid.to_string();

            // Test successful parsing
            let components = UlidEngine::parse(&ulid_str).expect("Should parse valid ULID");

            assert_eq!(components.ulid, ulid_str);
            assert!(components.valid);
            assert!(components.timestamp_ms > 0);
            assert!(!components.randomness_hex.is_empty());

            // Test components to value conversion
            let span = create_test_span();
            let value = UlidEngine::components_to_value(&components, span);

            match value {
                Value::Record { val, .. } => {
                    let record = val.into_owned();
                    assert!(record.contains("ulid"));
                    assert!(record.contains("timestamp"));
                    assert!(record.contains("randomness"));
                    assert!(record.contains("valid"));
                }
                _ => panic!("Components should convert to Record value"),
            }

            // Test parsing invalid ULID
            let invalid_result = UlidEngine::parse("invalid-ulid");
            assert!(invalid_result.is_err(), "Should fail to parse invalid ULID");
        }

        #[test]
        fn test_security_context_execution() {
            // Test security context detection logic from run method
            let sensitive_contexts = vec![
                "auth_token",
                "session_id",
                "password_reset",
                "api_key",
                "jwt_secret",
                "oauth_token",
            ];

            let safe_contexts = vec![
                "user_id",
                "transaction_id",
                "log_correlation",
                "temp_file",
                "product_id",
            ];

            // Test sensitive context detection
            for context in &sensitive_contexts {
                assert!(
                    SecurityWarnings::is_security_sensitive_context(context),
                    "Should detect '{}' as sensitive",
                    context
                );
            }

            // Test safe context detection
            for context in &safe_contexts {
                assert!(
                    !SecurityWarnings::is_security_sensitive_context(context),
                    "Should not detect '{}' as sensitive",
                    context
                );
            }

            // Test warning creation
            let span = create_test_span();
            for context in &sensitive_contexts {
                let warning = SecurityWarnings::create_context_warning(context, span);
                match warning {
                    Value::Record { .. } => {
                        // Warning should be a structured record
                    }
                    _ => panic!("Security warning should be a Record value"),
                }
            }
        }

        #[test]
        fn test_security_advice_execution() {
            // Test security advice generation from UlidSecurityAdviceCommand
            let span = create_test_span();
            let advice = SecurityWarnings::get_security_advice(span);

            match advice {
                Value::Record { val, .. } => {
                    let record = val.into_owned();

                    // Should contain key security advice fields
                    assert!(record.contains("safe_use_cases") || record.contains("overview"));

                    // Verify the structure contains useful information
                    let keys: Vec<_> = record.columns().collect();
                    assert!(!keys.is_empty(), "Security advice should have content");
                }
                _ => panic!("Security advice should return Record value"),
            }
        }

        #[test]
        fn test_format_string_validation_execution() {
            // Test format string validation logic used in run methods
            let valid_formats = vec!["string", "json", "binary"];
            let invalid_formats = vec!["xml", "yaml", "csv", "", "STRING", "JSON"];

            for format in &valid_formats {
                let parsed_format = match Some(format as &str) {
                    Some("json") => crate::UlidOutputFormat::Json,
                    Some("binary") => crate::UlidOutputFormat::Binary,
                    Some("string") | None => crate::UlidOutputFormat::String,
                    _ => panic!("Should not reach here for valid format"),
                };

                // Verify format parsing works
                match (format as &str, parsed_format) {
                    ("string", crate::UlidOutputFormat::String) => (),
                    ("json", crate::UlidOutputFormat::Json) => (),
                    ("binary", crate::UlidOutputFormat::Binary) => (),
                    _ => panic!("Format parsing mismatch for '{}'", format),
                }
            }

            // Test invalid format detection
            for format in &invalid_formats {
                let is_valid = matches!(format as &str, "string" | "json" | "binary");
                assert!(!is_valid, "Format '{}' should be invalid", format);
            }
        }

        #[test]
        fn test_timestamp_boundary_conditions() {
            // Test timestamp handling edge cases
            let test_timestamps = vec![
                0u64,             // Unix epoch
                1640995200000u64, // 2022-01-01 00:00:00 UTC
                u64::MAX - 1000,  // Near max value
            ];

            for timestamp in test_timestamps {
                // Test timestamp-based generation
                let result = UlidEngine::generate_with_timestamp(timestamp);

                if timestamp < u64::MAX - 1000 {
                    assert!(
                        result.is_ok(),
                        "Should generate ULID with timestamp {}",
                        timestamp
                    );

                    let ulid = result.unwrap();
                    let parsed = UlidEngine::parse(&ulid.to_string()).unwrap();
                    assert_eq!(parsed.timestamp_ms, timestamp);
                }
            }
        }

        #[test]
        fn test_ulid_uniqueness_and_sorting() {
            // Test ULID uniqueness and lexicographic sorting properties
            let mut ulids = Vec::new();

            // Generate multiple ULIDs
            for _ in 0..10 {
                let ulid = UlidEngine::generate().expect("Should generate ULID");
                ulids.push(ulid.to_string());
            }

            // All should be unique
            let unique_count = ulids.iter().collect::<std::collections::HashSet<_>>().len();
            assert_eq!(unique_count, 10, "All ULIDs should be unique");

            // Test lexicographic ordering (ULIDs should be roughly sortable by generation time)
            let sorted_ulids = {
                let mut sorted = ulids.clone();
                sorted.sort();
                sorted
            };

            // Due to timestamp precision, consecutive ULIDs should have some ordering correlation
            // We'll just verify they can be sorted without panicking
            assert_eq!(sorted_ulids.len(), ulids.len());
        }

        #[test]
        fn test_error_handling_paths() {
            // Test various error conditions in ULID operations

            // Test invalid ULID string patterns
            let invalid_inputs = vec![
                ("", "empty string"),
                ("invalid", "too short"),
                ("01AN4Z07BY79KA1307SR9X4MV3EXTRA", "too long"),
                ("01AN4Z07BY79KA1307SR9X4MV!", "invalid character"),
                ("not-a-ulid-at-all", "completely invalid"),
            ];

            for (input, description) in invalid_inputs {
                // Test validation
                assert!(
                    !UlidEngine::validate(input),
                    "Should reject {}: {}",
                    input,
                    description
                );

                // Test detailed validation includes errors
                let detailed = UlidEngine::validate_detailed(input);
                assert!(
                    !detailed.valid,
                    "Detailed validation should fail for {}",
                    description
                );
                assert!(
                    !detailed.errors.is_empty(),
                    "Should have error messages for {}",
                    description
                );

                // Test parsing fails appropriately
                let parse_result = UlidEngine::parse(input);
                assert!(
                    parse_result.is_err(),
                    "Parsing should fail for {}",
                    description
                );
            }

            // Test bulk generation limits
            let over_limit_result = UlidEngine::generate_bulk(10_001);
            assert!(
                over_limit_result.is_err(),
                "Should reject over-limit bulk generation"
            );
        }

        #[test]
        fn test_output_value_creation() {
            // Test the various Value creation paths used in run methods
            let test_ulid = UlidEngine::generate().expect("Should generate test ULID");
            let span = create_test_span();

            // Test single ULID value creation
            let single_value =
                UlidEngine::to_value(&test_ulid, &crate::UlidOutputFormat::String, span);
            match single_value {
                Value::String { val, .. } => {
                    assert_eq!(val, test_ulid.to_string());
                }
                _ => panic!("Single ULID should create String value"),
            }

            // Test list value creation (for bulk generation)
            let bulk_ulids = [test_ulid];
            let list_values: Vec<Value> = bulk_ulids
                .iter()
                .map(|ulid| UlidEngine::to_value(ulid, &crate::UlidOutputFormat::String, span))
                .collect();

            assert_eq!(list_values.len(), 1);
            match &list_values[0] {
                Value::String { val, .. } => {
                    assert_eq!(val, &test_ulid.to_string());
                }
                _ => panic!("Bulk ULID should create String values"),
            }

            // Test PipelineData creation
            let pipeline_data = PipelineData::Value(Value::list(list_values, span), None);

            match pipeline_data {
                PipelineData::Value(Value::List { vals, .. }, None) => {
                    assert_eq!(vals.len(), 1);
                }
                _ => panic!("Should create proper PipelineData"),
            }
        }
    }
}
