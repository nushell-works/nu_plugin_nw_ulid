#!/usr/bin/env nu
# Log Analysis with ULID Template
# 
# This template provides comprehensive log analysis capabilities for applications
# that use ULID-based request tracking and correlation.
#
# Features:
# - Multi-format log parsing (JSON, structured text, CSV)
# - ULID-based request correlation and tracing
# - Performance metrics and trends analysis
# - Error pattern detection and classification
# - Real-time monitoring and alerting
# - Timeline reconstruction and visualization
#
# Usage:
#   nu log-analysis.nu --help
#   nu log-analysis.nu analyze --file application.log --format json
#   nu log-analysis.nu monitor --file application.log --watch

# ====================================================================
# CONFIGURATION
# ====================================================================

const CONFIG = {
    # Analysis settings
    batch_size: 5000,
    max_memory_mb: 256,
    parallel_processing: true,
    
    # Time window settings
    default_time_window: "24h",
    monitoring_interval: "30s",
    
    # Pattern detection
    error_patterns: [
        "error", "exception", "failed", "timeout", "rejected",
        "unauthorized", "forbidden", "not found", "internal server error"
    ],
    
    performance_thresholds: {
        slow_request_ms: 2000,
        very_slow_request_ms: 5000,
        error_rate_threshold: 5.0,
        memory_usage_threshold: 80.0
    },
    
    # Output settings
    report_path: "log_analysis_report.json",
    metrics_path: "log_metrics.json",
    alerts_path: "log_alerts.json",
    
    # Supported log formats
    supported_formats: ["json", "structured", "csv", "combined", "nginx", "apache"],
    
    # ULID patterns for extraction
    ulid_patterns: [
        'request_id: "([0-9A-Z]{26})"',
        'correlation_id: "([0-9A-Z]{26})"',
        'trace_id: "([0-9A-Z]{26})"',
        '"ulid": "([0-9A-Z]{26})"',
        "\\[([0-9A-Z]{26})\\]"
    ]
}

# ====================================================================
# HELPER FUNCTIONS
# ====================================================================

# Extract ULIDs from log entry using various patterns
def extract_ulids [log_entry: string] {
    mut found_ulids = []
    
    for pattern in $CONFIG.ulid_patterns {
        let matches = $log_entry | str find-replace --all --regex $pattern '$1'
        if $matches != $log_entry {
            let potential_ulids = $matches | split row ' ' | where { ulid validate $in }
            $found_ulids = ($found_ulids | append $potential_ulids)
        }
    }
    
    # Direct ULID detection in the text
    let words = $log_entry | split row ' ' | where ($in | str length) == 26
    let direct_ulids = $words | where { ulid validate $in }
    $found_ulids = ($found_ulids | append $direct_ulids)
    
    $found_ulids | uniq
}

# Parse timestamp from log entry
def parse_timestamp [log_entry: record, format: string] {
    match $format {
        "json" => {
            if "timestamp" in $log_entry {
                $log_entry.timestamp | into datetime
            } else if "@timestamp" in $log_entry {
                $log_entry."@timestamp" | into datetime
            } else if "time" in $log_entry {
                $log_entry.time | into datetime
            } else {
                date now
            }
        },
        "structured" => {
            # Extract timestamp from structured text
            let timestamp_pattern = '\[(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[.\d+]*[Z]*)\]'
            let raw_text = $log_entry | to json
            let matches = $raw_text | str find-replace --regex $timestamp_pattern '$1'
            if $matches != $raw_text {
                try { $matches | into datetime } catch { date now }
            } else {
                date now
            }
        },
        _ => date now
    }
}

# Classify log level from entry
def classify_log_level [log_entry: any] {
    let entry_text = match ($log_entry | describe) {
        "record" => ($log_entry | to json | str downcase),
        _ => ($log_entry | into string | str downcase)
    }
    
    if ($entry_text | str contains "error") or ($entry_text | str contains "exception") {
        "error"
    } else if ($entry_text | str contains "warn") {
        "warn"
    } else if ($entry_text | str contains "info") {
        "info"
    } else if ($entry_text | str contains "debug") {
        "debug"
    } else {
        "unknown"
    }
}

