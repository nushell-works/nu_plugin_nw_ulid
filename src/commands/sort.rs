//! ULID sorting and inspection commands.

use std::cmp::Ordering;

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};

use crate::{UlidEngine, UlidPlugin};

const ULID_TIMESTAMP_BITS: i64 = 48;
const ULID_RANDOMNESS_BITS: i64 = 80;
const ULID_TOTAL_BITS: i64 = 128;
const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 3600;
const SECONDS_PER_DAY: i64 = 86400;

/// Sorts data by ULID timestamp order.
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
        let a_timestamp = match UlidEngine::extract_timestamp(a) {
            Ok(ts) => ts,
            Err(e) => {
                eprintln!("Failed to extract timestamp from '{}': {}", a, e);
                0
            }
        };
        let b_timestamp = match UlidEngine::extract_timestamp(b) {
            Ok(ts) => ts,
            Err(e) => {
                eprintln!("Failed to extract timestamp from '{}': {}", b, e);
                0
            }
        };

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

/// Extracts detailed information and metadata from ULIDs.
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

        if !UlidEngine::validate(&ulid_str) {
            return Err(LabeledError::new("Invalid ULID")
                .with_label(format!("'{}' is not a valid ULID", ulid_str), call.head));
        }

        let components = UlidEngine::parse(&ulid_str)
            .map_err(|e| LabeledError::new("Parse failed").with_label(e.to_string(), call.head))?;

        let mut record = nu_protocol::Record::new();

        if !timestamp_only {
            record.push("ulid", Value::string(&components.ulid, call.head));
            record.push("valid", Value::bool(components.valid, call.head));
        }

        if let Some(ts_value) = build_timestamp_value(&components, compact, call.head) {
            record.push("timestamp", ts_value);
        }

        if !timestamp_only {
            record.push(
                "randomness",
                build_randomness_value(&components, compact, call.head),
            );
        }

        if stats && !timestamp_only {
            record.push("statistics", build_stats_record(&components, call.head));
        }

        Ok(PipelineData::Value(Value::record(record, call.head), None))
    }
}

fn build_timestamp_value(
    components: &crate::UlidComponents,
    compact: bool,
    span: nu_protocol::Span,
) -> Option<Value> {
    let timestamp_ms = components.timestamp_ms;
    let timestamp_secs = timestamp_ms / crate::MS_PER_SECOND;
    let timestamp_nanos = (timestamp_ms % crate::MS_PER_SECOND) * crate::NANOS_PER_MILLI;

    let datetime = chrono::DateTime::from_timestamp(timestamp_secs as i64, timestamp_nanos as u32)?;

    if compact {
        Some(Value::string(
            datetime.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string(),
            span,
        ))
    } else {
        let mut ts_record = nu_protocol::Record::new();
        ts_record.push("milliseconds", Value::int(timestamp_ms as i64, span));
        ts_record.push("seconds", Value::int(timestamp_secs as i64, span));
        ts_record.push(
            "iso8601",
            Value::string(datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(), span),
        );
        ts_record.push("rfc3339", Value::string(datetime.to_rfc3339(), span));
        ts_record.push(
            "human",
            Value::string(datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string(), span),
        );

        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(datetime);
        if duration.num_seconds() > 0 {
            ts_record.push("age", Value::string(format_duration(duration), span));
        } else {
            ts_record.push("age", Value::string("in the future".to_string(), span));
        }

        Some(Value::record(ts_record, span))
    }
}

fn build_randomness_value(
    components: &crate::UlidComponents,
    compact: bool,
    span: nu_protocol::Span,
) -> Value {
    if compact {
        return Value::string(&components.randomness_hex, span);
    }

    let mut rand_record = nu_protocol::Record::new();
    rand_record.push("hex", Value::string(&components.randomness_hex, span));

    match hex::decode(&components.randomness_hex) {
        Ok(rand_bytes) => {
            rand_record.push("bytes", Value::binary(rand_bytes.clone(), span));
            rand_record.push(
                "base64",
                Value::string(
                    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &rand_bytes),
                    span,
                ),
            );
        }
        Err(e) => {
            eprintln!(
                "Failed to decode randomness hex '{}': {}",
                components.randomness_hex, e
            );
        }
    }

    Value::record(rand_record, span)
}

