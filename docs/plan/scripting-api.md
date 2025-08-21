# Nushell Scripting API Design

**Phase 3.4: Scripting API & Automation Support**  
**Goal**: Enable comprehensive automation and user script integration

## Overview

The nu_plugin_nw_ulid scripting API provides complete programmatic access to ULID functionality for Nushell users building automation workflows, data processing pipelines, and custom scripts.

## Design Principles

### 1. **Pipeline-First Design**
All commands designed for natural pipeline integration:
```nushell
# Data flows naturally through ULID operations
$records | insert id { ulid generate } | save output.json
```

### 2. **Batch-Friendly Operations**
Efficient bulk processing for automation:
```nushell
# Generate multiple ULIDs efficiently
1..1000 | each { ulid generate }
```

### 3. **Error-Resilient Patterns**
Graceful error handling for production scripts:
```nushell
# Validate with fallback behavior
$data | where { |row| ($row.ulid | ulid validate) }
```

## Core Scripting Patterns

### 1. **Data Pipeline Integration**

#### Record Enhancement
```nushell
# Add ULID to existing records
def add-ulids [data] {
    $data | each { |record|
        $record | insert ulid { ulid generate }
    }
}

# Usage
$customer_data | add-ulids | save customers-with-ids.json
```

#### Correlation ID Generation
```nushell
# Generate correlation IDs for log processing
def correlate-logs [logs] {
    $logs | group-by session_id | each { |group|
        let correlation_id = (ulid generate)
        $group.items | each { |log|
            $log | insert correlation_id $correlation_id
        }
    } | flatten
}
```

### 2. **Batch Processing Utilities**

#### Bulk Generation
```nushell
# Generate specified number of ULIDs
def generate-ulids [count: int] {
    1..$count | each { ulid generate }
}

# Generate with custom timestamps
def generate-historical-ulids [timestamps: list] {
    $timestamps | each { |ts|
        ulid generate --timestamp $ts
    }
}
```

#### Data Validation Workflows
```nushell
# Validate ULID fields in dataset
def validate-ulid-data [data, field: string] {
    let invalid = ($data | where { |row|
        ($row | get $field | ulid validate) == false
    })
    
    if ($invalid | length) > 0 {
        error make {
            msg: $"Found ($invalid | length) invalid ULIDs in field ($field)"
            span: (metadata $data).span
        }
    }
    
    $data
}
```

### 3. **Automation Helpers**

#### Database Preparation
```nushell
# Prepare records for database insertion
def prepare-for-insert [records] {
    $records | each { |record|
        $record 
        | insert id { ulid generate }
        | insert created_at { date now | date to-timezone UTC }
    }
}
```

#### File Processing Workflows
```nushell
# Process files with ULID naming
def process-files-with-ulids [directory: string] {
    ls $directory | each { |file|
        let ulid_name = $"($ulid generate).($file.name | path extension)"
        let output_path = ($file.name | path dirname) | path join $ulid_name
        
        # Process file and save with ULID name
        {
            original: $file.name,
            processed: $output_path,
            ulid: ($ulid_name | path parse | get stem)
        }
    }
}
```

## Advanced Integration Patterns

### 1. **Time-Series Data**

#### Time-Ordered ID Generation
```nushell
# Generate ULIDs maintaining chronological order
def generate-time-series-ids [events] {
    $events | each { |event|
        $event | insert ulid { 
            ulid generate --timestamp $event.timestamp 
        }
    }
}
```

#### Historical Data Migration
```nushell
# Migrate legacy data with historical ULIDs
def migrate-with-ulids [legacy_data] {
    $legacy_data | each { |record|
        let historical_ulid = (ulid generate --timestamp $record.created_date)
        $record | insert new_id $historical_ulid
    }
}
```

### 2. **Error Handling & Recovery**