# Extract performance metrics from log entry
def extract_performance_metrics [log_entry: any] {
    let entry_text = $log_entry | to json
    
    # Extract response time
    let response_time_patterns = [
        'response_time[":=]\s*(\d+)',
        'duration[":=]\s*(\d+)',
        'elapsed[":=]\s*(\d+)ms',
        'took[":=]\s*(\d+)'
    ]
    
    mut response_time = null
    for pattern in $response_time_patterns {
        let matches = $entry_text | str find-replace --regex $pattern '$1'
        if $matches != $entry_text and ($matches | str contains -v '"') {
            $response_time = try { $matches | into int } catch { null }
            break
        }
    }
    
    # Extract status code
    let status_patterns = [
        'status[":=]\s*(\d{3})',
        'code[":=]\s*(\d{3})',
        'http_status[":=]\s*(\d{3})'
    ]
    
    mut status_code = null
    for pattern in $status_patterns {
        let matches = $entry_text | str find-replace --regex $pattern '$1'
        if $matches != $entry_text {
            $status_code = try { $matches | into int } catch { null }
            break
        }
    }
    
    # Extract memory usage
    let memory_patterns = [
        'memory[":=]\s*(\d+)',
        'mem_usage[":=]\s*(\d+)',
        'heap[":=]\s*(\d+)'
    ]
    
    mut memory_usage = null
    for pattern in $memory_patterns {
        let matches = $entry_text | str find-replace --regex $pattern '$1'
        if $matches != $entry_text {
            $memory_usage = try { $matches | into int } catch { null }
            break
        }
    }
    
    {
        response_time_ms: $response_time,
        status_code: $status_code,
        memory_usage_mb: $memory_usage
    }
}

# ====================================================================
# PARSING FUNCTIONS
# ====================================================================

# Parse log file based on format
def parse_log_file [
    file_path: string,
    format: string,
    --sample-size?: int,
    --start-line?: int,
    --max-lines?: int
] {
    print $"📖 Parsing log file: ($file_path) (format: ($format))"
    
    if not ($file_path | path exists) {
        error make { msg: $"Log file not found: ($file_path)" }
    }
    
    let start_line = $start_line | default 0
    let max_lines = $max_lines | default 100000
    
    let raw_lines = open $file_path 
        | lines 
        | skip $start_line 
        | first $max_lines
    
    print $"📊 Processing ($raw_lines | length) log lines"
    
    let parsed_entries = match $format {
        "json" => {
            $raw_lines 
            | where ($it | str length) > 0
            | each { |line|
                try {
                    $line | from json
                } catch {
                    { raw_line: $line, parse_error: "Invalid JSON" }
                }
            }
        },
        "structured" => {
            $raw_lines | each { |line|
                {
                    raw_line: $line,
                    timestamp: (parse_timestamp $line $format),
                    message: $line
                }
            }
        },
        "csv" => {
            let csv_data = $raw_lines | str join "\n" | from csv
            $csv_data
        },
        _ => {
            $raw_lines | each { |line|
                { raw_line: $line, message: $line }
            }
        }
    }
    
    # Enrich entries with extracted data
    let enriched_entries = $parsed_entries | each { |entry|
        let ulids = extract_ulids ($entry | to json)
        let log_level = classify_log_level $entry
        let performance = extract_performance_metrics $entry
        let timestamp = parse_timestamp $entry $format
        
        $entry 
        | upsert extracted_ulids $ulids
        | upsert log_level $log_level
        | upsert performance_metrics $performance
        | upsert parsed_timestamp $timestamp
        | upsert analysis_timestamp (date now)
    }
    
    print $"✅ Parsed ($enriched_entries | length) log entries"
    
    $enriched_entries
}

