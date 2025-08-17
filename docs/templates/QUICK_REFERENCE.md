# ULID Templates Quick Reference

## Template Overview

| Template | Use Case | Complexity | Key Features |
|----------|----------|------------|--------------|
| [api-request-tracking.nu](api-request-tracking.nu) | Track API requests across microservices | ⭐⭐ Medium | Request correlation, performance metrics, distributed tracing |
| [database-migration.nu](database-migration.nu) | Migrate databases to ULID primary keys | ⭐⭐⭐ Hard | Safe migration, foreign key mapping, data integrity validation |
| [log-analysis.nu](log-analysis.nu) | Analyze logs with ULID request tracking | ⭐⭐ Medium | Multi-format parsing, performance analysis, error detection |

## Quick Start Commands

### API Request Tracking
```bash
# Track a single API request
nu api-request-tracking.nu track-request --service auth --endpoint /login --method POST

# Track a request chain
nu api-request-tracking.nu track-chain

# Analyze traces
nu api-request-tracking.nu analyze --since "2023-08-01"

# Generate metrics
nu api-request-tracking.nu metrics --save

# Run tests
nu api-request-tracking.nu test
```

### Database Migration
```bash
# Analyze dataset for migration planning
nu database-migration.nu analyze --source data.json

# Perform migration with backup
nu database-migration.nu migrate --source data.json --target migrated.json

# Validate migration integrity
nu database-migration.nu validate original.json migrated.json

# Run migration tests
nu database-migration.nu test
```

### Log Analysis
```bash
# Analyze JSON logs
nu log-analysis.nu analyze --file app.log --format json

# Monitor logs in real-time
nu log-analysis.nu monitor --file app.log --format structured

# Parse and preview log structure
nu log-analysis.nu parse --file app.log --format csv

# Run analysis tests
nu log-analysis.nu test
```

## Common Configuration Patterns

### Batch Processing
```nu
const CONFIG = {
    batch_size: 1000,              # Process 1000 items at a time
    parallel_processing: true,      # Enable parallel processing
    memory_limit_mb: 512,          # Memory usage limit
    max_retries: 3                 # Maximum retry attempts
}
```

### Error Handling
```nu
def safe_operation [data: any] {
    try {
        # Your operation here
        $data | process_data
    } catch { |e|
        log error $"Operation failed: ($e.msg)"
        { success: false, error: $e.msg, data: null }
    }
}
```

### Logging Pattern
```nu
def log [level: string, message: string] {
    let timestamp = date now | format date "%Y-%m-%d %H:%M:%S"
    let log_entry = $"[$timestamp] [($level | str upcase)] ($message)"
    
    print $log_entry
    $log_entry | save --append application.log
}
```

### ULID Generation with Context
```nu
def generate_contextual_ulid [context: string] {
    # Check security context
    let security_advice = ulid security-advice --context $context
    
    if ($security_advice.warning_level == "high") {
        print $"WARNING: High security risk for context: ($context)"
    }
    
    ulid generate --context $context
}
```

## Performance Optimization Tips

### 1. Streaming for Large Datasets
```nu
# Instead of loading everything into memory
$large_dataset | each { |item| process_item $item }

# Use streaming operations
$large_dataset | ulid stream validate --batch-size 1000 --parallel
```

### 2. Adaptive Batch Sizing
```nu
def adaptive_processing [data: list] {
    let count = ($data | length)
    let batch_size = if $count > 100000 {
        5000  # Large batches for huge datasets
    } else if $count > 10000 {
        1000  # Medium batches for large datasets
    } else {
        100   # Small batches for smaller datasets
    }
    
    $data | chunks $batch_size | each { |chunk|
        $chunk | process_batch
    }
}
```

### 3. Memory-Efficient Processing
```nu
def process_file_in_chunks [file_path: string] {
    open $file_path
    | lines
    | chunks 1000  # Process 1000 lines at a time
    | each { |chunk|
        $chunk | process_lines
    }
    | flatten
}
```

## Security Best Practices

### 1. Context-Aware ULID Generation
```nu
def secure_ulid_generation [context: string] {
    # Always check security context first
    let security_check = ulid security-advice --context $context
    
    if ($security_check.warning_level in ["high", "critical"]) {
        let confirmation = input $"Security warning for context ($context). Continue? (y/N): "
        if $confirmation != "y" {
            error make { msg: "Operation cancelled due to security concerns" }
        }
    }
    
    ulid generate --context $context
}
```

### 2. Input Validation
```nu
def validate_ulid_input [ulid: string] {
    if not (ulid validate $ulid) {
        error make { msg: $"Invalid ULID format: ($ulid)" }
    }
    
    # Additional security checks
    let inspection = ulid inspect $ulid --security-check
    if ($inspection.security.predictability_risk == "high") {
        print $"WARNING: ULID may be predictable"
    }
}
```

