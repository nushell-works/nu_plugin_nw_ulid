use nu_plugin_nw_ulid::UlidEngine;
use std::collections::HashMap;

#[cfg(test)]
mod security_tests {
    use super::*;

    /// Test cryptographic randomness quality in ULID generation
    #[test]
    fn test_cryptographic_randomness_quality() {
        const SAMPLE_SIZE: usize = 10000;
        let mut ulids = Vec::with_capacity(SAMPLE_SIZE);

        // Generate sample ULIDs
        for _ in 0..SAMPLE_SIZE {
            let ulid = UlidEngine::generate().expect("ULID generation should succeed");
            ulids.push(ulid.to_string());
        }

        // Test 1: Uniqueness - no duplicates
        let mut unique_ulids = std::collections::HashSet::new();
        for ulid in &ulids {
            assert!(
                unique_ulids.insert(ulid.clone()),
                "ULID collision detected: {}",
                ulid
            );
        }

        // Test 2: Randomness distribution in randomness component
        test_randomness_distribution(&ulids);

        // Test 3: No obvious patterns
        test_no_sequential_patterns(&ulids);
    }

    fn test_randomness_distribution(ulids: &[String]) {
        // Extract randomness components (last 16 characters)
        let randomness_parts: Vec<String> = ulids
            .iter()
            .map(|ulid| ulid[10..].to_string()) // Skip timestamp, get randomness
            .collect();

        // Test character frequency distribution
        let mut char_frequency = HashMap::new();
        let total_chars = randomness_parts.len() * 16;

        for part in &randomness_parts {
            for ch in part.chars() {
                *char_frequency.entry(ch).or_insert(0) += 1;
            }
        }

        // Each character should appear roughly 1/32 of the time (Crockford Base32)
        let expected_frequency = total_chars as f64 / 32.0;
        let tolerance = expected_frequency * 0.2; // 20% tolerance

        for (ch, count) in char_frequency {
            let frequency = count as f64;
            assert!(
                (frequency - expected_frequency).abs() < tolerance,
                "Character '{}' appears {} times, expected ~{} (±{})",
                ch,
                count,
                expected_frequency as usize,
                tolerance as usize
            );
        }
    }

    fn test_no_sequential_patterns(ulids: &[String]) {
        // Check that consecutive ULIDs don't have identical randomness parts
        for i in 1..ulids.len().min(100) {
            // Check first 100 pairs
            let prev_randomness = &ulids[i - 1][10..];
            let curr_randomness = &ulids[i][10..];

            assert_ne!(
                prev_randomness,
                curr_randomness,
                "Sequential ULIDs have identical randomness: {} vs {}",
                ulids[i - 1],
                ulids[i]
            );
        }
    }

    /// Test input validation security
    #[test]
    fn test_input_validation_security() {
        // Test malformed ULID inputs
        let large_string = "A".repeat(1000);
        let malicious_inputs = vec![
            "",                                                     // Empty string
            large_string.as_str(),                                  // Very long string
            "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0", // Null bytes
            "01AN4Z07BY79KA1307SR9X4MV3\x00",                       // ULID with null terminator
            "01AN4Z07BY79KA1307SR9X4MV3\n",                         // ULID with newline
            "01AN4Z07BY79KA1307SR9X4MV3<script>",                   // HTML injection attempt
            "../../../etc/passwd",                                  // Path traversal attempt
            "'; DROP TABLE ulids; --",                              // SQL injection attempt
            "$(rm -rf /)",                                          // Command injection attempt
            "\x7F\x01\x02\x03",                                     // Control characters
        ];

        for input in malicious_inputs {
            // Validation should safely return false without panicking
            let is_valid = UlidEngine::validate(input);
            assert!(
                !is_valid,
                "Malicious input '{}' should not validate as ULID",
                input
                    .chars()
                    .map(|c| if c.is_control() { '�' } else { c })
                    .collect::<String>()
            );

            // Parsing should return error without panicking
            let parse_result = UlidEngine::parse(input);
            assert!(
                parse_result.is_err(),
                "Malicious input should not parse successfully"
            );
        }
    }

    /// Test memory safety with large inputs
    #[test]
    fn test_memory_safety_large_inputs() {
        // Test with extremely large strings
        let large_input = "A".repeat(1_000_000); // 1MB string

        // Should handle gracefully without memory issues
        let is_valid = UlidEngine::validate(&large_input);
        assert!(!is_valid);

        let parse_result = UlidEngine::parse(&large_input);
        assert!(parse_result.is_err());
    }

    /// Test boundary conditions for numeric inputs
    #[test]
    fn test_numeric_boundary_security() {
        // Test timestamp boundaries
        let test_timestamps = vec![
            0,                               // Epoch start
            u64::MAX,                        // Maximum value
            1692000000000,                   // Normal timestamp
            1692000000000 + u32::MAX as u64, // Large timestamp
        ];

        for timestamp in test_timestamps {
            // Should either succeed or fail gracefully
            let result = UlidEngine::generate_with_timestamp(timestamp);

            if let Ok(ulid) = result {
                // If successful, should be valid
                assert!(UlidEngine::validate(&ulid.to_string()));

                // Should be able to extract timestamp
                let extracted = UlidEngine::extract_timestamp(&ulid.to_string());
                assert!(extracted.is_ok());
            }
            // If error, that's also acceptable for boundary values
        }
    }

