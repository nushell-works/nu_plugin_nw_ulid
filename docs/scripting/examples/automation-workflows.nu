#!/usr/bin/env nu
# Automation Workflows Example with ULID Plugin
# Demonstrates real-world automation scenarios using ULIDs

# Source helper functions
source ../helpers.nu

def main [] {
    print "ULID Automation Workflows"
    print "========================="
    
    # Example 1: Database record processing
    database_record_processing
    
    # Example 2: Log file analysis
    log_file_analysis
    
    # Example 3: API request tracking
    api_request_tracking
    
    # Example 4: Data synchronization
    data_synchronization
}

# Example 1: Automated database record processing
def database_record_processing [] {
    print "\n1. Database Record Processing Workflow"
    print "--------------------------------------"
    
    # Simulate database records that need ULID assignment
    let raw_records = [
        { name: "Product A", category: "electronics", price: 299.99 }
        { name: "Product B", category: "books", price: 19.99 }
        { name: "Product C", category: "electronics", price: 599.99 }
        { name: "Product D", category: "clothing", price: 49.99 }
        { name: "Product E", category: "books", price: 24.99 }
    ]
    
    print $"Processing ($raw_records | length) database records..."
    
    # Step 1: Assign ULIDs to records
    let records_with_ids = (add_ulids_to_records $raw_records)
    print "✓ ULIDs assigned to all records"
    
    # Step 2: Validate all assigned IDs
    let validation_check = ($records_with_ids | each { |record|
        $record | upsert id_valid (ulid validate $record.id)
    })
    
    let valid_count = ($validation_check | where id_valid | length)
    print $"✓ Validation complete: ($valid_count)/($records_with_ids | length) records have valid IDs"
    
    # Step 3: Add timestamp metadata for auditing
    let records_with_metadata = ($validation_check | each { |record|
        if $record.id_valid {
            let parsed = (ulid parse $record.id)
            $record 
            | upsert created_at $parsed.timestamp.iso8601
            | upsert created_timestamp $parsed.timestamp.milliseconds
        } else {
            $record | upsert created_at null | upsert created_timestamp null
        }
    })
    
    print "✓ Timestamp metadata added"
    
    # Step 4: Sort by creation time and categorize
    let processed_records = ($records_with_metadata 
        | ulid sort --column id
        | group-by category
    )
    
    print $"✓ Records processed and grouped by category: ($processed_records | columns | length) categories"
    
    # Step 5: Generate summary report
    let summary = ($processed_records | transpose category records | each { |group|
        {
            category: $group.category,
            count: ($group.records | length),
            total_value: ($group.records | get price | math sum),
            earliest_id: ($group.records | first | get id),
            latest_id: ($group.records | last | get id)
        }
    })
    
    print $"📊 Processing Summary: ($summary)"
}

# Example 2: Log file analysis automation
def log_file_analysis [] {
    print "\n2. Log File Analysis Workflow"
    print "-----------------------------"
    
    # Simulate log entries with request IDs (ULIDs)
    let log_entries = (generate_test_ulids 15 | each { |request_id|
        {
            request_id: $request_id,
            method: (["GET", "POST", "PUT", "DELETE"] | get (random int 0..3)),
            endpoint: (["/api/users", "/api/products", "/api/orders", "/api/auth"] | get (random int 0..3)),
            status_code: (["200", "201", "400", "404", "500"] | get (random int 0..4)),
            response_time: (random int 10..5000),
            user_id: $"user_((random int 1000..9999))"
        }
    })
    
    print $"Analyzing ($log_entries | length) log entries..."
    
    # Step 1: Sort logs chronologically by request ID
    let sorted_logs = ($log_entries | ulid sort --column request_id)
    print "✓ Logs sorted chronologically"
    
    # Step 2: Extract time periods for analysis
    let logs_with_time = ($sorted_logs | each { |log|
        let parsed = (ulid parse $log.request_id)
        $log 
        | upsert timestamp $parsed.timestamp.iso8601
        | upsert hour ($parsed.timestamp.iso8601 | str substring 11..13 | into int)
    })
    
    # Step 3: Analyze by time periods
    let hourly_analysis = ($logs_with_time | group-by hour | transpose hour logs | each { |group|
        let logs = $group.logs
        {
            hour: $group.hour,
            request_count: ($logs | length),
            avg_response_time: ($logs | get response_time | math avg | math round),
            error_rate: (($logs | where status_code in ["400", "404", "500"] | length) / ($logs | length) * 100 | math round),
            endpoints: ($logs | get endpoint | uniq | length)
        }
    })
    
    print $"📈 Hourly Analysis: ($hourly_analysis)"
    
    # Step 4: Identify slow requests using ULID timestamps
    let slow_requests = ($logs_with_time | where response_time > 1000 | ulid sort --column request_id)
    print $"🐌 Slow requests identified: ($slow_requests | length) requests > 1000ms"
    
    # Step 5: Generate time-series data for monitoring
    let time_series = ($logs_with_time | each { |log|
        let parsed = (ulid parse $log.request_id)
        {
            timestamp_ms: $parsed.timestamp.milliseconds,
            response_time: $log.response_time,
            is_error: ($log.status_code in ["400", "404", "500"])
        }
    } | sort-by timestamp_ms)
    
    print $"📊 Time-series data prepared: ($time_series | length) data points"
}

