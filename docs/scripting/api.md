# ULID Plugin API Reference

**Version**: 0.1.0  
**Last Updated**: August 17, 2025  
**Compatibility**: Nushell 0.106.1+

Complete reference for all ULID plugin commands and their programmatic usage. This document provides detailed API specifications, parameter descriptions, return types, and advanced usage patterns for developers and script authors.

## Quick Reference

| Command | Purpose | Input Types | Output Types |
|---------|---------|-------------|-------------|
| `ulid generate` | Generate ULIDs | Nothing, Number | String, List<String> |
| `ulid validate` | Validate ULID format | String, List<String> | Bool, List<Record> |
| `ulid parse` | Parse ULID components | String, List<String> | Record, List<Record> |
| `ulid inspect` | Detailed ULID analysis | String | Record |
| `ulid sort` | Sort by ULID timestamp | List<String>, Table | List<String>, Table |
| `ulid stream` | Stream process large datasets | List<Any> | List<Record> |
| `ulid generate-stream` | Bulk ULID generation | Number | List<String> |
| `ulid security-advice` | Security recommendations | Nothing | Record |
| `ulid time` | Time operations | Various | Record |
| `ulid encode/decode` | Encoding operations | String, Binary | String, Binary |
| `ulid hash` | Cryptographic operations | String, Binary | String |
| `ulid uuid` | UUID compatibility | String | String, Bool, Record |
| `ulid info` | Plugin information | Nothing | Record |

## Core Commands

### `ulid generate`
Generate cryptographically secure ULIDs with optional parameters.

**Full Syntax:**
```nu
ulid generate [--count <int>] [--timestamp <int>] [--format <string>] [--context <string>]
```

**Parameters:**
- `--count <int>`: Number of ULIDs to generate (1-100,000, default: 1)
- `--timestamp <int>`: Custom timestamp in milliseconds since Unix epoch (optional)
- `--format <string>`: Output format - "standard" (default), "compact", "json"
- `--context <string>`: Security context for validation ("user-session", "api-keys", etc.)

**Input Types:**
- `Nothing`: Generate based on parameters
- `Number`: Use as count parameter

**Output Types:**
- Single ULID: `String` (26 characters)
- Multiple ULIDs: `List<String>`

**Return Value Schema:**
```nu
# Single ULID
"01K2W41TWG3FKYYSK430SR8KW6"

# Multiple ULIDs
[
    "01K2W41TWG3FKYYSK430SR8KW6",
    "01K2W41TWG3FKYYSK430SR8KW7",
    "01K2W41TWG3FKYYSK430SR8KW8"
]

# JSON format
{
    ulid: "01K2W41TWG3FKYYSK430SR8KW6",
    timestamp: 1692817394611,
    randomness: "TWG3FKYYSK430SR8KW6",
    iso8601: "2023-08-23T18:49:54.611Z"
}
```

**Advanced Examples:**
```nu
# Generate with current timestamp
let current_id = ulid generate

# Generate batch with custom timestamp
let batch_ids = ulid generate --count 100 --timestamp 1692000000000

# Generate with security context validation
let secure_id = ulid generate --context "api-keys"

# Generate in JSON format for detailed output
let detailed_ulid = ulid generate --format json

# Use in pipeline
1..10 | each { ulid generate } | str join ","

# Generate time-ordered sequence
let ordered_ids = (0..5 | each { |i| 
    ulid generate --timestamp (date now | into int | $in + ($i * 1000))
})
```

**Error Conditions:**
- `InvalidParameter`: Count exceeds maximum (100,000)
- `InvalidTimestamp`: Timestamp is negative or too large
- `SecurityWarning`: Context indicates potential security risk

### `ulid validate`
Validate ULID format, structure, and integrity.

**Full Syntax:**
```nu
ulid validate <ulid> [--strict] [--details] [--context <string>]
```

**Parameters:**
- `<ulid>`: ULID string to validate (required)
- `--strict`: Enable strict validation (checks timestamp bounds)
- `--details`: Return detailed validation results instead of boolean
- `--context <string>`: Security context for validation

**Input Types:**
- `String`: Single ULID to validate
- `List<String>`: Multiple ULIDs to validate

**Output Types:**
- Single validation: `Bool` (default) or `Record` (with --details)
- Multiple validations: `List<Record>`