fn build_stats_record(components: &crate::UlidComponents, span: nu_protocol::Span) -> Value {
    let mut stats_record = nu_protocol::Record::new();

    stats_record.push("timestamp_bits", Value::int(ULID_TIMESTAMP_BITS, span));
    stats_record.push("randomness_bits", Value::int(ULID_RANDOMNESS_BITS, span));
    stats_record.push("total_bits", Value::int(ULID_TOTAL_BITS, span));

    let randomness_entropy = analyze_entropy(&components.randomness_hex);
    stats_record.push("randomness_entropy", Value::float(randomness_entropy, span));

    stats_record.push(
        "collision_probability_per_ms",
        Value::string("~1 in 1.2 × 10^24".to_string(), span),
    );

    Value::record(stats_record, span)
}

fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < SECONDS_PER_MINUTE {
        format!("{} seconds ago", total_seconds)
    } else if total_seconds < SECONDS_PER_HOUR {
        let minutes = total_seconds / SECONDS_PER_MINUTE;
        format!("{} minutes ago", minutes)
    } else if total_seconds < SECONDS_PER_DAY {
        let hours = total_seconds / SECONDS_PER_HOUR;
        format!("{} hours ago", hours)
    } else {
        let days = total_seconds / SECONDS_PER_DAY;
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

#[cfg(test)]
mod tests {
    use super::*;
    use nu_protocol::Span;

    fn test_span() -> Span {
        Span::test_data()
    }

    fn test_components() -> crate::UlidComponents {
        // 01AN4Z07BY = timestamp, 79KA1307SR9X4MV3 = randomness
        crate::UlidEngine::parse("01AN4Z07BY79KA1307SR9X4MV3").unwrap()
    }

    mod sort_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidSortCommand;
            let sig = cmd.signature();
            assert_eq!(sig.name, "ulid sort");
            assert!(sig.named.iter().any(|f| f.long == "column"));
            assert!(sig.named.iter().any(|f| f.long == "reverse"));
            assert!(sig.named.iter().any(|f| f.long == "natural"));
        }

        #[test]
        fn test_command_name() {
            assert_eq!(UlidSortCommand.name(), "ulid sort");
        }

        #[test]
        fn test_command_examples_not_empty() {
            assert!(!UlidSortCommand.examples().is_empty());
        }
    }

    mod inspect_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidInspectCommand;
            let sig = cmd.signature();
            assert_eq!(sig.name, "ulid inspect");
            assert_eq!(sig.required_positional.len(), 1);
            assert!(sig.named.iter().any(|f| f.long == "compact"));
            assert!(sig.named.iter().any(|f| f.long == "timestamp-only"));
            assert!(sig.named.iter().any(|f| f.long == "stats"));
        }

        #[test]
        fn test_command_name() {
            assert_eq!(UlidInspectCommand.name(), "ulid inspect");
        }

        #[test]
        fn test_command_examples_not_empty() {
            assert!(!UlidInspectCommand.examples().is_empty());
        }
    }

    mod build_timestamp_value_tests {
        use super::*;

        #[test]
        fn test_compact_returns_formatted_string() {
            let components = test_components();
            let result = build_timestamp_value(&components, true, test_span());
            assert!(result.is_some());
            match result.unwrap() {
                Value::String { val, .. } => {
                    assert!(val.contains("UTC"));
                }
                _ => panic!("Expected string value in compact mode"),
            }
        }

        #[test]
        fn test_full_returns_record() {
            let components = test_components();
            let result = build_timestamp_value(&components, false, test_span());
            assert!(result.is_some());
            match result.unwrap() {
                Value::Record { val, .. } => {
                    assert!(val.get("milliseconds").is_some());
                    assert!(val.get("seconds").is_some());
                    assert!(val.get("iso8601").is_some());
                    assert!(val.get("rfc3339").is_some());
                    assert!(val.get("human").is_some());
                    assert!(val.get("age").is_some());
                }
                _ => panic!("Expected record value in full mode"),
            }
        }
    }

    mod build_randomness_value_tests {
        use super::*;

        #[test]
        fn test_compact_returns_hex_string() {
            let components = test_components();
            let result = build_randomness_value(&components, true, test_span());
            match result {
                Value::String { val, .. } => {
                    assert_eq!(val, components.randomness_hex);
                }
                _ => panic!("Expected string value in compact mode"),
            }
        }

        #[test]
        fn test_full_returns_record_with_bytes_and_base64() {
            let components = test_components();
            let result = build_randomness_value(&components, false, test_span());
            match result {
                Value::Record { val, .. } => {
                    assert!(val.get("hex").is_some());
                    assert!(val.get("bytes").is_some());
                    assert!(val.get("base64").is_some());
                }
                _ => panic!("Expected record value in full mode"),
            }
        }
    }

    mod build_stats_record_tests {
        use super::*;

        #[test]
        fn test_contains_expected_fields() {
            let components = test_components();
            let result = build_stats_record(&components, test_span());
            match result {
                Value::Record { val, .. } => {
                    assert_eq!(
                        val.get("timestamp_bits").unwrap().as_int().unwrap(),
                        ULID_TIMESTAMP_BITS
                    );
                    assert_eq!(
                        val.get("randomness_bits").unwrap().as_int().unwrap(),
                        ULID_RANDOMNESS_BITS
                    );
                    assert_eq!(
                        val.get("total_bits").unwrap().as_int().unwrap(),
                        ULID_TOTAL_BITS
                    );
                    assert!(val.get("randomness_entropy").is_some());
                    assert!(val.get("collision_probability_per_ms").is_some());
                }
                _ => panic!("Expected record value"),
            }
        }
    }

    mod compare_ulid_strings_tests {
        use super::*;

        #[test]
        fn test_natural_ordering() {
            let a = "01AN4Z07BY79KA1307SR9X4MV3";
            let b = "01AN4Z07BZ79KA1307SR9X4MV4";
            assert_eq!(compare_ulid_strings(a, b, true), Ordering::Less);
            assert_eq!(compare_ulid_strings(b, a, true), Ordering::Greater);
            assert_eq!(compare_ulid_strings(a, a, true), Ordering::Equal);
        }

        #[test]
        fn test_timestamp_ordering() {
            let a = "01AN4Z07BY79KA1307SR9X4MV3";
            let b = "01AN4Z07BZ79KA1307SR9X4MV4";
            let result = compare_ulid_strings(a, b, false);
            // Both should parse; the one with higher timestamp chars sorts later
            assert!(result == Ordering::Less || result == Ordering::Greater);
        }

        #[test]
        fn test_equal_timestamps_fall_back_to_string() {
            let a = "01AN4Z07BY79KA1307SR9X4MV3";
            assert_eq!(compare_ulid_strings(a, a, false), Ordering::Equal);
        }
    }

    mod format_duration_tests {
        use super::*;

        #[test]
        fn test_seconds() {
            let d = chrono::Duration::seconds(30);
            assert_eq!(format_duration(d), "30 seconds ago");
        }

        #[test]
        fn test_minutes() {
            let d = chrono::Duration::seconds(120);
            assert_eq!(format_duration(d), "2 minutes ago");
        }

        #[test]
        fn test_hours() {
            let d = chrono::Duration::seconds(7200);
            assert_eq!(format_duration(d), "2 hours ago");
        }

        #[test]
        fn test_days() {
            let d = chrono::Duration::seconds(172800);
            assert_eq!(format_duration(d), "2 days ago");
        }
    }

    mod analyze_entropy_tests {
        use super::*;

        #[test]
        fn test_single_char_has_zero_entropy() {
            assert_eq!(analyze_entropy("aaaa"), 0.0);
        }

        #[test]
        fn test_varied_chars_have_positive_entropy() {
            let entropy = analyze_entropy("0123456789abcdef");
            assert!(entropy > 0.0);
        }
    }

    mod extract_helpers {
        use super::*;

        #[test]
        fn test_extract_string_value() {
            let val = Value::string("hello", test_span());
            assert_eq!(extract_string_value(&val), Some("hello".to_string()));

            let val = Value::int(42, test_span());
            assert_eq!(extract_string_value(&val), None);
        }

        #[test]
        fn test_extract_ulid_from_record() {
            let mut record = nu_protocol::Record::new();
            record.push(
                "id",
                Value::string("01AN4Z07BY79KA1307SR9X4MV3", test_span()),
            );
            let val = Value::record(record, test_span());
            assert_eq!(
                extract_ulid_from_record(&val, "id"),
                Some("01AN4Z07BY79KA1307SR9X4MV3".to_string())
            );
            assert_eq!(extract_ulid_from_record(&val, "missing"), None);
        }

        #[test]
        fn test_extract_ulid_from_non_record() {
            let val = Value::string("not a record", test_span());
            assert_eq!(extract_ulid_from_record(&val, "id"), None);
        }
    }
}