### 3. Audit Trail Implementation
```nu
def log_ulid_operation [operation: string, ulids: list, context: string] {
    let audit_entry = {
        audit_id: (ulid generate --context "audit"),
        timestamp: (date now),
        operation: $operation,
        context: $context,
        ulid_count: ($ulids | length),
        user: (whoami),
        checksum: ([$operation, ($ulids | str join ","), $context] | str join "|" | hash sha256)
    }
    
    $audit_entry | to json | save --append audit.jsonl
}
```

## Testing Patterns

### Unit Testing
```nu
def test_ulid_generation [] {
    let ulid = ulid generate
    assert (ulid validate $ulid) "Generated ULID should be valid"
    
    let parsed = ulid parse $ulid
    assert ($parsed.valid == true) "Parsed ULID should be marked as valid"
    
    print "✅ ULID generation test passed"
}
```

### Integration Testing
```nu
def test_end_to_end_workflow [] {
    # Test complete workflow
    let correlation_id = ulid generate --context "test"
    
    # Simulate operations
    let results = [
        { operation: "auth", success: true },
        { operation: "process", success: true },
        { operation: "notify", success: true }
    ]
    
    # Validate results
    assert (ulid validate $correlation_id) "Correlation ID should be valid"
    assert (($results | all { |r| $r.success }) == true) "All operations should succeed"
    
    print "✅ End-to-end workflow test passed"
}
```

### Performance Testing
```nu
def test_performance [operation: closure, data_sizes: list] {
    $data_sizes | each { |size|
        let test_data = 0..$size | each { ulid generate }
        let start = date now | into int
        
        let result = do $operation $test_data
        
        let end = date now | into int
        let duration = $end - $start
        
        {
            data_size: $size,
            duration_ms: $duration,
            throughput: ($size / ($duration / 1000)),
            success: ($result | length) == $size
        }
    }
}
```

## Troubleshooting Common Issues

### 1. Invalid ULID Errors
```nu
# Problem: Getting "Invalid ULID format" errors
# Solution: Validate ULIDs before processing
def safe_ulid_parse [ulid: string] {
    if not (ulid validate $ulid) {
        print $"Invalid ULID: ($ulid)"
        return null
    }
    ulid parse $ulid
}
```

### 2. Memory Issues with Large Datasets
```nu
# Problem: Running out of memory with large datasets
# Solution: Use streaming and reduce batch sizes
let large_dataset = open huge_file.json | get ulids
$large_dataset | chunks 100 | each { |chunk|
    $chunk | ulid stream validate --batch-size 50
} | flatten
```

### 3. Performance Issues
```nu
# Problem: Slow processing
# Solution: Enable parallel processing and optimize batch sizes
$data | ulid stream parse --parallel --batch-size 2000
```

## Template Customization

### Adding New Commands
1. Add command function to the template
2. Update the `main` function to handle the new command
3. Add help text and examples
4. Include error handling and validation

### Extending Configuration
```nu
# Add custom configuration options
const CUSTOM_CONFIG = {
    # Merge with existing CONFIG
    ($CONFIG),
    
    # Add custom settings
    custom_setting: "value",
    advanced_options: {
        feature_enabled: true,
        threshold: 100
    }
}
```

### Adding Custom Validation
```nu
def custom_validator [data: any] {
    # Implement custom validation logic
    if not (validate_custom_rules $data) {
        error make { msg: "Custom validation failed" }
    }
    
    # Return validated data
    $data
}
```

## Integration Examples

### With External APIs
```nu
def call_external_api [endpoint: string, data: record] {
    let correlation_id = ulid generate --context "external-api"
    
    let response = http post $endpoint {
        body: $data,
        headers: {
            "X-Correlation-ID": $correlation_id,
            "Content-Type": "application/json"
        }
    }
    
    { correlation_id: $correlation_id, response: $response }
}
```

### With Databases
```nu
def insert_with_ulid [table: string, data: record] {
    let id = ulid generate --context "database"
    let record_with_id = $data | upsert id $id | upsert created_at (date now)
    
    # Simulate database insert
    print $"INSERT INTO ($table) VALUES ($record_with_id | to json)"
    
    $record_with_id
}
```

### With File Processing
```nu
def process_file_with_tracking [file_path: string] {
    let processing_id = ulid generate --context "file-processing"
    
    let result = {
        processing_id: $processing_id,
        file_path: $file_path,
        start_time: (date now),
        records_processed: 0,
        errors: []
    }
    
    # Process file and update result
    $result
}
```

---

For more detailed information, see the individual template files and the main [Templates README](README.md).