**Return Value Schema:**
```nu
# Simple validation
true  # or false

# Detailed validation
{
    ulid: "01K2W41TWG3FKYYSK430SR8KW6",
    valid: true,
    format_valid: true,
    timestamp_valid: true,
    randomness_valid: true,
    length_valid: true,
    encoding_valid: true,
    warnings: []
}

# Multiple ULIDs
[
    { ulid: "01K2W41TWG3FKYYSK430SR8KW6", valid: true },
    { ulid: "invalid-ulid", valid: false, error: "Invalid length" }
]
```

**Advanced Examples:**
```nu
# Simple validation
if (ulid validate "01K2W41TWG3FKYYSK430SR8KW6") {
    print "Valid ULID"
}

# Detailed validation with error information
let validation = ulid validate "01K2W41TWG3FKYYSK430SR8KW6" --details
if not $validation.valid {
    print $"Validation failed: ($validation.error)"
}

# Validate with strict timestamp checking
let strict_result = ulid validate $ulid --strict

# Batch validation with error handling
let results = $ulid_list | each { |id|
    try {
        { ulid: $id, valid: (ulid validate $id), error: null }
    } catch { |e|
        { ulid: $id, valid: false, error: $e.msg }
    }
}

# Filter valid ULIDs from mixed data
let valid_ulids = $mixed_data 
    | where { ulid validate $in.id }
    | get id

# Security context validation
let secure_validation = ulid validate $api_key --context "api-keys" --details
if ($secure_validation.warnings | length) > 0 {
    print $"Security warnings: ($secure_validation.warnings)"
}
```

**Error Conditions:**
- `InvalidInput`: Input is not a string
- `EmptyInput`: Input string is empty
- `InvalidFormat`: ULID format is incorrect

### `ulid parse`
Parse ULID into timestamp, randomness, and metadata components.

**Full Syntax:**
```nu
ulid parse <ulid> [--format <string>] [--timezone <string>] [--validate]
```

**Parameters:**
- `<ulid>`: ULID string to parse (required)
- `--format <string>`: Output format - "standard" (default), "compact", "json", "timestamp-only"
- `--timezone <string>`: Timezone for timestamp formatting ("UTC", "local", or IANA timezone)
- `--validate`: Validate ULID before parsing (recommended)

**Input Types:**
- `String`: Single ULID to parse
- `List<String>`: Multiple ULIDs to parse

**Output Types:**
- Single parse: `Record`
- Multiple parses: `List<Record>`

**Return Value Schema:**
```nu
# Standard format
{
    ulid: "01K2W41TWG3FKYYSK430SR8KW6",
    timestamp: {
        milliseconds: 1692817394611,
        iso8601: "2023-08-23T18:49:54.611Z",
        human: "2023-08-23 18:49:54 UTC",
        unix: 1692817394
    },
    randomness: {
        hex: "F2Y5SK430SR8KW6",
        bytes: [242, 89, 83, 75, 52, 48, 83, 82, 56, 75, 87, 54]
    },
    valid: true
}

# Compact format
{
    ulid: "01K2W41TWG3FKYYSK430SR8KW6",
    ts: 1692817394611,
    rand: "F2Y5SK430SR8KW6",
    valid: true
}

# Timestamp-only format
{
    ulid: "01K2W41TWG3FKYYSK430SR8KW6",
    timestamp: 1692817394611,
    iso8601: "2023-08-23T18:49:54.611Z"
}
```

**Advanced Examples:**
```nu
# Parse with validation
let parsed = ulid parse "01K2W41TWG3FKYYSK430SR8KW6" --validate
if not $parsed.valid {
    error make { msg: "Invalid ULID" }
}

# Extract timestamp in local timezone
let local_time = ulid parse $ulid --timezone local | get timestamp.human

# Batch parsing with error handling
let parsed_ulids = $ulid_list | each { |id|
    try {
        ulid parse $id --validate
    } catch {
        { ulid: $id, valid: false, error: "Parse failed" }
    }
}

# Extract timestamps for sorting/filtering
let timestamps = $ulids | each { |id| 
    ulid parse $id --format timestamp-only | get timestamp 
}

# Parse and convert to different timezone
let tokyo_time = ulid parse $ulid --timezone "Asia/Tokyo" | get timestamp.human

# Compact parsing for memory efficiency
let compact_results = $large_ulid_list | each { |id|
    ulid parse $id --format compact
}

# Extract randomness for uniqueness analysis
let randomness_values = $ulids | each { |id|
    ulid parse $id | get randomness.hex
} | uniq | length
```

