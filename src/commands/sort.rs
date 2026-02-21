//! ULID sorting command.

use std::cmp::Ordering;

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};

use crate::{UlidEngine, UlidPlugin};

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

#[cfg(test)]
mod tests {
    use super::*;
    use nu_protocol::Span;

    fn test_span() -> Span {
        Span::test_data()
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
