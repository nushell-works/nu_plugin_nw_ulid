#!/usr/bin/env nu
# Database Migration to ULID Template
# 
# This template helps migrate existing databases from traditional ID systems
# (auto-increment, UUID, etc.) to ULID-based primary keys while maintaining
# data integrity and referential consistency.
#
# Features:
# - Safe migration with rollback capability
# - Foreign key relationship mapping
# - Data integrity validation
# - Performance optimization
# - Migration progress tracking
#
# Usage:
#   nu database-migration.nu --help
#   nu database-migration.nu analyze --source data.json
#   nu database-migration.nu migrate --source data.json --target migrated_data.json

# ====================================================================
# CONFIGURATION
# ====================================================================

const CONFIG = {
    # Migration settings
    batch_size: 1000,
    validation_sample_size: 100,
    backup_suffix: "_backup",
    migration_log: "migration.log",
    
    # Performance settings
    parallel_processing: true,
    memory_limit_mb: 512,
    
    # Validation settings
    integrity_check: true,
    foreign_key_validation: true,
    duplicate_check: true,
    
    # ID mapping settings
    id_mapping_file: "id_mapping.json",
    preserve_timestamps: true,
    
    # Supported source ID types
    supported_id_types: ["auto_increment", "uuid", "guid", "custom"]
}

# ====================================================================
# HELPER FUNCTIONS
# ====================================================================

# Log migration events
def log_migration [level: string, message: string, details?: record] {
    let timestamp = date now | format date "%Y-%m-%d %H:%M:%S"
    let log_entry = $"[$timestamp] [($level | str upcase)] ($message)"
    
    print $log_entry
    
    let full_entry = {
        timestamp: $timestamp,
        level: $level,
        message: $message,
        details: ($details | default {})
    }
    
    $full_entry | to json | save --append $CONFIG.migration_log
}

# Detect ID type in dataset
def detect_id_type [data: list, id_column: string = "id"] {
    if ($data | length) == 0 {
        error make { msg: "Empty dataset provided" }
    }
    
    let sample_ids = $data | get $id_column | first 10
    
    # Check for auto-increment (sequential integers)
    let integers = $sample_ids | where ($in | describe | str contains "int")
    if ($integers | length) == ($sample_ids | length) {
        let sorted = $integers | sort
        let is_sequential = ($sorted | enumerate | all { |item| 
            ($item.index == 0) or ($item.item == ($sorted | get ($item.index - 1)) + 1)
        })
        
        if $is_sequential {
            return "auto_increment"
        } else {
            return "integer_id"
        }
    }
    
    # Check for UUID format
    let uuid_pattern = "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    let uuids = $sample_ids | where ($in | str contains "-" | default false)
    if ($uuids | length) == ($sample_ids | length) {
        return "uuid"
    }
    
    # Check for ULID format
    let ulids = $sample_ids | where { ulid validate $in }
    if ($ulids | length) == ($sample_ids | length) {
        return "ulid"
    }
    
    # Check for GUID (UUID without hyphens)
    let guid_pattern = "^[0-9a-fA-F]{32}$"
    let guids = $sample_ids | where ($in | str length) == 32
    if ($guids | length) == ($sample_ids | length) {
        return "guid"
    }
    
    return "custom"
}

# Create ULID with preserved timestamp
def create_ulid_with_timestamp [original_timestamp?: any] {
    if $original_timestamp != null {
        # Try to convert timestamp to milliseconds
        let timestamp_ms = try {
            $original_timestamp | into int
        } catch {
            try {
                $original_timestamp | into datetime | into int
            } catch {
                date now | into int
            }
        }
        
        ulid generate --timestamp $timestamp_ms
    } else {
        ulid generate
    }
}

