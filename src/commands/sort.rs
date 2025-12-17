use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};
use std::cmp::Ordering;

use crate::{UlidEngine, UlidPlugin};

pub struct UlidSortCommand;

impl PluginCommand for UlidSortCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid sort"
    }

    fn description(&self) -> &str {
        "Sort data by ULID timestamp order"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .named(
                "column",
                SyntaxShape::String,
                "Column containing ULIDs to sort by",
                Some('c'),
            )
            .switch(
                "reverse",
                "Sort in descending order (newest first)",
                Some('r'),
            )
            .switch(
                "natural",
                "Use natural ULID string sorting instead of timestamp",
                Some('n'),
            )
            .input_output_types(vec![
                (
                    Type::List(Box::new(Type::String)),
                    Type::List(Box::new(Type::String)),
                ),
                (
                    Type::List(Box::new(Type::Record(vec![].into()))),
                    Type::List(Box::new(Type::Record(vec![].into()))),
                ),
            ])
            .category(Category::Filters)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: r#"["01AN4Z07BY79KA1307SR9X4MV3", "01AN4Z07BZ79KA1307SR9X4MV4"] | ulid sort"#,
                description: "Sort a list of ULIDs by timestamp",
                result: None,
            },
            Example {
                example: r#"[{id: "01AN4Z07BZ79KA1307SR9X4MV4", name: "second"}, {id: "01AN4Z07BY79KA1307SR9X4MV3", name: "first"}] | ulid sort --column id"#,
                description: "Sort records by ULID in a specific column",
                result: None,
            },
            Example {
                example: r#"["01AN4Z07BY79KA1307SR9X4MV3", "01AN4Z07BZ79KA1307SR9X4MV4"] | ulid sort --reverse"#,
                description: "Sort ULIDs in descending order (newest first)",
                result: None,
            },
            Example {
                example: r#"["01AN4Z07BY79KA1307SR9X4MV3", "01AN4Z07BZ79KA1307SR9X4MV4"] | ulid sort --natural"#,
                description: "Sort ULIDs using natural string ordering",
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
        let column: Option<String> = call.get_flag("column")?;
        let reverse: bool = call.has_flag("reverse")?;
        let natural: bool = call.has_flag("natural")?;

        match input {
            PipelineData::Value(
                Value::List {
                    vals,
                    internal_span,
                    ..
                },
                _,
            ) => {
                let mut sorted_vals = vals;

                // Sort based on whether we have a column specified
                if let Some(col_name) = column {
                    // Sort records by ULID in specified column
                    sorted_vals.sort_by(|a, b| {
                        compare_records_by_column(a, b, &col_name, natural, reverse)
                    });
                } else {
                    // Sort list of ULID strings directly
                    sorted_vals.sort_by(|a, b| compare_ulid_values(a, b, natural, reverse));
                }

                Ok(PipelineData::Value(
                    Value::list(sorted_vals, internal_span),
                    None,
                ))
            }
            PipelineData::Empty => Ok(PipelineData::Empty),
            _ => Err(LabeledError::new("Invalid input").with_label(
                "Expected a list of ULIDs or records containing ULIDs",
                call.head,
            )),
        }
    }
}

