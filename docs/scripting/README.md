# Nushell ULID Scripting Guide

This guide shows how to integrate ULID functionality into your Nushell scripts for automation workflows.

## Quick Start Patterns

### Basic ULID Operations
```nu
# Generate a single ULID
let id = ulid generate

# Generate multiple ULIDs
let ids = ulid generate-stream 100

# Validate ULIDs from data
$data | each { |row| ulid validate $row.id }

# Parse ULIDs for timestamp extraction
$ids | each { |id| ulid parse $id | get timestamp.milliseconds }
```

### Data Processing Patterns
```nu
# Sort records by ULID timestamp
$records | ulid sort --column id

# Filter records by ULID timestamp range
$records | where {
    let ts = ($in.id | ulid parse | get timestamp.milliseconds)
    $ts > 1692000000000 and $ts < 1692086400000
}

# Transform data with ULID validation
$data | each { |row|
    if (ulid validate $row.id) {
        $row | upsert valid true
    } else {
        $row | upsert valid false | upsert error "Invalid ULID"
    }
}
```

### Bulk Processing
```nu
# Process large datasets efficiently
$large_dataset | ulid stream validate --batch-size 1000 --continue-on-error

# Generate ULIDs in batches for large datasets
ulid generate-stream 50000 --batch-size 500 | save ulid_batch.json
```

## Advanced Integration Patterns

### Error Handling
```nu
def safe_ulid_parse [ulid: string] {
    try {
        ulid parse $ulid
    } catch {
        { error: "Invalid ULID", ulid: $ulid }
    }
}
```

### Custom Validation
```nu
def validate_ulid_age [ulid: string, max_age_hours: int] {
    let parsed = (ulid parse $ulid)
    let age_hours = (date now | into int) - ($parsed.timestamp.milliseconds | into int) | math floor | $in / 3600000
    $age_hours <= $max_age_hours
}
```

### Batch ID Generation
```nu
def generate_ids_for_records [records: list] {
    let count = ($records | length)
    let ids = (ulid generate-stream $count)
    $records | enumerate | each { |row|
        $row.item | upsert id ($ids | get $row.index)
    }
}
```

## Security Considerations

Always use the security advice command when working with ULIDs in sensitive contexts:

```nu
# Check security context before ULID operations
ulid security-advice --context "user-session"
```

## Performance Tips

1. Use streaming commands for large datasets (>1000 items)
2. Enable parallel processing for CPU-intensive operations
3. Use batch processing to manage memory usage
4. Cache parsed ULID components when processing the same ULIDs multiple times

## See Also

- [API Reference](api.md) - Complete command documentation
- [Examples](examples/) - Ready-to-use script templates
- [Helpers](helpers.nu) - Reusable utility functions