# Architecture Design

**Phase 2.2: Architecture Design**  
**Date**: August 17, 2025  
**Status**: In Progress - Command Interface Design  

## Executive Summary

This document defines the production architecture for nu_plugin_nw_ulid, transitioning from Phase 1.2 demo commands to a complete ULID implementation. The design prioritizes security-first principles, performance optimization, and seamless Nushell integration based on Phase 2.1 research findings.

## Core Architecture Principles

### 1. **Security-First Design**
- **Prominent warnings** for inappropriate use cases
- **Educational approach** to prevent misuse in authentication contexts
- **Clear guidance** on safe vs unsafe ULID applications

### 2. **Performance-Optimized**
- **Sub-microsecond** generation targets (<100ns achieved: ~31ns)
- **Bulk operations** optimized for streaming and automation
- **Memory-efficient** processing for large datasets

### 3. **Nushell-Native Integration**
- **Structured data** support for ULID components
- **Pipeline-friendly** operations and error handling
- **Type-safe** conversions and validations

## Command Interface Design

### Core ULID Commands

#### 1. **`ulid generate`** - ULID Generation
```nushell
# Basic generation
ulid generate

# Bulk generation with count
ulid generate --count 1000

# Custom timestamp (for historical data)
ulid generate --timestamp "2024-01-01T00:00:00Z"

# Monotonic generation (same millisecond handling)
ulid generate --monotonic --count 10

# Output format options
ulid generate --format json     # Structured output
ulid generate --format string   # Default string format
ulid generate --format binary   # Binary representation
```

**Security Integration**:
```nushell
ulid generate --help
# ⚠️  WARNING: ULIDs are not suitable for security-sensitive use cases
# ✅  Safe for: Database IDs, log correlation, file naming
# ❌  Unsafe for: Auth tokens, session IDs, API keys
# 📖  See: ulid security-advice
```

#### 2. **`ulid parse`** - Component Extraction
```nushell
# Parse into structured components
"01AN4Z07BY79KA1307SR9X4MV3" | ulid parse

# Output format:
{
  ulid: "01AN4Z07BY79KA1307SR9X4MV3",
  timestamp: {
    ms: 1469918176000,
    iso8601: "2016-07-30T23:36:16.000Z",
    unix: 1469918176
  },
  randomness: {
    hex: "79KA1307SR9X4MV3",
    bytes: [121, 75, 161, 48, 112, 83, 169, 216, 77, 86]
  },
  valid: true
}

# Extract specific components
"01AN4Z07BY79KA1307SR9X4MV3" | ulid parse | get timestamp.ms
"01AN4Z07BY79KA1307SR9X4MV3" | ulid parse | get randomness.hex
```

#### 3. **`ulid validate`** - Format Validation
```nushell
# Single validation (returns boolean)
"01AN4Z07BY79KA1307SR9X4MV3" | ulid validate
# true

# Detailed validation (returns structure)
"01AN4Z07BY79KA1307SR9X4MV3" | ulid validate --detailed

# Output format:
{
  valid: true,
  length: 26,
  charset: true,
  timestamp_valid: true,
  errors: []
}

# Batch validation
["01AN4Z07BY79KA1307SR9X4MV3", "invalid", "01BN4Z07BY79KA1307SR9X4MV3"] 
| each { |ulid| { ulid: $ulid, valid: ($ulid | ulid validate) } }
```

#### 4. **`ulid timestamp`** - Timestamp Operations
```nushell
# Extract timestamp from ULID
"01AN4Z07BY79KA1307SR9X4MV3" | ulid timestamp
# 1469918176000

# Convert timestamp to different formats
"01AN4Z07BY79KA1307SR9X4MV3" | ulid timestamp --format iso8601
# "2016-07-30T23:36:16.000Z"

"01AN4Z07BY79KA1307SR9X4MV3" | ulid timestamp --format unix
# 1469918176

# Generate ULID for specific timestamp
"2024-01-01T00:00:00Z" | ulid timestamp generate
# 01HK5YK05C...
```

#### 5. **`ulid random`** - Randomness Operations
```nushell
# Extract randomness component
"01AN4Z07BY79KA1307SR9X4MV3" | ulid random
# "79KA1307SR9X4MV3"

# Get as hex bytes
"01AN4Z07BY79KA1307SR9X4MV3" | ulid random --format hex
# "796b61313037537239583456"

# Get as binary
"01AN4Z07BY79KA1307SR9X4MV3" | ulid random --format binary
# [121, 75, 161, 48, 112, 83, 169, 216, 77, 86]
```

