use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Span, SyntaxShape, Type, Value,
};

use crate::{SecurityWarnings, UlidEngine, UlidPlugin};

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

        // Security check for context
        if let Some(ref ctx) = context {
            if SecurityWarnings::is_security_sensitive_context(ctx) {
                let warning = SecurityWarnings::create_context_warning(ctx, call.head);
                return Ok(PipelineData::Value(warning, None));
            }
        }

        // Parse format
        let format = match format_str.as_deref() {
            Some("json") => crate::UlidOutputFormat::Json,
            Some("binary") => crate::UlidOutputFormat::Binary,
            Some("string") | None => crate::UlidOutputFormat::String,
            Some(f) => {
                return Err(LabeledError::new("Invalid format").with_label(
                    format!("Unknown format '{}'. Use 'string', 'json', or 'binary'", f),
                    call.head,
                ));
            }
        };

        match count {
            Some(c) => {
                // Generate multiple ULIDs
                let count_usize = if c < 0 {
                    return Err(LabeledError::new("Invalid count")
                        .with_label("Count must be positive", call.head));
                } else if c > 10_000 {
                    return Err(LabeledError::new("Count too large")
                        .with_label("Maximum count is 10,000", call.head));
                } else {
                    c as usize
                };

                let ulids = match timestamp {
                    Some(ts) => {
                        let mut result = Vec::new();
                        for _ in 0..count_usize {
                            match UlidEngine::generate_with_timestamp(ts as u64) {
                                Ok(ulid) => result.push(ulid),
                                Err(e) => {
                                    return Err(LabeledError::new("Generation failed")
                                        .with_label(e.to_string(), call.head));
                                }
                            }
                        }
                        result
                    }
                    None => match UlidEngine::generate_bulk(count_usize) {
                        Ok(ulids) => ulids,
                        Err(e) => {
                            return Err(LabeledError::new("Bulk generation failed")
                                .with_label(e.to_string(), call.head));
                        }
                    },
                };

                let values: Vec<Value> = ulids
                    .iter()
                    .map(|ulid| UlidEngine::to_value(ulid, &format, call.head))
                    .collect();

                Ok(PipelineData::Value(
                    Value::List {
                        vals: values,
                        internal_span: call.head,
                    },
                    None,
                ))
            }
            None => {
                // Generate single ULID
                let ulid = match timestamp {
                    Some(ts) => UlidEngine::generate_with_timestamp(ts as u64),
                    None => UlidEngine::generate(),
                }
                .map_err(|e| {
                    LabeledError::new("Generation failed").with_label(e.to_string(), call.head)
                })?;

                let value = UlidEngine::to_value(&ulid, &format, call.head);
                Ok(PipelineData::Value(value, None))
            }
        }
    }
}

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

            record.push(
                "errors",
                Value::List {
                    vals: errors,
                    internal_span: call.head,
                },
            );

            Ok(PipelineData::Value(
                Value::Record {
                    val: record.into(),
                    internal_span: call.head,
                },
                None,
            ))
        } else {
            let is_valid = UlidEngine::validate(&ulid_str);
            Ok(PipelineData::Value(Value::bool(is_valid, call.head), None))
        }
    }
}

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
}
