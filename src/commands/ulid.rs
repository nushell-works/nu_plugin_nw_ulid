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
                ))
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
                                        .with_label(e.to_string(), call.head))
                                }
                            }
                        }
                        result
                    }
                    None => match UlidEngine::generate_bulk(count_usize) {
                        Ok(ulids) => ulids,
                        Err(e) => {
                            return Err(LabeledError::new("Bulk generation failed")
                                .with_label(e.to_string(), call.head))
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