#### Robust Validation Pipeline
```nushell
# Complete validation with detailed reporting
def validate-and-report [data, ulid_fields: list] {
    let results = ($ulid_fields | each { |field|
        let invalid_records = ($data | enumerate | where { |item|
            ($item.item | get $field | ulid validate) == false
        })
        
        {
            field: $field,
            invalid_count: ($invalid_records | length),
            invalid_indices: ($invalid_records | get index)
        }
    })
    
    let total_errors = ($results | get invalid_count | math sum)
    
    if $total_errors > 0 {
        print $"Validation failed: ($total_errors) total errors"
        $results | where invalid_count > 0
    } else {
        print "All ULID validations passed"
        $data
    }
}
```

### 3. **Performance Optimization**

#### Streaming Processing
```nushell
# Process large datasets in chunks
def process-large-dataset [input_file: string, chunk_size: int = 1000] {
    open $input_file 
    | enumerate 
    | group-by { |item| ($item.index // $chunk_size) }
    | each { |chunk|
        $chunk.items 
        | get item
        | each { |record| $record | insert ulid { ulid generate } }
    }
    | flatten
}
```

## Testing Patterns

### 1. **ULID Property Testing**
```nushell
# Verify ULID properties in test data
def test-ulid-properties [ulids: list] {
    # Test 1: All valid format
    let format_test = ($ulids | all { |ulid| $ulid | ulid validate })
    
    # Test 2: Lexicographic ordering matches timestamp ordering
    let sorted_by_ulid = ($ulids | sort)
    let sorted_by_timestamp = ($ulids | each { |ulid| 
        $ulid | ulid parse | get timestamp 
    } | sort)
    
    {
        all_valid_format: $format_test,
        maintains_time_order: ($sorted_by_ulid == $sorted_by_timestamp)
    }
}
```

### 2. **Performance Benchmarks**
```nushell
# Benchmark ULID generation performance
def benchmark-generation [count: int] {
    let start_time = (date now)
    let ulids = (1..$count | each { ulid generate })
    let end_time = (date now)
    let duration = (($end_time - $start_time) | into int)
    
    {
        count: $count,
        duration_ms: $duration,
        rate_per_second: ($count * 1000 / $duration)
    }
}
```

## Documentation Examples

### 1. **Quick Start Templates**
```nushell
# Template: Add ULIDs to JSON data
cat data.json 
| from json 
| each { |item| $item | insert id { ulid generate } }
| to json 
| save data-with-ulids.json

# Template: Validate existing ULIDs
cat data-with-ulids.json
| from json
| where { |item| ($item.id | ulid validate) == false }
```

### 2. **Common Workflows**
```nushell
# Workflow 1: Database preparation
def prepare-user-data [] {
    open users.csv
    | each { |user|
        $user 
        | insert user_id { ulid generate }
        | insert created_at { date now | format date iso8601 }
    }
    | to json
    | save users-prepared.json
}

# Workflow 2: Log correlation
def correlate-request-logs [logs] {
    $logs 
    | group-by request_id
    | each { |group|
        let trace_id = (ulid generate)
        $group.items | each { insert trace_id $trace_id }
    }
    | flatten
}
```

## API Design Guidelines

### 1. **Command Composition**
- All commands work naturally in pipelines
- Support both single values and lists/streams
- Consistent parameter naming across commands

### 2. **Error Propagation**
- Commands fail gracefully with helpful error messages
- Support `try`/`catch` patterns for error handling
- Provide validation commands for pre-checking data

### 3. **Performance Considerations**
- Bulk operations optimized for large datasets
- Streaming support for memory-efficient processing
- Benchmark utilities for performance monitoring

## Implementation Requirements

### Phase 3.4 Deliverables
- [ ] Complete scripting pattern documentation
- [ ] Template script collection
- [ ] Performance optimization for bulk operations
- [ ] Error handling best practices guide
- [ ] Integration testing with real-world workflows
- [ ] User cookbook with common automation patterns

This scripting API design ensures that nu_plugin_nw_ulid becomes a powerful tool for automation and data processing workflows, not just interactive command-line usage.