### Utility Commands

#### 6. **`ulid sort`** - Lexicographic Sorting
```nushell
# Sort ULIDs (lexicographic = chronological)
["01BN4Z07BY79KA1307SR9X4MV3", "01AN4Z07BY79KA1307SR9X4MV3"] | ulid sort

# Sort structured data by ULID field
$data | sort-by { |row| $row.ulid_id }

# Validate sort order
$ulids | ulid sort --validate
```

#### 7. **`ulid convert`** - Format Conversions
```nushell
# ULID to UUID (where compatible)
"01AN4Z07BY79KA1307SR9X4MV3" | ulid convert uuid

# UUID to ULID (timestamp extraction where possible)
"550e8400-e29b-41d4-a716-446655440000" | ulid convert ulid --timestamp "2024-01-01T00:00:00Z"

# Base32 encoding operations
"hello world" | encode utf8 | ulid convert base32
"CSQPYRK1E8" | ulid convert from-base32
```

#### 8. **`ulid info`** - Plugin Information & Diagnostics
```nushell
# Plugin metadata (enhanced from demo)
ulid info

# Performance benchmarks
ulid info --bench

# Security guidance
ulid info --security
```

#### 9. **`ulid security-advice`** - Security Education
```nushell
# Security guidance and best practices
ulid security-advice

# Output:
# 🚨 ULID Security Considerations
# 
# ✅ SAFE USE CASES:
# - Database primary keys
# - Log correlation IDs  
# - File/object naming
# - Sortable identifiers
# 
# ❌ UNSAFE USE CASES:
# - Authentication tokens
# - Session identifiers
# - Password reset tokens
# - API keys or secrets
# 
# 🛡️ SECURE ALTERNATIVES:
# - Auth tokens: Use 256-bit random strings
# - Sessions: Use UUID v4 or dedicated generators
# - API keys: Use proper key derivation functions
```

## Data Type Integration

### Nushell Value Types

#### ULID as String (Default)
```nushell
ulid generate
# "01AN4Z07BY79KA1307SR9X4MV3"
```

#### ULID as Structured Record
```nushell
ulid generate --format json
# {
#   ulid: "01AN4Z07BY79KA1307SR9X4MV3",
#   timestamp_ms: 1469918176000,
#   randomness: "79KA1307SR9X4MV3"
# }
```

#### ULID as Binary
```nushell
ulid generate --format binary
# 0x[01, 6A, 2D, 3E, 4D, B0, 79, 6B, 61, 31, 30, 37, 53, 72, 39, 58]
```

### Pipeline Integration Patterns

#### Data Enhancement
```nushell
# Add ULID to records
$customers | each { |customer|
    $customer | insert customer_id { ulid generate }
}

# Add with timestamp preservation
$orders | each { |order|
    $order | insert order_id { 
        ulid generate --timestamp $order.created_at 
    }
}
```

#### Validation Workflows
```nushell
# Filter valid ULIDs
$data | where { |row| $row.ulid_field | ulid validate }

# Report validation errors
$data | each { |row| 
    {
        row_id: $row.id,
        ulid_valid: ($row.ulid_field | ulid validate),
        ulid: $row.ulid_field
    }
} | where ulid_valid == false
```

## Error Handling Strategy

### Error Categories

#### 1. **Format Errors**
```rust
// Invalid ULID format
UlidError::InvalidFormat {
    input: String,
    reason: FormatErrorReason,
    position: Option<usize>,
}

// Reasons: InvalidLength, InvalidCharacter, InvalidTimestamp
```

#### 2. **Generation Errors**
```rust
// Timestamp out of range
UlidError::TimestampOutOfRange {
    timestamp: i64,
    max_timestamp: i64,
}

// Monotonic overflow (too many in same millisecond)
UlidError::MonotonicOverflow {
    timestamp: i64,
    attempts: u32,
}
```

#### 3. **Conversion Errors**
```rust
// UUID conversion not possible
UlidError::ConversionError {
    from_type: String,
    to_type: String,
    reason: String,
}
```

### Nushell Error Integration

#### Graceful Error Messages
```nushell
# Invalid ULID format
"invalid-ulid" | ulid validate
# Error: Invalid ULID format
#   ╭─[<input>:1:1]
#   │
# 1 │ "invalid-ulid" | ulid validate
#   │  ──────┬──────
#   │        ╰── ULID must be exactly 26 characters, got 12
#   ╰────
#   help: Valid ULID format: 01AN4Z07BY79KA1307SR9X4MV3
```