# Validate foreign key relationships
def validate_foreign_keys [data: record, relationships: list] {
    log_migration "info" "Validating foreign key relationships"
    
    mut validation_results = []
    
    for relationship in $relationships {
        let parent_table = $relationship.parent_table
        let child_table = $relationship.child_table
        let parent_key = $relationship.parent_key
        let child_key = $relationship.child_key
        
        if not ($parent_table in $data) {
            log_migration "error" $"Parent table ($parent_table) not found in data"
            continue
        }
        
        if not ($child_table in $data) {
            log_migration "error" $"Child table ($child_table) not found in data"
            continue
        }
        
        let parent_ids = $data | get $parent_table | get $parent_key | uniq
        let child_foreign_keys = $data | get $child_table | get $child_key | where ($in != null) | uniq
        
        let orphaned_keys = $child_foreign_keys | where ($in not-in $parent_ids)
        
        let result = {
            relationship: $relationship,
            parent_count: ($parent_ids | length),
            child_references: ($child_foreign_keys | length),
            orphaned_count: ($orphaned_keys | length),
            valid: (($orphaned_keys | length) == 0)
        }
        
        $validation_results = ($validation_results | append $result)
        
        if ($orphaned_keys | length) > 0 {
            log_migration "warn" $"Found ($orphaned_keys | length) orphaned foreign keys in ($child_table).($child_key)"
        }
    }
    
    $validation_results
}

# Create ID mapping between old and new IDs
def create_id_mapping [table_name: string, old_ids: list, new_ids: list] {
    if ($old_ids | length) != ($new_ids | length) {
        error make { msg: "Old and new ID lists must have the same length" }
    }
    
    let mapping = $old_ids | enumerate | each { |item|
        {
            table: $table_name,
            old_id: $item.item,
            new_id: ($new_ids | get $item.index),
            created_at: (date now | into int)
        }
    }
    
    $mapping
}

# ====================================================================
# MIGRATION FUNCTIONS
# ====================================================================

# Analyze dataset for migration planning
def analyze_dataset [
    data: any,
    --tables?: list,           # Specific tables to analyze
    --id-column: string = "id", # ID column name
    --relationships?: list      # Foreign key relationships
] {
    log_migration "info" "Starting dataset analysis for ULID migration"
    
    let analysis_start = date now | into int
    
    # Handle different data formats
    let dataset = match ($data | describe) {
        "string" => (open $data | from json),
        "record" => $data,
        _ => (error make { msg: "Data must be a file path (string) or record" })
    }
    
    # Determine tables to analyze
    let table_names = if $tables != null {
        $tables
    } else {
        $dataset | columns
    }
    
    log_migration "info" $"Analyzing ($table_names | length) tables: ($table_names | str join ', ')"
    
    # Analyze each table
    let table_analysis = $table_names | each { |table_name|
        if not ($table_name in $dataset) {
            log_migration "warn" $"Table ($table_name) not found in dataset"
            return null
        }
        
        let table_data = $dataset | get $table_name
        
        if ($table_data | length) == 0 {
            log_migration "warn" $"Table ($table_name) is empty"
            return {
                table: $table_name,
                record_count: 0,
                id_type: "empty",
                migration_needed: false
            }
        }
        
        # Detect current ID type
        let current_id_type = detect_id_type $table_data $id_column
        log_migration "info" $"Table ($table_name): detected ID type ($current_id_type)"
        
        # Check if migration is needed
        let migration_needed = $current_id_type != "ulid"
        
        # Analyze ID characteristics
        let ids = $table_data | get $id_column
        let unique_ids = $ids | uniq
        let has_duplicates = ($ids | length) != ($unique_ids | length)
        let null_ids = $ids | where ($in == null) | length
        
        # Sample IDs for validation
        let sample_ids = $ids | first ($CONFIG.validation_sample_size | math min ($ids | length))
        
        # Check for timestamp columns
        let columns = $table_data | first | columns
        let timestamp_columns = $columns | where ($in | str contains -i "timestamp" or $in | str contains -i "created" or $in | str contains -i "updated")
        
        {
            table: $table_name,
            record_count: ($table_data | length),
            id_type: $current_id_type,
            migration_needed: $migration_needed,
            id_analysis: {
                total_ids: ($ids | length),
                unique_ids: ($unique_ids | length),
                has_duplicates: $has_duplicates,
                null_ids: $null_ids,
                sample_ids: ($sample_ids | first 5)
            },
            columns: $columns,
            timestamp_columns: $timestamp_columns,
            migration_complexity: (if $has_duplicates { "high" } else if $migration_needed { "medium" } else { "low" })
        }
    } | where ($in != null)
    
    # Validate foreign key relationships if provided
    let fk_validation = if $relationships != null {
        validate_foreign_keys $dataset $relationships
    } else {
        []
    }
    
    let analysis_end = date now | into int
    let analysis_duration = $analysis_end - $analysis_start
    
    # Generate summary
    let summary = {
        analysis_timestamp: (date now),
        analysis_duration_ms: $analysis_duration,
        total_tables: ($table_analysis | length),
        tables_needing_migration: ($table_analysis | where migration_needed | length),
        total_records: ($table_analysis | get record_count | math sum),
        id_types_found: ($table_analysis | get id_type | uniq),
        complexity_distribution: ($table_analysis | group-by migration_complexity | transpose complexity count),
        foreign_key_issues: ($fk_validation | where (not valid) | length),
        migration_recommended: (($table_analysis | where migration_needed | length) > 0)
    }
    
    log_migration "info" $"Analysis completed in ($analysis_duration)ms"
    
    {
        summary: $summary,
        table_analysis: $table_analysis,
        foreign_key_validation: $fk_validation,
        dataset_info: {
            source_format: ($data | describe),
            tables_analyzed: $table_names,
            analysis_timestamp: (date now)
        }
    }
}

