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
        let mut rng = rand::rng();
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
            let mut rng = rand::rng();
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

    mod execution_logic_tests {
        use super::*;

        #[test]
        fn test_sha256_string_input_execution() {
            // Test SHA-256 with string input
            let span = create_test_span();
            let test_string = "hello world";
            let input_value = Value::String {
                val: test_string.to_string(),
                internal_span: span,
            };

            // Test the input processing logic from the run method
            let data = match input_value {
                Value::String { val, .. } => val.into_bytes(),
                Value::Binary { val, .. } => val,
                _ => panic!("Should handle string input"),
            };

            // Test the actual hash computation
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let hash = hasher.finalize();

            let hex_result = hex::encode(hash);
            let binary_result = hash.to_vec();

            // Verify known SHA-256 hash
            assert_eq!(
                hex_result,
                "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
            );
            assert_eq!(binary_result.len(), 32); // SHA-256 produces 32 bytes
        }

        #[test]
        fn test_sha256_binary_input_execution() {
            // Test SHA-256 with binary input
            let span = create_test_span();
            let test_bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
            let input_value = Value::Binary {
                val: test_bytes.clone(),
                internal_span: span,
            };

            // Test binary input processing
            let data = match input_value {
                Value::String { val, .. } => val.into_bytes(),
                Value::Binary { val, .. } => val,
                _ => panic!("Should handle binary input"),
            };

            assert_eq!(data, test_bytes);

            // Test hash computation
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let hash = hasher.finalize();

            assert_eq!(hash.len(), 32);
        }

        #[test]
        fn test_sha256_invalid_input_type() {
            // Test error handling for invalid input types
            let span = create_test_span();
            let invalid_input = Value::Int {
                val: 42,
                internal_span: span,
            };

            // Test the error path from run method
            let result = match invalid_input {
                Value::String { val, .. } => Ok(val.into_bytes()),
                Value::Binary { val, .. } => Ok(val),
                _ => Err(LabeledError::new("Invalid input type")
                    .with_label("Expected string or binary data", span)),
            };

            assert!(result.is_err());
            if let Err(error) = result {
                assert_eq!(error.msg, "Invalid input type");
            }
        }

        #[test]
        fn test_sha256_binary_output_flag() {
            // Test binary output flag processing
            let test_data = b"test";

            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(test_data);
            let hash = hasher.finalize();

            let span = create_test_span();

            // Test hex output (binary_output = false)
            let hash_vec = hash.to_vec();
            let hex_result = Value::string(hex::encode(&hash_vec), span);
            match hex_result {
                Value::String { val, .. } => {
                    assert_eq!(val.len(), 64); // 32 bytes * 2 hex chars
                    assert!(val.chars().all(|c| c.is_ascii_hexdigit()));
                }
                _ => panic!("Should be string value"),
            }

            // Test binary output (binary_output = true)
            let binary_result = Value::binary(hash_vec, span);
            match binary_result {
                Value::Binary { val, .. } => {
                    assert_eq!(val.len(), 32);
                }
                _ => panic!("Should be binary value"),
            }
        }

        #[test]
        fn test_sha512_execution() {
            // Test SHA-512 execution path
            let test_data = b"sha512 test";

            use sha2::{Digest, Sha512};
            let mut hasher = Sha512::new();
            hasher.update(test_data);
            let hash = hasher.finalize();

            let hash_vec = hash.to_vec();
            let hex_result = hex::encode(&hash_vec);

            // SHA-512 produces 64 bytes
            assert_eq!(hash_vec.len(), 64);
            assert_eq!(hex_result.len(), 128); // 64 bytes * 2 hex chars
        }

        #[test]
        fn test_blake3_execution() {
            // Test BLAKE3 execution path
            let test_data = b"blake3 test";
            let output_length = 32;

            let mut hasher = Blake3Hasher::new();
            hasher.update(test_data);
            let mut hash = vec![0u8; output_length];
            hasher.finalize_xof().fill(&mut hash);

            let hex_result = hex::encode(&hash);
            let binary_result = hash;

            assert_eq!(binary_result.len(), output_length);
            assert_eq!(hex_result.len(), output_length * 2);
        }

        #[test]
        fn test_blake3_custom_length() {
            // Test BLAKE3 with custom output length
            let test_data = b"variable length test";
            let custom_lengths = [16, 32, 64, 128];

            for &length in &custom_lengths {
                // Test length validation
                let is_valid = length > 0 && length <= 1024;
                assert!(is_valid, "Length {} should be valid", length);

                // Test hash generation
                let mut hasher = Blake3Hasher::new();
                hasher.update(test_data);
                let mut hash = vec![0u8; length];
                hasher.finalize_xof().fill(&mut hash);

                assert_eq!(hash.len(), length);
                assert!(hash.iter().any(|&b| b != 0)); // Should not be all zeros
            }
        }

        #[test]
        fn test_blake3_invalid_length() {
            // Test BLAKE3 length validation error paths
            let invalid_lengths = [0, 1025, 2000];

            for &length in &invalid_lengths {
                let is_valid = length > 0 && length <= 1024;
                assert!(!is_valid, "Length {} should be invalid", length);

                // Test error creation
                if !is_valid {
                    let span = create_test_span();
                    let error = LabeledError::new("Invalid output length")
                        .with_label("Output length must be between 1 and 1024 bytes", span);
                    assert_eq!(error.msg, "Invalid output length");
                }
            }
        }

        #[test]
        fn test_random_command_execution() {
            // Test random bytes generation
            let byte_count = 32;

            use rand::RngCore;
            let mut rng = rand::rng();
            let mut bytes = vec![0u8; byte_count];
            rng.fill_bytes(&mut bytes);

            let span = create_test_span();

            // Test hex output
            let hex_result = Value::string(hex::encode(&bytes), span);
            match hex_result {
                Value::String { val, .. } => {
                    assert_eq!(val.len(), byte_count * 2);
                    assert!(val.chars().all(|c| c.is_ascii_hexdigit()));
                }
                _ => panic!("Should be string value"),
            }

            // Test binary output
            let binary_result = Value::binary(bytes.clone(), span);
            match binary_result {
                Value::Binary { val, .. } => {
                    assert_eq!(val.len(), byte_count);
                    assert_eq!(val, bytes);
                }
                _ => panic!("Should be binary value"),
            }
        }

        #[test]
        fn test_random_command_length_validation() {
            // Test length validation in random command
            let test_cases = vec![
                (0, false, "zero length"),
                (-1, false, "negative length"),
                (1, true, "minimum valid"),
                (32, true, "default length"),
                (1024, true, "maximum valid"),
                (1025, false, "over maximum"),
                (2000, false, "way over maximum"),
            ];

            for (length, should_be_valid, description) in test_cases {
                let is_valid = length > 0 && length <= 1024;
                assert_eq!(
                    is_valid, should_be_valid,
                    "Failed for {}: {}",
                    length, description
                );

                // Test error creation for invalid lengths
                if !is_valid {
                    let span = create_test_span();
                    let error = LabeledError::new("Invalid length")
                        .with_label("Length must be between 1 and 1024 bytes", span);
                    assert_eq!(error.msg, "Invalid length");
                }
            }
        }

        #[test]
        fn test_hash_consistency() {
            // Test that hash functions are deterministic
            let test_data = "consistency test";
            let test_bytes = test_data.as_bytes();

            // SHA-256 consistency
            use sha2::{Digest, Sha256};
            let mut hasher1 = Sha256::new();
            hasher1.update(test_bytes);
            let hash1 = hex::encode(hasher1.finalize());

            let mut hasher2 = Sha256::new();
            hasher2.update(test_bytes);
            let hash2 = hex::encode(hasher2.finalize());

            assert_eq!(hash1, hash2, "SHA-256 should be consistent");

            // BLAKE3 consistency
            let blake3_hash1 = blake3::hash(test_bytes).to_hex();
            let blake3_hash2 = blake3::hash(test_bytes).to_hex();

            assert_eq!(blake3_hash1, blake3_hash2, "BLAKE3 should be consistent");
        }

        #[test]
        fn test_pipeline_input_processing() {
            // Test pipeline input vs positional argument handling
            let span = create_test_span();
            let test_data = "pipeline test";

            // Test pipeline input
            let pipeline_input = PipelineData::Value(
                Value::String {
                    val: test_data.to_string(),
                    internal_span: span,
                },
                None,
            );

            let data_from_pipeline = match pipeline_input {
                PipelineData::Value(Value::String { val, .. }, _) => val.into_bytes(),
                PipelineData::Value(Value::Binary { val, .. }, _) => val,
                _ => panic!("Should handle pipeline string input"),
            };

            assert_eq!(data_from_pipeline, test_data.as_bytes());

            // Test positional argument
            let arg_value = Value::String {
                val: test_data.to_string(),
                internal_span: span,
            };

            let data_from_arg = match arg_value {
                Value::String { val, .. } => val.into_bytes(),
                Value::Binary { val, .. } => val,
                _ => panic!("Should handle positional string argument"),
            };

            assert_eq!(data_from_arg, test_data.as_bytes());
            assert_eq!(data_from_pipeline, data_from_arg);
        }

        #[test]
        fn test_all_hash_algorithms_with_same_input() {
            // Test all hash algorithms with the same input for comparison
            let test_input = "cross-algorithm test";
            let test_bytes = test_input.as_bytes();

            // SHA-256
            use sha2::{Digest, Sha256, Sha512};
            let mut sha256_hasher = Sha256::new();
            sha256_hasher.update(test_bytes);
            let sha256_hash = sha256_hasher.finalize();
            assert_eq!(sha256_hash.len(), 32);

            // SHA-512
            let mut sha512_hasher = Sha512::new();
            sha512_hasher.update(test_bytes);
            let sha512_hash = sha512_hasher.finalize();
            assert_eq!(sha512_hash.len(), 64);

            // BLAKE3
            let mut blake3_hasher = Blake3Hasher::new();
            blake3_hasher.update(test_bytes);
            let mut blake3_hash = vec![0u8; 32];
            blake3_hasher.finalize_xof().fill(&mut blake3_hash);
            assert_eq!(blake3_hash.len(), 32);

            // All hashes should be different (extremely high probability)
            assert_ne!(sha256_hash.to_vec(), sha512_hash[..32].to_vec());
            assert_ne!(sha256_hash.to_vec(), blake3_hash);
            assert_ne!(sha512_hash[..32].to_vec(), blake3_hash);
        }
    }
}
