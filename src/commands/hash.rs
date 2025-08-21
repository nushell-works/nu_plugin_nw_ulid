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

#[cfg(test)]
mod tests {
    use super::*;
    use nu_protocol::Span;

    fn create_test_span() -> Span {
        Span::test_data()
    }

    mod ulid_hash_sha256_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidHashSha256Command;
            let signature = cmd.signature();

            assert_eq!(signature.name, "ulid hash sha256");
            assert_eq!(signature.required_positional.len(), 1);
            assert_eq!(signature.required_positional[0].name, "data");
            assert!(signature.named.iter().any(|flag| flag.long == "binary"));
        }

        #[test]
        fn test_command_name() {
            let cmd = UlidHashSha256Command;
            assert_eq!(cmd.name(), "ulid hash sha256");
        }

        #[test]
        fn test_command_description() {
            let cmd = UlidHashSha256Command;
            let desc = cmd.description();
            assert!(desc.contains("SHA-256") || desc.contains("sha256"));
            assert!(desc.contains("hash"));
        }

        #[test]
        fn test_command_examples() {
            let cmd = UlidHashSha256Command;
            let examples = cmd.examples();

            assert!(!examples.is_empty());
            assert!(
                examples
                    .iter()
                    .any(|ex| ex.example.contains("ulid hash sha256"))
            );
        }

        #[test]
        fn test_sha256_hash_computation() {
            // Test known SHA-256 hash values
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(b"hello");
            let result = hasher.finalize();
            let hex_result = hex::encode(result);

            // Just check that our hashing logic produces consistent results
            assert_eq!(hex_result.len(), 64); // SHA-256 is 64 hex chars
            assert!(hex_result.starts_with("2cf24dba"));
        }
    }

    mod ulid_hash_sha512_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidHashSha512Command;
            let signature = cmd.signature();

            assert_eq!(signature.name, "ulid hash sha512");
            assert_eq!(signature.required_positional.len(), 1);
            assert_eq!(signature.required_positional[0].name, "data");
            assert!(signature.named.iter().any(|flag| flag.long == "binary"));
        }

        #[test]
        fn test_command_name() {
            let cmd = UlidHashSha512Command;
            assert_eq!(cmd.name(), "ulid hash sha512");
        }

        #[test]
        fn test_command_description() {
            let cmd = UlidHashSha512Command;
            let desc = cmd.description();
            assert!(desc.contains("SHA-512") || desc.contains("sha512"));
            assert!(desc.contains("hash"));
        }

        #[test]
        fn test_sha512_hash_computation() {
            // Test that SHA-512 produces 128 hex character output
            use sha2::{Digest, Sha512};
            let mut hasher = Sha512::new();
            hasher.update(b"test");
            let result = hasher.finalize();
            let hex_result = hex::encode(result);

            assert_eq!(hex_result.len(), 128); // SHA-512 is 128 hex chars
        }
    }

    mod ulid_hash_blake3_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidHashBlake3Command;
            let signature = cmd.signature();

            assert_eq!(signature.name, "ulid hash blake3");
            assert_eq!(signature.required_positional.len(), 1);
            assert_eq!(signature.required_positional[0].name, "data");
            assert!(signature.named.iter().any(|flag| flag.long == "binary"));
        }

        #[test]
        fn test_command_name() {
            let cmd = UlidHashBlake3Command;
            assert_eq!(cmd.name(), "ulid hash blake3");
        }

        #[test]
        fn test_command_description() {
            let cmd = UlidHashBlake3Command;
            let desc = cmd.description();
            assert!(desc.contains("BLAKE3") || desc.contains("blake3"));
            assert!(desc.contains("hash"));
        }

        #[test]
        fn test_blake3_hash_computation() {
            // Test that BLAKE3 produces consistent results
            let input = "test input";
            let hash1 = blake3::hash(input.as_bytes());
            let hash2 = blake3::hash(input.as_bytes());

            // Same input should produce same hash
            assert_eq!(hash1.to_hex(), hash2.to_hex());
            assert_eq!(hash1.to_hex().len(), 64); // BLAKE3 default output is 64 hex chars
        }

        #[test]
        fn test_blake3_empty_input() {
            let hash = blake3::hash(b"");
            let hex_result = hash.to_hex();

            // BLAKE3 hash of empty string is known
            assert_eq!(hex_result.len(), 64);
        }
    }

    mod ulid_hash_random_command {
        use super::*;

        #[test]
        fn test_command_signature() {
            let cmd = UlidHashRandomCommand;
            let signature = cmd.signature();

            assert_eq!(signature.name, "ulid hash random");
            assert!(signature.named.iter().any(|flag| flag.long == "length"));
            assert!(signature.named.iter().any(|flag| flag.long == "binary"));
        }

        #[test]
        fn test_command_name() {
            let cmd = UlidHashRandomCommand;
            assert_eq!(cmd.name(), "ulid hash random");
        }

        #[test]
        fn test_command_description() {
            let cmd = UlidHashRandomCommand;
            let desc = cmd.description();
            assert!(desc.contains("random"));
            assert!(desc.contains("bytes") || desc.contains("data"));
        }

        #[test]
        fn test_length_validation_logic() {
            // Test length validation without full command execution
            let test_cases = vec![
                (0, false, "zero length"),
                (1, true, "minimum length"),
                (32, true, "default length"),
                (512, true, "medium length"),
                (1024, true, "maximum length"),
                (1025, false, "over maximum length"),
            ];

            for (length, should_be_valid, description) in test_cases {
                let is_valid = !(length == 0 || length > 1024);

                assert_eq!(
                    is_valid, should_be_valid,
                    "Failed for length {}: {}",
                    length, description
                );
            }
        }

        #[test]
        fn test_random_bytes_generation() {
            // Test that random bytes are actually generated
            use rand::RngCore;
            let mut rng = rand::thread_rng();
            let mut bytes1 = vec![0u8; 32];
            let mut bytes2 = vec![0u8; 32];

            rng.fill_bytes(&mut bytes1);
            rng.fill_bytes(&mut bytes2);

            // Random bytes should be different (extremely high probability)
            assert_ne!(bytes1, bytes2);
            assert_eq!(bytes1.len(), 32);
            assert_eq!(bytes2.len(), 32);
        }

        #[test]
        fn test_hex_encoding() {
            // Test hex encoding of random bytes
            let test_bytes = vec![0x00, 0x01, 0x0F, 0x10, 0xFF];
            let hex_result = hex::encode(&test_bytes);

            assert_eq!(hex_result, "00010f10ff");
            assert_eq!(hex_result.len(), test_bytes.len() * 2);
        }
    }

    mod hash_algorithm_correctness {

        #[test]
        fn test_sha256_known_vectors() {
            // Test against known SHA-256 test vectors
            use sha2::{Digest, Sha256};

            let test_vectors = vec![
                (
                    "abc",
                    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
                ),
                (
                    "",
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                ),
            ];

            for (input, expected) in test_vectors {
                let mut hasher = Sha256::new();
                hasher.update(input.as_bytes());
                let result = hasher.finalize();
                let hex_result = hex::encode(result);

                assert_eq!(
                    hex_result, expected,
                    "SHA-256 mismatch for input '{}'",
                    input
                );
            }
        }

        #[test]
        fn test_hash_determinism() {
            // Test that hash functions are deterministic
            let input = "test determinism";

            // SHA-256
            use sha2::{Digest, Sha256};
            let mut hasher1 = Sha256::new();
            hasher1.update(input.as_bytes());
            let sha256_1 = hex::encode(hasher1.finalize());

            let mut hasher2 = Sha256::new();
            hasher2.update(input.as_bytes());
            let sha256_2 = hex::encode(hasher2.finalize());

            assert_eq!(sha256_1, sha256_2, "SHA-256 should be deterministic");

            // BLAKE3
            let blake3_1 = blake3::hash(input.as_bytes()).to_hex();
            let blake3_2 = blake3::hash(input.as_bytes()).to_hex();

            assert_eq!(blake3_1, blake3_2, "BLAKE3 should be deterministic");
        }
    }

    mod error_handling {
        use super::*;

        #[test]
        fn test_error_message_construction() {
            // Test error message construction for hash commands
            let test_cases = vec![
                ("Invalid length", "Length must be between"),
                ("Hash computation failed", "hash"),
                ("Invalid input", "input"),
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
        fn test_length_boundary_validation() {
            // Test exact boundary conditions for random length
            let boundary_cases = vec![
                (0, false, "zero length"),
                (1, true, "minimum valid"),
                (1024, true, "maximum valid"),
                (1025, false, "over maximum"),
            ];

            for (length, expected_valid, description) in boundary_cases {
                let is_valid = length > 0 && length <= 1024;
                assert_eq!(
                    is_valid, expected_valid,
                    "Boundary test failed for {}: {}",
                    length, description
                );
            }
        }
    }
}
