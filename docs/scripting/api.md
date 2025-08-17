# ULID Plugin API Reference

Complete reference for all ULID plugin commands and their programmatic usage.

## Core Commands

### `ulid generate`
Generate new ULIDs with optional timestamp.

**Syntax:**
```nu
ulid generate [--count <int>] [--timestamp <int>]
```

**Scripting Examples:**
```nu
# Single ULID
let id = ulid generate

# Multiple ULIDs
let ids = ulid generate --count 10

# ULID with specific timestamp
let id = ulid generate --timestamp 1692000000000
```

### `ulid validate`
Validate ULID format and structure.

**Syntax:**
```nu
ulid validate <ulid>
```

**Scripting Examples:**
```nu
# Validate single ULID
if (ulid validate $id) { print "Valid" } else { print "Invalid" }

# Validate list of ULIDs
$ulids | each { |id| { ulid: $id, valid: (ulid validate $id) } }
```

### `ulid parse`
Parse ULID into components (timestamp, randomness).

**Syntax:**
```nu
ulid parse <ulid> [--format <string>]
```

**Scripting Examples:**
```nu
# Parse ULID
let components = ulid parse $id

# Extract timestamp
let timestamp = ulid parse $id | get timestamp.milliseconds

# Compact format
let compact = ulid parse $id --format compact
```

### `ulid inspect`
Detailed ULID analysis with metadata.

**Syntax:**
```nu
ulid inspect <ulid> [--compact] [--timestamp-only] [--stats]
```

**Scripting Examples:**
```nu
# Full inspection
let details = ulid inspect $id

# Just timestamp info
let ts_info = ulid inspect $id --timestamp-only

# Statistical analysis
let stats = ulid inspect $id --stats
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
Stream-process large ULID datasets.

**Syntax:**
```nu
ulid stream <operation> [--batch-size <int>] [--output-format <string>] [--parallel] [--continue-on-error]
```

**Operations:** validate, parse, extract-timestamp, transform

**Scripting Examples:**
```nu
# Validate large dataset
$large_ulids | ulid stream validate --batch-size 500

# Parse with error handling
$ulids | ulid stream parse --continue-on-error

# Parallel timestamp extraction
$huge_dataset | ulid stream extract-timestamp --parallel
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