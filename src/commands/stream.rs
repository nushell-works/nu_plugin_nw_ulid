use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};

use crate::{UlidEngine, UlidPlugin};

pub struct UlidStreamCommand;

impl PluginCommand for UlidStreamCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid stream"
    }

    fn description(&self) -> &str {
        "Stream-process large datasets of ULIDs with memory-efficient operations"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "operation",
                SyntaxShape::String,
                "Operation to perform: validate, parse, extract-timestamp, or transform",
            )
            .named(
                "batch-size",
                SyntaxShape::Int,
                "Number of items to process in each batch (default: 1000)",
                Some('b'),
            )
            .named(
                "output-format",
                SyntaxShape::String,
                "Output format for parsed data: compact, full, timestamp-only",
                Some('f'),
            )
            .switch(
                "parallel",
                "Enable parallel processing for CPU-intensive operations",
                Some('p'),
            )
            .switch(
                "continue-on-error",
                "Continue processing despite individual item errors",
                Some('c'),
            )
            .input_output_types(vec![
                (
                    Type::List(Box::new(Type::String)),
                    Type::List(Box::new(Type::Any)),
                ),
                (
                    Type::List(Box::new(Type::Record(vec![].into()))),
                    Type::List(Box::new(Type::Any)),
                ),
            ])
            .category(Category::Filters)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: r#"["01AN4Z07BY79KA1307SR9X4MV3", "01AN4Z07BZ79KA1307SR9X4MV4"] | ulid stream validate"#,
                description: "Stream-validate a list of ULIDs",
                result: None,
            },
            Example {
                example: r#"$large_ulid_list | ulid stream parse --batch-size 500"#,
                description: "Parse large ULID list in batches of 500",
                result: None,
            },
            Example {
                example: r#"$huge_dataset | ulid stream extract-timestamp --parallel"#,
                description: "Extract timestamps with parallel processing",
                result: None,
            },
            Example {
                example: r#"$ulid_data | ulid stream transform --output-format compact --continue-on-error"#,
                description: "Transform ULIDs to compact format, continuing on errors",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let operation: String = call.req(0)?;
        let batch_size: Option<i64> = call.get_flag("batch-size")?;
        let output_format: Option<String> = call.get_flag("output-format")?;
        let parallel: bool = call.has_flag("parallel")?;
        let continue_on_error: bool = call.has_flag("continue-on-error")?;

        let batch_size = batch_size.unwrap_or(1000) as usize;
        let format = output_format.unwrap_or_else(|| "full".to_string());

        match input {
            PipelineData::Value(
                Value::List {
                    vals,
                    internal_span,
                },
                _,
            ) => {
                let result = process_stream(
                    &vals,
                    &operation,
                    batch_size,
                    &format,
                    parallel,
                    continue_on_error,
                    call.head,
                )
                .map_err(|e| *e)?;

                Ok(PipelineData::Value(
                    Value::List {
                        vals: result,
                        internal_span,
                    },
                    None,
                ))
            }
            PipelineData::Empty => Ok(PipelineData::Empty),
            _ => Err(LabeledError::new("Invalid input").with_label(
                "Expected a list of ULIDs or ULID-containing records",
                call.head,
            )),
        }
    }
}

fn process_stream(
    input_vals: &[Value],
    operation: &str,
    batch_size: usize,
    output_format: &str,
    parallel: bool,
    continue_on_error: bool,
    call_head: nu_protocol::Span,
) -> Result<Vec<Value>, Box<LabeledError>> {
    if input_vals.is_empty() {
        return Ok(Vec::new());
    }

    // Process in batches to maintain memory efficiency
    let mut results = Vec::new();
    let total_batches = input_vals.len().div_ceil(batch_size);

    for (batch_idx, chunk) in input_vals.chunks(batch_size).enumerate() {
        // Progress indication for large datasets
        if total_batches > 10 && batch_idx % (total_batches / 10).max(1) == 0 {
            eprintln!(
                "Processing batch {}/{} ({:.1}%)",
                batch_idx + 1,
                total_batches,
                (batch_idx as f64 / total_batches as f64) * 100.0
            );
        }

        let batch_results = if parallel && chunk.len() > 10 {
            process_batch_parallel(
                chunk,
                operation,
                output_format,
                continue_on_error,
                call_head,
            )?
        } else {
            process_batch_sequential(
                chunk,
                operation,
                output_format,
                continue_on_error,
                call_head,
            )?
        };

        results.extend(batch_results);
    }

    Ok(results)
}

