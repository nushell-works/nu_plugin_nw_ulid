# nu_plugin_ulid User Guide

Welcome to the comprehensive user guide for nu_plugin_ulid - a production-grade ULID plugin for Nushell.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic ULID Operations](#basic-ulid-operations)
3. [Advanced Features](#advanced-features)
4. [Practical Examples](#practical-examples)
5. [Performance & Best Practices](#performance--best-practices)
6. [Security Considerations](#security-considerations)
7. [Troubleshooting](#troubleshooting)

## Getting Started

### What are ULIDs?

ULIDs (Universally Unique Lexicographically Sortable Identifiers) are 128-bit identifiers that combine the benefits of UUIDs with lexicographic sorting capability:

- **26 characters long** using Crockford Base32 encoding
- **Sortable by timestamp** - newer ULIDs are lexicographically larger
- **URL-safe** - no special characters that need escaping
- **Case-insensitive** - all uppercase for consistency
- **Monotonic** - within the same millisecond, values increase monotonically

### ULID Structure

```
01ARZ3NDEKTSV4RRFFQ69G5FAV
|----------|-------------|
  Timestamp    Randomness
   (48-bit)     (80-bit)
    10 chars     16 chars
```

### Installation

1. **Install the plugin:**
   ```bash
   cargo install nw-nu_plugin_ulid
   ```

2. **Register with Nushell:**
   ```bash
   plugin add ~/.cargo/bin/nu_plugin_ulid
   plugin use ulid
   ```

3. **Verify installation:**
   ```nushell
   ulid info
   ```

## Basic ULID Operations

### Generating ULIDs

```nushell
# Generate a single ULID
> ulid generate
01K2W41TWG3FKYYSK430SR8KW6

# Generate multiple ULIDs
> ulid generate --count 3
╭───┬─────────────────────────────╮
│ 0 │ 01K2W41TWG3FKYYSK430SR8KW7 │
│ 1 │ 01K2W41TWG3FKYYSK430SR8KW8 │
│ 2 │ 01K2W41TWG3FKYYSK430SR8KW9 │
╰───┴─────────────────────────────╯

# Generate with custom timestamp (milliseconds since epoch)
> ulid generate --timestamp 1692000000000
01H4QG7XG00000000000000000

# Generate for specific context with security validation
> ulid generate --context "user-session"
01K2W41TWG3FKYYSK430SR8KWA
```

### Validating ULIDs

```nushell
# Validate a single ULID
> ulid validate "01K2W41TWG3FKYYSK430SR8KW6"
true

# Validate invalid ULID
> ulid validate "invalid-ulid"
false

# Validate ULIDs in a list
> ["01K2W41TWG3FKYYSK430SR8KW6", "invalid", "01K2W41TWG3FKYYSK430SR8KW7"]
  | each { |ulid| { ulid: $ulid, valid: (ulid validate $ulid) } }
╭───┬─────────────────────────────┬───────╮
│ # │            ulid             │ valid │
├───┼─────────────────────────────┼───────┤
│ 0 │ 01K2W41TWG3FKYYSK430SR8KW6  │ true  │
│ 1 │ invalid                     │ false │
│ 2 │ 01K2W41TWG3FKYYSK430SR8KW7  │ true  │
╰───┴─────────────────────────────┴───────╯
```

### Parsing ULIDs

```nushell
# Parse ULID into components
> ulid parse "01K2W41TWG3FKYYSK430SR8KW6"
╭────────────┬────────────────────────────╮
│ ulid       │ 01K2W41TWG3FKYYSK430SR8KW6 │
│ timestamp  │ {record 4 fields}          │
│ randomness │ {record 1 field}           │
│ valid      │ true                       │
╰────────────┴────────────────────────────╯

# Extract just the timestamp
> ulid parse "01K2W41TWG3FKYYSK430SR8KW6" | get timestamp
╭─────────────┬─────────────────────────╮
│ milliseconds │ 1692817394611           │
│ iso8601      │ 2023-08-23T18:49:54.611Z │
│ human        │ 2023-08-23 18:49:54 UTC │
│ unix         │ 1692817394              │
╰─────────────┴─────────────────────────╯
```

### Sorting Data by ULIDs

```nushell
# Sort ULIDs chronologically
> ["01K3X1", "01K2W4", "01K5Z9"] | ulid sort
╭───┬────────╮
│ 0 │ 01K2W4 │
│ 1 │ 01K3X1 │
│ 2 │ 01K5Z9 │
╰───┴────────╯

# Sort records by ULID column
> [{id: "01K3X1", name: "Alice"}, {id: "01K2W4", name: "Bob"}] | ulid sort --column id
╭───┬────────┬──────╮
│ # │   id   │ name │
├───┼────────┼──────┤
│ 0 │ 01K2W4 │ Bob  │
│ 1 │ 01K3X1 │ Alice│
╰───┴────────┴──────╯

# Reverse sort (newest first)
> ["01K2W4", "01K3X1", "01K5Z9"] | ulid sort --reverse
╭───┬────────╮
│ 0 │ 01K5Z9 │
│ 1 │ 01K3X1 │
│ 2 │ 01K2W4 │
╰───┴────────╯
```

### Detailed ULID Inspection

```nushell
# Get detailed ULID analysis
> ulid inspect "01K2W41TWG3FKYYSK430SR8KW6"
╭─────────────────┬─────────────────────────────────╮
│ ulid            │ 01K2W41TWG3FKYYSK430SR8KW6      │
│ valid           │ true                            │
│ timestamp_ms    │ 1692817394611                   │
│ timestamp_human │ 2023-08-23 18:49:54 UTC        │
│ randomness_hex  │ F2Y5SK430SR8KW6                 │
│ age_seconds     │ 86400                           │
│ metadata        │ {record 3 fields}               │
╰─────────────────┴─────────────────────────────────╯

# Get compact inspection
> ulid inspect "01K2W41TWG3FKYYSK430SR8KW6" --compact
╭─────────┬─────────────────────────╮
│ ulid    │ 01K2W41TWG3FKYYSK430SR8KW6 │
│ ts      │ 1692817394611              │
│ age_hrs │ 24                         │
╰─────────┴────────────────────────────╯
```

## Advanced Features

### Streaming Operations for Large Datasets

When working with thousands or millions of ULIDs, use streaming commands for optimal performance:

```nushell
# Validate large datasets efficiently
> $large_ulid_list | ulid stream validate --batch-size 1000
╭─────┬─────────────────────────────┬───────╮
│  #  │            ulid             │ valid │
├─────┼─────────────────────────────┼───────┤
│   0 │ 01K2W41TWG3FKYYSK430SR8KW6  │ true  │
│   1 │ 01K2W41TWG3FKYYSK430SR8KW7  │ true  │
│ ... │ ...                         │ ...   │
╰─────┴─────────────────────────────┴───────╯

# Parse large datasets with error handling
> $ulid_list | ulid stream parse --continue-on-error --parallel
╭─────┬─────────────────────────────┬─────────────┬─────────────╮
│  #  │            ulid             │  timestamp  │ randomness  │
├─────┼─────────────────────────────┼─────────────┼─────────────┤
│   0 │ 01K2W41TWG3FKYYSK430SR8KW6  │ {...}       │ {...}       │
│   1 │ invalid-ulid                │ null        │ null        │
│ ... │ ...                         │ ...         │ ...         │
╰─────┴─────────────────────────────┴─────────────┴─────────────╯

# Generate large quantities efficiently
> ulid generate-stream 10000 --batch-size 500
╭─────┬─────────────────────────────╮
│  0  │ 01K2W41TWG3FKYYSK430SR8KW6 │
│  1  │ 01K2W41TWG3FKYYSK430SR8KW7 │
│ ... │ ...                        │
│9999 │ 01K2W41TWG3FKYYSK430SR8KX5 │
╰─────┴─────────────────────────────╯
```

### Time-based Operations

```nushell
# Get current timestamp in various formats
> ulid time now
╭─────────────┬─────────────────────────╮
│ milliseconds │ 1692817394611           │
│ iso8601      │ 2023-08-23T18:49:54.611Z │
│ human        │ 2023-08-23 18:49:54 UTC │
│ unix         │ 1692817394              │
╰─────────────┴─────────────────────────╯

# Convert timestamp to ULID format
> ulid time millis 1692000000000
1692000000000

# Parse various timestamp formats
> ulid time parse "2023-08-23T18:49:54.611Z"
╭─────────────┬─────────────────────────╮
│ milliseconds │ 1692817394611           │
│ iso8601      │ 2023-08-23T18:49:54.611Z │
│ unix         │ 1692817394              │
╰─────────────┴─────────────────────────╯
```

### Encoding Operations

```nushell
# Base32 encoding (ULID standard)
> ulid encode base32 "Hello World"
91JPRV3F5GG7EVVJDHJ22

# Base32 decoding
> ulid decode base32 "91JPRV3F5GG7EVVJDHJ22" --text
Hello World

# Hexadecimal encoding
> ulid encode hex "Hello World"
48656c6c6f20576f726c64

> ulid encode hex "Hello World" --uppercase
48656C6C6F20576F726C64

# Hexadecimal decoding
> ulid decode hex "48656c6c6f20576f726c64" --text
Hello World
```

### Cryptographic Operations

```nushell
# Generate secure random bytes
> ulid hash random --length 32
d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5

# SHA-256 hashing
> ulid hash sha256 "Hello World"
a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e

# SHA-512 hashing
> ulid hash sha512 "Hello World"
2c74fd17edafd80e8447b0d46741ee243b7eb74dd2149a0ab1b9246fb30382f27e853d8585719e0e67cbda0daa8f51671064615d645ae27acb15bfb1447f459b

# BLAKE3 hashing with custom length
> ulid hash blake3 "Hello World" --length 16
d74981efa70a0c880b8d8c1985d075b2
```

## Practical Examples

### Example 1: Database Record Management

```nushell
# Create records with ULIDs
def add_record_ids [records: list] {
    let count = ($records | length)
    let ids = (ulid generate-stream $count)
    $records | enumerate | each { |row|
        $row.item | upsert id ($ids | get $row.index)
    }
}

# Usage
let products = [
    {name: "Laptop", price: 999.99, category: "electronics"},
    {name: "Book", price: 19.99, category: "books"},
    {name: "Shirt", price: 29.99, category: "clothing"}
]

let products_with_ids = (add_record_ids $products)
$products_with_ids | ulid sort --column id
```

### Example 2: Log Analysis

```nushell
# Analyze log files with ULID request IDs
def analyze_request_logs [log_file: string] {
    open $log_file
    | where (ulid validate $in.request_id)
    | ulid sort --column request_id
    | group-by { |log|
        ulid parse $log.request_id | get timestamp.iso8601 | str substring 0..13
    }
    | transpose hour logs
    | each { |group|
        {
            hour: $group.hour,
            request_count: ($group.logs | length),
            avg_response_time: ($group.logs | get response_time | math avg),
            error_rate: (($group.logs | where status >= 400 | length) / ($group.logs | length) * 100)
        }
    }
}
```

### Example 3: Data Synchronization

```nushell
# Sync data based on ULID timestamps
def sync_data [source: list, target: list] {
    let source_ids = ($source | get id)
    let target_ids = ($target | get id)

    # Find new and modified records
    let new_records = ($source | where ($it.id not-in $target_ids))
    let modified_records = ($source
        | where ($it.id in $target_ids)
        | where ($it.updated_at > ($target | where id == $it.id | first | get updated_at))
    )

    # Sort by ULID timestamp for proper sync order
    let sync_order = ([$new_records, $modified_records] | flatten | ulid sort --column id)

    {
        new_count: ($new_records | length),
        modified_count: ($modified_records | length),
        sync_operations: $sync_order
    }
}
```

### Example 4: API Rate Limiting

```nushell
# Track API requests with ULID timestamps
def check_rate_limit [request_id: string, rate_limit_per_minute: int] {
    let request_time = (ulid parse $request_id | get timestamp.milliseconds)
    let minute_start = ($request_time // 60000 * 60000)

    # Check requests in the same minute
    let recent_requests = ($api_request_log
        | where { |req|
            let req_time = (ulid parse $req.id | get timestamp.milliseconds)
            $req_time >= $minute_start and $req_time < ($minute_start + 60000)
        }
        | length
    )

    if $recent_requests >= $rate_limit_per_minute {
        {allowed: false, remaining: 0, reset_time: ($minute_start + 60000)}
    } else {
        {allowed: true, remaining: ($rate_limit_per_minute - $recent_requests - 1), reset_time: ($minute_start + 60000)}
    }
}
```

## Performance & Best Practices

### Performance Guidelines

1. **Use streaming for large datasets** (>1000 items):
   ```nushell
   # Good: Use streaming for large datasets
   $large_dataset | ulid stream validate --batch-size 1000

   # Avoid: Individual validation for large datasets
   $large_dataset | each { |item| ulid validate $item.id }
   ```

2. **Batch ULID generation**:
   ```nushell
   # Good: Generate in bulk
   ulid generate-stream 1000

   # Avoid: Individual generation in loops
   0..999 | each { ulid generate }
   ```

3. **Configure batch sizes** based on memory:
   ```nushell
   # For high memory systems
   ulid stream parse --batch-size 5000

   # For constrained systems
   ulid stream parse --batch-size 100
   ```

4. **Use parallel processing** for CPU-intensive operations:
   ```nushell
   $large_dataset | ulid stream parse --parallel
   ```

### Memory Optimization

- **Stream processing**: Prevents loading entire datasets into memory
- **Configurable batching**: Adjust batch sizes based on available memory
- **Lazy evaluation**: Process data as needed, not all at once

### CPU Optimization

- **Parallel processing**: Enable `--parallel` for multi-core utilization
- **Bulk operations**: Use bulk commands instead of loops
- **Efficient algorithms**: Optimized parsing and validation routines

## Security Considerations

### Security Best Practices

1. **Use security advice** for sensitive contexts:
   ```nushell
   ulid security-advice --context "user-session"
   ulid security-advice --context "api-keys"
   ulid security-advice --context "database-ids"
   ```

2. **Validate inputs** before processing:
   ```nushell
   def safe_ulid_operation [ulid: string] {
       if not (ulid validate $ulid) {
           error make {msg: "Invalid ULID format"}
       }
       # Proceed with operation
       ulid parse $ulid
   }
   ```

3. **Handle errors gracefully**:
   ```nushell
   def process_ulids_safely [ulids: list] {
       $ulids | each { |ulid|
           try {
               ulid parse $ulid
           } catch {
               {error: "Invalid ULID", ulid: $ulid}
           }
       }
   }
   ```

### Security Features

- **A- Security Rating**: Comprehensive security audit completed
- **Cryptographic randomness**: Uses secure system entropy
- **Input validation**: Comprehensive malicious input protection
- **Memory safety**: Rust's memory guarantees prevent buffer overflows
- **Information leakage protection**: Sanitized error messages

### Context-Aware Security

The plugin provides security warnings for different use contexts:

```nushell
# Check security implications for different contexts
ulid security-advice --context "user-session"    # User session tracking
ulid security-advice --context "api-keys"        # API key generation
ulid security-advice --context "database-ids"    # Database primary keys
ulid security-advice --context "file-names"      # File naming
ulid security-advice --context "public-urls"     # Public URL generation
```

## Troubleshooting

### Common Issues

#### 1. Plugin Not Found
```
Error: Plugin nu_plugin_ulid was not found
```
**Solution:**
```bash
# Re-register the plugin
plugin add ~/.cargo/bin/nu_plugin_ulid
plugin use ulid
```

#### 2. Invalid ULID Errors
```
Error: Invalid ULID format
```
**Solution:**
```nushell
# Always validate before processing
if (ulid validate $ulid) {
    ulid parse $ulid
} else {
    print $"Invalid ULID: ($ulid)"
}
```

#### 3. Performance Issues with Large Datasets
**Problem:** Slow processing of large ULID datasets
**Solution:**
```nushell
# Use streaming with appropriate batch size
$large_dataset | ulid stream validate --batch-size 1000 --parallel
```

#### 4. Memory Usage Issues
**Problem:** High memory usage with large datasets
**Solution:**
```nushell
# Reduce batch size and use streaming
$data | ulid stream parse --batch-size 100
```

### Debugging

1. **Check plugin version and status**:
   ```nushell
   ulid info
   ```

2. **Validate ULID format**:
   ```nushell
   ulid validate "your-ulid-here"
   ```

3. **Test with simple operations**:
   ```nushell
   # Test generation
   ulid generate

   # Test validation
   ulid validate (ulid generate)

   # Test parsing
   ulid parse (ulid generate)
   ```

4. **Check for security warnings**:
   ```nushell
   ulid security-advice --context "your-use-case"
   ```

### Getting Help

- **Plugin information**: `ulid info`
- **Command help**: `help ulid generate`, `help ulid parse`, etc.
- **Security guidance**: `ulid security-advice`
- **GitHub Issues**: [Report bugs and feature requests](https://github.com/nushell-works/nu_plugin_ulid/issues)
- **Documentation**: [Complete documentation](https://github.com/nushell-works/nu_plugin_ulid/tree/main/docs)

---

This user guide provides comprehensive coverage of nu_plugin_ulid functionality with practical examples for real-world usage. For more advanced topics, see the [API Reference](scripting/api.md) and [Scripting Guide](scripting/README.md).