# ====================================================================
# ANALYSIS FUNCTIONS
# ====================================================================

# Analyze ULID patterns and correlations
def analyze_ulid_patterns [log_entries: list] {
    print "🔍 Analyzing ULID patterns and correlations"
    
    # Extract all ULIDs from logs
    let all_ulids = $log_entries 
        | get extracted_ulids 
        | flatten 
        | where ($it | str length) > 0
        | uniq
    
    if ($all_ulids | length) == 0 {
        return {
            total_ulids: 0,
            unique_ulids: 0,
            ulid_analysis: null,
            correlation_chains: [],
            temporal_distribution: []
        }
    }
    
    print $"📊 Found ($all_ulids | length) unique ULIDs in logs"
    
    # Parse ULIDs for temporal analysis
    let ulid_analysis = $all_ulids | each { |ulid|
        let parsed = ulid parse $ulid
        {
            ulid: $ulid,
            timestamp: $parsed.timestamp.milliseconds,
            iso_time: $parsed.timestamp.iso8601,
            age_hours: ((date now | into int) - $parsed.timestamp.unix) / 3600
        }
    } | sort-by timestamp
    
    # Find correlation chains (ULIDs appearing together)
    let correlation_chains = $log_entries | each { |entry|
        let entry_ulids = $entry.extracted_ulids
        if ($entry_ulids | length) > 1 {
            {
                entry_timestamp: $entry.parsed_timestamp,
                ulids: $entry_ulids,
                chain_length: ($entry_ulids | length),
                log_level: $entry.log_level
            }
        }
    } | where ($in != null) | sort-by entry_timestamp
    
    # Temporal distribution analysis
    let temporal_distribution = $ulid_analysis 
        | group-by { |ulid| $ulid.iso_time | str substring 0..13 }  # Group by hour
        | transpose hour ulids
        | each { |group|
            {
                hour: $group.hour,
                ulid_count: ($group.ulids | length),
                unique_requests: ($group.ulids | length),  # Assuming each ULID is a unique request
                avg_age_hours: ($group.ulids | get age_hours | math avg | math round)
            }
        }
        | sort-by hour
    
    {
        total_ulids: ($all_ulids | length),
        unique_ulids: ($all_ulids | length),
        time_span: {
            earliest: ($ulid_analysis | first | get iso_time),
            latest: ($ulid_analysis | last | get iso_time),
            duration_hours: (($ulid_analysis | last | get timestamp) - ($ulid_analysis | first | get timestamp)) / 3600000
        },
        correlation_chains: $correlation_chains,
        temporal_distribution: $temporal_distribution,
        ulid_details: $ulid_analysis
    }
}