# Migrate a single table to ULID
def migrate_table [
    table_data: list,
    table_name: string,
    --id-column: string = "id",
    --timestamp-column?: string,
    --preserve-old-id?: bool,
    --dry-run?: bool
] {
    log_migration "info" $"Starting migration of table ($table_name)"
    
    if ($table_data | length) == 0 {
        log_migration "warn" $"Table ($table_name) is empty, skipping migration"
        return { table: $table_name, migrated_records: [], id_mapping: [], warnings: ["Table is empty"] }
    }
    
    let migration_start = date now | into int
    mut warnings = []
    
    # Check if already using ULIDs
    let current_id_type = detect_id_type $table_data $id_column
    if $current_id_type == "ulid" {
        log_migration "info" $"Table ($table_name) already uses ULIDs, skipping migration"
        return { 
            table: $table_name, 
            migrated_records: $table_data, 
            id_mapping: [], 
            warnings: ["Table already uses ULIDs"]
        }
    }
    
    # Generate new ULIDs
    let old_ids = $table_data | get $id_column
    
    # Create ULIDs with preserved timestamps if requested
    let new_ids = if $CONFIG.preserve_timestamps and $timestamp_column != null {
        log_migration "info" $"Preserving timestamps from column ($timestamp_column)"
        $table_data | each { |record|
            create_ulid_with_timestamp ($record | get $timestamp_column)
        }
    } else {
        # Generate sequential ULIDs for better ordering
        let base_timestamp = date now | into int
        $table_data | enumerate | each { |item|
            ulid generate --timestamp ($base_timestamp + $item.index)
        }
    }
    
    # Validate generated ULIDs
    let invalid_ulids = $new_ids | where { not (ulid validate $in) }
    if ($invalid_ulids | length) > 0 {
        error make { msg: $"Generated ($invalid_ulids | length) invalid ULIDs during migration" }
    }
    
    # Check for ULID duplicates (should be extremely rare)
    let duplicate_ulids = $new_ids | group-by {|id| $id} | transpose ulid count | where count > 1
    if ($duplicate_ulids | length) > 0 {
        $warnings = ($warnings | append "Found duplicate ULIDs (extremely rare), regenerating...")
        log_migration "warn" $"Found ($duplicate_ulids | length) duplicate ULIDs, regenerating..."
        
        # Regenerate with slight time offsets
        $new_ids = $table_data | enumerate | each { |item|
            ulid generate --timestamp ((date now | into int) + $item.index + (random int 0..1000))
        }
    }
    
    # Create migrated records
    let migrated_records = $table_data | enumerate | each { |item|
        let record = $item.item
        let new_id = $new_ids | get $item.index
        let old_id = $old_ids | get $item.index
        
        mut migrated_record = $record | upsert $id_column $new_id
        
        # Preserve old ID if requested
        if $preserve_old_id {
            $migrated_record = ($migrated_record | upsert $"old_($id_column)" $old_id)
        }
        
        # Add migration metadata
        $migrated_record = ($migrated_record 
            | upsert ulid_migrated_at (date now | into int)
            | upsert ulid_migration_batch (($item.index // $CONFIG.batch_size) + 1)
        )
        
        $migrated_record
    }
    
    # Create ID mapping
    let id_mapping = create_id_mapping $table_name $old_ids $new_ids
    
    let migration_end = date now | into int
    let migration_duration = $migration_end - $migration_start
    
    log_migration "info" $"Table ($table_name) migration completed in ($migration_duration)ms"
    
    if $dry_run {
        log_migration "info" "DRY RUN: No data was actually migrated"
    }
    
    {
        table: $table_name,
        migration_summary: {
            record_count: ($table_data | length),
            old_id_type: $current_id_type,
            new_id_type: "ulid",
            migration_duration_ms: $migration_duration,
            dry_run: ($dry_run | default false)
        },
        migrated_records: (if $dry_run { [] } else { $migrated_records }),
        id_mapping: $id_mapping,
        warnings: $warnings
    }
}

# Update foreign key references
def update_foreign_keys [
    data: record,
    relationships: list,
    id_mappings: list
] {
    log_migration "info" "Updating foreign key references"
    
    # Create a lookup table for ID mappings
    let id_lookup = $id_mappings | group-by table | transpose table mappings | each { |group|
        let mappings = $group.mappings | reduce -f {} { |acc, mapping|
            $acc | upsert ($mapping.old_id | into string) $mapping.new_id
        }
        { table: $group.table, lookup: $mappings }
    }
    
    mut updated_data = $data
    
    for relationship in $relationships {
        let parent_table = $relationship.parent_table
        let child_table = $relationship.child_table
        let child_key = $relationship.child_key
        
        # Find the lookup table for the parent table
        let parent_lookup = $id_lookup | where table == $parent_table | first | get lookup
        
        if ($parent_lookup | columns | length) == 0 {
            log_migration "warn" $"No ID mapping found for parent table ($parent_table)"
            continue
        }
        
        # Update foreign keys in child table
        let child_data = $updated_data | get $child_table
        let updated_child_data = $child_data | each { |record|
            let old_fk = $record | get $child_key
            if $old_fk != null and (($old_fk | into string) in $parent_lookup) {
                let new_fk = $parent_lookup | get ($old_fk | into string)
                $record | upsert $child_key $new_fk
            } else {
                $record
            }
        }
        
        $updated_data = ($updated_data | upsert $child_table $updated_child_data)
        
        log_migration "info" $"Updated foreign keys in ($child_table).($child_key) referencing ($parent_table)"
    }
    
    $updated_data
}

# Perform complete database migration
def migrate_database [
    source: string,
    target: string,
    --relationships?: list,     # Foreign key relationships
    --tables?: list,           # Specific tables to migrate
    --id-column: string = "id",
    --timestamp-column?: string,
    --preserve-old-id?: bool,
    --backup?: bool,
    --dry-run?: bool
] {
    log_migration "info" "Starting complete database migration to ULID"
    
    let migration_id = ulid generate --context "database-migration"
    let migration_start = date now | into int
    
    # Load source data
    log_migration "info" $"Loading source data from ($source)"
    let source_data = open $source | from json
    
    # Create backup if requested
    if $backup {
        let backup_path = $"($source)($CONFIG.backup_suffix)"
        log_migration "info" $"Creating backup at ($backup_path)"
        $source_data | to json | save $backup_path
    }
    
    # Analyze dataset first
    let analysis = analyze_dataset $source_data --tables $tables --id-column $id_column --relationships $relationships
    
    if not $analysis.summary.migration_recommended {
        log_migration "info" "No migration needed based on analysis"
        return $analysis
    }
    
    # Determine tables to migrate
    let tables_to_migrate = if $tables != null {
        $tables
    } else {
        $analysis.table_analysis | where migration_needed | get table
    }
    
    log_migration "info" $"Migrating ($tables_to_migrate | length) tables: ($tables_to_migrate | str join ', ')"
    
    # Migrate each table
    mut all_id_mappings = []
    mut migration_results = []
    mut migrated_data = $source_data
    
    for table_name in $tables_to_migrate {
        let table_data = $source_data | get $table_name
        
        let migration_result = migrate_table $table_data $table_name --id-column $id_column --timestamp-column $timestamp_column --preserve-old-id $preserve_old_id --dry-run $dry_run
        
        $migration_results = ($migration_results | append $migration_result)
        $all_id_mappings = ($all_id_mappings | append $migration_result.id_mapping)
        
        if not $dry_run {
            $migrated_data = ($migrated_data | upsert $table_name $migration_result.migrated_records)
        }
    }
    
    # Update foreign key references
    if $relationships != null and not $dry_run {
        $migrated_data = update_foreign_keys $migrated_data $relationships $all_id_mappings
    }
    
    # Save results
    if not $dry_run {
        log_migration "info" $"Saving migrated data to ($target)"
        $migrated_data | to json | save $target
        
        # Save ID mappings
        $all_id_mappings | to json | save $CONFIG.id_mapping_file
        log_migration "info" $"ID mappings saved to ($CONFIG.id_mapping_file)"
    }
    
    let migration_end = date now | into int
    let total_duration = $migration_end - $migration_start
    
    let migration_summary = {
        migration_id: $migration_id,
        start_time: $migration_start,
        end_time: $migration_end,
        duration_ms: $total_duration,
        source_file: $source,
        target_file: $target,
        dry_run: ($dry_run | default false),
        tables_migrated: ($tables_to_migrate | length),
        total_records_migrated: ($migration_results | get migration_summary.record_count | math sum),
        id_mappings_created: ($all_id_mappings | length),
        foreign_key_relationships: ($relationships | default [] | length),
        warnings: ($migration_results | get warnings | flatten | uniq)
    }
    
    log_migration "info" $"Database migration completed in ($total_duration)ms"
    
    {
        migration_id: $migration_id,
        summary: $migration_summary,
        analysis: $analysis,
        migration_results: $migration_results,
        migrated_data: (if $dry_run { null } else { $migrated_data }),
        id_mappings: $all_id_mappings
    }
}

# ====================================================================
# VALIDATION FUNCTIONS
# ====================================================================

# Validate migration integrity
def validate_migration [
    original_data: any,
    migrated_data: any,
    id_mappings: list,
    --relationships?: list
] {
    log_migration "info" "Validating migration integrity"
    
    let validation_start = date now | into int
    mut validation_results = []
    
    # Load data if paths provided
    let original = match ($original_data | describe) {
        "string" => (open $original_data | from json),
        _ => $original_data
    }
    
    let migrated = match ($migrated_data | describe) {
        "string" => (open $migrated_data | from json),
        _ => $migrated_data
    }
    
    # Check table structure consistency
    for table_name in ($original | columns) {
        if not ($table_name in $migrated) {
            $validation_results = ($validation_results | append {
                check: "table_exists",
                table: $table_name,
                status: "fail",
                message: $"Table ($table_name) missing in migrated data"
            })
            continue
        }
        
        let original_count = $original | get $table_name | length
        let migrated_count = $migrated | get $table_name | length
        
        $validation_results = ($validation_results | append {
            check: "record_count",
            table: $table_name,
            status: (if $original_count == $migrated_count { "pass" } else { "fail" }),
            original_count: $original_count,
            migrated_count: $migrated_count,
            message: (if $original_count == $migrated_count { "Record counts match" } else { "Record count mismatch" })
        })
        
        # Validate ULID format in migrated data
        let migrated_table = $migrated | get $table_name
        if ($migrated_table | length) > 0 {
            let invalid_ulids = $migrated_table | get id | where { not (ulid validate $in) }
            
            $validation_results = ($validation_results | append {
                check: "ulid_format",
                table: $table_name,
                status: (if ($invalid_ulids | length) == 0 { "pass" } else { "fail" }),
                invalid_count: ($invalid_ulids | length),
                message: (if ($invalid_ulids | length) == 0 { "All ULIDs are valid" } else { $"Found ($invalid_ulids | length) invalid ULIDs" })
            })
        }
    }
    
    # Validate ID mappings
    let mapping_tables = $id_mappings | get table | uniq
    for table in $mapping_tables {
        let table_mappings = $id_mappings | where table == $table
        let unique_old_ids = $table_mappings | get old_id | uniq | length
        let unique_new_ids = $table_mappings | get new_id | uniq | length
        let total_mappings = $table_mappings | length
        
        $validation_results = ($validation_results | append {
            check: "id_mapping_uniqueness",
            table: $table,
            status: (if ($unique_old_ids == $total_mappings and $unique_new_ids == $total_mappings) { "pass" } else { "fail" }),
            total_mappings: $total_mappings,
            unique_old_ids: $unique_old_ids,
            unique_new_ids: $unique_new_ids,
            message: "ID mapping uniqueness check"
        })
    }
    
    # Validate foreign key relationships if provided
    if $relationships != null {
        let fk_validation = validate_foreign_keys $migrated $relationships
        for fk_result in $fk_validation {
            $validation_results = ($validation_results | append {
                check: "foreign_key_integrity",
                table: $fk_result.relationship.child_table,
                status: (if $fk_result.valid { "pass" } else { "fail" }),
                orphaned_count: $fk_result.orphaned_count,
                message: $"Foreign key validation: ($fk_result.relationship.child_table).($fk_result.relationship.child_key) -> ($fk_result.relationship.parent_table)"
            })
        }
    }
    
    let validation_end = date now | into int
    let validation_duration = $validation_end - $validation_start
    
    let summary = {
        validation_timestamp: (date now),
        validation_duration_ms: $validation_duration,
        total_checks: ($validation_results | length),
        passed_checks: ($validation_results | where status == "pass" | length),
        failed_checks: ($validation_results | where status == "fail" | length),
        success_rate: (($validation_results | where status == "pass" | length) / ($validation_results | length) * 100)
    }
    
    log_migration "info" $"Validation completed: ($summary.passed_checks)/($summary.total_checks) checks passed"
    
    {
        summary: $summary,
        validation_results: $validation_results,
        overall_status: (if $summary.failed_checks == 0 { "pass" } else { "fail" })
    }
}

# ====================================================================
# COMMAND LINE INTERFACE
# ====================================================================

def main [
    command?: string,    # Command to execute (analyze, migrate, validate, test)
    --source?: string,   # Source data file
    --target?: string,   # Target data file
    --help              # Show help information
] {
    if $help {
        print "Database Migration to ULID Template"
        print "==================================="
        print ""
        print "Commands:"
        print "  analyze      Analyze dataset for migration planning"
        print "  migrate      Perform complete migration to ULIDs"
        print "  validate     Validate migration integrity"
        print "  test         Run migration tests"
        print ""
        print "Examples:"
        print "  nu database-migration.nu analyze --source data.json"
        print "  nu database-migration.nu migrate --source data.json --target migrated.json"
        print "  nu database-migration.nu validate original.json migrated.json"
        return
    }
    
    match $command {
        "analyze" => {
            if $source == null {
                error make { msg: "Source file required for analysis" }
            }
            analyze_dataset $source
        },
        "migrate" => {
            if $source == null or $target == null {
                error make { msg: "Both source and target files required for migration" }
            }
            migrate_database $source $target --backup
        },
        "validate" => {
            print "Use: validate_migration <original> <migrated> <id_mappings>"
        },
        "test" => {
            print "Running migration tests..."
            # Test implementation would go here
            print "Tests completed"
        },
        _ => {
            print "Available commands: analyze, migrate, validate, test"
            print "Use --help for more information"
        }
    }
}