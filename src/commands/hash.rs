use blake3::Hasher as Blake3Hasher;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};
use sha2::{Digest, Sha256, Sha512};

use crate::UlidPlugin;

pub struct UlidHashSha256Command;

impl PluginCommand for UlidHashSha256Command {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid hash sha256"
    }

    fn description(&self) -> &str {
        "Compute SHA-256 hash of data"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("data", SyntaxShape::Any, "Data to hash (string or binary)")
            .switch("binary", "Output as binary instead of hex", Some('b'))
            .input_output_types(vec![
                (Type::String, Type::String),
                (Type::Binary, Type::String),
                (Type::String, Type::Binary),
                (Type::Binary, Type::Binary),
            ])
            .category(Category::Hash)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid hash sha256 'hello world'",
                description: "Hash a string with SHA-256",
                result: None,
            },
            Example {
                example: "ulid hash sha256 'hello world' --binary",
                description: "Hash a string and output as binary",
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
        let binary_output = call.has_flag("binary")?;

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

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hasher.finalize();

        let result = if binary_output {
            Value::binary(hash.to_vec(), call.head)
        } else {
            Value::string(hex::encode(hash), call.head)
        };

        Ok(PipelineData::Value(result, None))
    }
}

pub struct UlidHashSha512Command;

impl PluginCommand for UlidHashSha512Command {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid hash sha512"
    }

    fn description(&self) -> &str {
        "Compute SHA-512 hash of data"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("data", SyntaxShape::Any, "Data to hash (string or binary)")
            .switch("binary", "Output as binary instead of hex", Some('b'))
            .input_output_types(vec![
                (Type::String, Type::String),
                (Type::Binary, Type::String),
                (Type::String, Type::Binary),
                (Type::Binary, Type::Binary),
            ])
            .category(Category::Hash)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid hash sha512 'hello world'",
                description: "Hash a string with SHA-512",
                result: None,
            },
            Example {
                example: "ulid hash sha512 'hello world' --binary",
                description: "Hash a string and output as binary",
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
        let binary_output = call.has_flag("binary")?;

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

        let mut hasher = Sha512::new();
        hasher.update(&data);
        let hash = hasher.finalize();

        let result = if binary_output {
            Value::binary(hash.to_vec(), call.head)
        } else {
            Value::string(hex::encode(hash), call.head)
        };

        Ok(PipelineData::Value(result, None))
    }
}

pub struct UlidHashBlake3Command;

impl PluginCommand for UlidHashBlake3Command {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid hash blake3"
    }

    fn description(&self) -> &str {
        "Compute BLAKE3 hash of data (fast, secure, modern hash function)"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("data", SyntaxShape::Any, "Data to hash (string or binary)")
            .switch("binary", "Output as binary instead of hex", Some('b'))
            .named(
                "length",
                SyntaxShape::Int,
                "Output length in bytes (default: 32)",
                Some('l'),
            )
            .input_output_types(vec![
                (Type::String, Type::String),
                (Type::Binary, Type::String),
                (Type::String, Type::Binary),
                (Type::Binary, Type::Binary),
            ])
            .category(Category::Hash)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid hash blake3 'hello world'",
                description: "Hash a string with BLAKE3",
                result: None,
            },
            Example {
                example: "ulid hash blake3 'hello world' --binary",
                description: "Hash a string and output as binary",
                result: None,
            },
            Example {
                example: "ulid hash blake3 'hello world' --length 16",
                description: "Hash a string with 16-byte output",
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
        let binary_output = call.has_flag("binary")?;
        let length: Option<i64> = call.get_flag("length")?;
        let output_length = length.unwrap_or(32) as usize;

        if output_length == 0 || output_length > 1024 {
            return Err(LabeledError::new("Invalid output length")
                .with_label("Output length must be between 1 and 1024 bytes", call.head));
        }

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

        let mut hasher = Blake3Hasher::new();
        hasher.update(&data);
        let mut hash = vec![0u8; output_length];
        hasher.finalize_xof().fill(&mut hash);

        let result = if binary_output {
            Value::binary(hash, call.head)
        } else {
            Value::string(hex::encode(hash), call.head)
        };

        Ok(PipelineData::Value(result, None))
    }
}

pub struct UlidHashRandomCommand;

impl PluginCommand for UlidHashRandomCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid hash random"
    }

    fn description(&self) -> &str {
        "Generate cryptographically secure random bytes"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .named(
                "length",
                SyntaxShape::Int,
                "Number of random bytes to generate (default: 32)",
                Some('l'),
            )
            .switch("binary", "Output as binary instead of hex", Some('b'))
            .input_output_types(vec![
                (Type::Nothing, Type::String),
                (Type::Nothing, Type::Binary),
            ])
            .category(Category::Random)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid hash random",
                description: "Generate 32 random bytes as hex",
                result: None,
            },
            Example {
                example: "ulid hash random --length 16",
                description: "Generate 16 random bytes as hex",
                result: None,
            },
            Example {
                example: "ulid hash random --binary",
                description: "Generate random bytes as binary",
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
        let length: Option<i64> = call.get_flag("length")?;
        let binary_output = call.has_flag("binary")?;
        let byte_count = length.unwrap_or(32) as usize;

        if byte_count == 0 || byte_count > 1024 {
            return Err(LabeledError::new("Invalid length")
                .with_label("Length must be between 1 and 1024 bytes", call.head));
        }

        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = vec![0u8; byte_count];
        rng.fill_bytes(&mut bytes);

        let result = if binary_output {
            Value::binary(bytes, call.head)
        } else {
            Value::string(hex::encode(bytes), call.head)
        };

        Ok(PipelineData::Value(result, None))
    }
}