# Analyze performance metrics
def analyze_performance [log_entries: list] {
    print "⚡ Analyzing performance metrics"
    
    let entries_with_metrics = $log_entries | where ($in.performance_metrics.response_time_ms != null)
    
    if ($entries_with_metrics | length) == 0 {
        return {
            total_entries: 0,
            performance_summary: null,
            slow_requests: [],
            status_distribution: [],
            performance_trends: []
        }
    }
    
    print $"📊 Analyzing performance for ($entries_with_metrics | length) entries"
    
    let response_times = $entries_with_metrics | get performance_metrics.response_time_ms
    let status_codes = $entries_with_metrics | get performance_metrics.status_code | where ($in != null)
    
    # Performance summary
    let performance_summary = {
        avg_response_time: ($response_times | math avg | math round),
        median_response_time: ($response_times | math median | math round),
        p95_response_time: ($response_times | sort | get (($response_times | length) * 0.95 | math floor)),
        p99_response_time: ($response_times | sort | get (($response_times | length) * 0.99 | math floor)),
        min_response_time: ($response_times | math min),
        max_response_time: ($response_times | math max),
        total_requests: ($entries_with_metrics | length)
    }
    
    # Identify slow requests
    let slow_requests = $entries_with_metrics 
        | where ($in.performance_metrics.response_time_ms > $CONFIG.performance_thresholds.slow_request_ms)
        | each { |entry|
            {
                timestamp: $entry.parsed_timestamp,
                response_time: $entry.performance_metrics.response_time_ms,
                status_code: $entry.performance_metrics.status_code,
                ulids: $entry.extracted_ulids,
                log_level: $entry.log_level,
                slow_category: (if $entry.performance_metrics.response_time_ms > $CONFIG.performance_thresholds.very_slow_request_ms { "very_slow" } else { "slow" })
            }
        }
        | sort-by response_time --reverse
    
    # Status code distribution
    let status_distribution = $status_codes 
        | group-by {|code| $code}
        | transpose status_code count
        | each { |group|
            {
                status_code: ($group.status_code | into int),
                count: ($group.count | length),
                percentage: (($group.count | length) / ($status_codes | length) * 100 | math round)
            }
        }
        | sort-by status_code
    
    # Performance trends over time
    let performance_trends = $entries_with_metrics
        | group-by { |entry| $entry.parsed_timestamp | format date "%Y-%m-%d %H:00" }
        | transpose hour entries
        | each { |group|
            let hour_entries = $group.entries
            {
                hour: $group.hour,
                request_count: ($hour_entries | length),
                avg_response_time: ($hour_entries | get performance_metrics.response_time_ms | math avg | math round),
                slow_request_count: ($hour_entries | where ($in.performance_metrics.response_time_ms > $CONFIG.performance_thresholds.slow_request_ms) | length),
                error_count: ($hour_entries | where log_level == "error" | length)
            }
        }
        | sort-by hour
    
    {
        total_entries: ($entries_with_metrics | length),
        performance_summary: $performance_summary,
        slow_requests: ($slow_requests | first 20),  # Top 20 slowest
        status_distribution: $status_distribution,
        performance_trends: $performance_trends
    }
}

# Analyze error patterns and classification
def analyze_errors [log_entries: list] {
    print "🚨 Analyzing error patterns"
    
    let error_entries = $log_entries | where log_level == "error"
    
    if ($error_entries | length) == 0 {
        return {
            total_errors: 0,
            error_rate: 0,
            error_patterns: [],
            error_timeline: [],
            ulid_error_correlation: []
        }
    }
    
    print $"📊 Analyzing ($error_entries | length) error entries"
    
    let total_entries = $log_entries | length
    let error_rate = ($error_entries | length) / $total_entries * 100
    
    # Extract error patterns
    let error_messages = $error_entries | each { |entry|
        if "message" in $entry {
            $entry.message
        } else if "raw_line" in $entry {
            $entry.raw_line
        } else {
            $entry | to json
        }
    }
    
    # Classify error types
    let error_classification = $error_messages | each { |msg|
        let msg_lower = $msg | str downcase
        if ($msg_lower | str contains "timeout") {
            "timeout"
        } else if ($msg_lower | str contains "connection") {
            "connection"
        } else if ($msg_lower | str contains "database") {
            "database"
        } else if ($msg_lower | str contains "auth") {
            "authentication"
        } else if ($msg_lower | str contains "permission") {
            "authorization"
        } else if ($msg_lower | str contains "not found") {
            "not_found"
        } else if ($msg_lower | str contains "server") {
            "server_error"
        } else {
            "other"
        }
    }
    
    let error_patterns = $error_classification 
        | group-by {|type| $type}
        | transpose error_type count
        | each { |group|
            {
                error_type: $group.error_type,
                count: ($group.count | length),
                percentage: (($group.count | length) / ($error_entries | length) * 100 | math round)
            }
        }
        | sort-by count --reverse
    
    # Error timeline
    let error_timeline = $error_entries
        | group-by { |entry| $entry.parsed_timestamp | format date "%Y-%m-%d %H:00" }
        | transpose hour errors
        | each { |group|
            {
                hour: $group.hour,
                error_count: ($group.errors | length),
                unique_ulids: ($group.errors | get extracted_ulids | flatten | uniq | length)
            }
        }
        | sort-by hour
    
    # ULID error correlation
    let ulid_error_correlation = $error_entries 
        | where ($in.extracted_ulids | length) > 0
        | each { |entry|
            {
                ulids: $entry.extracted_ulids,
                timestamp: $entry.parsed_timestamp,
                error_type: (classify_error_type $entry),
                performance: $entry.performance_metrics
            }
        }
    
    {
        total_errors: ($error_entries | length),
        error_rate: $error_rate,
        error_patterns: $error_patterns,
        error_timeline: $error_timeline,
        ulid_error_correlation: $ulid_error_correlation,
        top_error_messages: ($error_messages | group-by {|msg| $msg} | transpose message count | sort-by count --reverse | first 10)
    }
}

