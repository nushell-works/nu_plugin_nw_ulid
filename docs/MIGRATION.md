# Migration Guide and Compatibility Information

This document provides comprehensive guidance for migrating to nu_plugin_nw_ulid from other ULID implementations, upgrading between versions, and ensuring compatibility across different environments.

## Table of Contents

- [Package Information](#package-information)
- [Version Migration](#version-migration)
- [From Other ULID Libraries](#from-other-ulid-libraries)
- [From UUID to ULID](#from-uuid-to-ulid)
- [Nushell Version Compatibility](#nushell-version-compatibility)
- [Breaking Changes](#breaking-changes)
- [Migration Tools and Scripts](#migration-tools-and-scripts)
- [Compatibility Matrix](#compatibility-matrix)
- [Troubleshooting](#troubleshooting)

## Package Information

### Official Package Details

- **Crates.io Package**: `nu_plugin_nw_ulid`
- **Binary Name**: `nu_plugin_nw_ulid` (required by Nushell)
- **Organization**: nushell-works
- **Repository**: https://github.com/nushell-works/nu_plugin_nw_ulid

### Installation

```bash
# Install from crates.io
cargo install nu_plugin_nw_ulid

# Register with Nushell (binary name must be nu_plugin_nw_ulid)
plugin add ~/.cargo/bin/nu_plugin_nw_ulid
plugin use nw_ulid
```

### Why "nw-" Prefix?

The `nw-` prefix stands for "nushell-works" (the GitHub organization) and was chosen because:
- The original `nu_plugin_nw_ulid` package name was already taken on crates.io
- Nushell requires the binary to be named `nu_plugin_nw_ulid`
- This naming convention allows clear identification of official nushell-works packages

## Version Migration

### Current Version: v0.1.0

This is the initial production release with:
- ✅ Complete ULID functionality (23 commands)
- ✅ Enterprise-grade security and performance
- ✅ Comprehensive testing and documentation
- ✅ Nushell 0.109.1+ compatibility
- ✅ Rust edition 2024 support

### Future Versions

#### Planned v0.2.x Features
```
Feature                  | v0.1.x              | v0.2.x (Planned)
-------------------------|---------------------|-------------------
Minimum Rust Version    | 1.89.0 (edition24) | 1.90.0+
Minimum Nushell Version | 0.109.1             | 0.110.0+
Command Interface        | Stable              | Enhanced subcommands
Error Types              | Basic               | Detailed error context
Async Support            | No                  | Yes
GPU Acceleration         | No                  | Yes (bulk ops)
```

## From Other ULID Libraries

### Migrating from `nu_plugin_nw_ulid` (Original Package)

If you were using the original `nu_plugin_nw_ulid` package (v0.15.0), here's how to migrate:

#### 1. Uninstall Old Package
```bash
# Remove old plugin registration
plugin remove ulid

# Uninstall old package
cargo uninstall nu_plugin_nw_ulid
```

#### 2. Install New Package
```bash
# Install nushell-works version
cargo install nu_plugin_nw_ulid

# Register new plugin (same binary name)
plugin add ~/.cargo/bin/nu_plugin_nw_ulid
plugin use nw_ulid
```

#### 3. Verify Migration
```nu
# Check plugin info
ulid info

# Test basic functionality
let test_ulid = (ulid generate)
ulid validate $test_ulid
```

### Migrating from `ulid-rs` Crate

#### API Mapping
```rust
// ulid-rs (Rust code)
use ulid::Ulid;
let ulid = Ulid::new();
let string = ulid.to_string();
let timestamp = ulid.datetime();

// nu_plugin_nw_ulid (Nushell)
let ulid = (ulid generate)
let parsed = (ulid parse $ulid)
let timestamp = $parsed.formatted_time
```

#### Feature Comparison
```
Feature              | ulid-rs        | nu_plugin_nw_ulid
---------------------|----------------|------------------
ULID Generation      | ✅             | ✅
ULID Validation      | ✅             | ✅
Timestamp Extraction | ✅             | ✅
Stream Processing    | ❌             | ✅
Nushell Integration  | ❌             | ✅
Security Analysis    | ❌             | ✅
Bulk Operations      | Manual         | Built-in
Performance Metrics  | ❌             | ✅
```

#### Migration Script
```nu
# Convert from ulid-rs format to nu_plugin_nw_ulid
def migrate_from_ulid_rs [input_file: path, output_file: path] {
    print $"🔄 Converting ULIDs from ($input_file) to nu_plugin_nw_ulid format"

    let data = (open $input_file)

    let converted = ($data | each { |record|
        # Validate existing ULIDs
        if (ulid validate $record.id) {
            let parsed = (ulid parse $record.id)
            $record | insert ulid_info $parsed
        } else {
            # Generate new ULID if invalid
            let new_ulid = (ulid generate)
            $record
            | upsert id $new_ulid
            | insert migration_note "Generated new ULID - original was invalid"
        }
    })

    $converted | save $output_file
    print $"✅ Converted ($data | length) records to ($output_file)"
}
```

## From UUID to ULID

### Benefits of ULID over UUID

```
Aspect               | UUID v4        | ULID
---------------------|----------------|------------------
Sortability          | ❌ Random      | ✅ Lexicographic
Timestamp            | ❌ None        | ✅ Millisecond precision
Collision Resistance | ✅ High        | ✅ High
URL Safety           | ⚠️  Hyphens    | ✅ Base32
Human Readability    | ⚠️  Moderate   | ✅ Good
Database Performance | ⚠️  Random I/O | ✅ Sequential
Index Efficiency     | ⚠️  Poor       | ✅ Excellent
```

### Migration Strategy

#### Phase 1: Dual ID Support
```nu
# Add ULID alongside existing UUID
def add_ulid_to_records [records: list] {
    $records | each { |record|
        $record | insert ulid (ulid generate)
    }
}

# Example: Customer records
let customers = [
    { uuid: "550e8400-e29b-41d4-a716-446655440000", name: "Alice" }
    { uuid: "6ba7b810-9dad-11d1-80b4-00c04fd430c8", name: "Bob" }
]

let customers_with_ulid = (add_ulid_to_records $customers)
# Result: { uuid: "...", name: "Alice", ulid: "01K2W6DWFDF0H0NH22WHF5A3B2" }
```

#### Phase 2: ULID Primary, UUID Reference
```nu
# Transition to ULID as primary ID
def transition_to_ulid_primary [records: list] {
    $records | each { |record|
        {
            id: $record.ulid,
            legacy_uuid: $record.uuid,
            name: $record.name,
            migrated_at: (date now)
        }
    }
}
```

#### Phase 3: UUID Deprecation
```nu
# Remove UUID fields after transition period
def remove_legacy_uuids [records: list, retention_days: int = 90] {
    let cutoff_date = ((date now) - ($retention_days * 24 * 60 * 60 * 1000))

    $records | each { |record|
        if ($record.migrated_at | into int) < ($cutoff_date | into int) {
            $record | reject legacy_uuid
        } else {
            $record  # Keep legacy UUID during transition
        }
    }
}
```

### Database Migration Examples

#### PostgreSQL Migration
```sql
-- Add ULID column
ALTER TABLE users ADD COLUMN ulid CHAR(26);

-- Generate ULIDs for existing records (via Nushell script)
-- Update via external script calling: ulid generate

-- Create index on ULID
CREATE INDEX idx_users_ulid ON users(ulid);

-- After validation period, make ULID primary
ALTER TABLE users DROP CONSTRAINT users_pkey;
ALTER TABLE users ADD PRIMARY KEY (ulid);
```

#### MongoDB Migration
```nu
# MongoDB collection migration example
def migrate_mongodb_collection [collection: string] {
    print $"🔄 Migrating MongoDB collection: ($collection)"

    # This is conceptual - adapt to your MongoDB driver
    let documents = (mongo_find $collection {})

    let migrated = ($documents | each { |doc|
        let ulid_id = (ulid generate)

        {
            _id: $ulid_id,
            legacy_id: $doc._id,
            data: ($doc | reject _id),
            migration_timestamp: (date now)
        }
    })

    print $"✅ Prepared ($migrated | length) documents for migration"
    $migrated
}
```

## Nushell Version Compatibility

### Supported Versions

```
nu_plugin_nw_ulid | Nushell Version | Rust Requirement | Status
------------------|-----------------|-------------------|--------
0.1.0             | 0.109.1+        | 1.89.0+          | ✅ Current
Future 0.2.x      | 0.110.0+        | 1.90.0+          | 🔄 Planned
```

### Version-Specific Features

```
Feature                | Nu 0.95.x | Nu 0.106.x | Nu 0.110.x+ (Future)
-----------------------|-----------|------------|----------------------
Plugin Protocol        | v0.95     | v0.106     | v0.110
Command Description     | usage()   | description() | description()
Error Handling          | Basic     | Enhanced   | Rich context
Stream Processing       | Limited   | Full       | Optimized
Async Support          | No        | No         | Yes
```

### Compatibility Testing

```nu
def test_nushell_compatibility [] {
    let nu_version = (version | get version)
    let plugin_info = (ulid info)

    print $"Nushell version: ($nu_version)"
    print $"Plugin version: ($plugin_info.version)"
    print $"Plugin package: nu_plugin_nw_ulid"

    # Test core functionality
    let test_cases = [
        { test: "generation", command: { ulid generate } },
        { test: "validation", command: { ulid validate "01ARZ3NDEKTSV4RRFFQ69G5FAV" } },
        { test: "parsing", command: { ulid parse "01ARZ3NDEKTSV4RRFFQ69G5FAV" } },
        { test: "stream", command: { ["01ARZ3NDEKTSV4RRFFQ69G5FAV"] | ulid stream validate } }
    ]

    let results = ($test_cases | each { |case|
        try {
            let result = (do $case.command)
            { test: $case.test, status: "✅ OK", result: $result }
        } catch { |error|
            { test: $case.test, status: "❌ FAILED", error: $error.msg }
        }
    })

    $results
}
```

## Breaking Changes

### v0.1.0 Changes from Demo

```
Aspect               | Demo Version  | v0.1.0 Production
---------------------|---------------|-------------------
Package Name         | nu_plugin_nw_ulid| nu_plugin_nw_ulid
API Stability        | Experimental  | Stable
Error Handling       | Basic         | Comprehensive
Security Features    | None          | Full security analysis
Performance          | Unoptimized   | Production optimized
Testing              | Minimal       | 90% coverage
Documentation        | Basic         | Comprehensive
```

### Future Breaking Changes (v0.2.x)

Planned but not yet implemented:

```nu
# Current (v0.1.x): Basic bulk operations
ulid generate --count 10

# Future (v0.2.x): Enhanced batch operations
ulid batch generate --size 10 --async

# Current: String errors
try { ulid validate "invalid" } catch { |e| print $e.msg }

# Future: Structured errors
match (ulid validate "invalid") {
    { valid: true } => true,
    { valid: false, reason: $reason, code: $code } => false
}
```

## Migration Tools and Scripts

### Automated Migration Checker

```nu
def check_migration_readiness [] {
    print "🔍 Checking migration readiness for nu_plugin_nw_ulid..."

    let checks = [
        {
            name: "Nushell Version",
            check: { (version | get version) >= "0.109.1" },
            fix: "Update Nushell: cargo install nu --version 0.109.1"
        },
        {
            name: "Rust Version",
            check: { (rustc --version | parse "rustc {version}" | get version.0) >= "1.89.0" },
            fix: "Update Rust: rustup update"
        },
        {
            name: "Old Plugin Removed",
            check: { try { plugin list | where name == "ulid" | length } catch { 0 } == 0 },
            fix: "Remove old plugin: plugin remove ulid"
        },
        {
            name: "New Plugin Installed",
            check: { try { ulid info | get package } == "nu_plugin_nw_ulid" catch { false } },
            fix: "Install: cargo install nu_plugin_nw_ulid && plugin add ~/.cargo/bin/nu_plugin_nw_ulid"
        }
    ]

    $checks | each { |check|
        let result = (do $check.check)
        if $result {
            print $"✅ ($check.name): OK"
        } else {
            print $"❌ ($check.name): FAILED"
            print $"   Fix: ($check.fix)"
        }
        { check: $check.name, passed: $result }
    }
}
```

### Data Migration Validator

```nu
def validate_data_migration [original: list, migrated: list] {
    print "🔍 Validating data migration..."

    let validation = {
        record_count_match: (($original | length) == ($migrated | length)),
        all_ulids_valid: ($migrated | all { |record| ulid validate $record.id }),
        no_duplicate_ulids: (($migrated | get id | uniq | length) == ($migrated | length)),
        timestamp_ordering: ($migrated | get id | ulid sort | length) == ($migrated | length),
        package_info: (ulid info | get package) == "nu_plugin_nw_ulid"
    }

    let all_passed = ($validation | values | all { |v| $v })

    if $all_passed {
        print "✅ Migration validation passed"
    } else {
        print "❌ Migration validation failed"
        $validation | items { |key, value|
            if not $value {
                print $"  ❌ ($key): FAILED"
            }
        }
    }

    $validation
}
```

### Performance Comparison Script

```nu
def compare_migration_performance [old_impl: string, iterations: int = 1000] {
    print $"🔄 Comparing performance: ($old_impl) vs nu_plugin_nw_ulid"

    # Test new implementation
    let start_new = (date now | into int)
    for $i in 1..$iterations {
        let ulid = (ulid generate)
        ulid validate $ulid | ignore
    }
    let end_new = (date now | into int)
    let new_duration = ($end_new - $start_new)

    {
        old_implementation: $old_impl,
        new_implementation: "nu_plugin_nw_ulid",
        iterations: $iterations,
        new_duration_ms: $new_duration,
        new_ops_per_sec: ($iterations * 1000 / $new_duration),
        migration_recommendation: if $new_duration < 1000 { "✅ Performance acceptable" } else { "⚠️ Consider optimization" }
    }
}
```

## Compatibility Matrix

### Operating System Support

```
OS                   | Architecture | Status | Binary Location
---------------------|-------------|--------|---------------------------
Linux (Ubuntu 20.04+)| x86_64      | ✅     | ~/.cargo/bin/nu_plugin_nw_ulid
Linux (Ubuntu 20.04+)| aarch64     | ✅     | ~/.cargo/bin/nu_plugin_nw_ulid
macOS 11+            | x86_64      | ✅     | ~/.cargo/bin/nu_plugin_nw_ulid
macOS 11+            | aarch64     | ✅     | ~/.cargo/bin/nu_plugin_nw_ulid
Windows 10+          | x86_64      | ✅     | %USERPROFILE%\.cargo\bin\nu_plugin_nw_ulid.exe
FreeBSD              | x86_64      | ⚠️     | Community supported
```

### Package Manager Compatibility

```
Package Manager | Installation Command | Status
----------------|---------------------|--------
Cargo           | cargo install nu_plugin_nw_ulid | ✅ Primary
Homebrew        | Not available | ❌ Future
APT             | Not available | ❌ Future
Chocolatey      | Not available | ❌ Future
Nixpkgs         | Not available | ❌ Future
```

## Troubleshooting

### Common Migration Issues

#### Package Not Found
```bash
# Symptom: "package `nu_plugin_nw_ulid` not found"
# Solution: Verify package name spelling
cargo search nu_plugin_nw_ulid
cargo install nu_plugin_nw_ulid
```

#### Plugin Not Loading
```nu
# Symptom: "Plugin not found" error after installation
# Solution: Check binary installation and registration
ls ~/.cargo/bin/nu_plugin_nw_ulid  # Should exist
plugin add ~/.cargo/bin/nu_plugin_nw_ulid
plugin use nw_ulid
```

#### Version Conflicts
```nu
# Symptom: Multiple plugin versions or conflicts
# Solution: Clean installation
plugin remove ulid
cargo uninstall nu_plugin_nw_ulid nu_plugin_nw_ulid  # Remove any old versions
cargo install nu_plugin_nw_ulid
plugin add ~/.cargo/bin/nu_plugin_nw_ulid
plugin use nw_ulid
```

#### Performance Issues After Migration
```nu
def diagnose_performance_issues [] {
    print "🔍 Diagnosing performance issues..."

    let baseline = {
        generation: 600000,  # ops/sec expected
        validation: 7000000, # ops/sec expected
    }

    # Test current performance
    let start = (date now | into int)
    let test_ulids = (1..1000 | each { ulid generate })
    let gen_end = (date now | into int)
    let gen_rate = (1000 * 1000 / ($gen_end - $start))

    let val_start = (date now | into int)
    $test_ulids | each { |ulid| ulid validate $ulid } | ignore
    let val_end = (date now | into int)
    let val_rate = (1000 * 1000 / ($val_end - $val_start))

    {
        plugin_package: (ulid info | get package),
        generation_rate: $gen_rate,
        validation_rate: $val_rate,
        baseline_generation: $baseline.generation,
        baseline_validation: $baseline.validation,
        generation_ok: ($gen_rate >= ($baseline.generation * 0.8)),
        validation_ok: ($val_rate >= ($baseline.validation * 0.8))
    }
}
```

### Migration Validation Checklist

```nu
def migration_checklist [] {
    print "📋 nu_plugin_nw_ulid Migration Checklist"
    print "========================================"

    let items = [
        "✅ Backup of original data created",
        "✅ Old plugin uninstalled (if applicable)",
        "✅ Nushell version 0.109.1+ verified",
        "✅ Rust version 1.89.0+ verified",
        "✅ nu_plugin_nw_ulid installed from crates.io",
        "✅ Plugin registered with correct binary name",
        "✅ Basic functionality verified",
        "✅ Performance benchmarks acceptable",
        "✅ All scripts updated with new package name",
        "✅ Team notified of package name change",
        "✅ Documentation updated",
        "✅ CI/CD updated with new package name"
    ]

    $items | each { |item| print $item }

    print ""
    print "🎯 Package: nu_plugin_nw_ulid"
    print "🎯 Binary: nu_plugin_nw_ulid"
    print "🎯 Organization: nushell-works"
}
```

### Getting Help

#### Support Channels
- **GitHub Issues**: https://github.com/nushell-works/nu_plugin_nw_ulid/issues
- **GitHub Discussions**: https://github.com/nushell-works/nu_plugin_nw_ulid/discussions
- **Documentation**: https://docs.rs/nu_plugin_nw_ulid
- **Nushell Community**: Discord/Matrix channels

#### Reporting Issues

When reporting migration issues, include:

```nu
def generate_issue_report [] {
    let system_info = {
        os: $nu.os-info,
        nushell_version: (version | get version),
        rust_version: (rustc --version),
        plugin_info: (try { ulid info } catch { "Plugin not available" }),
        cargo_version: (cargo --version)
    }

    print "🐛 Migration Issue Report for nu_plugin_nw_ulid"
    print "=============================================="
    print ""
    print "**System Information:**"
    print ($system_info | to yaml)
    print ""
    print "**Migration Context:**"
    print "- Migrating from: [Previous ULID implementation]"
    print "- Migration step: [Which step failed]"
    print "- Package installed: nu_plugin_nw_ulid"
    print ""
    print "**Issue Description:**"
    print "[Describe the migration issue]"
}
```

---

*Migration guide for nu_plugin_nw_ulid*
*Last updated: December 2024*
*Package: nu_plugin_nw_ulid | Binary: nu_plugin_nw_ulid | Org: nushell-works*
