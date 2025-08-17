#!/usr/bin/env nu
# Batch Operations Example with ULID Plugin
# Demonstrates efficient processing of large ULID datasets

# Source helper functions
source ../helpers.nu

def main [] {
    print "ULID Batch Operations Examples"
    print "=============================="
    
    # Example 1: Large-scale ULID generation
    large_scale_generation
    
    # Example 2: Bulk validation with error handling
    bulk_validation_with_errors
    
    # Example 3: Streaming data transformation
    streaming_data_transformation
    
    # Example 4: Performance comparison
    performance_comparison
}

# Example 1: Generate large quantities of ULIDs efficiently
def large_scale_generation [] {
    print "\n1. Large-Scale ULID Generation"
    print "-------------------------------"
    
    # Generate 1000 ULIDs for batch processing
    print "Generating 1000 ULIDs in batches..."
    let batch_ids = (ulid generate-stream 1000 --batch-size 200)
    print $"Generated ($batch_ids | length) ULIDs"
    
    # Generate time-ordered ULIDs for sequential processing
    print "Generating 500 time-ordered ULIDs..."
    let ordered_ids = (ulid generate-stream 500 --unique-timestamps)
    print $"Generated ($ordered_ids | length) ordered ULIDs"
    
    # Verify ordering
    let is_ordered = (verify_ulid_ordering $ordered_ids)
    print $"ULIDs are properly ordered: ($is_ordered)"
    
    # Sample the first and last ULIDs to show time progression
    let first_id = ($ordered_ids | first)
    let last_id = ($ordered_ids | last)
    let first_time = (ulid parse $first_id | get timestamp.human)
    let last_time = (ulid parse $last_id | get timestamp.human)
    
    print $"Time range: ($first_time) to ($last_time)"
}

# Example 2: Bulk validation with comprehensive error handling
def bulk_validation_with_errors [] {
    print "\n2. Bulk Validation with Error Handling"
    print "---------------------------------------"
    
    # Create a mix of valid and invalid ULIDs for testing
    let valid_ulids = (ulid generate-stream 100)
    let invalid_ulids = [
        "INVALID_ULID_1",
        "NOT_A_ULID_2", 
        "WRONG_LENGTH",
        "01INVALID78901234567890123456",
        "SHORT"
    ]
    
    let mixed_ulids = ($valid_ulids | append $invalid_ulids | shuffle)
    print $"Testing with ($mixed_ulids | length) ULIDs (($valid_ulids | length) valid, ($invalid_ulids | length) invalid)"
    
    # Stream validation with error continuation
    print "Running bulk validation with error handling..."
    let validation_results = ($mixed_ulids | ulid stream validate --batch-size 25 --continue-on-error)
    
    # Analyze results
    let valid_count = ($validation_results | where ($in == true) | length)
    let invalid_count = ($validation_results | where ($in == false or ($in | describe) == "record") | length)
    
    print $"Validation complete: ($valid_count) valid, ($invalid_count) invalid"
    
    # Show detailed error information for invalid ULIDs
    let error_details = ($mixed_ulids | enumerate | each { |row|
        let ulid = $row.item
        let index = $row.index
        let result = ($validation_results | get $index)
        
        if ($result == false or ($result | describe) == "record") {
            { index: $index, ulid: $ulid, error: "Validation failed" }
        } else {
            null
        }
    } | compact)
    
    print $"Error details: ($error_details)"
}

# Example 3: Streaming data transformation
def streaming_data_transformation [] {
    print "\n3. Streaming Data Transformation"
    print "--------------------------------"
    
    # Generate sample dataset
    let dataset = (ulid generate-stream 200)
    print $"Processing dataset of ($dataset | length) ULIDs"
    
    # Transform ULIDs to compact format using streaming
    print "Extracting timestamps using streaming..."
    let timestamps = ($dataset | ulid stream extract-timestamp --batch-size 50)
    print $"Extracted ($timestamps | length) timestamps"
    
    # Parse ULIDs in compact format
    print "Parsing to compact format..."
    let compact_data = ($dataset | ulid stream parse --output-format compact --batch-size 50)
    print $"Parsed ($compact_data | length) records in compact format"
    
    # Show sample of transformed data
    let sample = ($compact_data | first 3)
    print $"Sample transformed data: ($sample)"
    
    # Calculate timestamp statistics
    let min_timestamp = ($timestamps | math min)
    let max_timestamp = ($timestamps | math max)
    let avg_timestamp = ($timestamps | math avg | math round)
    
    print $"Timestamp range: ($min_timestamp) to ($max_timestamp)"
    print $"Average timestamp: ($avg_timestamp)"
}

# Example 4: Performance comparison between different approaches
def performance_comparison [] {
    print "\n4. Performance Comparison"
    print "-------------------------"
    
    let test_size = 500
    print $"Comparing performance with ($test_size) ULIDs"
    
    # Generate test data
    let test_ulids = (ulid generate-stream $test_size)
    
    # Method 1: Individual parsing (simulated - slower)
    print "Method 1: Individual parsing (each command)"
    let start_time = (date now | into int)
    let individual_results = ($test_ulids | each { |ulid| 
        try { 
            ulid parse $ulid | get timestamp.milliseconds 
        } catch { 
            0 
        } 
    })
    let individual_time = ((date now | into int) - $start_time)
    print $"Individual parsing completed: ($individual_results | length) results in ($individual_time)ms"
    
    # Method 2: Streaming (faster)
    print "Method 2: Streaming extraction"
    let stream_start = (date now | into int)
    let streaming_results = ($test_ulids | ulid stream extract-timestamp --batch-size 100)
    let streaming_time = ((date now | into int) - $stream_start)
    print $"Streaming extraction completed: ($streaming_results | length) results in ($streaming_time)ms"
    
    # Compare results
    if ($individual_time > 0 and $streaming_time > 0) {
        let speedup = ($individual_time / $streaming_time)
        print $"Performance improvement: {:.2}x speedup with streaming" | format $speedup
    }
    
    # Memory efficiency demonstration
    print "\nMemory efficiency test with larger dataset..."
    let large_size = 2000
    let large_ulids = (ulid generate-stream $large_size --batch-size 500)
    
    # Process in batches to show memory management
    let batch_results = (process_ulids_batch $large_ulids "validate" --batch-size 250)
    print $"Processed ($batch_results | length) ULIDs in memory-efficient batches"
}

# Utility function to demonstrate custom batch processing
def process_custom_batch [data: list, batch_size: int] {
    let total = ($data | length)
    let batches = (($total + $batch_size - 1) / $batch_size | math floor)
    
    print $"Processing ($total) items in ($batches) batches of ($batch_size)"
    
    mut results = []
    for batch_num in 0..($batches - 1) {
        let start_idx = ($batch_num * $batch_size)
        let end_idx = (($start_idx + $batch_size) | math min $total)
        let batch = ($data | skip $start_idx | first ($end_idx - $start_idx))
        
        print $"Processing batch ($batch_num + 1)/($batches)..."
        let batch_result = ($batch | each { |item| ulid validate $item })
        $results = ($results | append $batch_result)
    }
    
    $results
}

# Run the examples
main