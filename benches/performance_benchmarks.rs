use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use nu_plugin_ulid::UlidEngine;

fn benchmark_ulid_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ulid_generation");

    // Single ULID generation
    group.bench_function("generate_single", |b| {
        b.iter(|| {
            let ulid = UlidEngine::generate().expect("ULID generation should succeed");
            black_box(ulid)
        })
    });

    // Bulk ULID generation with different sizes
    for size in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("generate_bulk", size), size, |b, &size| {
            b.iter(|| {
                let ulids =
                    UlidEngine::generate_bulk(size).expect("Bulk generation should succeed");
                black_box(ulids)
            })
        });
    }

    // ULID generation with custom timestamp
    group.bench_function("generate_with_timestamp", |b| {
        let timestamp = 1692000000000u64;
        b.iter(|| {
            let ulid =
                UlidEngine::generate_with_timestamp(timestamp).expect("Generation should succeed");
            black_box(ulid)
        })
    });

    group.finish();
}

fn benchmark_ulid_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ulid_validation");

    // Generate sample ULIDs for testing
    let valid_ulids: Vec<String> = (0..1000)
        .map(|_| UlidEngine::generate().unwrap().to_string())
        .collect();

    let invalid_ulids = [
        "invalid_ulid".to_string(),
        "".to_string(),
        "01AN4Z07BY79KA1307SR9X4MV".to_string(),  // Too short
        "01AN4Z07BY79KA1307SR9X4MVX".to_string(), // Too long
        "01AN4Z07BY79KA1307SR9X4M!@".to_string(), // Invalid chars
    ];

    // Valid ULID validation
    group.bench_function("validate_valid", |b| {
        let ulid = &valid_ulids[0];
        b.iter(|| {
            let is_valid = UlidEngine::validate(black_box(ulid));
            black_box(is_valid)
        })
    });

    // Invalid ULID validation
    group.bench_function("validate_invalid", |b| {
        let ulid = &invalid_ulids[0];
        b.iter(|| {
            let is_valid = UlidEngine::validate(black_box(ulid));
            black_box(is_valid)
        })
    });

    // Batch validation
    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("validate_batch", size),
            size,
            |b, &size| {
                let batch: Vec<&str> = valid_ulids.iter().take(size).map(|s| s.as_str()).collect();
                b.iter(|| {
                    for ulid in &batch {
                        let is_valid = UlidEngine::validate(black_box(ulid));
                        black_box(is_valid);
                    }
                })
            },
        );
    }

    group.finish();
}

fn benchmark_ulid_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("ulid_parsing");

    // Generate sample ULIDs for testing
    let valid_ulids: Vec<String> = (0..100)
        .map(|_| UlidEngine::generate().unwrap().to_string())
        .collect();

    // Single ULID parsing
    group.bench_function("parse_single", |b| {
        let ulid = &valid_ulids[0];
        b.iter(|| {
            let components = UlidEngine::parse(black_box(ulid)).expect("Parsing should succeed");
            black_box(components)
        })
    });

    // Timestamp extraction
    group.bench_function("extract_timestamp", |b| {
        let ulid = &valid_ulids[0];
        b.iter(|| {
            let timestamp =
                UlidEngine::extract_timestamp(black_box(ulid)).expect("Extraction should succeed");
            black_box(timestamp)
        })
    });

    // Batch parsing
    for size in [10, 100].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("parse_batch", size), size, |b, &size| {
            let batch: Vec<&str> = valid_ulids.iter().take(size).map(|s| s.as_str()).collect();
            b.iter(|| {
                for ulid in &batch {
                    let components =
                        UlidEngine::parse(black_box(ulid)).expect("Parsing should succeed");
                    black_box(components);
                }
            })
        });
    }

    group.finish();
}

fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    // Memory allocation patterns for different operations
    group.bench_function("string_allocation", |b| {
        b.iter(|| {
            let ulid = UlidEngine::generate().expect("Generation should succeed");
            let ulid_str = ulid.to_string();
            black_box(ulid_str)
        })
    });

    // Large batch memory usage
    group.bench_function("large_batch_generation", |b| {
        b.iter(|| {
            let ulids = UlidEngine::generate_bulk(1000).expect("Bulk generation should succeed");
            // Convert to strings to measure string allocation overhead
            let ulid_strings: Vec<String> = ulids.iter().map(|u| u.to_string()).collect();
            black_box(ulid_strings)
        })
    });

    group.finish();
}

fn benchmark_comparison_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison_operations");

    // Generate ULIDs for comparison testing
    let ulids: Vec<String> = (0..1000)
        .map(|_| UlidEngine::generate().unwrap().to_string())
        .collect();

    // String comparison (lexicographic)
    group.bench_function("string_comparison", |b| {
        b.iter(|| {
            for i in 1..ulids.len().min(100) {
                let result = ulids[i - 1].cmp(&ulids[i]);
                black_box(result);
            }
        })
    });

    // Timestamp-based comparison
    group.bench_function("timestamp_comparison", |b| {
        b.iter(|| {
            for i in 1..ulids.len().min(100) {
                let ts1 = UlidEngine::extract_timestamp(&ulids[i - 1]).unwrap();
                let ts2 = UlidEngine::extract_timestamp(&ulids[i]).unwrap();
                let result = ts1.cmp(&ts2);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");

    // Single-threaded baseline
    group.bench_function("sequential_generation", |b| {
        b.iter(|| {
            let mut ulids = Vec::with_capacity(100);
            for _ in 0..100 {
                let ulid = UlidEngine::generate().expect("Generation should succeed");
                ulids.push(ulid);
            }
            black_box(ulids)
        })
    });

    // Multi-threaded generation
    group.bench_function("concurrent_generation", |b| {
        use std::thread;

        b.iter(|| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    thread::spawn(|| {
                        let mut ulids = Vec::with_capacity(25);
                        for _ in 0..25 {
                            let ulid = UlidEngine::generate().expect("Generation should succeed");
                            ulids.push(ulid);
                        }
                        ulids
                    })
                })
                .collect();

            let mut all_ulids = Vec::new();
            for handle in handles {
                let ulids = handle.join().expect("Thread should complete");
                all_ulids.extend(ulids);
            }

            black_box(all_ulids)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_ulid_generation,
    benchmark_ulid_validation,
    benchmark_ulid_parsing,
    benchmark_memory_efficiency,
    benchmark_comparison_operations,
    benchmark_concurrent_operations
);

criterion_main!(benches);
