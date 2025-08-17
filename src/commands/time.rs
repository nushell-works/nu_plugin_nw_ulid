use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Span, SyntaxShape, Type, Value,
};

use crate::UlidPlugin;

pub struct UlidTimeNowCommand;

impl PluginCommand for UlidTimeNowCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid time now"
    }

    fn usage(&self) -> &str {
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
            Some("iso8601") | None => Value::string(now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(), call.head),
            Some(fmt) => {
                return Err(LabeledError::new("Invalid format")
                    .with_label(
                        format!("Unknown format '{}'. Valid formats: iso8601, rfc3339, millis, seconds", fmt),
                        call.head,
                    ))
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

    fn usage(&self) -> &str {
        "Parse a timestamp string or number into various formats"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("timestamp", SyntaxShape::Any, "Timestamp to parse (string, int, or number)")
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
                if val > 1_000_000_000_000i64 {
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
                    .with_label("Expected string, int, or float", call.head))
            }
        };

        let record = Value::record(
            [
                ("iso8601".into(), Value::string(datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(), call.head)),
                ("rfc3339".into(), Value::string(datetime.to_rfc3339(), call.head)),
                ("unix_seconds".into(), Value::int(datetime.timestamp(), call.head)),
                ("unix_millis".into(), Value::int(datetime.timestamp_millis(), call.head)),
                ("year".into(), Value::int(datetime.year() as i64, call.head)),
                ("month".into(), Value::int(datetime.month() as i64, call.head)),
                ("day".into(), Value::int(datetime.day() as i64, call.head)),
                ("hour".into(), Value::int(datetime.hour() as i64, call.head)),
                ("minute".into(), Value::int(datetime.minute() as i64, call.head)),
                ("second".into(), Value::int(datetime.second() as i64, call.head)),
                ("nanosecond".into(), Value::int(datetime.nanosecond() as i64, call.head)),
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

    fn usage(&self) -> &str {
        "Convert various timestamp formats to milliseconds (ULID timestamp format)"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .optional("timestamp", SyntaxShape::Any, "Timestamp to convert (defaults to now)")
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
                if val > 1_000_000_000_000i64 {
                    // Already milliseconds
                    val
                } else {
                    // Seconds, convert to milliseconds
                    val * 1000
                }
            }
            Some(Value::Float { val, .. }) => {
                if val > 1_000_000_000_000.0 {
                    // Already milliseconds
                    val as i64
                } else {
                    // Seconds, convert to milliseconds
                    (val * 1000.0) as i64
                }
            }
            Some(_) => {
                return Err(LabeledError::new("Invalid input type")
                    .with_label("Expected string, int, or float", call.head))
            }
        };

        Ok(PipelineData::Value(Value::int(millis, call.head), None))
    }
}