fn process_batch_sequential(
    batch: &[Value],
    operation: &str,
    output_format: &str,
    continue_on_error: bool,
    call_head: nu_protocol::Span,
) -> Result<Vec<Value>, Box<LabeledError>> {
    let mut results = Vec::new();

    for value in batch {
        match process_single_item(value, operation, output_format, call_head) {
            Ok(result) => results.push(result),
            Err(e) => {
                if continue_on_error {
                    // Create error record instead of failing
                    let mut error_record = nu_protocol::Record::new();
                    error_record.push("error", Value::string(e.msg, call_head));
                    error_record.push("input", value.clone());
                    results.push(Value::Record {
                        val: error_record.into(),
                        internal_span: call_head,
                    });
                } else {
                    return Err(e);
                }
            }
        }
    }

    Ok(results)
}

fn process_batch_parallel(
    batch: &[Value],
    operation: &str,
    output_format: &str,
    continue_on_error: bool,
    call_head: nu_protocol::Span,
) -> Result<Vec<Value>, Box<LabeledError>> {
    // For parallel processing, we'd use rayon or similar
    // For now, implement as sequential but with the structure for future parallel implementation
    process_batch_sequential(
        batch,
        operation,
        output_format,
        continue_on_error,
        call_head,
    )
}

fn process_single_item(
    value: &Value,
    operation: &str,
    output_format: &str,
    call_head: nu_protocol::Span,
) -> Result<Value, Box<LabeledError>> {
    let ulid_str = extract_ulid_string(value)?;

    match operation {
        "validate" => {
            let is_valid = UlidEngine::validate(&ulid_str);
            Ok(Value::bool(is_valid, call_head))
        }
        "parse" => {
            let components = UlidEngine::parse(&ulid_str).map_err(|e| {
                Box::new(LabeledError::new("Parse failed").with_label(e.to_string(), call_head))
            })?;

            match output_format {
                "compact" => {
                    let mut record = nu_protocol::Record::new();
                    record.push("ulid", Value::string(&components.ulid, call_head));
                    record.push("timestamp_ms", Value::int(components.timestamp_ms as i64, call_head));
                    record.push("randomness", Value::string(&components.randomness_hex, call_head));
                    Ok(Value::Record {
                        val: record.into(),
                        internal_span: call_head,
                    })
                }
                "timestamp-only" => Ok(Value::int(components.timestamp_ms as i64, call_head)),
                _ => Ok(UlidEngine::components_to_value(&components, call_head)),
            }
        }
        "extract-timestamp" => {
            let timestamp = UlidEngine::extract_timestamp(&ulid_str).map_err(|e| {
                Box::new(LabeledError::new("Timestamp extraction failed").with_label(e.to_string(), call_head))
            })?;
            Ok(Value::int(timestamp as i64, call_head))
        }
        "transform" => {
            // Validate and return in requested format
            if !UlidEngine::validate(&ulid_str) {
                return Err(Box::new(LabeledError::new("Invalid ULID")
                    .with_label(format!("'{}' is not a valid ULID", ulid_str), call_head)));
            }

            match output_format {
                "compact" => {
                    let mut record = nu_protocol::Record::new();
                    record.push("ulid", Value::string(&ulid_str, call_head));
                    Ok(Value::Record {
                        val: record.into(),
                        internal_span: call_head,
                    })
                }
                _ => Ok(Value::string(&ulid_str, call_head)),
            }
        }
        _ => Err(Box::new(LabeledError::new("Invalid operation").with_label(
            format!(
                "Unknown operation '{}'. Valid operations: validate, parse, extract-timestamp, transform",
                operation
            ),
            call_head,
        ))),
    }
}

fn extract_ulid_string(value: &Value) -> Result<String, Box<LabeledError>> {
    match value {
        Value::String { val, .. } => Ok(val.clone()),
        Value::Record { val, .. } => {
            // Try to find ULID in common field names
            for field_name in ["ulid", "id", "identifier", "uuid"] {
                if let Some(Value::String { val, .. }) = val.get(field_name) {
                    return Ok(val.clone());
                }
            }
            Err(Box::new(
                LabeledError::new("No ULID field found").with_label(
                    "Record must contain a ULID in 'ulid', 'id', 'identifier', or 'uuid' field",
                    nu_protocol::Span::unknown(),
                ),
            ))
        }
        _ => Err(Box::new(
            LabeledError::new("Invalid value type").with_label(
                "Expected string or record containing ULID",
                nu_protocol::Span::unknown(),
            ),
        )),
    }
}