**Error Conditions:**
- `InvalidFormat`: ULID format is incorrect
- `ParseError`: Unable to parse timestamp or randomness
- `InvalidTimezone`: Specified timezone is not recognized

### `ulid inspect`
Comprehensive ULID analysis with detailed metadata and statistics.

**Full Syntax:**
```nu
ulid inspect <ulid> [--compact] [--timestamp-only] [--stats] [--security-check] [--format <string>]
```

**Parameters:**
- `<ulid>`: ULID string to inspect (required)
- `--compact`: Return condensed output format
- `--timestamp-only`: Return only timestamp-related information
- `--stats`: Include statistical analysis (entropy, patterns)
- `--security-check`: Perform security context analysis
- `--format <string>`: Output format - "standard", "json", "table"

**Input Types:**
- `String`: Single ULID to inspect

**Output Types:**
- `Record`: Detailed inspection results

**Return Value Schema:**
```nu
# Standard inspection
{
    ulid: "01K2W41TWG3FKYYSK430SR8KW6",
    valid: true,
    timestamp: {
        milliseconds: 1692817394611,
        iso8601: "2023-08-23T18:49:54.611Z",
        human: "2023-08-23 18:49:54 UTC",
        unix: 1692817394,
        age_seconds: 86400,
        age_human: "1 day ago"
    },
    randomness: {
        hex: "F2Y5SK430SR8KW6",
        bytes: [242, 89, 83, 75, 52, 48, 83, 82, 56, 75, 87, 54],
        entropy_bits: 80
    },
    metadata: {
        length: 26,
        encoding: "crockford_base32",
        version: "ulid",
        monotonic: true
    }
}

# With statistics
{
    # ... standard fields ...
    statistics: {
        character_distribution: {...},
        entropy_analysis: {...},
        pattern_detection: {...}
    }
}

# Security check
{
    # ... standard fields ...
    security: {
        context_warnings: [],
        predictability_risk: "low",
        collision_probability: 2.3e-24
    }
}
```

**Advanced Examples:**
```nu
# Full inspection with all options
let full_analysis = ulid inspect $ulid --stats --security-check

# Quick timestamp check
let age = ulid inspect $ulid --timestamp-only | get age_human

# Compact inspection for logging
let log_entry = ulid inspect $request_id --compact

# Security analysis for API keys
let security_info = ulid inspect $api_key --security-check
if ($security_info.security.predictability_risk == "high") {
    print "WARNING: Predictable ULID detected"
}

# Batch inspection for analysis
let inspections = $ulid_list | each { |id|
    ulid inspect $id --compact
}

# Extract creation times for timeline analysis
let timeline = $ulids | each { |id|
    let inspection = ulid inspect $id --timestamp-only
    {
        ulid: $id,
        created: $inspection.timestamp.iso8601,
        age: $inspection.age_human
    }
} | sort-by created
```

### `ulid sort`
Sort data by ULID timestamp order.

**Syntax:**
```nu
ulid sort [--column <string>] [--reverse] [--natural]
```

**Scripting Examples:**
```nu
# Sort ULID list
$ulids | ulid sort

# Sort records by ULID column
$records | ulid sort --column id

# Reverse chronological order
$ulids | ulid sort --reverse
```

## Streaming Commands

### `ulid stream`
High-performance streaming operations for large ULID datasets with memory-efficient processing.

**Full Syntax:**
```nu
ulid stream <operation> [--batch-size <int>] [--parallel] [--continue-on-error] [--output-format <string>] [--progress] [--max-errors <int>]
```

**Operations:**
- `validate`: Validate ULID format and integrity
- `parse`: Parse ULIDs into components
- `extract-timestamp`: Extract timestamps only
- `extract-randomness`: Extract randomness components only
- `transform`: Transform ULIDs with custom formatting
- `analyze`: Statistical analysis of ULID patterns

