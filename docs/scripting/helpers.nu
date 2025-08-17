# ULID Helper Functions for Nushell Scripts
# Source this file in your scripts: source helpers.nu

# Safe ULID parsing with error handling
def safe_ulid_parse [ulid: string] {
    try {
        ulid parse $ulid
    } catch {
        { error: "Invalid ULID", ulid: $ulid, valid: false }
    }
}

# Validate ULID age (returns true if ULID is newer than max_age_hours)
def validate_ulid_age [ulid: string, max_age_hours: int] {
    try {
        let parsed = (ulid parse $ulid)
        let now_ms = (date now | into int)
        let ulid_ms = ($parsed.timestamp.milliseconds)
        let age_hours = ($now_ms - $ulid_ms) / 3600000
        $age_hours <= $max_age_hours
    } catch {
        false
    }
}

# Generate ULIDs for a list of records, adding them as 'id' field
def add_ulids_to_records [records: list] {
    let count = ($records | length)
    let ids = (ulid generate-stream $count)
    $records | enumerate | each { |row|
        $row.item | upsert id ($ids | get $row.index)
    }
}

# Extract timestamps from ULID list and return as milliseconds
def extract_ulid_timestamps [ulids: list] {
    $ulids | each { |ulid|
        try {
            ulid parse $ulid | get timestamp.milliseconds
        } catch {
            null
        }
    }
}

# Filter records by ULID timestamp range
def filter_by_ulid_timestamp [
    records: list,
    ulid_column: string,
    start_timestamp: int,
    end_timestamp: int
] {
    $records | where {
        let ulid_val = ($in | get $ulid_column)
        try {
            let ts = (ulid parse $ulid_val | get timestamp.milliseconds)
            $ts >= $start_timestamp and $ts <= $end_timestamp
        } catch {
            false
        }
    }
}

# Validate and clean ULID data, removing invalid entries
def clean_ulid_data [data: list, ulid_column: string] {
    $data | where {
        let ulid_val = ($in | get $ulid_column)
        ulid validate $ulid_val
    }
}

# Group records by ULID timestamp hour
def group_by_ulid_hour [records: list, ulid_column: string] {
    $records | group-by {
        let ulid_val = ($in | get $ulid_column)
        try {
            let ts = (ulid parse $ulid_val | get timestamp.milliseconds)
            let hour = ($ts / 3600000 | math floor)
            $hour
        } catch {
            "invalid"
        }
    }
}

# Convert ULID to different formats
def ulid_to_formats [ulid: string] {
    try {
        let parsed = (ulid parse $ulid)
        {
            ulid: $ulid,
            timestamp: $parsed.timestamp.milliseconds,
            iso8601: $parsed.timestamp.iso8601,
            human: $parsed.timestamp.human,
            randomness_hex: $parsed.randomness.hex
        }
    } catch {
        { error: "Invalid ULID", ulid: $ulid }
    }
}

# Batch process ULIDs with progress indication
def process_ulids_batch [
    ulids: list,
    operation: string,
    --batch-size: int = 1000
] {
    let total = ($ulids | length)
    print $"Processing ($total) ULIDs in batches of ($batch_size)..."
    
    $ulids | ulid stream $operation --batch-size $batch_size
}

# Generate time-ordered ULIDs for testing
def generate_test_ulids [count: int, --start-time: int] {
    let base_time = if ($start_time != null) { $start_time } else { (date now | into int) }
    ulid generate-stream $count --timestamp $base_time --unique-timestamps
}

# Verify ULID ordering (returns true if ULIDs are in chronological order)
def verify_ulid_ordering [ulids: list] {
    let timestamps = ($ulids | extract_ulid_timestamps)
    let sorted_timestamps = ($timestamps | sort)
    $timestamps == $sorted_timestamps
}

# Security check for ULID context
def check_ulid_security [context: string] {
    ulid security-advice --context $context --format compact | get warning_level
}

# Example usage function
def example_ulid_workflow [] {
    print "ULID Helper Functions Example Workflow"
    print "======================================"
    
    # Generate test data
    let test_ulids = (generate_test_ulids 5)
    print $"Generated ULIDs: ($test_ulids)"
    
    # Parse and display information
    let parsed_info = ($test_ulids | each { |ulid| ulid_to_formats $ulid })
    print $"Parsed information: ($parsed_info)"
    
    # Verify ordering
    let is_ordered = (verify_ulid_ordering $test_ulids)
    print $"ULIDs are chronologically ordered: ($is_ordered)"
    
    # Security check
    let security_level = (check_ulid_security "example")
    print $"Security warning level: ($security_level)"
}