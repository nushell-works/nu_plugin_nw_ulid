use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Span, SyntaxShape, Type, Value,
};

use crate::UlidPlugin;

pub struct UlidEncodeBase32Command;

impl PluginCommand for UlidEncodeBase32Command {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid encode base32"
    }

    fn description(&self) -> &str {
        "Encode data to Base32 (Crockford variant, used by ULIDs)"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "data",
                SyntaxShape::Any,
                "Data to encode (string or binary)",
            )
            .input_output_types(vec![
                (Type::String, Type::String),
                (Type::Binary, Type::String),
            ])
            .category(Category::Hash)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid encode base32 'hello world'",
                description: "Encode a string to Base32",
                result: None,
            },
            Example {
                example: "0x48656c6c6f20776f726c64 | ulid encode base32",
                description: "Encode binary data to Base32",
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
        let data = if let Ok(arg) = call.req::<Value>(0) {
            // Using positional argument
            match arg {
                Value::String { val, .. } => val.into_bytes(),
                Value::Binary { val, .. } => val,
                _ => {
                    return Err(LabeledError::new("Invalid input type")
                        .with_label("Expected string or binary data", call.head));
                }
            }
        } else {
            // Using pipeline input
            match input {
                PipelineData::Value(Value::String { val, .. }, _) => val.into_bytes(),
                PipelineData::Value(Value::Binary { val, .. }, _) => val,
                _ => {
                    return Err(LabeledError::new("Invalid input type")
                        .with_label("Expected string or binary data from pipeline", call.head));
                }
            }
        };

        let encoded = base32::encode(base32::Alphabet::Crockford, &data);
        Ok(PipelineData::Value(Value::string(encoded, call.head), None))
    }
}

pub struct UlidDecodeBase32Command;

impl PluginCommand for UlidDecodeBase32Command {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid decode base32"
    }

    fn description(&self) -> &str {
        "Decode Base32 data (Crockford variant, used by ULIDs)"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("data", SyntaxShape::String, "Base32 string to decode")
            .switch("text", "Output as text instead of binary", Some('t'))
            .input_output_types(vec![
                (Type::String, Type::Binary),
                (Type::String, Type::String),
            ])
            .category(Category::Hash)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid decode base32 'CSQPYRK1E8'",
                description: "Decode Base32 to binary",
                result: None,
            },
            Example {
                example: "ulid decode base32 'CSQPYRK1E8' --text",
                description: "Decode Base32 to text",
                result: Some(Value::string("hello", Span::test_data())),
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
        let data: String = call.req(0)?;
        let as_text = call.has_flag("text")?;

        match base32::decode(base32::Alphabet::Crockford, &data) {
            Some(decoded) => {
                let result = if as_text {
                    match String::from_utf8(decoded) {
                        Ok(text) => Value::string(text, call.head),
                        Err(_) => {
                            return Err(LabeledError::new("Invalid UTF-8")
                                .with_label("Decoded data is not valid UTF-8 text", call.head));
                        }
                    }
                } else {
                    Value::binary(decoded, call.head)
                };

                Ok(PipelineData::Value(result, None))
            }
            None => Err(LabeledError::new("Invalid Base32")
                .with_label("Failed to decode Base32 data", call.head)),
        }
    }
}

pub struct UlidEncodeHexCommand;

impl PluginCommand for UlidEncodeHexCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid encode hex"
    }

    fn description(&self) -> &str {
        "Encode data to hexadecimal"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "data",
                SyntaxShape::Any,
                "Data to encode (string or binary)",
            )
            .switch("uppercase", "Use uppercase hex letters", Some('u'))
            .input_output_types(vec![
                (Type::String, Type::String),
                (Type::Binary, Type::String),
            ])
            .category(Category::Hash)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid encode hex 'hello'",
                description: "Encode a string to hex",
                result: Some(Value::string("68656c6c6f", Span::test_data())),
            },
            Example {
                example: "ulid encode hex 'hello' --uppercase",
                description: "Encode a string to uppercase hex",
                result: Some(Value::string("68656C6C6F", Span::test_data())),
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
        let uppercase = call.has_flag("uppercase")?;

        let data = if let Ok(arg) = call.req::<Value>(0) {
            // Using positional argument
            match arg {
                Value::String { val, .. } => val.into_bytes(),
                Value::Binary { val, .. } => val,
                _ => {
                    return Err(LabeledError::new("Invalid input type")
                        .with_label("Expected string or binary data", call.head));
                }
            }
        } else {
            // Using pipeline input
            match input {
                PipelineData::Value(Value::String { val, .. }, _) => val.into_bytes(),
                PipelineData::Value(Value::Binary { val, .. }, _) => val,
                _ => {
                    return Err(LabeledError::new("Invalid input type")
                        .with_label("Expected string or binary data from pipeline", call.head));
                }
            }
        };

        let encoded = if uppercase {
            hex::encode_upper(&data)
        } else {
            hex::encode(&data)
        };

        Ok(PipelineData::Value(Value::string(encoded, call.head), None))
    }
}

pub struct UlidDecodeHexCommand;

impl PluginCommand for UlidDecodeHexCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid decode hex"
    }

    fn description(&self) -> &str {
        "Decode hexadecimal data"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("data", SyntaxShape::String, "Hex string to decode")
            .switch("text", "Output as text instead of binary", Some('t'))
            .input_output_types(vec![
                (Type::String, Type::Binary),
                (Type::String, Type::String),
            ])
            .category(Category::Hash)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid decode hex '68656c6c6f'",
                description: "Decode hex to binary",
                result: None,
            },
            Example {
                example: "ulid decode hex '68656c6c6f' --text",
                description: "Decode hex to text",
                result: Some(Value::string("hello", Span::test_data())),
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
        let data: String = call.req(0)?;
        let as_text = call.has_flag("text")?;

        match hex::decode(&data) {
            Ok(decoded) => {
                let result = if as_text {
                    match String::from_utf8(decoded) {
                        Ok(text) => Value::string(text, call.head),
                        Err(_) => {
                            return Err(LabeledError::new("Invalid UTF-8")
                                .with_label("Decoded data is not valid UTF-8 text", call.head));
                        }
                    }
                } else {
                    Value::binary(decoded, call.head)
                };

                Ok(PipelineData::Value(result, None))
            }
            Err(e) => Err(LabeledError::new("Invalid hex")
                .with_label(format!("Failed to decode hex data: {}", e), call.head)),
        }
    }
}
