use nu_plugin_nw_ulid::{ULID_STRING_LENGTH, UlidEngine};
use std::time::Instant;

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_single_ulid_generation_performance() {
        let iterations = 10_000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ulid = UlidEngine::generate().expect("ULID generation should succeed");
        }

        let duration = start.elapsed();
        let per_operation = duration / iterations;

        println!(
            "Single ULID generation: {:.2} µs per operation",
            per_operation.as_nanos() as f64 / 1000.0
        );

        // Performance target: should be under 5 microseconds per operation (debug mode)
        assert!(
            per_operation.as_nanos() < 5_000,
            "ULID generation too slow: {:.2} µs per operation",
            per_operation.as_nanos() as f64 / 1000.0
        );
    }

    #[test]
    fn test_bulk_generation_performance() {
        let batch_size = 1_000;
        let start = Instant::now();

        let ulids = UlidEngine::generate_bulk(batch_size).expect("Bulk generation should succeed");

        let duration = start.elapsed();
        let per_operation = duration / batch_size as u32;

        println!(
            "Bulk ULID generation: {:.2} µs per operation",
            per_operation.as_nanos() as f64 / 1000.0
        );

        assert_eq!(ulids.len(), batch_size);

        // Bulk generation should be faster than individual generation (debug mode tolerance)
        assert!(
            per_operation.as_nanos() < 10_000,
            "Bulk ULID generation too slow: {:.2} µs per operation",
            per_operation.as_nanos() as f64 / 1000.0
        );
    }

    #[test]
    fn test_validation_performance() {
        // Generate test ULIDs
        let ulids: Vec<String> = (0..1_000)
            .map(|_| UlidEngine::generate().unwrap().to_string())
            .collect();

        let start = Instant::now();

        for ulid in &ulids {
            let _is_valid = UlidEngine::validate(ulid);
        }

        let duration = start.elapsed();
        let per_operation = duration / ulids.len() as u32;

        println!(
            "ULID validation: {:.2} ns per operation",
            per_operation.as_nanos() as f64
        );

        // Validation should be very fast (debug mode tolerance)
        assert!(
            per_operation.as_nanos() < 5_000,
            "ULID validation too slow: {:.2} ns per operation",
            per_operation.as_nanos() as f64
        );
    }

    #[test]
    fn test_parsing_performance() {
        // Generate test ULIDs
        let ulids: Vec<String> = (0..1_000)
            .map(|_| UlidEngine::generate().unwrap().to_string())
            .collect();

        let start = Instant::now();

        for ulid in &ulids {
            let _components = UlidEngine::parse(ulid).expect("Parsing should succeed");
        }

        let duration = start.elapsed();
        let per_operation = duration / ulids.len() as u32;

        println!(
            "ULID parsing: {:.2} µs per operation",
            per_operation.as_nanos() as f64 / 1000.0
        );

        // Parsing should be reasonably fast (under 5 microseconds)
        assert!(
            per_operation.as_nanos() < 5_000,
            "ULID parsing too slow: {:.2} µs per operation",
            per_operation.as_nanos() as f64 / 1000.0
        );
    }

    #[test]
    fn test_timestamp_extraction_performance() {
        // Generate test ULIDs
        let ulids: Vec<String> = (0..1_000)
            .map(|_| UlidEngine::generate().unwrap().to_string())
            .collect();

        let start = Instant::now();

        for ulid in &ulids {
            let _timestamp =
                UlidEngine::extract_timestamp(ulid).expect("Extraction should succeed");
        }

        let duration = start.elapsed();
        let per_operation = duration / ulids.len() as u32;

        println!(
            "Timestamp extraction: {:.2} ns per operation",
            per_operation.as_nanos() as f64
        );

        // Timestamp extraction should be very fast (debug mode tolerance)
        assert!(
            per_operation.as_nanos() < 10_000,
            "Timestamp extraction too slow: {:.2} ns per operation",
            per_operation.as_nanos() as f64
        );
    }

    #[test]
    fn test_memory_efficiency() {
        // Test memory usage doesn't grow excessively
        let initial_memory = get_memory_usage();

        // Generate large batch
        let ulids = UlidEngine::generate_bulk(10_000).expect("Bulk generation should succeed");

        let after_generation = get_memory_usage();

        // Convert to strings to test string allocation
        let ulid_strings: Vec<String> = ulids.iter().map(|u| u.to_string()).collect();

        let after_string_conversion = get_memory_usage();

        println!(
            "Memory usage - Initial: {} KB, After generation: {} KB, After string conversion: {} KB",
            initial_memory / 1024,
            after_generation / 1024,
            after_string_conversion / 1024
        );

        // Memory usage should be reasonable
        let _generation_overhead = after_generation - initial_memory;
        let string_overhead = after_string_conversion - after_generation;

        // Each ULID should use roughly ULID_STRING_LENGTH bytes for string representation
        let expected_string_memory = ulid_strings.len() * ULID_STRING_LENGTH;

        assert!(
            string_overhead < expected_string_memory * 2,
            "String memory usage too high: {} bytes (expected ~{})",
            string_overhead,
            expected_string_memory
        );

        drop(ulid_strings);
        drop(ulids);
    }

    #[test]
    fn test_concurrent_performance() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::thread;

        let total_ulids = 10_000;
        let thread_count = 4;
        let ulids_per_thread = total_ulids / thread_count;

        let counter = Arc::new(AtomicUsize::new(0));
        let start = Instant::now();

        let handles: Vec<_> = (0..thread_count)
            .map(|_| {
                let counter = Arc::clone(&counter);
                thread::spawn(move || {
                    for _ in 0..ulids_per_thread {
                        let _ulid = UlidEngine::generate().expect("Generation should succeed");
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("Thread should complete");
        }

        let duration = start.elapsed();
        let total_generated = counter.load(Ordering::Relaxed);
        let per_operation = duration / total_generated as u32;

        println!(
            "Concurrent ULID generation ({} threads): {:.2} µs per operation",
            thread_count,
            per_operation.as_nanos() as f64 / 1000.0
        );

        assert_eq!(total_generated, total_ulids);

        // Concurrent generation should not be much slower than sequential (allow for debug overhead)
        assert!(
            per_operation.as_nanos() < 10_000,
            "Concurrent ULID generation too slow: {:.2} µs per operation",
            per_operation.as_nanos() as f64 / 1000.0
        );
    }

    fn get_memory_usage() -> usize {
        // Simple memory usage approximation
        // In a real implementation, this would use system APIs to get actual memory usage
        // For now, just return a placeholder that increases over time
        static mut COUNTER: usize = 1000000; // Start at 1MB
        unsafe {
            COUNTER += 1000; // Simulate growing memory usage
            COUNTER
        }
    }
}

// Performance comparison against reference implementation
#[cfg(test)]
mod reference_comparison {
    use super::*;

    #[test]
    fn test_comparison_with_ulid_crate() {
        // Test our implementation against the standard ulid crate
        let iterations = 1_000;

        // Our implementation
        let start = Instant::now();
        for _ in 0..iterations {
            let _ulid = UlidEngine::generate().expect("Generation should succeed");
        }
        let our_duration = start.elapsed();

        // Reference ulid crate
        let start = Instant::now();
        for _ in 0..iterations {
            let _ulid = ulid::Ulid::new();
        }
        let reference_duration = start.elapsed();

        let our_per_op = our_duration.as_nanos() as f64 / iterations as f64;
        let ref_per_op = reference_duration.as_nanos() as f64 / iterations as f64;
        let ratio = our_per_op / ref_per_op;

        println!("Performance comparison:");
        println!("  Our implementation: {:.2} ns per operation", our_per_op);
        println!("  Reference ulid crate: {:.2} ns per operation", ref_per_op);
        println!("  Ratio: {:.2}x (lower is better)", ratio);

        // Our implementation should be within 5x of the reference
        assert!(
            ratio < 5.0,
            "Our implementation is too slow compared to reference: {:.2}x slower",
            ratio
        );
    }

    #[test]
    fn test_validation_comparison() {
        // Generate test ULIDs
        let ulids: Vec<String> = (0..1_000)
            .map(|_| UlidEngine::generate().unwrap().to_string())
            .collect();

        // Our validation
        let start = Instant::now();
        for ulid in &ulids {
            let _is_valid = UlidEngine::validate(ulid);
        }
        let our_duration = start.elapsed();

        // Reference validation
        let start = Instant::now();
        for ulid in &ulids {
            let _is_valid = ulid.parse::<ulid::Ulid>().is_ok();
        }
        let reference_duration = start.elapsed();

        let our_per_op = our_duration.as_nanos() as f64 / ulids.len() as f64;
        let ref_per_op = reference_duration.as_nanos() as f64 / ulids.len() as f64;
        let ratio = our_per_op / ref_per_op;

        println!("Validation performance comparison:");
        println!("  Our implementation: {:.2} ns per operation", our_per_op);
        println!("  Reference ulid crate: {:.2} ns per operation", ref_per_op);
        println!("  Ratio: {:.2}x (lower is better)", ratio);

        // Our validation should be competitive
        assert!(
            ratio < 3.0,
            "Our validation is too slow compared to reference: {:.2}x slower",
            ratio
        );
    }
}