**Parameters:**
- `<operation>`: Stream operation to perform (required)
- `--batch-size <int>`: Processing batch size (100-10000, default: 1000)
- `--parallel`: Enable parallel processing across CPU cores
- `--continue-on-error`: Continue processing after errors
- `--output-format <string>`: Output format - "standard", "compact", "json"
- `--progress`: Show progress indicator for large datasets
- `--max-errors <int>`: Maximum errors before stopping (default: unlimited)

**Input Types:**
- `List<String>`: ULIDs to process
- `List<Record>`: Records containing ULID fields
- `Table`: Structured data with ULID columns

**Output Types:**
- `List<Record>`: Processed results with status information

**Return Value Schema:**
```nu
# Stream validation
[
    { ulid: "01K2W41TWG3FKYYSK430SR8KW6", valid: true, batch: 0 },
    { ulid: "01K2W41TWG3FKYYSK430SR8KW7", valid: true, batch: 0 },
    { ulid: "invalid-ulid", valid: false, error: "Invalid format", batch: 0 }
]

# Stream parsing
[
    {
        ulid: "01K2W41TWG3FKYYSK430SR8KW6",
        timestamp: 1692817394611,
        randomness: "F2Y5SK430SR8KW6",
        success: true,
        batch: 0
    }
]

# Stream analysis
{
    processed: 10000,
    valid: 9987,
    invalid: 13,
    errors: 0,
    processing_time_ms: 1234,
    throughput_per_sec: 8103,
    batches: 10
}
```

**Advanced Examples:**
```nu
# High-performance validation of large dataset
let validation_results = $million_ulids 
    | ulid stream validate --batch-size 5000 --parallel --progress

# Parse with comprehensive error handling
let parsed_data = $ulid_dataset 
    | ulid stream parse --continue-on-error --max-errors 100
    | where success == true

# Extract timestamps for time-series analysis
let timestamps = $log_ulids 
    | ulid stream extract-timestamp --parallel
    | get timestamp
    | sort

# Memory-efficient processing of huge datasets
let processed = open huge_ulid_file.json
    | get ulids
    | ulid stream validate --batch-size 1000 --output-format compact

# Transform ULIDs with custom formatting
let formatted = $ulids 
    | ulid stream transform --output-format json
    | each { |item| $item | upsert formatted_id $"ULID_($item.ulid)" }

# Statistical analysis of ULID patterns
let analysis = $dataset 
    | ulid stream analyze --parallel
    | get statistics
```

### `ulid generate-stream`
Generate large quantities of ULIDs efficiently.

**Syntax:**
```nu
ulid generate-stream <count> [--batch-size <int>] [--timestamp <int>] [--unique-timestamps]
```

**Scripting Examples:**
```nu
# Generate 10,000 ULIDs
let bulk_ids = ulid generate-stream 10000

# Time-ordered ULIDs
let ordered_ids = ulid generate-stream 1000 --unique-timestamps

# Custom batch size
let ids = ulid generate-stream 50000 --batch-size 1000
```

## Utility Commands

### `ulid security-advice`
Get security recommendations for ULID usage.

**Syntax:**
```nu
ulid security-advice [--context <string>] [--format <string>]
```

**Scripting Examples:**
```nu
# Check security for context
let advice = ulid security-advice --context "user-session"

# Compact security info
let warning = ulid security-advice --format compact | get warning_level
```

## Error Handling Patterns

### Try-Catch Pattern
```nu
def safe_ulid_operation [ulid: string] {
    try {
        ulid parse $ulid
    } catch {
        { error: $"Invalid ULID: ($ulid)", success: false }
    }
}
```

### Validation Pattern
```nu
def process_if_valid [ulid: string] {
    if (ulid validate $ulid) {
        # Process valid ULID
        ulid parse $ulid
    } else {
        # Handle invalid ULID
        { error: "Invalid ULID", ulid: $ulid }
    }
}
```

### Bulk Processing Pattern
```nu
def process_ulid_list [ulids: list] {
    $ulids | each { |ulid|
        if (ulid validate $ulid) {
            { ulid: $ulid, parsed: (ulid parse $ulid), valid: true }
        } else {
            { ulid: $ulid, error: "Invalid", valid: false }
        }
    }
}
```

## Performance Guidelines

### Memory Efficiency
- Use streaming commands for datasets > 1000 items
- Adjust batch sizes based on available memory
- Process data in chunks rather than loading everything at once