    /// Test resistance to timing attacks (basic)
    #[test]
    #[ignore = "Timing-sensitive test disabled for CI stability"]
    fn test_timing_attack_resistance() {
        let valid_ulid = "01AN4Z07BY79KA1307SR9X4MV3";
        let invalid_prefix = "01AN4Z07BY79KA1307SR9X4MV4"; // Different last char
        let invalid_start = "XX"; // Invalid from start

        // This is a basic test - in production, more sophisticated timing analysis needed
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            UlidEngine::validate(valid_ulid);
        }
        let valid_time = start.elapsed();

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            UlidEngine::validate(invalid_prefix);
        }
        let invalid_prefix_time = start.elapsed();

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            UlidEngine::validate(invalid_start);
        }
        let invalid_start_time = start.elapsed();

        // Times should be relatively similar (within order of magnitude)
        // This is a basic check - sophisticated timing attacks require more analysis
        let max_time = valid_time.max(invalid_prefix_time).max(invalid_start_time);
        let min_time = valid_time.min(invalid_prefix_time).min(invalid_start_time);

        // Allow significant variance due to system scheduling and optimization
        // This is a basic smoke test - real timing attack analysis requires specialized tools
        let ratio = max_time.as_nanos() as f64 / min_time.as_nanos() as f64;

        // Very permissive check - just ensuring no catastrophic timing differences
        assert!(
            ratio < 100.0,
            "Extreme timing variance detected: {:.2}x difference",
            ratio
        );

        // Log timing information for manual analysis
        println!(
            "Timing analysis: valid={:?}, invalid_prefix={:?}, invalid_start={:?}, ratio={:.2}x",
            valid_time, invalid_prefix_time, invalid_start_time, ratio
        );
    }

    /// Test bulk operation resource limits
    #[test]
    fn test_bulk_operation_limits() {
        // Test that bulk operations respect limits
        let result = UlidEngine::generate_bulk(100_001); // Exceeds limit
        assert!(
            result.is_err(),
            "Should reject bulk generation beyond limit"
        );

        // Test successful bulk operation
        let result = UlidEngine::generate_bulk(1000);
        assert!(result.is_ok(), "Should accept reasonable bulk generation");

        if let Ok(ulids) = result {
            assert_eq!(ulids.len(), 1000);

            // Verify all generated ULIDs are unique and valid
            let mut unique_set = std::collections::HashSet::new();
            for ulid in ulids {
                let ulid_str = ulid.to_string();
                assert!(
                    UlidEngine::validate(&ulid_str),
                    "Generated ULID should be valid"
                );
                assert!(
                    unique_set.insert(ulid_str),
                    "Generated ULIDs should be unique"
                );
            }
        }
    }

    /// Test error handling doesn't leak information
    #[test]
    fn test_error_information_leakage() {
        let test_inputs = vec![
            "invalid_ulid",
            "",
            "01AN4Z07BY79KA1307SR9X4MV",  // Too short
            "01AN4Z07BY79KA1307SR9X4M!@", // Invalid characters
            "ZZZZZZZZZZZZZZZZZZZZZZZZZZ", // Invalid timestamp
        ];

        for input in test_inputs {
            if let Err(error) = UlidEngine::parse(input) {
                let error_msg = error.to_string();

                // Error message should not contain:
                // - File paths
                // - Memory addresses
                // - Internal implementation details
                assert!(
                    !error_msg.contains("/"),
                    "Error should not contain file paths: {}",
                    error_msg
                );
                assert!(
                    !error_msg.contains("\\"),
                    "Error should not contain file paths: {}",
                    error_msg
                );
                assert!(
                    !error_msg.contains("0x"),
                    "Error should not contain memory addresses: {}",
                    error_msg
                );
                assert!(
                    !error_msg.contains("panic"),
                    "Error should not contain panic details: {}",
                    error_msg
                );

                // Error message should be user-friendly
                assert!(!error_msg.is_empty(), "Error message should not be empty");
                assert!(
                    error_msg.len() < 200,
                    "Error message should be reasonably short"
                );
            }
        }
    }

    /// Test concurrent access safety
    #[test]
    fn test_concurrent_safety() {
        use std::sync::Arc;
        use std::thread;

        const NUM_THREADS: usize = 10;
        const ULIDS_PER_THREAD: usize = 100;

        let mut handles = Vec::new();
        let all_ulids = Arc::new(std::sync::Mutex::new(Vec::new()));

        // Generate ULIDs concurrently
        for _ in 0..NUM_THREADS {
            let ulids_ref = Arc::clone(&all_ulids);

            let handle = thread::spawn(move || {
                let mut thread_ulids = Vec::new();

                for _ in 0..ULIDS_PER_THREAD {
                    let ulid = UlidEngine::generate().expect("ULID generation should succeed");
                    thread_ulids.push(ulid.to_string());
                }

                // Add to shared collection
                let mut all_ulids = ulids_ref.lock().unwrap();
                all_ulids.extend(thread_ulids);
            });

            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }

        // Verify all ULIDs are unique
        let all_ulids = all_ulids.lock().unwrap();
        assert_eq!(all_ulids.len(), NUM_THREADS * ULIDS_PER_THREAD);

        let unique_ulids: std::collections::HashSet<_> = all_ulids.iter().collect();
        assert_eq!(
            unique_ulids.len(),
            all_ulids.len(),
            "All ULIDs should be unique across threads"
        );
    }
}
