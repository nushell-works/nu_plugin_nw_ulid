#!/usr/bin/env nu
# Data Processing Example with ULID Plugin
# Demonstrates common data processing patterns using ULIDs

# Source helper functions
source ../helpers.nu

def main [] {
    print "ULID Data Processing Examples"
    print "============================="
    
    # Example 1: Processing user events with ULIDs
    process_user_events
    
    # Example 2: Log analysis with temporal sorting
    analyze_request_logs
    
    # Example 3: Batch ID assignment
    batch_id_assignment
    
    # Example 4: Data validation and cleanup
    data_validation_cleanup
}

# Example 1: Process user events with ULID timestamps
def process_user_events [] {
    print "\n1. Processing User Events"
    print "-------------------------"
    
    # Simulate user events with ULIDs
    let events = [
        { event_id: "01H5JZQX1X7M8P9Q2R3S4T5U6V", user_id: "user_123", action: "login" }
        { event_id: "01H5JZQY2Y8N9Q0R1S2T3U4V5W", user_id: "user_123", action: "view_page" }
        { event_id: "01H5JZQZ3Z9O0R1S2T3U4V5W6X", user_id: "user_456", action: "login" }
        { event_id: "01H5JZR04A0P1S2T3U4V5W6X7Y", user_id: "user_123", action: "logout" }
    ]
    
    # Sort events chronologically
    let sorted_events = ($events | ulid sort --column event_id)
    print $"Chronologically sorted events: ($sorted_events)"
    
    # Extract event timestamps
    let event_times = ($events | each { |event|
        let parsed = (ulid parse $event.event_id)
        $event | upsert timestamp $parsed.timestamp.human | upsert timestamp_ms $parsed.timestamp.milliseconds
    })
    print $"Events with timestamps: ($event_times)"
    
    # Find events in the last hour (simulated)
    let now_ms = (date now | into int)
    let hour_ago = ($now_ms - 3600000)
    let recent_events = (filter_by_ulid_timestamp $events "event_id" $hour_ago $now_ms)
    print $"Recent events: ($recent_events | length) events"
}

# Example 2: Analyze request logs by temporal patterns
def analyze_request_logs [] {
    print "\n2. Request Log Analysis"
    print "-----------------------"
    
    # Generate sample request logs with ULIDs
    let request_count = 20
    let request_ids = (generate_test_ulids $request_count)
    
    let logs = ($request_ids | enumerate | each { |row|
        let id = $row.item
        let parsed = (ulid parse $id)
        {
            request_id: $id,
            timestamp: $parsed.timestamp.human,
            status: (["200", "404", "500"] | get (random int 0..2)),
            response_time_ms: (random int 50..2000)
        }
    })
    
    print $"Generated ($request_count) request logs"
    
    # Group by hour (simplified - group by request order for demo)
    let hourly_groups = ($logs | group_by_ulid_hour "request_id")
    print $"Grouped into ($hourly_groups | columns | length) time periods"
    
    # Analyze response times by chronological order
    let response_analysis = ($logs 
        | ulid sort --column request_id 
        | select request_id status response_time_ms
        | each { |log| 
            $log | upsert order_position ($logs | find --predicate { |x| $x.request_id == $log.request_id } | length)
        }
    )
    print $"Response time analysis: ($response_analysis)"
}

# Example 3: Batch ID assignment for new records
def batch_id_assignment [] {
    print "\n3. Batch ID Assignment"
    print "----------------------"
    
    # Simulate new user records without IDs
    let new_users = [
        { name: "Alice", email: "alice@example.com", role: "admin" }
        { name: "Bob", email: "bob@example.com", role: "user" }
        { name: "Charlie", email: "charlie@example.com", role: "moderator" }
        { name: "Diana", email: "diana@example.com", role: "user" }
    ]
    
    print $"Original records (without IDs): ($new_users)"
    
    # Add ULIDs to all records
    let users_with_ids = (add_ulids_to_records $new_users)
    print $"Records with ULIDs assigned: ($users_with_ids)"
    
    # Verify all IDs are valid
    let validation_results = ($users_with_ids | each { |user|
        {
            name: $user.name,
            id: $user.id,
            id_valid: (ulid validate $user.id)
        }
    })
    print $"ID validation: ($validation_results)"
}

# Example 4: Data validation and cleanup
def data_validation_cleanup [] {
    print "\n4. Data Validation and Cleanup"
    print "------------------------------"
    
    # Sample data with some invalid ULIDs
    let mixed_data = [
        { id: "01H5JZQX1X7M8P9Q2R3S4T5U6V", name: "Valid Record 1" }
        { id: "INVALID_ULID_123", name: "Invalid Record" }
        { id: "01H5JZQY2Y8N9Q0R1S2T3U4V5W", name: "Valid Record 2" }
        { id: "NOT_A_ULID", name: "Another Invalid" }
        { id: "01H5JZQZ3Z9O0R1S2T3U4V5W6X", name: "Valid Record 3" }
    ]
    
    print $"Original data: ($mixed_data | length) records"
    
    # Clean data - remove invalid ULIDs
    let clean_data = (clean_ulid_data $mixed_data "id")
    print $"Cleaned data: ($clean_data | length) valid records"
    
    # Detailed validation report
    let validation_report = ($mixed_data | each { |record|
        let is_valid = (ulid validate $record.id)
        if $is_valid {
            let parsed = (safe_ulid_parse $record.id)
            $record | upsert valid true | upsert timestamp $parsed.timestamp.human
        } else {
            $record | upsert valid false | upsert error "Invalid ULID format"
        }
    })
    
    print $"Validation report: ($validation_report)"
    
    # Summary statistics
    let total = ($mixed_data | length)
    let valid = ($validation_report | where valid | length)
    let invalid = ($total - $valid)
    
    print $"Summary: ($valid)/($total) valid records, ($invalid) invalid records"
}

# Run the examples
main