### CPU Optimization
- Enable parallel processing for CPU-intensive operations
- Use bulk generation instead of individual ULID generation
- Cache parsed results when processing the same ULIDs multiple times

### Pipeline Integration
```nu
# Efficient pipeline pattern
$data 
| where valid_record 
| ulid sort --column id 
| select id timestamp data
| save processed_data.json
```

## Common Integration Patterns

### Database ID Generation
```nu
def add_ids_to_records [records: list] {
    let count = ($records | length)
    let ids = (ulid generate-stream $count)
    $records | enumerate | each { |row|
        $row.item | upsert id ($ids | get $row.index)
    }
}
```

### Log Processing
```nu
def process_logs_by_time [logs: list] {
    $logs 
    | ulid sort --column request_id 
    | group-by { |log| 
        ulid parse $log.request_id | get timestamp.iso8601 | str substring 0..13
    }
}
```

### Data Validation
```nu
def validate_data_integrity [data: list] {
    let total = ($data | length)
    let valid = ($data | where { ulid validate $in.id } | length)
    { total: $total, valid: $valid, invalid: ($total - $valid) }
}
```

## Advanced Usage Patterns

### Batch Processing Strategies

#### Memory-Efficient Processing
```nu
# Process large datasets without loading everything into memory
def process_ulid_file [file_path: string] {
    open $file_path
    | get ulids
    | chunks 1000  # Process in 1000-item chunks
    | each { |chunk|
        $chunk | ulid stream validate --batch-size 500 --parallel
    }
    | flatten
}
```

#### Adaptive Batch Sizing
```nu
# Adjust batch size based on available memory and dataset size
def adaptive_ulid_processing [ulids: list] {
    let count = ($ulids | length)
    let batch_size = if $count > 100000 {
        5000  # Large batches for huge datasets
    } else if $count > 10000 {
        1000  # Medium batches for large datasets
    } else {
        100   # Small batches for smaller datasets
    }
    
    $ulids | ulid stream parse --batch-size $batch_size --parallel
}
```

### Error Handling Patterns

#### Graceful Error Recovery
```nu
def robust_ulid_processor [ulids: list] {
    $ulids | each { |ulid|
        try {
            let parsed = ulid parse $ulid --validate
            { 
                ulid: $ulid, 
                success: true, 
                timestamp: $parsed.timestamp.milliseconds,
                error: null 
            }
        } catch { |e|
            { 
                ulid: $ulid, 
                success: false, 
                timestamp: null,
                error: $e.msg 
            }
        }
    }
}
```

#### Error Aggregation and Reporting
```nu
def process_with_error_report [ulids: list] {
    let results = $ulids | robust_ulid_processor
    let errors = $results | where success == false
    let successes = $results | where success == true
    
    {
        total_processed: ($results | length),
        successful: ($successes | length),
        failed: ($errors | length),
        success_rate: (($successes | length) / ($results | length) * 100),
        error_summary: ($errors | group-by error | transpose error count),
        results: $results
    }
}
```

### Performance Optimization

#### Parallel Processing with Work Distribution
```nu
# Distribute work across multiple parallel streams
def parallel_ulid_analysis [ulids: list, num_workers: int = 4] {
    let chunk_size = (($ulids | length) / $num_workers | math ceil)
    
    $ulids 
    | chunks $chunk_size
    | par-each { |chunk|
        $chunk | ulid stream parse --parallel --batch-size 1000
    }
    | flatten
}
```

#### Caching and Memoization
```nu
# Cache parsed ULID results for repeated access
mut $ulid_cache = {}

def cached_ulid_parse [ulid: string] {
    if $ulid in $ulid_cache {
        $ulid_cache | get $ulid
    } else {
        let parsed = ulid parse $ulid
        $ulid_cache = ($ulid_cache | upsert $ulid $parsed)
        $parsed
    }
}
```

### Security-Aware Processing

#### Context-Sensitive Validation
```nu
def secure_ulid_validator [ulids: list, context: string] {
    $ulids | each { |ulid|
        # Basic validation
        let basic_valid = ulid validate $ulid
        
        # Security context check
        let security_check = ulid security-advice --context $context
        
        # Enhanced validation for sensitive contexts
        let enhanced_valid = if $context in ["api-keys", "session-tokens"] {
            let inspection = ulid inspect $ulid --security-check
            $basic_valid and ($inspection.security.predictability_risk != "high")
        } else {
            $basic_valid
        }
        
        {
            ulid: $ulid,
            basic_valid: $basic_valid,
            security_valid: $enhanced_valid,
            context: $context,
            warnings: $security_check.warnings
        }
    }
}
```