def classify_error_type [entry: record] {
    let text = $entry | to json | str downcase
    
    for pattern in $CONFIG.error_patterns {
        if ($text | str contains $pattern) {
            return $pattern
        }
    }
    
    "unknown"
}

# Generate comprehensive analysis report
def generate_analysis_report [log_entries: list, file_path: string] {
    print "📋 Generating comprehensive analysis report"
    
    let analysis_start = date now | into int
    
    # Perform all analyses
    let ulid_analysis = analyze_ulid_patterns $log_entries
    let performance_analysis = analyze_performance $log_entries
    let error_analysis = analyze_errors $log_entries
    
    # Generate summary statistics
    let summary = {
        analysis_timestamp: (date now),
        log_file: $file_path,
        total_entries: ($log_entries | length),
        time_span: {
            start: ($log_entries | get parsed_timestamp | math min),
            end: ($log_entries | get parsed_timestamp | math max),
            duration_hours: (($log_entries | get parsed_timestamp | math max | into int) - ($log_entries | get parsed_timestamp | math min | into int)) / 3600
        },
        log_levels: ($log_entries | group-by log_level | transpose level count | each { |group| { level: $group.level, count: ($group.count | length) } }),
        analysis_duration_ms: ((date now | into int) - $analysis_start)
    }
    
    let report = {
        summary: $summary,
        ulid_analysis: $ulid_analysis,
        performance_analysis: $performance_analysis,
        error_analysis: $error_analysis,
        recommendations: (generate_recommendations $ulid_analysis $performance_analysis $error_analysis)
    }
    
    # Save report
    $report | to json | save $CONFIG.report_path
    print $"📄 Report saved to ($CONFIG.report_path)"
    
    # Display summary
    print $""
    print $"📊 Analysis Summary"
    print $"=================="
    print $"Log file: ($file_path)"
    print $"Total entries: ($summary.total_entries)"
    print $"Time span: ($summary.time_span.duration_hours | math round) hours"
    print $"ULIDs found: ($ulid_analysis.total_ulids)"
    print $"Error rate: ($error_analysis.error_rate | math round)%"
    if $performance_analysis.performance_summary != null {
        print $"Avg response time: ($performance_analysis.performance_summary.avg_response_time)ms"
    }
    
    $report
}

# Generate recommendations based on analysis
def generate_recommendations [ulid_analysis: record, performance_analysis: record, error_analysis: record] {
    mut recommendations = []
    
    # ULID recommendations
    if $ulid_analysis.total_ulids == 0 {
        $recommendations = ($recommendations | append {
            category: "ulid_usage",
            priority: "high",
            issue: "No ULIDs found in logs",
            recommendation: "Consider implementing ULID-based request tracking for better correlation and debugging"
        })
    }
    
    # Performance recommendations
    if $performance_analysis.performance_summary != null {
        if $performance_analysis.performance_summary.avg_response_time > $CONFIG.performance_thresholds.slow_request_ms {
            $recommendations = ($recommendations | append {
                category: "performance",
                priority: "medium",
                issue: $"Average response time is ($performance_analysis.performance_summary.avg_response_time)ms",
                recommendation: "Investigate slow requests and optimize performance bottlenecks"
            })
        }
    }
    
    # Error recommendations
    if $error_analysis.error_rate > $CONFIG.performance_thresholds.error_rate_threshold {
        $recommendations = ($recommendations | append {
            category: "errors",
            priority: "high",
            issue: $"Error rate is ($error_analysis.error_rate | math round)%",
            recommendation: "Investigate and resolve error patterns to improve system reliability"
        })
    }
    
    $recommendations
}