#### Error Recovery Patterns
```nushell
# Try/catch pattern for validation
try {
    $data | each { |row| $row.ulid | ulid validate }
} catch { |err|
    print $"Validation failed: ($err.msg)"
    $data | where { |row| $row.ulid | ulid validate }
}

# Default value pattern
def safe-ulid-parse [ulid_str] {
    try {
        $ulid_str | ulid parse
    } catch {
        { ulid: $ulid_str, valid: false, error: "Invalid format" }
    }
}
```

## Configuration Framework

### Plugin Configuration

#### Settings Structure
```toml
# ~/.config/nushell/plugins/ulid.toml
[ulid]
default_format = "string"          # string|json|binary
timestamp_format = "iso8601"       # iso8601|unix|ms
security_warnings = true           # Show security warnings
bulk_chunk_size = 1000            # Bulk operation chunk size
performance_mode = "balanced"      # fast|balanced|secure

[ulid.generation]
clock_sequence_node = "random"     # For monotonic generation
timestamp_source = "system"       # system|custom
```

#### Runtime Configuration
```nushell
# Configure plugin behavior
ulid config set default_format json
ulid config set security_warnings false
ulid config get

# Temporary overrides
ulid generate --config security_warnings=false
```

### Extensibility Framework

#### Plugin Architecture
```rust
// Core trait for ULID operations
pub trait UlidOperation {
    fn execute(&self, input: &Value, call: &Call, input: &Value) -> Result<Value, ShellError>;
    fn signature(&self) -> Signature;
}

// Command registration system
pub struct UlidPlugin {
    commands: Vec<Box<dyn UlidOperation>>,
    config: UlidConfig,
}
```

#### Future Extension Points
- Custom timestamp sources
- Alternative encoding schemes  
- Integration with external ID systems
- Custom validation rules
- Performance monitoring hooks

## Implementation Strategy

### Phase 3.1: Core Engine
1. **ULID generation** using `ulid` crate v1.1.3
2. **Component extraction** and parsing
3. **Validation** with detailed error reporting
4. **Security warnings** integrated into help system

### Phase 3.2: Command Implementation
1. **Core commands**: generate, parse, validate, timestamp, random
2. **Utility commands**: sort, convert, info, security-advice
3. **Error handling** with user-friendly messages
4. **Help system** with security education

### Phase 3.3: Advanced Features
1. **Bulk operations** optimized for performance
2. **Pipeline integration** patterns and examples
3. **Configuration** system and user preferences
4. **Performance monitoring** and benchmarking

### Phase 3.4: Scripting API
1. **Automation helpers** and template functions
2. **Error recovery** patterns for production scripts
3. **Performance utilities** for bulk processing
4. **Integration examples** and user cookbook

## Performance Targets

### Generation Performance
- **Single ULID**: <100ns (achieved: ~31ns)
- **Bulk 1K ULIDs**: <100μs (achieved: ~31μs)  
- **Bulk 10K ULIDs**: <1ms (projected: ~310μs)

### Memory Usage
- **Single ULID**: 16 bytes (128-bit) + 26 bytes string
- **Bulk generation**: O(n) linear scaling
- **Target**: <1MB for 10K ULIDs in memory

### Validation Performance
- **Parse ULID**: <50ns (achieved: ~13ns)
- **Format validation**: <50ns (achieved: ~13ns)
- **Bulk validation**: Linear with input size

## Security Implementation

### Warning Integration
```rust
impl UlidGenerateCommand {
    fn signature(&self) -> Signature {
        Signature::build("ulid generate")
            .long_description(
                "Generate ULIDs for general-purpose identification.\n\n\
                ⚠️  WARNING: ULIDs are not suitable for security contexts.\n\
                ✅  Safe: Database IDs, log correlation, file naming\n\
                ❌  Unsafe: Auth tokens, session IDs, API keys\n\
                📖  Learn more: ulid security-advice"
            )
            // ... rest of signature
    }
}
```

### Educational Commands
The `ulid security-advice` command provides comprehensive security education, preventing misuse while enabling appropriate applications.

## Next Steps

1. **Complete Architecture Review** - Validate design with Phase 2.1 research
2. **Begin Phase 3.1** - Implement core ULID engine
3. **Prototype Commands** - Start with `ulid generate` and `ulid parse`
4. **Security Integration** - Build warning system into command help
5. **Performance Validation** - Confirm targets with `ulid` crate integration

This architecture provides a solid foundation for transitioning from demo functionality to production ULID implementation while maintaining security-first principles and optimal performance.