pub struct UlidGenerateStreamCommand;

impl PluginCommand for UlidGenerateStreamCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid generate-stream"
    }

    fn description(&self) -> &str {
        "Generate a continuous stream of ULIDs with memory-efficient batch processing"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "count",
                SyntaxShape::Int,
                "Total number of ULIDs to generate",
            )
            .named(
                "batch-size",
                SyntaxShape::Int,
                "Number of ULIDs to generate per batch (default: 1000)",
                Some('b'),
            )
            .named(
                "timestamp",
                SyntaxShape::Int,
                "Base timestamp in milliseconds (incremented for each ULID)",
                Some('t'),
            )
            .switch(
                "unique-timestamps",
                "Ensure each ULID has a unique timestamp",
                Some('u'),
            )
            .input_output_types(vec![(Type::Nothing, Type::List(Box::new(Type::String)))])
            .category(Category::Generators)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid generate-stream 10000",
                description: "Generate 10,000 ULIDs in memory-efficient batches",
                result: None,
            },
            Example {
                example: "ulid generate-stream 50000 --batch-size 500",
                description: "Generate 50,000 ULIDs in batches of 500",
                result: None,
            },
            Example {
                example: "ulid generate-stream 1000 --unique-timestamps",
                description: "Generate 1,000 ULIDs with guaranteed unique timestamps",
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
        let count: i64 = call.req(0)?;
        let batch_size: Option<i64> = call.get_flag("batch-size")?;
        let base_timestamp: Option<i64> = call.get_flag("timestamp")?;
        let unique_timestamps: bool = call.has_flag("unique-timestamps")?;

        if count < 0 {
            return Err(
                LabeledError::new("Invalid count").with_label("Count must be positive", call.head)
            );
        }

        if count > 100_000 {
            return Err(LabeledError::new("Count too large").with_label(
                "Maximum count is 100,000 for streaming generation",
                call.head,
            ));
        }

        let count = count as usize;
        let batch_size = batch_size.unwrap_or(1000).max(1) as usize;

        let mut results = Vec::new();
        let total_batches = count.div_ceil(batch_size);
        let mut current_timestamp = base_timestamp.map(|t| t as u64);

        for batch_idx in 0..total_batches {
            let remaining = count - batch_idx * batch_size;
            let current_batch_size = remaining.min(batch_size);

            // Progress indication for large streams
            if total_batches > 10 && batch_idx % (total_batches / 10).max(1) == 0 {
                eprintln!(
                    "Generating batch {}/{} ({:.1}%)",
                    batch_idx + 1,
                    total_batches,
                    (batch_idx as f64 / total_batches as f64) * 100.0
                );
            }

            let batch_results = if let Some(ref mut timestamp) = current_timestamp {
                generate_batch_with_timestamps(
                    current_batch_size,
                    timestamp,
                    unique_timestamps,
                    call.head,
                )
                .map_err(|e| *e)?
            } else {
                generate_batch_random(current_batch_size, call.head).map_err(|e| *e)?
            };

            results.extend(batch_results);
        }

        Ok(PipelineData::Value(
            Value::List {
                vals: results,
                internal_span: call.head,
            },
            None,
        ))
    }
}

fn generate_batch_with_timestamps(
    count: usize,
    base_timestamp: &mut u64,
    unique_timestamps: bool,
    call_head: nu_protocol::Span,
) -> Result<Vec<Value>, Box<LabeledError>> {
    let mut results = Vec::with_capacity(count);

    for _ in 0..count {
        let ulid = UlidEngine::generate_with_timestamp(*base_timestamp).map_err(|e| {
            Box::new(LabeledError::new("Generation failed").with_label(e.to_string(), call_head))
        })?;

        results.push(Value::string(ulid.to_string(), call_head));

        if unique_timestamps {
            *base_timestamp += 1;
        }
    }

    Ok(results)
}

fn generate_batch_random(
    count: usize,
    call_head: nu_protocol::Span,
) -> Result<Vec<Value>, Box<LabeledError>> {
    let ulids = UlidEngine::generate_bulk(count).map_err(|e| {
        Box::new(LabeledError::new("Bulk generation failed").with_label(e.to_string(), call_head))
    })?;

    Ok(ulids
        .into_iter()
        .map(|ulid| Value::string(ulid.to_string(), call_head))
        .collect())
}