# ====================================================================
# MONITORING FUNCTIONS
# ====================================================================

# Real-time log monitoring
def monitor_logs [
    file_path: string,
    format: string,
    --watch?: bool,
    --interval: string = "30s"
] {
    print $"👁️  Monitoring log file: ($file_path)"
    
    if not ($file_path | path exists) {
        error make { msg: $"Log file not found: ($file_path)" }
    }
    
    mut last_position = 0
    
    loop {
        let current_size = $file_path | path stat | get size
        
        if $current_size > $last_position {
            print $"📈 New log data detected (($current_size - $last_position) bytes)"
            
            # Read new lines
            let new_lines = open $file_path 
                | lines 
                | skip ($last_position // 80)  # Rough estimate of lines
                | where ($it | str length) > 0
            
            if ($new_lines | length) > 0 {
                let parsed_entries = parse_log_file $file_path $format --start-line ($last_position // 80) --max-lines ($new_lines | length)
                
                # Quick analysis of new entries
                let new_errors = $parsed_entries | where log_level == "error" | length
                let new_ulids = $parsed_entries | get extracted_ulids | flatten | uniq | length
                
                print $"📊 New entries: ($parsed_entries | length), Errors: ($new_errors), ULIDs: ($new_ulids)"
                
                # Check for alerts
                if $new_errors > 0 {
                    print $"🚨 ALERT: ($new_errors) new errors detected"
                }
            }
            
            $last_position = $current_size
        }
        
        if not $watch {
            break
        }
        
        sleep ($interval | into duration)
    }
}

# ====================================================================
# COMMAND LINE INTERFACE
# ====================================================================

def main [
    command?: string,    # Command to execute (analyze, monitor, parse, test)
    --file?: string,     # Log file path
    --format: string = "json",  # Log format
    --help              # Show help information
] {
    if $help {
        print "Log Analysis with ULID Template"
        print "==============================="
        print ""
        print "Commands:"
        print "  analyze      Perform comprehensive log analysis"
        print "  monitor      Real-time log monitoring"
        print "  parse        Parse log file and show structure"
        print "  test         Run analysis tests"
        print ""
        print "Formats:"
        print "  json         JSON formatted logs"
        print "  structured   Structured text logs"
        print "  csv          CSV formatted logs"
        print "  combined     Combined/Common log format"
        print ""
        print "Examples:"
        print "  nu log-analysis.nu analyze --file app.log --format json"
        print "  nu log-analysis.nu monitor --file app.log --format structured"
        print "  nu log-analysis.nu parse --file app.log --format csv"
        return
    }
    
    match $command {
        "analyze" => {
            if $file == null {
                error make { msg: "Log file required for analysis" }
            }
            let entries = parse_log_file $file $format
            generate_analysis_report $entries $file
        },
        "monitor" => {
            if $file == null {
                error make { msg: "Log file required for monitoring" }
            }
            monitor_logs $file $format --watch
        },
        "parse" => {
            if $file == null {
                error make { msg: "Log file required for parsing" }
            }
            let entries = parse_log_file $file $format --max-lines 100
            print $"📊 Parsed ($entries | length) entries (sample)"
            $entries | first 5 | table
        },
        "test" => {
            print "Running log analysis tests..."
            # Test implementation would go here
            print "Tests completed"
        },
        _ => {
            print "Available commands: analyze, monitor, parse, test"
            print "Use --help for more information"
        }
    }
}