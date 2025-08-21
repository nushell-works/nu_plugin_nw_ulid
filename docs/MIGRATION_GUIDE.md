# nu_plugin_nw_ulid Migration Guide

**Version**: 0.1.0
**Last Updated**: August 18, 2025
**Target Audience**: System administrators, developers, DevOps teams

This guide provides comprehensive instructions for migrating from existing ID systems to ULID-based implementations using nu_plugin_nw_ulid.

## Table of Contents

1. [Migration Overview](#migration-overview)
2. [Pre-Migration Assessment](#pre-migration-assessment)
3. [Migration Strategies](#migration-strategies)
4. [Step-by-Step Migration](#step-by-step-migration)
5. [Database Migrations](#database-migrations)
6. [Application Integration](#application-integration)
7. [Testing & Validation](#testing--validation)
8. [Rollback Procedures](#rollback-procedures)
9. [Post-Migration Optimization](#post-migration-optimization)
10. [Troubleshooting](#troubleshooting)

## Migration Overview

### Why Migrate to ULIDs?

ULIDs offer significant advantages over traditional ID systems:

- **Lexicographic Sorting**: ULIDs sort chronologically as strings
- **URL-Safe**: No special characters requiring encoding
- **Compact**: 26 characters vs 36 for UUIDs with hyphens
- **Monotonic**: Within same millisecond, values increase
- **Database Friendly**: Better index performance than UUIDs
- **Distributed System Ready**: No coordination required for generation

### Migration Scope

This guide covers migration from:
- **Auto-increment integers** (MySQL, PostgreSQL sequences)
- **UUIDs** (UUID v1, v4, custom variants)
- **Custom ID systems** (base64, custom encoding schemes)
- **Legacy identifiers** (composite keys, string IDs)

## Pre-Migration Assessment

### 1. Current System Analysis

Use the migration analysis template to assess your current system:

```nushell
# Analyze your current ID usage
def analyze_current_ids [data_source: string] {
    let sample_data = open $data_source | first 1000

    let id_analysis = $sample_data | each { |record|
        if "id" in $record {
            {
                id: $record.id,
                type: (detect_id_type $record.id),
                length: ($record.id | str length),
                format: (analyze_id_format $record.id)
            }
        }
    } | where ($in != null)

    {
        total_records: ($sample_data | length),
        id_types: ($id_analysis | group-by type | transpose type count),
        avg_length: ($id_analysis | get length | math avg),
        migration_complexity: (assess_migration_complexity $id_analysis)
    }
}

def detect_id_type [id: any] {
    let id_str = $id | into string

    if ($id_str | str contains "-") and ($id_str | str length) == 36 {
        "uuid"
    } else if ($id_str | str length) == 26 and (ulid validate $id_str) {
        "ulid"
    } else if ($id_str | str contains -E "^\\d+$") {
        "integer"
    } else {
        "custom"
    }
}

def assess_migration_complexity [id_analysis: list] {
    let types = $id_analysis | get type | uniq
    let has_foreign_keys = true  # Assess based on your schema
    let record_count = $id_analysis | length

    if ($types | length) > 1 or $has_foreign_keys or $record_count > 1000000 {
        "high"
    } else if $record_count > 100000 {
        "medium"
    } else {
        "low"
    }
}
```

### 2. Dependency Mapping

Identify all systems that depend on your current IDs:

```nushell
# Map ID dependencies
def map_id_dependencies [] {
    {
        databases: [
            { name: "primary_db", tables: ["users", "orders", "products"], foreign_keys: ["user_id", "order_id"] },
            { name: "analytics_db", tables: ["events"], foreign_keys: ["user_id"] }
        ],
        services: [
            { name: "api_service", endpoints: ["/users/{id}", "/orders/{id}"] },
            { name: "auth_service", tokens: ["session_id", "refresh_token"] }
        ],
        external_systems: [
            { name: "payment_gateway", id_fields: ["transaction_id"] },
            { name: "crm_system", id_fields: ["customer_id"] }
        ],
        file_systems: [
            { path: "/logs/", id_patterns: ["request_id", "trace_id"] },
            { path: "/exports/", id_patterns: ["batch_id"] }
        ]
    }
}
```

### 3. Migration Planning Checklist

- [ ] **Data Volume Assessment**: Count records in each table
- [ ] **Foreign Key Mapping**: Identify all relationships
- [ ] **External Dependencies**: List all systems using current IDs
- [ ] **Downtime Requirements**: Determine acceptable maintenance windows
- [ ] **Backup Strategy**: Plan comprehensive backups
- [ ] **Rollback Plan**: Design rollback procedures
- [ ] **Testing Environment**: Set up migration testing
- [ ] **Performance Impact**: Assess migration performance requirements

## Migration Strategies

### 1. Big Bang Migration

**Description**: Migrate all systems simultaneously during a maintenance window.

**Pros:**
- Clean, complete migration
- No dual-system complexity
- Immediate benefits

**Cons:**
- Requires downtime
- Higher risk
- Complex coordination

**Best For:**
- Small to medium systems
- Acceptable downtime windows
- Simple dependencies

```nushell
# Big bang migration workflow
def big_bang_migration [config: record] {
    print "🚀 Starting Big Bang Migration"

    # 1. System shutdown
    print "📴 Shutting down services..."
    stop_all_services $config.services

    # 2. Database backup
    print "💾 Creating backups..."
    create_full_backup $config.backup_location

    # 3. Migration execution
    print "🔄 Executing migration..."
    let migration_result = execute_migration $config.migration_plan

    # 4. Validation
    print "✅ Validating migration..."
    let validation_result = validate_migration $migration_result

    if $validation_result.success {
        # 5. Service restart
        print "🟢 Starting services..."
        start_all_services $config.services
        print "✅ Big Bang Migration completed successfully"
    } else {
        # Rollback
        print "❌ Migration validation failed, rolling back..."
        rollback_migration $config.backup_location
        start_all_services $config.services
        error make { msg: "Migration failed and was rolled back" }
    }
}
```

### 2. Gradual Migration (Blue-Green)

**Description**: Migrate systems incrementally with dual ID support.

**Pros:**
- Zero downtime
- Lower risk
- Gradual validation

**Cons:**
- More complex
- Longer timeline
- Dual-system overhead

**Best For:**
- Large systems
- 24/7 availability requirements
- Complex dependencies

```nushell
# Gradual migration workflow
def gradual_migration [config: record] {
    print "🔄 Starting Gradual Migration"

    # Phase 1: Add ULID columns
    print "📋 Phase 1: Adding ULID columns..."
    add_ulid_columns $config.tables

    # Phase 2: Populate ULIDs
    print "🔢 Phase 2: Populating ULIDs..."
    populate_ulids $config.tables

    # Phase 3: Update foreign keys
    print "🔗 Phase 3: Updating foreign key references..."
    update_foreign_key_references $config.relationships

    # Phase 4: Application updates
    print "💻 Phase 4: Updating applications..."
    update_applications_gradual $config.applications

    # Phase 5: Switch primary keys
    print "🔄 Phase 5: Switching to ULID primary keys..."
    switch_to_ulid_primary_keys $config.tables

    # Phase 6: Cleanup
    print "🧹 Phase 6: Cleaning up old ID columns..."
    cleanup_old_id_columns $config.tables

    print "✅ Gradual Migration completed successfully"
}
```

### 3. Hybrid Approach

**Description**: Maintain both ID systems permanently for compatibility.

**Pros:**
- Maximum compatibility
- Gradual adoption
- Flexible rollback

**Cons:**
- Permanent complexity
- Storage overhead
- Maintenance burden

**Best For:**
- Legacy system integration
- External API compatibility
- Long-term transitions

```nushell
# Hybrid approach implementation
def hybrid_migration [config: record] {
    print "🔀 Starting Hybrid Migration"

    # Add ULIDs alongside existing IDs
    add_ulid_columns_hybrid $config.tables

    # Set up ID synchronization
    setup_id_synchronization $config

    # Update applications to use ULIDs where possible
    update_applications_hybrid $config.applications

    # Maintain mapping between old and new IDs
    maintain_id_mapping $config

    print "✅ Hybrid Migration setup completed"
}
```

## Step-by-Step Migration

### Phase 1: Preparation

#### 1.1 Environment Setup

```bash
# Install nu_plugin_nw_ulid
cargo install nu_plugin_nw_ulid

# Register plugin
plugin add ~/.cargo/bin/nu_plugin_nw_ulid
plugin use nw_ulid

# Verify installation
ulid info
```

#### 1.2 Backup Creation

```nushell
# Create comprehensive backup
def create_migration_backup [backup_location: string] {
    let timestamp = date now | format date "%Y%m%d_%H%M%S"
    let backup_dir = $"($backup_location)/migration_backup_($timestamp)"

    mkdir $backup_dir

    # Database backup
    print "💾 Creating database backup..."
    pg_dump your_database > $"($backup_dir)/database_backup.sql"

    # Configuration backup
    print "⚙️  Backing up configurations..."
    cp -r /path/to/config $"($backup_dir)/config"

    # Application code backup
    print "📁 Backing up application code..."
    git archive --format=tar.gz HEAD > $"($backup_dir)/app_backup.tar.gz"

    print $"✅ Backup created at ($backup_dir)"
    $backup_dir
}
```

#### 1.3 Migration Scripts Preparation

```nushell
# Generate migration scripts
def generate_migration_scripts [schema: record] {
    let migration_dir = "migration_scripts"
    mkdir $migration_dir

    # Generate table migration scripts
    $schema.tables | each { |table|
        let script_content = generate_table_migration_script $table
        $script_content | save $"($migration_dir)/migrate_($table.name).sql"
    }

    # Generate data migration scripts
    let data_script = generate_data_migration_script $schema
    $data_script | save $"($migration_dir)/migrate_data.nu"

    # Generate rollback scripts
    let rollback_script = generate_rollback_script $schema
    $rollback_script | save $"($migration_dir)/rollback.nu"

    print $"✅ Migration scripts generated in ($migration_dir)"
}
```

### Phase 2: Schema Migration

#### 2.1 Add ULID Columns

```sql
-- Example SQL for adding ULID columns
ALTER TABLE users ADD COLUMN ulid_id VARCHAR(26);
ALTER TABLE orders ADD COLUMN ulid_id VARCHAR(26);
ALTER TABLE products ADD COLUMN ulid_id VARCHAR(26);

-- Add indexes for performance
CREATE INDEX idx_users_ulid ON users(ulid_id);
CREATE INDEX idx_orders_ulid ON orders(ulid_id);
CREATE INDEX idx_products_ulid ON products(ulid_id);
```

#### 2.2 Populate ULIDs

```nushell
# Populate ULID columns with timestamp preservation
def populate_table_ulids [table_name: string, preserve_timestamps: bool = true] {
    print $"🔢 Populating ULIDs for table: ($table_name)"

    # Get records that need ULIDs
    let records = query_database $"SELECT id, created_at FROM ($table_name) WHERE ulid_id IS NULL"

    print $"📊 Found ($records | length) records to process"

    # Process in batches for performance
    $records | chunks 1000 | enumerate | each { |batch|
        print $"Processing batch ($batch.index + 1)..."

        let batch_updates = $batch.item | each { |record|
            let ulid = if $preserve_timestamps and ($record.created_at != null) {
                let timestamp_ms = $record.created_at | into datetime | into int
                ulid generate --timestamp $timestamp_ms
            } else {
                ulid generate
            }

            {
                id: $record.id,
                ulid: $ulid
            }
        }

        # Execute batch update
        update_batch_ulids $table_name $batch_updates
    }

    print $"✅ ULID population completed for ($table_name)"
}

def update_batch_ulids [table: string, updates: list] {
    # Generate batch UPDATE statement
    let update_cases = $updates | each { |update|
        $"WHEN id = ($update.id) THEN '($update.ulid)'"
    } | str join " "

    let ids = $updates | get id | str join ", "

    let sql = $"
        UPDATE ($table)
        SET ulid_id = CASE
            ($update_cases)
        END
        WHERE id IN (($ids))
    "

    execute_sql $sql
}
```

### Phase 3: Foreign Key Migration

#### 3.1 Update Foreign Key References

```nushell
# Update foreign key references to use ULIDs
def migrate_foreign_keys [relationships: list] {
    $relationships | each { |rel|
        print $"🔗 Migrating foreign keys: ($rel.child_table).($rel.foreign_key) -> ($rel.parent_table)"

        let sql = $"
            UPDATE ($rel.child_table)
            SET ($rel.foreign_key)_ulid = (
                SELECT ulid_id
                FROM ($rel.parent_table)
                WHERE ($rel.parent_table).id = ($rel.child_table).($rel.foreign_key)
            )
            WHERE ($rel.foreign_key) IS NOT NULL
        "

        execute_sql $sql

        # Verify migration
        let verification = verify_foreign_key_migration $rel
        if not $verification.success {
            error make { msg: $"Foreign key migration failed for ($rel.child_table).($rel.foreign_key)" }
        }

        print $"✅ Foreign key migration completed: ($rel.child_table).($rel.foreign_key)"
    }
}

def verify_foreign_key_migration [relationship: record] {
    let sql = $"
        SELECT COUNT(*) as total_count,
               COUNT(($relationship.foreign_key)_ulid) as migrated_count
        FROM ($relationship.child_table)
        WHERE ($relationship.foreign_key) IS NOT NULL
    "

    let result = query_database $sql | first

    {
        success: ($result.total_count == $result.migrated_count),
        total: $result.total_count,
        migrated: $result.migrated_count,
        missing: ($result.total_count - $result.migrated_count)
    }
}
```

### Phase 4: Application Migration

#### 4.1 Update Application Code

```nushell
# Application migration checklist
def migrate_application_code [app_config: record] {
    print $"💻 Migrating application: ($app_config.name)"

    # 1. Update database models
    update_database_models $app_config.models_path

    # 2. Update API endpoints
    update_api_endpoints $app_config.api_path

    # 3. Update frontend components
    update_frontend_components $app_config.frontend_path

    # 4. Update configuration files
    update_configuration_files $app_config.config_path

    # 5. Update tests
    update_test_files $app_config.tests_path

    print $"✅ Application migration completed: ($app_config.name)"
}

# Example: Update database queries
def update_database_queries [file_path: string] {
    let content = open $file_path

    # Replace old ID patterns with ULID patterns
    let updated_content = $content
        | str replace --all "WHERE id = ?" "WHERE ulid_id = ?"
        | str replace --all "SELECT id," "SELECT ulid_id as id,"
        | str replace --all "INSERT INTO.*VALUES.*\\?" "INSERT INTO table (ulid_id, ...) VALUES (?, ...)"

    $updated_content | save $file_path

    print $"✅ Updated database queries in ($file_path)"
}
```

## Database Migrations

### PostgreSQL Migration

```sql
-- PostgreSQL-specific ULID migration
BEGIN;

-- Add ULID extension (if using pgcrypto for generation)
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Add ULID columns
ALTER TABLE users ADD COLUMN ulid_id VARCHAR(26);
ALTER TABLE orders ADD COLUMN ulid_id VARCHAR(26);

-- Populate ULIDs (example with timestamp preservation)
UPDATE users
SET ulid_id = generate_ulid_with_timestamp(EXTRACT(EPOCH FROM created_at) * 1000)
WHERE ulid_id IS NULL;

-- Add constraints and indexes
ALTER TABLE users ADD CONSTRAINT users_ulid_unique UNIQUE (ulid_id);
CREATE INDEX CONCURRENTLY idx_users_ulid ON users(ulid_id);

-- Update foreign keys
ALTER TABLE orders ADD COLUMN user_ulid_id VARCHAR(26);
UPDATE orders
SET user_ulid_id = (SELECT ulid_id FROM users WHERE users.id = orders.user_id);

-- Switch primary keys (careful operation)
ALTER TABLE users DROP CONSTRAINT users_pkey;
ALTER TABLE users ADD CONSTRAINT users_pkey PRIMARY KEY (ulid_id);

COMMIT;
```

### MySQL Migration

```sql
-- MySQL-specific ULID migration
START TRANSACTION;

-- Add ULID columns
ALTER TABLE users ADD COLUMN ulid_id VARCHAR(26);
ALTER TABLE orders ADD COLUMN ulid_id VARCHAR(26);

-- Populate ULIDs
UPDATE users
SET ulid_id = GENERATE_ULID_WITH_TIMESTAMP(UNIX_TIMESTAMP(created_at) * 1000)
WHERE ulid_id IS NULL;

-- Add indexes
ALTER TABLE users ADD UNIQUE KEY idx_users_ulid (ulid_id);
ALTER TABLE orders ADD KEY idx_orders_user_ulid (user_ulid_id);

-- Update foreign keys
ALTER TABLE orders ADD COLUMN user_ulid_id VARCHAR(26);
UPDATE orders o
JOIN users u ON o.user_id = u.id
SET o.user_ulid_id = u.ulid_id;

COMMIT;
```

### MongoDB Migration

```javascript
// MongoDB ULID migration script
db.users.find({ulid_id: {$exists: false}}).forEach(function(doc) {
    var timestamp = doc.created_at ? doc.created_at.getTime() : Date.now();
    var ulid = generateULIDWithTimestamp(timestamp);

    db.users.updateOne(
        {_id: doc._id},
        {$set: {ulid_id: ulid}}
    );
});

// Create index
db.users.createIndex({ulid_id: 1}, {unique: true});

// Update references in other collections
db.orders.find({user_ulid_id: {$exists: false}}).forEach(function(order) {
    var user = db.users.findOne({_id: order.user_id});
    if (user && user.ulid_id) {
        db.orders.updateOne(
            {_id: order._id},
            {$set: {user_ulid_id: user.ulid_id}}
        );
    }
});
```

## Testing & Validation

### Migration Testing Framework

```nushell
# Comprehensive migration testing
def test_migration [test_config: record] {
    print "🧪 Starting migration testing"

    let test_results = {
        schema_tests: (test_schema_migration $test_config),
        data_integrity_tests: (test_data_integrity $test_config),
        performance_tests: (test_migration_performance $test_config),
        application_tests: (test_application_compatibility $test_config),
        rollback_tests: (test_rollback_procedures $test_config)
    }

    let overall_success = $test_results | values | all { |result| $result.success }

    {
        overall_success: $overall_success,
        test_results: $test_results,
        summary: (generate_test_summary $test_results)
    }
}

def test_data_integrity [config: record] {
    print "📊 Testing data integrity"

    # Test 1: Record count consistency
    let count_test = $config.tables | each { |table|
        let original_count = query_database $"SELECT COUNT(*) as count FROM ($table.name)" | first | get count
        let migrated_count = query_database $"SELECT COUNT(*) as count FROM ($table.name) WHERE ulid_id IS NOT NULL" | first | get count

        {
            table: $table.name,
            original_count: $original_count,
            migrated_count: $migrated_count,
            success: ($original_count == $migrated_count)
        }
    }

    # Test 2: ULID format validation
    let format_test = $config.tables | each { |table|
        let invalid_ulids = query_database $"SELECT ulid_id FROM ($table.name) WHERE ulid_id IS NOT NULL"
            | get ulid_id
            | where { not (ulid validate $in) }

        {
            table: $table.name,
            invalid_count: ($invalid_ulids | length),
            success: (($invalid_ulids | length) == 0)
        }
    }

    # Test 3: Foreign key integrity
    let fk_test = $config.relationships | each { |rel|
        let orphaned_count = query_database $"
            SELECT COUNT(*) as count
            FROM ($rel.child_table) c
            LEFT JOIN ($rel.parent_table) p ON c.($rel.foreign_key)_ulid = p.ulid_id
            WHERE c.($rel.foreign_key)_ulid IS NOT NULL AND p.ulid_id IS NULL
        " | first | get count

        {
            relationship: $"($rel.child_table).($rel.foreign_key)",
            orphaned_count: $orphaned_count,
            success: ($orphaned_count == 0)
        }
    }

    {
        success: (($count_test | all { |t| $t.success }) and ($format_test | all { |t| $t.success }) and ($fk_test | all { |t| $t.success })),
        count_tests: $count_test,
        format_tests: $format_test,
        foreign_key_tests: $fk_test
    }
}
```

### Performance Validation

```nushell
# Test migration performance impact
def test_migration_performance [config: record] {
    print "⚡ Testing migration performance"

    let benchmark_queries = [
        "SELECT * FROM users WHERE ulid_id = ?",
        "SELECT * FROM orders WHERE user_ulid_id = ?",
        "SELECT COUNT(*) FROM users WHERE ulid_id LIKE '01%'",
        "SELECT * FROM orders ORDER BY ulid_id LIMIT 100"
    ]

    let performance_results = $benchmark_queries | each { |query|
        let start = date now | into int

        # Execute query multiple times
        let iterations = 100
        for i in 1..$iterations {
            execute_query $query
        }

        let end = date now | into int
        let avg_time = ($end - $start) / $iterations

        {
            query: $query,
            avg_execution_time_ms: $avg_time,
            iterations: $iterations,
            total_time_ms: ($end - $start)
        }
    }

    {
        success: true,
        performance_results: $performance_results,
        summary: {
            avg_query_time: ($performance_results | get avg_execution_time_ms | math avg),
            max_query_time: ($performance_results | get avg_execution_time_ms | math max),
            total_test_time: ($performance_results | get total_time_ms | math sum)
        }
    }
}
```

## Rollback Procedures

### Emergency Rollback

```nushell
# Emergency rollback procedure
def emergency_rollback [backup_location: string] {
    print "🚨 EMERGENCY ROLLBACK INITIATED"

    try {
        # 1. Stop all services
        print "📴 Stopping all services..."
        stop_all_services

        # 2. Restore database from backup
        print "💾 Restoring database from backup..."
        restore_database_backup $backup_location

        # 3. Restore application configurations
        print "⚙️  Restoring configurations..."
        restore_configuration_backup $backup_location

        # 4. Restart services with old configuration
        print "🟢 Restarting services..."
        start_all_services_old_config

        # 5. Validate rollback
        print "✅ Validating rollback..."
        let validation = validate_rollback

        if $validation.success {
            print "✅ EMERGENCY ROLLBACK COMPLETED SUCCESSFULLY"
        } else {
            print "❌ ROLLBACK VALIDATION FAILED - MANUAL INTERVENTION REQUIRED"
            error make { msg: "Rollback validation failed" }
        }

    } catch { |e|
        print $"❌ ROLLBACK FAILED: ($e.msg)"
        print "📞 CONTACT EMERGENCY RESPONSE TEAM IMMEDIATELY"
        error make { msg: $"Emergency rollback failed: ($e.msg)" }
    }
}
```

### Graceful Rollback

```nushell
# Graceful rollback with validation
def graceful_rollback [rollback_config: record] {
    print "🔄 Starting graceful rollback"

    # 1. Pre-rollback validation
    let pre_validation = validate_current_state
    if not $pre_validation.safe_to_rollback {
        error make { msg: "System state is not safe for rollback" }
    }

    # 2. Create rollback checkpoint
    let checkpoint = create_rollback_checkpoint

    # 3. Switch applications back to old ID system
    print "💻 Switching applications to old ID system..."
    switch_applications_to_old_ids $rollback_config.applications

    # 4. Remove ULID constraints and indexes
    print "🗑️  Removing ULID constraints..."
    remove_ulid_constraints $rollback_config.tables

    # 5. Drop ULID columns (optional)
    if $rollback_config.drop_ulid_columns {
        print "🗑️  Dropping ULID columns..."
        drop_ulid_columns $rollback_config.tables
    }

    # 6. Validate rollback
    let validation = validate_rollback_state

    if $validation.success {
        print "✅ Graceful rollback completed successfully"
    } else {
        print "❌ Rollback validation failed, attempting emergency restore..."
        emergency_rollback $rollback_config.emergency_backup
    }
}
```

## Post-Migration Optimization

### Index Optimization

```sql
-- Optimize indexes after migration
-- Drop old ID indexes
DROP INDEX IF EXISTS idx_users_old_id;
DROP INDEX IF EXISTS idx_orders_old_id;

-- Create optimized ULID indexes
CREATE INDEX CONCURRENTLY idx_users_ulid_prefix ON users(LEFT(ulid_id, 10));
CREATE INDEX CONCURRENTLY idx_orders_ulid_timestamp ON orders((ulid_id::bytea)[1:6]);

-- Composite indexes for common queries
CREATE INDEX CONCURRENTLY idx_orders_user_ulid_status ON orders(user_ulid_id, status);
CREATE INDEX CONCURRENTLY idx_users_ulid_email ON users(ulid_id, email);
```

### Performance Monitoring

```nushell
# Post-migration performance monitoring
def monitor_post_migration_performance [] {
    print "📊 Starting post-migration performance monitoring"

    loop {
        let metrics = {
            timestamp: (date now),
            database_metrics: (collect_database_metrics),
            application_metrics: (collect_application_metrics),
            ulid_usage_metrics: (collect_ulid_usage_metrics)
        }

        # Check for performance issues
        let issues = detect_performance_issues $metrics

        if ($issues | length) > 0 {
            print $"⚠️  Performance issues detected: ($issues)"
            alert_performance_issues $issues
        }

        # Save metrics
        $metrics | to json | save --append post_migration_metrics.jsonl

        sleep 5min
    }
}

def collect_ulid_usage_metrics [] {
    {
        ulid_queries_per_second: (get_query_rate "ulid_id"),
        ulid_index_usage: (get_index_usage_stats "ulid"),
        avg_ulid_query_time: (get_avg_query_time "ulid_id"),
        ulid_validation_errors: (get_ulid_validation_error_count)
    }
}
```

## Troubleshooting

### Common Migration Issues

#### Issue 1: ULID Generation Performance

**Symptoms:**
- Slow ULID population
- High CPU usage during migration
- Migration timeouts

**Diagnosis:**
```nushell
def diagnose_ulid_generation_performance [] {
    let test_count = 10000
    let start = date now | into int

    let test_ulids = ulid generate-stream $test_count

    let end = date now | into int
    let duration = $end - $start
    let rate = $test_count / ($duration / 1000)

    print $"ULID generation rate: ($rate | math round) per second"

    if $rate < 1000 {
        print "❌ ISSUE: ULID generation is too slow"
        print "💡 SOLUTION: Use batch generation with ulid generate-stream"
    } else {
        print "✅ ULID generation performance is acceptable"
    }
}
```

**Solutions:**
- Use `ulid generate-stream` for batch generation
- Implement parallel processing for large datasets
- Optimize batch sizes based on available memory

#### Issue 2: Foreign Key Orphans

**Symptoms:**
- Foreign key constraint violations
- Missing parent records
- Data inconsistency

**Diagnosis:**
```nushell
def diagnose_foreign_key_orphans [relationship: record] {
    let orphan_query = $"
        SELECT COUNT(*) as orphan_count
        FROM ($relationship.child_table) c
        LEFT JOIN ($relationship.parent_table) p
        ON c.($relationship.foreign_key)_ulid = p.ulid_id
        WHERE c.($relationship.foreign_key)_ulid IS NOT NULL
        AND p.ulid_id IS NULL
    "

    let result = query_database $orphan_query | first

    if $result.orphan_count > 0 {
        print $"❌ ISSUE: Found ($result.orphan_count) orphaned records"
        print $"💡 SOLUTION: Check data integrity before migration"

        # Find specific orphaned records
        let orphan_details = query_database $"
            SELECT c.*, c.($relationship.foreign_key)_ulid as orphaned_ulid
            FROM ($relationship.child_table) c
            LEFT JOIN ($relationship.parent_table) p
            ON c.($relationship.foreign_key)_ulid = p.ulid_id
            WHERE c.($relationship.foreign_key)_ulid IS NOT NULL
            AND p.ulid_id IS NULL
            LIMIT 10
        "

        print "Sample orphaned records:"
        $orphan_details | table
    } else {
        print "✅ No orphaned foreign key records found"
    }
}
```

**Solutions:**
- Clean data before migration
- Use CASCADE operations carefully
- Implement orphan detection and cleanup

#### Issue 3: Application Compatibility

**Symptoms:**
- API errors after migration
- UI display issues
- Integration failures

**Diagnosis:**
```nushell
def diagnose_application_compatibility [app_config: record] {
    print $"🔍 Diagnosing application compatibility for: ($app_config.name)"

    # Test API endpoints
    let api_tests = $app_config.api_endpoints | each { |endpoint|
        try {
            let response = http get $endpoint.url
            { endpoint: $endpoint.name, status: "success", response_code: 200 }
        } catch { |e|
            { endpoint: $endpoint.name, status: "failed", error: $e.msg }
        }
    }

    # Check database queries
    let query_tests = $app_config.critical_queries | each { |query|
        try {
            let result = execute_query $query.sql
            { query: $query.name, status: "success", result_count: ($result | length) }
        } catch { |e|
            { query: $query.name, status: "failed", error: $e.msg }
        }
    }

    let failed_apis = $api_tests | where status == "failed"
    let failed_queries = $query_tests | where status == "failed"

    if ($failed_apis | length) > 0 or ($failed_queries | length) > 0 {
        print "❌ Application compatibility issues found:"
        if ($failed_apis | length) > 0 {
            print "Failed API endpoints:"
            $failed_apis | table
        }
        if ($failed_queries | length) > 0 {
            print "Failed database queries:"
            $failed_queries | table
        }
    } else {
        print "✅ Application compatibility tests passed"
    }
}
```

### Migration Health Check

```nushell
# Comprehensive migration health check
def migration_health_check [] {
    print "🏥 Performing migration health check"

    let health_checks = {
        ulid_format_check: (check_ulid_formats),
        data_integrity_check: (check_data_integrity),
        performance_check: (check_performance_metrics),
        foreign_key_check: (check_foreign_key_integrity),
        application_check: (check_application_health),
        backup_check: (check_backup_status)
    }

    let overall_health = $health_checks | values | all { |check| $check.status == "healthy" }

    print "📊 Health Check Results:"
    $health_checks | transpose check_name result | each { |check|
        let status_icon = if $check.result.status == "healthy" { "✅" } else { "❌" }
        print $"  ($status_icon) ($check.check_name): ($check.result.status)"
        if $check.result.status != "healthy" {
            print $"      Issue: ($check.result.issue)"
            print $"      Action: ($check.result.recommended_action)"
        }
    }

    if $overall_health {
        print "🎉 Migration is healthy and ready for production!"
    } else {
        print "⚠️  Migration issues detected - review and resolve before proceeding"
    }

    $health_checks
}
```

---

This migration guide provides comprehensive procedures for safely migrating to ULID-based systems. Always test migrations thoroughly in a staging environment before applying to production systems.