#### Audit Trail Generation
```nu
def create_ulid_audit_trail [operations: list] {
    $operations | each { |op|
        let timestamp = date now
        let audit_id = ulid generate
        
        {
            audit_id: $audit_id,
            timestamp: $timestamp,
            operation: $op.type,
            ulid_processed: $op.ulid,
            result: $op.result,
            security_context: $op.context,
            user: (whoami),
            checksum: ([$op.ulid, $op.result, $timestamp] | str join "|" | hash sha256)
        }
    }
}
```

### Integration Patterns

#### Database Integration
```nu
# Prepare ULIDs for database insertion
def prepare_db_records [records: list] {
    let timestamp = date now | into int
    
    $records | each { |record|
        let id = ulid generate --timestamp $timestamp
        $record 
        | upsert id $id
        | upsert created_at (ulid parse $id | get timestamp.iso8601)
        | upsert created_timestamp $timestamp
    }
}
```

#### API Response Processing
```nu
# Process API responses containing ULIDs
def process_api_response [response: record] {
    let processed_items = $response.items | each { |item|
        if (ulid validate $item.id) {
            let parsed = ulid parse $item.id
            $item 
            | upsert created_time $parsed.timestamp.human
            | upsert age_seconds (date now | into int | $in - $parsed.timestamp.unix)
        } else {
            $item | upsert error "Invalid ULID"
        }
    }
    
    $response | upsert items $processed_items
}
```

### Monitoring and Analytics

#### Performance Metrics Collection
```nu
def collect_ulid_metrics [operations: list] {
    let start_time = date now | into int
    
    let results = $operations | each { |op|
        let op_start = date now | into int
        let result = (do $op.command)
        let op_end = date now | into int
        
        {
            operation: $op.name,
            duration_ms: ($op_end - $op_start),
            success: ($result != null),
            items_processed: ($result | length),
            throughput: (($result | length) / (($op_end - $op_start) / 1000))
        }
    }
    
    let end_time = date now | into int
    
    {
        total_duration_ms: ($end_time - $start_time),
        operations: $results,
        total_throughput: ($results | get items_processed | math sum) / (($end_time - $start_time) / 1000),
        success_rate: (($results | where success | length) / ($results | length) * 100)
    }
}
```

#### ULID Pattern Analysis
```nu
def analyze_ulid_patterns [ulids: list] {
    let parsed_ulids = $ulids | each { ulid parse $in }
    
    {
        total_ulids: ($ulids | length),
        time_span: {
            earliest: ($parsed_ulids | get timestamp.milliseconds | math min),
            latest: ($parsed_ulids | get timestamp.milliseconds | math max),
            duration_hours: (($parsed_ulids | get timestamp.milliseconds | math max) - ($parsed_ulids | get timestamp.milliseconds | math min)) / 3600000
        },
        distribution: {
            hourly: ($parsed_ulids | group-by { |p| $p.timestamp.iso8601 | str substring 0..13 } | transpose hour count),
            daily: ($parsed_ulids | group-by { |p| $p.timestamp.iso8601 | str substring 0..10 } | transpose day count)
        },
        randomness_analysis: {
            unique_randomness: ($parsed_ulids | get randomness.hex | uniq | length),
            entropy_estimate: ($parsed_ulids | get randomness.hex | uniq | length) / ($parsed_ulids | length)
        }
    }
}
```

## API Reference Summary

This comprehensive API reference provides detailed specifications for all nu_plugin_nw_ulid commands. Key features:

- **Type-safe operations** with comprehensive input/output type specifications
- **Performance optimization** through streaming and parallel processing
- **Security-first design** with context-aware validation and warnings
- **Error resilience** with graceful error handling and recovery patterns
- **Enterprise-grade quality** with comprehensive validation and audit capabilities

For additional examples and use cases, see:
- [User Guide](../USER_GUIDE.md) - Complete user documentation
- [Scripting Guide](README.md) - Automation patterns and workflows
- [Developer Guide](../DEVELOPER_GUIDE.md) - Internal architecture and contribution guidelines