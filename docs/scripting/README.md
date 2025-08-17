# Nushell ULID Scripting Guide

**Version**: 0.1.0  
**Last Updated**: August 17, 2025  
**Target Audience**: Script developers, automation engineers, DevOps teams

This comprehensive guide demonstrates how to integrate ULID functionality into your Nushell scripts for powerful automation workflows. It covers everything from basic operations to advanced enterprise patterns, performance optimization, and security best practices.

## Table of Contents

1. [Quick Start Patterns](#quick-start-patterns)
2. [Data Processing Workflows](#data-processing-workflows)
3. [Enterprise Automation Patterns](#enterprise-automation-patterns)
4. [Performance Optimization](#performance-optimization)
5. [Security Best Practices](#security-best-practices)
6. [Real-World Use Cases](#real-world-use-cases)
7. [Advanced Integration Patterns](#advanced-integration-patterns)
8. [Troubleshooting & Debugging](#troubleshooting--debugging)

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

## Enterprise Automation Patterns

### Service Orchestration

```nu
# Orchestrate microservices with ULID correlation tracking
def orchestrate_service_chain [request_data: record] {
    let correlation_id = ulid generate --context "service-orchestration"
    
    # Step 1: Authentication service
    let auth_result = call_auth_service $request_data $correlation_id
    
    # Step 2: Business logic service
    let business_result = if $auth_result.success {
        call_business_service $request_data $correlation_id
    } else {
        { success: false, reason: "Authentication failed" }
    }
    
    # Step 3: Notification service
    let notification_result = if $business_result.success {
        call_notification_service $business_result $correlation_id
    } else {
        { success: false, reason: "Business logic failed" }
    }
    
    # Generate execution report
    {
        correlation_id: $correlation_id,
        start_time: (ulid parse $correlation_id | get timestamp.iso8601),
        steps: [
            { service: "auth", success: $auth_result.success, duration: $auth_result.duration },
            { service: "business", success: $business_result.success, duration: $business_result.duration },
            { service: "notification", success: $notification_result.success, duration: $notification_result.duration }
        ],
        overall_success: ($auth_result.success and $business_result.success and $notification_result.success)
    }
}

def call_auth_service [data: record, correlation_id: string] {
    let start = date now | into int
    
    # Simulate API call with correlation ID
    let result = http post "https://auth.service.com/validate" {
        headers: { "X-Correlation-ID": $correlation_id },
        body: $data
    }
    
    let end = date now | into int
    
    {
        success: ($result.status == 200),
        duration: ($end - $start),
        correlation_id: $correlation_id,
        response: $result
    }
}
```

### Event-Driven Automation

```nu
# Process events with ULID-based event sourcing
def process_event_stream [events: list] {
    let processed_events = $events | each { |event|
        # Generate event ID if not present
        let event_id = if "id" in $event {
            $event.id
        } else {
            ulid generate --timestamp $event.timestamp
        }
        
        # Validate event ID
        if not (ulid validate $event_id) {
            error make { msg: $"Invalid event ID: ($event_id)" }
        }
        
        # Enrich event with metadata
        let enriched_event = $event 
            | upsert id $event_id
            | upsert processed_at (date now | into int)
            | upsert age_seconds ((date now | into int) - (ulid parse $event_id | get timestamp.unix))
        
        # Route event based on type
        match $enriched_event.type {
            "user_signup" => process_user_signup $enriched_event,
            "order_created" => process_order_created $enriched_event,
            "payment_processed" => process_payment $enriched_event,
            _ => { error: $"Unknown event type: ($enriched_event.type)" }
        }
    }
    
    # Generate processing summary
    let successful = $processed_events | where error == null
    let failed = $processed_events | where error != null
    
    {
        total_events: ($events | length),
        successful: ($successful | length),
        failed: ($failed | length),
        processing_rate: (($successful | length) / ($events | length) * 100),
        events: $processed_events
    }
}

def process_user_signup [event: record] {
    print $"Processing user signup: ($event.id)"
    
    # Create user record with ULID
    let user_id = ulid generate --context "user-registration"
    
    # Store user data
    {
        event_id: $event.id,
        user_id: $user_id,
        action: "user_created",
        timestamp: (date now | into int),
        success: true,
        error: null
    }
}
```

### Batch Job Management

```nu
# Manage batch jobs with ULID-based job tracking
def run_batch_job [job_definition: record] {
    let job_id = ulid generate --context "batch-processing"
    let start_time = date now | into int
    
    print $"Starting batch job ($job_id): ($job_definition.name)"
    
    # Initialize job status
    let job_status = {
        job_id: $job_id,
        name: $job_definition.name,
        status: "running",
        start_time: $start_time,
        progress: 0,
        total_items: ($job_definition.input_data | length),
        processed_items: 0,
        failed_items: 0,
        error_log: []
    }
    
    # Process items in batches
    let batch_size = $job_definition.batch_size | default 100
    let total_batches = (($job_definition.input_data | length) / $batch_size | math ceil)
    
    mut current_status = $job_status
    
    for batch_num in 0..<$total_batches {
        let start_idx = $batch_num * $batch_size
        let end_idx = (($start_idx + $batch_size) | math min ($job_definition.input_data | length))
        let batch_data = ($job_definition.input_data | skip $start_idx | first ($end_idx - $start_idx))
        
        print $"Processing batch ($batch_num + 1)/($total_batches): ($batch_data | length) items"
        
        # Process batch
        let batch_results = $batch_data | each { |item|
            try {
                let result = (do $job_definition.processor $item)
                { item: $item, success: true, result: $result, error: null }
            } catch { |e|
                { item: $item, success: false, result: null, error: $e.msg }
            }
        }
        
        # Update job status
        let successful_items = $batch_results | where success == true | length
        let failed_items = $batch_results | where success == false | length
        
        $current_status = $current_status 
            | upsert processed_items ($current_status.processed_items + $successful_items)
            | upsert failed_items ($current_status.failed_items + $failed_items)
            | upsert progress (($current_status.processed_items + $failed_items) / $current_status.total_items * 100)
            | upsert error_log ([$current_status.error_log, ($batch_results | where success == false | get error)] | flatten)
    }
    
    # Finalize job status
    let end_time = date now | into int
    let final_status = $current_status 
        | upsert status "completed"
        | upsert end_time $end_time
        | upsert duration_seconds ($end_time - $start_time)
        | upsert success_rate (($current_status.processed_items / $current_status.total_items) * 100)
    
    print $"Batch job ($job_id) completed: ($final_status.processed_items)/($final_status.total_items) items processed"
    
    $final_status
}
```

## Performance Optimization

### Memory-Efficient Streaming

```nu
# Process large ULID datasets without memory exhaustion
def stream_process_large_dataset [file_path: string, processor: closure] {
    let chunk_size = 1000
    mut total_processed = 0
    mut errors = []
    
    # Process file in chunks
    open $file_path 
        | lines 
        | chunks $chunk_size 
        | each { |chunk|
            let chunk_results = $chunk 
                | ulid stream validate --batch-size 500 --continue-on-error
                | where valid == true
                | get ulid
                | each $processor
            
            $total_processed = $total_processed + ($chunk_results | length)
            
            if ($total_processed % 10000) == 0 {
                print $"Processed ($total_processed) items..."
            }
            
            $chunk_results
        }
        | flatten
}

# Example usage
let processor = { |ulid|
    let parsed = ulid parse $ulid
    {
        ulid: $ulid,
        timestamp: $parsed.timestamp.milliseconds,
        hour: ($parsed.timestamp.iso8601 | str substring 11..13)
    }
}

let results = stream_process_large_dataset "large_ulid_file.txt" $processor
```

### Parallel Processing Optimization

```nu
# Distribute ULID processing across multiple cores
def parallel_ulid_processor [ulids: list, operation: string, num_workers: int = 4] {
    let chunk_size = (($ulids | length) / $num_workers | math ceil)
    
    $ulids 
        | chunks $chunk_size
        | par-each { |chunk|
            match $operation {
                "validate" => $chunk | ulid stream validate --parallel,
                "parse" => $chunk | ulid stream parse --parallel,
                "analyze" => $chunk | each { |ulid| ulid inspect $ulid --stats },
                _ => error make { msg: $"Unknown operation: ($operation)" }
            }
        }
        | flatten
}

# Performance benchmarking
def benchmark_ulid_operations [ulids: list] {
    let operations = [
        { name: "validate", func: { $ulids | each { ulid validate $in } } },
        { name: "validate_stream", func: { $ulids | ulid stream validate } },
        { name: "parse", func: { $ulids | each { ulid parse $in } } },
        { name: "parse_stream", func: { $ulids | ulid stream parse } }
    ]
    
    $operations | each { |op|
        let start = date now | into int
        let result = do $op.func
        let end = date now | into int
        
        {
            operation: $op.name,
            duration_ms: ($end - $start),
            items_processed: ($result | length),
            throughput: (($result | length) / (($end - $start) / 1000) | math round)
        }
    }
}
```

## Security Best Practices

### Secure ULID Generation

```nu
# Generate ULIDs with security context validation
def secure_ulid_generator [context: string, count: int = 1] {
    # Check security context
    let security_advice = ulid security-advice --context $context
    
    if ($security_advice.warning_level == "high") {
        print $"WARNING: High security risk for context: ($context)"
        print $"Recommendations: ($security_advice.recommendations)"
        
        let confirmation = input "Continue with ULID generation? (y/N): "
        if $confirmation != "y" {
            error make { msg: "ULID generation cancelled due to security concerns" }
        }
    }
    
    # Generate ULIDs with security tracking
    let ulids = if $count == 1 {
        [ulid generate --context $context]
    } else {
        ulid generate-stream $count --context $context
    }
    
    # Log security event
    let audit_entry = {
        timestamp: (date now | into int),
        action: "ulid_generation",
        context: $context,
        count: $count,
        security_level: $security_advice.warning_level,
        user: (whoami),
        ulids: $ulids
    }
    
    # Store audit log (implement your audit storage)
    $audit_entry | to json | save --append security_audit.jsonl
    
    $ulids
}

# Validate ULIDs with security checks
def secure_ulid_validator [ulids: list, context: string] {
    $ulids | each { |ulid|
        let basic_validation = ulid validate $ulid
        let security_inspection = ulid inspect $ulid --security-check
        
        {
            ulid: $ulid,
            valid: $basic_validation,
            security_risk: $security_inspection.security.predictability_risk,
            context_appropriate: ($security_inspection.security.context_warnings | is-empty),
            recommendations: $security_inspection.security.recommendations
        }
    }
}
```

### Audit Trail Implementation

```nu
# Comprehensive audit trail for ULID operations
const AUDIT_LOG_PATH = "ulid_audit.jsonl"

def log_ulid_operation [operation: string, ulids: list, context: string, metadata: record] {
    let audit_id = ulid generate --context "audit-logging"
    let timestamp = date now
    
    let audit_entry = {
        audit_id: $audit_id,
        timestamp: $timestamp,
        operation: $operation,
        context: $context,
        ulid_count: ($ulids | length),
        ulids_sample: ($ulids | first 5),  # Store only first 5 for privacy
        metadata: $metadata,
        user: (whoami),
        session_id: $env.SESSION_ID? | default "unknown",
        checksum: ([$operation, ($ulids | str join ","), $context] | str join "|" | hash sha256)
    }
    
    $audit_entry | to json | save --append $AUDIT_LOG_PATH
    
    print $"Audit logged: ($audit_id)"
}

# Query audit trail
def query_audit_trail [--operation: string, --context: string, --user: string, --since: string] {
    let entries = open $AUDIT_LOG_PATH | lines | each { from json }
    
    let filtered = $entries | where {
        let entry = $in
        
        (if $operation != null { $entry.operation == $operation } else { true }) and
        (if $context != null { $entry.context == $context } else { true }) and
        (if $user != null { $entry.user == $user } else { true }) and
        (if $since != null { $entry.timestamp > ($since | into datetime) } else { true })
    }
    
    $filtered | sort-by timestamp
}
```

## Real-World Use Cases

### API Request Tracking

```nu
# Track API requests across microservices
def api_request_tracker [] {
    # Generate correlation ID for request chain
    let correlation_id = ulid generate --context "api-request"
    
    # Mock API call chain
    let api_calls = [
        { service: "auth", endpoint: "/validate", method: "POST" },
        { service: "user", endpoint: "/profile", method: "GET" },
        { service: "billing", endpoint: "/calculate", method: "POST" },
        { service: "notification", endpoint: "/send", method: "POST" }
    ]
    
    let results = $api_calls | each { |call|
        let request_id = ulid generate --context "api-request"
        let start_time = date now | into int
        
        # Simulate API call
        let response = {
            status: (if (random bool) { 200 } else { 500 }),
            duration: (random int 50..2000),
            data: { message: "success" }
        }
        
        let end_time = date now | into int
        
        {
            correlation_id: $correlation_id,
            request_id: $request_id,
            service: $call.service,
            endpoint: $call.endpoint,
            method: $call.method,
            status: $response.status,
            duration_ms: ($end_time - $start_time),
            start_time: $start_time,
            end_time: $end_time,
            success: ($response.status < 400)
        }
    }
    
    # Analyze request chain
    let total_duration = ($results | get duration_ms | math sum)
    let success_count = ($results | where success | length)
    let failure_count = ($results | where not success | length)
    
    {
        correlation_id: $correlation_id,
        total_calls: ($results | length),
        successful_calls: $success_count,
        failed_calls: $failure_count,
        total_duration_ms: $total_duration,
        success_rate: ($success_count / ($results | length) * 100),
        call_details: $results
    }
}
```

### Database Record Management

```nu
# Manage database records with ULID primary keys
def database_record_manager [] {
    # Create sample records
    let records = [
        { name: "Product A", category: "electronics", price: 299.99 },
        { name: "Product B", category: "books", price: 19.99 },
        { name: "Product C", category: "clothing", price: 49.99 }
    ]
    
    # Add ULIDs and metadata
    let records_with_ids = $records | each { |record|
        let id = ulid generate --context "database-record"
        let timestamp = ulid parse $id | get timestamp
        
        $record 
            | upsert id $id
            | upsert created_at $timestamp.iso8601
            | upsert created_timestamp $timestamp.milliseconds
            | upsert updated_at $timestamp.iso8601
            | upsert version 1
    }
    
    # Simulate database operations
    {
        operation: "bulk_insert",
        records: $records_with_ids,
        count: ($records_with_ids | length),
        ids: ($records_with_ids | get id)
    }
}

# Update records with version tracking
def update_record [record_id: string, updates: record] {
    # Validate record ID
    if not (ulid validate $record_id) {
        error make { msg: $"Invalid record ID: ($record_id)" }
    }
    
    # Create update timestamp
    let update_timestamp = date now
    
    # Return updated record structure
    $updates 
        | upsert id $record_id
        | upsert updated_at ($update_timestamp | into int)
        | upsert version ($updates.version + 1)
        | upsert update_history ([
            $updates.update_history? | default [],
            {
                timestamp: $update_timestamp,
                changes: $updates,
                updated_by: (whoami)
            }
        ] | flatten)
}
```

### Log File Analysis

```nu
# Analyze log files with ULID request IDs
def analyze_log_patterns [log_file: string] {
    let log_entries = open $log_file | lines | each { from json }
    
    # Filter entries with valid ULIDs
    let valid_entries = $log_entries 
        | where ("request_id" in $in)
        | where { ulid validate $in.request_id }
    
    # Parse ULID timestamps for analysis
    let entries_with_timestamps = $valid_entries | each { |entry|
        let parsed = ulid parse $entry.request_id
        $entry 
            | upsert ulid_timestamp $parsed.timestamp.milliseconds
            | upsert ulid_age_seconds ((date now | into int) - $parsed.timestamp.unix)
    }
    
    # Group by time periods
    let hourly_stats = $entries_with_timestamps 
        | group-by { |entry| 
            $entry.ulid_timestamp // 3600000 * 3600000  # Group by hour
        }
        | transpose hour entries
        | each { |group|
            {
                hour: ($group.hour | into datetime | format date "%Y-%m-%d %H:00"),
                count: ($group.entries | length),
                error_rate: (($group.entries | where level == "error" | length) / ($group.entries | length) * 100),
                avg_response_time: ($group.entries | get response_time | math avg),
                unique_users: ($group.entries | get user_id | uniq | length)
            }
        }
    
    {
        total_entries: ($valid_entries | length),
        time_span_hours: (($entries_with_timestamps | get ulid_timestamp | math max) - ($entries_with_timestamps | get ulid_timestamp | math min)) / 3600000,
        hourly_breakdown: $hourly_stats,
        top_errors: ($entries_with_timestamps | where level == "error" | group-by error_code | transpose error count | sort-by count --reverse | first 10)
    }
}
```

## Advanced Integration Patterns

### CI/CD Pipeline Integration

```nu
# Generate build artifacts with ULID tracking
def generate_build_artifacts [project_info: record] {
    let build_id = ulid generate --context "ci-cd-build"
    let timestamp = date now
    
    # Create build manifest
    let build_manifest = {
        build_id: $build_id,
        project: $project_info.name,
        version: $project_info.version,
        branch: $project_info.branch,
        commit: $project_info.commit,
        timestamp: $timestamp,
        artifacts: []
    }
    
    # Generate artifact IDs
    let artifacts = [
        { type: "binary", name: $"($project_info.name).exe", size: 1024000 },
        { type: "documentation", name: "docs.tar.gz", size: 512000 },
        { type: "metadata", name: "manifest.json", size: 2048 }
    ]
    
    let artifacts_with_ids = $artifacts | each { |artifact|
        $artifact | upsert id (ulid generate --context "build-artifact")
    }
    
    $build_manifest | upsert artifacts $artifacts_with_ids
}

# Deploy with tracking
def deploy_with_tracking [build_manifest: record, environment: string] {
    let deployment_id = ulid generate --context "deployment"
    
    {
        deployment_id: $deployment_id,
        build_id: $build_manifest.build_id,
        environment: $environment,
        deployed_at: (date now),
        artifacts_deployed: ($build_manifest.artifacts | get id),
        status: "deployed"
    }
}
```

### Monitoring and Alerting

```nu
# Monitor system health with ULID-based event tracking
def system_health_monitor [] {
    let check_id = ulid generate --context "health-check"
    
    # Perform health checks
    let checks = [
        { name: "database", func: { check_database_health } },
        { name: "api", func: { check_api_health } },
        { name: "cache", func: { check_cache_health } },
        { name: "storage", func: { check_storage_health } }
    ]
    
    let results = $checks | each { |check|
        let start = date now | into int
        let result = try {
            do $check.func
        } catch { |e|
            { healthy: false, error: $e.msg }
        }
        let end = date now | into int
        
        {
            check_id: (ulid generate --context "health-check-item"),
            name: $check.name,
            healthy: $result.healthy,
            duration_ms: ($end - $start),
            timestamp: $start,
            error: ($result.error? | default null)
        }
    }
    
    let overall_health = ($results | all { |check| $check.healthy })
    
    {
        check_id: $check_id,
        timestamp: (date now),
        overall_healthy: $overall_health,
        individual_checks: $results,
        summary: {
            healthy_checks: ($results | where healthy | length),
            total_checks: ($results | length),
            avg_response_time: ($results | get duration_ms | math avg)
        }
    }
}

def check_database_health [] {
    # Simulate database health check
    { healthy: (random bool), response_time: (random int 10..500) }
}
```

## Troubleshooting & Debugging

### Debug ULID Operations

```nu
# Debug ULID parsing issues
def debug_ulid_parsing [ulid: string] {
    print $"Debugging ULID: ($ulid)"
    
    # Basic validation
    let valid = ulid validate $ulid
    print $"Valid format: ($valid)"
    
    if not $valid {
        print "Validation failed - checking common issues:"
        
        # Check length
        let length = ($ulid | str length)
        print $"Length: ($length) (expected: 26)"
        
        # Check characters
        let invalid_chars = ($ulid | split chars | where $in not-in ("0123456789ABCDEFGHJKMNPQRSTVWXYZ" | split chars))
        if ($invalid_chars | length) > 0 {
            print $"Invalid characters found: ($invalid_chars)"
        }
        
        return null
    }
    
    # Parse and inspect
    let parsed = ulid parse $ulid
    let inspection = ulid inspect $ulid --stats
    
    {
        ulid: $ulid,
        valid: $valid,
        parsed: $parsed,
        inspection: $inspection
    }
}

# Performance debugging
def debug_performance [operation: closure, data: list] {
    let sizes = [10, 100, 1000, 10000]
    
    $sizes | each { |size|
        let test_data = $data | first $size
        let start = date now | into int
        
        let result = do $operation $test_data
        
        let end = date now | into int
        let duration = $end - $start
        
        {
            data_size: $size,
            duration_ms: $duration,
            throughput: ($size / ($duration / 1000)),
            memory_efficient: ($duration < ($size * 0.1))  # Rule of thumb
        }
    }
}
```

## Performance Tips

1. **Use streaming commands** for large datasets (>1000 items)
2. **Enable parallel processing** for CPU-intensive operations
3. **Use batch processing** to manage memory usage
4. **Cache parsed ULID components** when processing the same ULIDs multiple times
5. **Implement adaptive batch sizing** based on dataset size and available resources
6. **Monitor performance metrics** and adjust parameters based on actual usage patterns

## See Also

- [API Reference](api.md) - Complete command documentation with detailed specifications
- [User Guide](../USER_GUIDE.md) - Comprehensive user documentation with examples
- [Developer Guide](../DEVELOPER_GUIDE.md) - Internal architecture and contribution guidelines
- [Examples](examples/) - Ready-to-use script templates for common scenarios
- [Helpers](helpers.nu) - Reusable utility functions and automation helpers
- [Security Guide](../security/audit-report.md) - Security best practices and audit results
- [Performance Report](../quality/phase4-completion-report.md) - Detailed performance analysis and benchmarks