# Example 3: API request tracking automation
def api_request_tracking [] {
    print "\n3. API Request Tracking Workflow"
    print "--------------------------------"
    
    # Simulate API requests with correlation IDs
    let base_time = (date now | into int)
    let correlation_ids = (ulid generate-stream 20 --timestamp $base_time --unique-timestamps)
    
    let api_requests = ($correlation_ids | enumerate | each { |row|
        let correlation_id = $row.item
        let request_num = $row.index
        {
            correlation_id: $correlation_id,
            service: (["auth", "user", "order", "payment", "notification"] | get ($request_num mod 5)),
            operation: (["create", "read", "update", "delete"] | get ($request_num mod 4)),
            duration_ms: (random int 50..2000),
            success: (random bool)
        }
    })
    
    print $"Tracking ($api_requests | length) API requests..."
    
    # Step 1: Group requests by service and analyze patterns
    let service_analysis = ($api_requests | group-by service | transpose service requests | each { |group|
        let requests = $group.requests
        {
            service: $group.service,
            total_requests: ($requests | length),
            avg_duration: ($requests | get duration_ms | math avg | math round),
            success_rate: (($requests | where success | length) / ($requests | length) * 100 | math round),
            correlation_ids: ($requests | get correlation_id | first 3)
        }
    })
    
    print $"🔍 Service Analysis: ($service_analysis)"
    
    # Step 2: Track request timeline using ULID ordering
    let request_timeline = ($api_requests 
        | ulid sort --column correlation_id 
        | enumerate 
        | each { |row|
            let request = $row.item
            let position = $row.index + 1
            let parsed = (ulid parse $request.correlation_id)
            {
                position: $position,
                correlation_id: $request.correlation_id,
                service: $request.service,
                timestamp: $parsed.timestamp.human,
                duration_ms: $request.duration_ms,
                success: $request.success
            }
        }
    )
    
    print $"⏱️  Request timeline prepared: ($request_timeline | length) sequential requests"
    
    # Step 3: Identify request chains and dependencies
    let request_chains = ($request_timeline | group-by service | transpose service chain | each { |group|
        let chain = $group.chain
        {
            service: $group.service,
            chain_length: ($chain | length),
            first_request: ($chain | first | get correlation_id),
            last_request: ($chain | last | get correlation_id),
            total_duration: ($chain | get duration_ms | math sum)
        }
    })
    
    print $"🔗 Request chains: ($request_chains)"
    
    # Step 4: Generate correlation report
    let correlation_report = {
        total_requests: ($api_requests | length),
        time_span_start: ($request_timeline | first | get timestamp),
        time_span_end: ($request_timeline | last | get timestamp),
        success_rate_overall: (($api_requests | where success | length) / ($api_requests | length) * 100 | math round),
        avg_duration_overall: ($api_requests | get duration_ms | math avg | math round)
    }
    
    print $"📋 Correlation Report: ($correlation_report)"
}

# Example 4: Data synchronization automation
def data_synchronization [] {
    print "\n4. Data Synchronization Workflow"
    print "--------------------------------"
    
    # Simulate source and target datasets with ULIDs
    let source_data = (generate_test_ulids 12 | each { |id|
        {
            id: $id,
            name: $"Record_((random int 1000..9999))",
            status: (["active", "pending", "inactive"] | get (random int 0..2)),
            version: 1
        }
    })
    
    # Simulate target data (subset with some modifications)
    let target_data = ($source_data | first 8 | each { |record|
        if (random bool) {
            $record | upsert status "updated" | upsert version 2
        } else {
            $record
        }
    })
    
    print $"Synchronizing data: ($source_data | length) source records, ($target_data | length) target records"
    
    # Step 1: Identify records to sync using ULID comparison
    let source_ids = ($source_data | get id)
    let target_ids = ($target_data | get id)
    
    let new_records = ($source_data | where ($it.id not-in $target_ids))
    let existing_records = ($source_data | where ($it.id in $target_ids))
    
    print $"📊 Sync analysis: ($new_records | length) new records, ($existing_records | length) existing records"
    
    # Step 2: Sort all records chronologically for processing order
    let sync_order = ($source_data | ulid sort --column id)
    print "✓ Sync order determined by ULID timestamps"
    
    # Step 3: Detect changes in existing records
    let change_detection = ($existing_records | each { |source_record|
        let target_record = ($target_data | where id == $source_record.id | first)
        if ($source_record.status != $target_record.status or $source_record.version != $target_record.version) {
            {
                id: $source_record.id,
                change_type: "modified",
                source_status: $source_record.status,
                target_status: $target_record.status,
                source_version: $source_record.version,
                target_version: $target_record.version
            }
        } else {
            {
                id: $source_record.id,
                change_type: "unchanged"
            }
        }
    })
    
    let changes = ($change_detection | where change_type == "modified")
    print $"🔄 Change detection: ($changes | length) modified records"
    
    # Step 4: Generate sync operations with timestamps
    let sync_operations = [
        ...(
            $new_records | each { |record|
                let parsed = (ulid parse $record.id)
                {
                    operation: "INSERT",
                    record_id: $record.id,
                    created_at: $parsed.timestamp.iso8601,
                    data: $record
                }
            }
        ),
        ...(
            $changes | each { |change|
                let parsed = (ulid parse $change.id)
                {
                    operation: "UPDATE",
                    record_id: $change.id,
                    created_at: $parsed.timestamp.iso8601,
                    changes: $change
                }
            }
        )
    ]
    
    # Sort operations by creation time for proper execution order
    let ordered_operations = ($sync_operations | sort-by created_at)
    
    print $"⚡ Sync operations prepared: ($ordered_operations | length) operations in chronological order"
    
    # Step 5: Generate sync summary
    let sync_summary = {
        total_operations: ($ordered_operations | length),
        insert_operations: ($ordered_operations | where operation == "INSERT" | length),
        update_operations: ($ordered_operations | where operation == "UPDATE" | length),
        earliest_record: ($ordered_operations | first | get created_at),
        latest_record: ($ordered_operations | last | get created_at)
    }
    
    print $"📋 Sync Summary: ($sync_summary)"
}

# Run the automation workflows
main