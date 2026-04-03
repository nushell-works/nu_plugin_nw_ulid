# nu_plugin_nw_ulid Developer Guide

This guide is for developers who want to contribute to nu_plugin_nw_ulid or understand its internal architecture.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Development Setup](#development-setup)
3. [Code Structure](#code-structure)
4. [Building and Testing](#building-and-testing)
5. [Plugin Development](#plugin-development)
6. [Contributing Guidelines](#contributing-guidelines)
7. [Performance Optimization](#performance-optimization)
8. [Security Considerations](#security-considerations)

## Architecture Overview

### Design Principles

nu_plugin_nw_ulid follows enterprise-grade development principles:

- **Modular Design**: Each command is implemented as a separate module
- **Security First**: All operations include security validation and context awareness
- **Performance Optimized**: Streaming operations and bulk processing for large datasets
- **Error Resilient**: Comprehensive error handling and graceful degradation
- **Nushell Native**: Full integration with Nushell's type system and pipeline model

### Component Architecture

The plugin entry point is `lib.rs`, which registers the `UlidPlugin` struct and its 23
commands with the Nushell plugin host. Each subcommand lives in its own file under
`commands/`, grouped by domain (see STYLE-0014 for the grouping convention). Core ULID
logic — generation, parsing, validation, and error types — lives in `ulid_engine.rs`.
Security rating and advisory logic lives in `security.rs`.

### Core Components

#### 1. Command Framework (`src/commands/`)

Each command implements the `PluginCommand` trait. See the code example in
[Command Implementation Pattern](#command-implementation-pattern) below.

#### 2. ULID Engine (`src/ulid_engine.rs`)

Core ULID operations — generation, parsing, validation, timestamp/randomness extraction —
are implemented as associated functions on `UlidEngine`. Error types are co-located in the
same file.

#### 3. Error Handling (`src/ulid_engine.rs`)

Engine functions return `UlidError`, keeping the core logic free of `nu-protocol`
dependencies. Commands convert to `LabeledError` at the call boundary (see STYLE-0016):

```rust
#[derive(Debug, Clone)]
pub enum UlidError {
    InvalidFormat { input: String, reason: String },
    InvalidInput { message: String },
    TimestampOutOfRange { timestamp: u64, max_timestamp: u64 },
    GenerationError { reason: String },
}
```

## Development Setup

### Prerequisites

- **Rust 1.89.0+** (required for Nushell 0.109.1)
- **Nushell 0.109.1+**
- **Git** for version control
- **cargo-audit** for security scanning
- **cargo-deny** for dependency management

### Environment Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/nushell-works/nu_plugin_nw_ulid.git
   cd nu_plugin_nw_ulid
   ```

2. **Install development dependencies:**
   ```bash
   cargo install cargo-audit cargo-deny cargo-machete
   ```

3. **Set up pre-commit hooks:**
   ```bash
   # Install pre-commit
   pip install pre-commit
   
   # Install hooks
   pre-commit install
   ```

4. **Verify development environment:**
   ```bash
   # Build
   cargo build
   
   # Run tests
   cargo test
   
   # Check code quality
   cargo clippy
   cargo fmt --check
   
   # Security audit
   cargo audit
   cargo deny check
   ```

### Development Workflow

1. **Create feature branch:**
   ```bash
   git checkout -b feature/new-command
   ```

2. **Implement changes:**
   - Add/modify code
   - Write tests
   - Update documentation

3. **Run quality checks:**
   ```bash
   ./scripts/dev_check.sh
   ```

4. **Commit changes:**
   ```bash
   git add .
   git commit -m "feat: add new ULID command"
   ```

5. **Push and create PR:**
   ```bash
   git push origin feature/new-command
   ```

## Code Structure

### Command Implementation Pattern

Every command follows this structure:

```rust
// src/commands/inspect.rs (simplified)
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};

use crate::{UlidEngine, UlidPlugin};

pub struct UlidInspectCommand;

impl PluginCommand for UlidInspectCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid inspect"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("ulid", SyntaxShape::String, "The ULID to analyze")
            .input_output_types(vec![(Type::Nothing, Type::Record(vec![].into()))])
            .category(Category::Strings)
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let ulid_str: String = call.req(0)?;

        // Engine returns UlidError; convert to LabeledError at the boundary
        let components = UlidEngine::parse(&ulid_str)
            .map_err(|e| LabeledError::new("Parse failed")
                .with_label(e.to_string(), call.head))?;

        let mut record = nu_protocol::Record::new();
        record.push("ulid", Value::string(&components.ulid, call.head));
        record.push("valid", Value::bool(components.valid, call.head));
        // ...
        Ok(Value::record(record, call.head).into_pipeline_data())
    }
}
```

### Testing Pattern

Each command should have comprehensive tests:

```rust
// tests/test_example.rs
use nu_plugin::test_helpers::*;
use nu_protocol::Value;
use crate::commands::UlidExample;

#[test]
fn test_example_single() {
    let command = UlidExample::new();
    let call = test_call();
    let input = Value::nothing(test_span());
    
    let result = command.run(&call, &input).unwrap();
    assert!(matches!(result, Value::String { .. }));
}

#[test]
fn test_example_batch() {
    let command = UlidExample::new();
    let call = test_call();
    let input = Value::list(vec![
        Value::string("test1", test_span()),
        Value::string("test2", test_span()),
    ], test_span());
    
    let result = command.run(&call, &input).unwrap();
    assert!(matches!(result, Value::List { .. }));
}

#[test]
fn test_example_error_handling() {
    let command = UlidExample::new();
    let call = test_call();
    let input = Value::int(42, test_span());
    
    let result = command.run(&call, &input);
    assert!(result.is_err());
}
```

## Building and Testing

### Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check

# Build with all features
cargo build --all-features
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test test_generate

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests

# Run security tests
cargo test --test security_tests

# Run performance tests
cargo test --test performance_tests --release
```

### Benchmarking

```bash
# Run performance benchmarks
cargo bench

# Run specific benchmark
cargo bench ulid_generation

# Compare with baseline
cargo bench -- --save-baseline main
git checkout feature-branch
cargo bench -- --baseline main
```

### Code Quality

```bash
# Lint with Clippy
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Security audit
cargo audit

# Dependency checking
cargo deny check

# Check for unused dependencies
cargo machete
```

## Plugin Development

### Adding New Commands

1. **Create command module:**
   ```bash
   # Create new file: src/commands/new_command.rs
   ```

2. **Implement command structure:**
   ```rust
   // Follow the command pattern shown above
   pub struct UlidNewCommand;
   
   impl UlidNewCommand {
       pub fn new() -> Self { Self }
       pub fn signature(&self) -> PluginSignature { /* ... */ }
       pub fn run(&self, call: &EvaluatedCall, input: &Value) -> Result<Value, LabeledError> { /* ... */ }
   }
   ```

3. **Add to module exports:**
   ```rust
   // src/commands/mod.rs
   pub mod new_command;
   pub use new_command::UlidNewCommand;
   ```

4. **Register in plugin:**
   ```rust
   // src/lib.rs
   impl Plugin for UlidPlugin {
       fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
           vec![
               // existing commands...
               Box::new(UlidNewCommand::new()),
           ]
       }
   }
   ```

5. **Add tests:**
   ```rust
   // tests/test_new_command.rs
   // Add comprehensive tests
   ```

### Extending ULID Functionality

1. **Core functionality goes in `src/ulid_engine.rs`**
2. **Follow security-first principles**
3. **Add comprehensive error handling**
4. **Include performance considerations**
5. **Add security context validation**

### Performance Optimization

#### Streaming Implementation

For large dataset operations:

```rust
fn process_stream(
    &self,
    call: &EvaluatedCall,
    input: &Value,
    batch_size: usize,
    parallel: bool,
) -> Result<Value, LabeledError> {
    match input {
        Value::List { vals, .. } => {
            let results = if parallel {
                self.process_parallel_batches(vals, batch_size)?
            } else {
                self.process_sequential_batches(vals, batch_size)?
            };
            Ok(Value::list(results, call.head))
        }
        _ => Err(/* error */)
    }
}

fn process_sequential_batches(
    &self,
    vals: &[Value],
    batch_size: usize,
) -> Result<Vec<Value>, LabeledError> {
    vals.chunks(batch_size)
        .map(|chunk| self.process_chunk(chunk))
        .collect::<Result<Vec<_>, _>>()
        .map(|batches| batches.into_iter().flatten().collect())
}
```

#### Memory Management

- Use streaming for large datasets
- Implement configurable batch sizes
- Avoid loading entire datasets into memory
- Use iterators instead of collecting intermediate results

## Contributing Guidelines

### Code Style

1. **Follow Rust conventions:**
   - Use `snake_case` for functions and variables
   - Use `PascalCase` for types and structs
   - Use `SCREAMING_SNAKE_CASE` for constants

2. **Documentation:**
   - All public functions must have doc comments
   - Include examples in doc comments
   - Document error conditions

3. **Error Handling:**
   - Use Result types for fallible operations
   - Provide context in error messages
   - Don't panic in library code

4. **Testing:**
   - Unit tests for individual functions
   - Integration tests for command workflows
   - Property-based tests for core algorithms
   - Security tests for attack resistance

### Performance Guidelines

1. **Prefer streaming for large datasets**
2. **Use bulk operations when possible**
3. **Implement configurable batch sizes**
4. **Enable parallel processing for CPU-intensive operations**
5. **Profile performance-critical code**

### Security Guidelines

1. **Validate all inputs**
2. **Use secure randomness for ULID generation**
3. **Implement context-aware security warnings**
4. **Sanitize error messages**
5. **Follow principle of least privilege**

### Documentation Standards

1. **Code comments for complex logic**
2. **API documentation with examples**
3. **User guide updates for new features**
4. **Security documentation for new functionality**
5. **Performance characteristics documentation**

## Performance Optimization

### Profiling

```bash
# CPU profiling
cargo install flamegraph
cargo flamegraph --bin nu_plugin_nw_ulid

# Memory profiling
valgrind --tool=massif target/release/nu_plugin_nw_ulid

# Benchmark profiling
cargo bench -- --profile-time=10
```

### Optimization Techniques

1. **Algorithm Selection:**
   - Use efficient algorithms for Base32 encoding
   - Optimize timestamp extraction
   - Implement fast validation routines

2. **Memory Management:**
   - Minimize allocations in hot paths
   - Use object pooling for frequently allocated objects
   - Implement zero-copy operations where possible

3. **Parallel Processing:**
   - Use Rayon for data parallelism
   - Implement work-stealing for load balancing
   - Consider NUMA topology for large datasets

4. **Caching:**
   - Cache parsed ULID components
   - Use lookup tables for encoding operations
   - Implement result memoization for expensive operations

## Security Considerations

### Security Validation

1. **Input Validation:**
   ```rust
   fn validate_ulid_format(ulid: &str) -> Result<(), UlidError> {
       if ulid.len() != ULID_STRING_LENGTH {
           return Err(UlidError::InvalidFormat {
               input: ulid.to_string(),
               reason: "ULID must be 26 characters".to_string(),
           });
       }
       
       Ok(())
   }
   ```

2. **Cryptographic Security:**
   ```rust
   use rand::{rngs::OsRng, RngCore};
   
   fn generate_secure_randomness() -> [u8; 10] {
       let mut rng = OsRng;
       let mut bytes = [0u8; 10];
       rng.fill_bytes(&mut bytes);
       bytes
   }
   ```

3. **Context Validation:**
   ```rust
   fn check_security_context(context: &str) -> SecurityLevel {
       match context {
           "user-session" => SecurityLevel::High,
           "api-keys" => SecurityLevel::Critical,
           "database-ids" => SecurityLevel::Medium,
           _ => SecurityLevel::Low,
       }
   }
   ```

### Attack Resistance

1. **Timing Attack Prevention:**
   - Use constant-time comparison for sensitive operations
   - Avoid early returns in validation logic
   - Implement rate limiting for repeated operations

2. **Resource Exhaustion Protection:**
   - Implement batch size limits
   - Add timeout protection for long operations
   - Monitor memory usage in streaming operations

3. **Input Sanitization:**
   - Validate all input parameters
   - Escape special characters in error messages
   - Implement length limits for input data

This developer guide provides comprehensive information for contributing to nu_plugin_nw_ulid while maintaining enterprise-grade quality standards.