fn compare_records_by_column(
    a: &Value,
    b: &Value,
    column: &str,
    natural: bool,
    reverse: bool,
) -> Ordering {
    let a_ulid = extract_ulid_from_record(a, column);
    let b_ulid = extract_ulid_from_record(b, column);

    match (a_ulid, b_ulid) {
        (Some(a_str), Some(b_str)) => {
            let ordering = compare_ulid_strings(&a_str, &b_str, natural);
            if reverse {
                ordering.reverse()
            } else {
                ordering
            }
        }
        (Some(_), None) => {
            if reverse {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        (None, Some(_)) => {
            if reverse {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (None, None) => Ordering::Equal,
    }
}

fn compare_ulid_values(a: &Value, b: &Value, natural: bool, reverse: bool) -> Ordering {
    let a_str = extract_string_value(a);
    let b_str = extract_string_value(b);

    match (a_str, b_str) {
        (Some(a_ulid), Some(b_ulid)) => {
            let ordering = compare_ulid_strings(&a_ulid, &b_ulid, natural);
            if reverse {
                ordering.reverse()
            } else {
                ordering
            }
        }
        (Some(_), None) => {
            if reverse {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        (None, Some(_)) => {
            if reverse {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (None, None) => Ordering::Equal,
    }
}

fn compare_ulid_strings(a: &str, b: &str, natural: bool) -> Ordering {
    if natural {
        // Natural string comparison - ULIDs are naturally sortable
        a.cmp(b)
    } else {
        // Compare by extracted timestamps
        let a_timestamp = UlidEngine::extract_timestamp(a).unwrap_or(0);
        let b_timestamp = UlidEngine::extract_timestamp(b).unwrap_or(0);

        match a_timestamp.cmp(&b_timestamp) {
            Ordering::Equal => {
                // If timestamps are equal, fall back to string comparison for randomness part
                a.cmp(b)
            }
            other => other,
        }
    }
}

fn extract_ulid_from_record(value: &Value, column: &str) -> Option<String> {
    match value {
        Value::Record { val, .. } => val.get(column).and_then(extract_string_value),
        _ => None,
    }
}

fn extract_string_value(value: &Value) -> Option<String> {
    match value {
        Value::String { val, .. } => Some(val.clone()),
        _ => None,
    }
}

pub struct UlidInspectCommand;

impl PluginCommand for UlidInspectCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid inspect"
    }

    fn description(&self) -> &str {
        "Extract detailed information and metadata from ULIDs"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("ulid", SyntaxShape::String, "The ULID to analyze")
            .switch("compact", "Show compact output format", Some('c'))
            .switch(
                "timestamp-only",
                "Show only timestamp information",
                Some('t'),
            )
            .switch("stats", "Include statistical information", Some('s'))
            .input_output_types(vec![(Type::Nothing, Type::Record(vec![].into()))])
            .category(Category::Strings)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid inspect '01AN4Z07BY79KA1307SR9X4MV3'",
                description: "Get detailed information about a ULID",
                result: None,
            },
            Example {
                example: "ulid inspect '01AN4Z07BY79KA1307SR9X4MV3' --compact",
                description: "Get compact ULID information",
                result: None,
            },
            Example {
                example: "ulid inspect '01AN4Z07BY79KA1307SR9X4MV3' --timestamp-only",
                description: "Show only timestamp information",
                result: None,
            },
            Example {
                example: "ulid inspect '01AN4Z07BY79KA1307SR9X4MV3' --stats",
                description: "Include statistical analysis of the ULID",
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
        let compact: bool = call.has_flag("compact")?;
        let timestamp_only: bool = call.has_flag("timestamp-only")?;
        let stats: bool = call.has_flag("stats")?;

        // Validate ULID first
        if !UlidEngine::validate(&ulid_str) {
            return Err(LabeledError::new("Invalid ULID")
                .with_label(format!("'{}' is not a valid ULID", ulid_str), call.head));
        }

        let components = UlidEngine::parse(&ulid_str)
            .map_err(|e| LabeledError::new("Parse failed").with_label(e.to_string(), call.head))?;

        let mut record = nu_protocol::Record::new();

        // Basic ULID information
        if !timestamp_only {
            record.push("ulid", Value::string(&components.ulid, call.head));
            record.push("valid", Value::bool(components.valid, call.head));
        }

        // Timestamp information
        let timestamp_ms = components.timestamp_ms;
        let timestamp_secs = timestamp_ms / 1000;
        let timestamp_nanos = (timestamp_ms % 1000) * 1_000_000;

        if let Some(datetime) =
            chrono::DateTime::from_timestamp(timestamp_secs as i64, timestamp_nanos as u32)
        {
            if compact {
                record.push(
                    "timestamp",
                    Value::string(
                        datetime.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string(),
                        call.head,
                    ),
                );
            } else {
                let mut ts_record = nu_protocol::Record::new();
                ts_record.push("milliseconds", Value::int(timestamp_ms as i64, call.head));
                ts_record.push("seconds", Value::int(timestamp_secs as i64, call.head));
                ts_record.push(
                    "iso8601",
                    Value::string(
                        datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                        call.head,
                    ),
                );
                ts_record.push("rfc3339", Value::string(datetime.to_rfc3339(), call.head));
                ts_record.push(
                    "human",
                    Value::string(
                        datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                        call.head,
                    ),
                );

                // Add relative time information
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(datetime);
                if duration.num_seconds() > 0 {
                    ts_record.push("age", Value::string(format_duration(duration), call.head));
                } else {
                    ts_record.push("age", Value::string("in the future".to_string(), call.head));
                }

                record.push("timestamp", Value::record(ts_record, call.head));
            }
        }

        // Randomness information (skip if timestamp-only)
        if !timestamp_only {
            if compact {
                record.push(
                    "randomness",
                    Value::string(&components.randomness_hex, call.head),
                );
            } else {
                let mut rand_record = nu_protocol::Record::new();
                rand_record.push("hex", Value::string(&components.randomness_hex, call.head));

                // Convert to different formats
                if let Ok(rand_bytes) = hex::decode(&components.randomness_hex) {
                    rand_record.push("bytes", Value::binary(rand_bytes.clone(), call.head));
                    rand_record.push(
                        "base64",
                        Value::string(
                            base64::Engine::encode(
                                &base64::engine::general_purpose::STANDARD,
                                &rand_bytes,
                            ),
                            call.head,
                        ),
                    );
                }

                record.push("randomness", Value::record(rand_record, call.head));
            }
        }

        // Statistical information (if requested)
        if stats && !timestamp_only {
            let mut stats_record = nu_protocol::Record::new();

            // ULID component analysis
            stats_record.push("timestamp_bits", Value::int(48, call.head));
            stats_record.push("randomness_bits", Value::int(80, call.head));
            stats_record.push("total_bits", Value::int(128, call.head));

            // Entropy analysis (simplified)
            let randomness_entropy = analyze_entropy(&components.randomness_hex);
            stats_record.push(
                "randomness_entropy",
                Value::float(randomness_entropy, call.head),
            );

            // Collision probability (theoretical)
            stats_record.push(
                "collision_probability_per_ms",
                Value::string("~1 in 1.2 × 10^24".to_string(), call.head),
            );

            record.push("statistics", Value::record(stats_record, call.head));
        }

        Ok(PipelineData::Value(Value::record(record, call.head), None))
    }
}

fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < 60 {
        format!("{} seconds ago", total_seconds)
    } else if total_seconds < 3600 {
        let minutes = total_seconds / 60;
        format!("{} minutes ago", minutes)
    } else if total_seconds < 86400 {
        let hours = total_seconds / 3600;
        format!("{} hours ago", hours)
    } else {
        let days = total_seconds / 86400;
        format!("{} days ago", days)
    }
}

fn analyze_entropy(hex_string: &str) -> f64 {
    // Simple entropy calculation based on character frequency
    let mut char_counts = std::collections::HashMap::new();
    let total_chars = hex_string.len() as f64;

    for ch in hex_string.chars() {
        *char_counts.entry(ch).or_insert(0) += 1;
    }

    let mut entropy = 0.0;
    for count in char_counts.values() {
        let probability = *count as f64 / total_chars;
        if probability > 0.0 {
            entropy -= probability * probability.log2();
        }
    }

    entropy
}
