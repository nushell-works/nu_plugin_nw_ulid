#!/usr/bin/env nu
# API Request Tracking Template
# 
# This template demonstrates how to track API requests across microservices
# using ULIDs for correlation and tracing.
#
# Features:
# - Request correlation tracking
# - Service chain monitoring
# - Performance metrics collection
# - Error rate analysis
# - Distributed tracing support
#
# Usage:
#   nu api-request-tracking.nu --help
#   nu api-request-tracking.nu track-request --service auth --endpoint /login
#   nu api-request-tracking.nu analyze-traces --since "2023-08-01"

# ====================================================================
# CONFIGURATION
# ====================================================================

const CONFIG = {
    # Request tracking settings
    default_timeout: 30000,        # 30 seconds
    max_retries: 3,
    correlation_header: "X-Correlation-ID",
    trace_header: "X-Trace-ID",
    
    # Storage settings
    trace_log_path: "api_traces.jsonl",
    metrics_path: "api_metrics.json",
    error_log_path: "api_errors.jsonl",
    
    # Performance thresholds
    slow_request_threshold: 2000,   # 2 seconds
    error_rate_threshold: 5.0,     # 5%
    
    # Service configuration
    services: {
        auth: { base_url: "https://auth.service.com", timeout: 10000 },
        user: { base_url: "https://user.service.com", timeout: 15000 },
        order: { base_url: "https://order.service.com", timeout: 20000 },
        payment: { base_url: "https://payment.service.com", timeout: 30000 },
        notification: { base_url: "https://notification.service.com", timeout: 5000 }
    }
}

# ====================================================================
# HELPER FUNCTIONS
# ====================================================================

# Generate correlation ID for request chain
def generate_correlation_id [context: string = "api-request"] {
    ulid generate --context $context
}

# Generate trace ID for individual request
def generate_trace_id [correlation_id: string] {
    ulid generate --context "api-trace"
}

# Log request trace
def log_trace [trace: record] {
    let timestamp = date now | format date "%Y-%m-%d %H:%M:%S.%f"
    let log_entry = $trace | upsert logged_at $timestamp
    
    $log_entry | to json | save --append $CONFIG.trace_log_path
    
    if $trace.level == "error" {
        $log_entry | to json | save --append $CONFIG.error_log_path
    }
}

# Create trace record
def create_trace [
    correlation_id: string,
    trace_id: string,
    service: string,
    endpoint: string,
    method: string,
    status_code: int,
    duration_ms: int,
    request_size?: int,
    response_size?: int,
    error?: string
] {
    let timestamp = date now | into int
    let level = if $status_code >= 500 {
        "error"
    } else if $status_code >= 400 {
        "warn"
    } else {
        "info"
    }
    
    {
        correlation_id: $correlation_id,
        trace_id: $trace_id,
        timestamp: $timestamp,
        timestamp_iso: (date now | format date "%Y-%m-%dT%H:%M:%S.%fZ"),
        service: $service,
        endpoint: $endpoint,
        method: $method,
        status_code: $status_code,
        duration_ms: $duration_ms,
        request_size: ($request_size | default 0),
        response_size: ($response_size | default 0),
        level: $level,
        error: $error,
        slow_request: ($duration_ms > $CONFIG.slow_request_threshold),
        success: ($status_code < 400)
    }
}

# Make HTTP request with tracing
def make_traced_request [
    service: string,
    endpoint: string,
    method: string,
    correlation_id: string,
    body?: any,
    headers?: record
] {
    let trace_id = generate_trace_id $correlation_id
    let service_config = $CONFIG.services | get $service
    let url = $"($service_config.base_url)($endpoint)"
    
    # Prepare headers with tracing information
    let trace_headers = {
        ($CONFIG.correlation_header): $correlation_id,
        ($CONFIG.trace_header): $trace_id,
        "Content-Type": "application/json",
        "User-Agent": "nu-ulid-tracker/1.0"
    }
    
    let final_headers = if $headers != null {
        $trace_headers | merge $headers
    } else {
        $trace_headers
    }
    
    print $"🚀 ($method) ($url) [trace: ($trace_id)]"
    
    let start_time = date now | into int
    
    let result = try {
        let response = match $method {
            "GET" => (http get $url --headers $final_headers --max-time $service_config.timeout),
            "POST" => (http post $url $body --headers $final_headers --max-time $service_config.timeout),
            "PUT" => (http put $url $body --headers $final_headers --max-time $service_config.timeout),
            "DELETE" => (http delete $url --headers $final_headers --max-time $service_config.timeout),
            _ => (error make { msg: $"Unsupported HTTP method: ($method)" })
        }
        
        {
            success: true,
            status_code: 200,
            response: $response,
            error: null
        }
    } catch { |e|
        {
            success: false,
            status_code: 500,
            response: null,
            error: $e.msg
        }
    }
    
    let end_time = date now | into int
    let duration = $end_time - $start_time
    
    # Create trace record
    let trace = create_trace $correlation_id $trace_id $service $endpoint $method $result.status_code $duration null null $result.error
    
    # Log the trace
    log_trace $trace
    
    # Return result with trace information
    {
        correlation_id: $correlation_id,
        trace_id: $trace_id,
        service: $service,
        endpoint: $endpoint,
        method: $method,
        success: $result.success,
        status_code: $result.status_code,
        duration_ms: $duration,
        response: $result.response,
        error: $result.error,
        trace: $trace
    }
}

# ====================================================================
# MAIN FUNCTIONS
# ====================================================================

# Track a single API request
def track_request [
    --service: string,           # Service name (auth, user, order, payment, notification)
    --endpoint: string,          # API endpoint
    --method: string = "GET",    # HTTP method
    --correlation-id?: string,   # Existing correlation ID (optional)
    --body?: string,             # Request body (JSON string)
    --headers?: string           # Additional headers (JSON string)
] {
    # Validate service
    if $service not-in ($CONFIG.services | columns) {
        error make { msg: $"Unknown service: ($service). Available: ($CONFIG.services | columns | str join ', ')" }
    }
    
    # Generate correlation ID if not provided
    let corr_id = if $correlation_id != null {
        $correlation_id
    } else {
        generate_correlation_id "single-request"
    }
    
    # Parse optional parameters
    let request_body = if $body != null { $body | from json } else { null }
    let request_headers = if $headers != null { $headers | from json } else { null }
    
    # Make the request
    let result = make_traced_request $service $endpoint $method $corr_id $request_body $request_headers
    
    # Display result
    if $result.success {
        print $"✅ Request completed successfully"
        print $"   Status: ($result.status_code)"
        print $"   Duration: ($result.duration_ms)ms"
        print $"   Correlation ID: ($result.correlation_id)"
        print $"   Trace ID: ($result.trace_id)"
    } else {
        print $"❌ Request failed"
        print $"   Error: ($result.error)"
        print $"   Duration: ($result.duration_ms)ms"
        print $"   Correlation ID: ($result.correlation_id)"
        print $"   Trace ID: ($result.trace_id)"
    }
    
    $result
}

# Track a chain of API requests
def track_request_chain [requests: list] {
    let correlation_id = generate_correlation_id "request-chain"
    
    print $"🔗 Starting request chain with correlation ID: ($correlation_id)"
    
    let results = $requests | each { |request|
        make_traced_request $request.service $request.endpoint $request.method $correlation_id $request.body $request.headers
    }
    
    # Analyze chain results
    let total_duration = $results | get duration_ms | math sum
    let success_count = $results | where success | length
    let failure_count = $results | where (not success) | length
    let success_rate = $success_count / ($results | length) * 100
    
    print $"📊 Chain Summary:"
    print $"   Total requests: ($results | length)"
    print $"   Successful: ($success_count)"
    print $"   Failed: ($failure_count)"
    print $"   Success rate: ($success_rate | math round)%"
    print $"   Total duration: ($total_duration)ms"
    
    {
        correlation_id: $correlation_id,
        total_requests: ($results | length),
        successful_requests: $success_count,
        failed_requests: $failure_count,
        success_rate: $success_rate,
        total_duration_ms: $total_duration,
        requests: $results
    }
}

# Analyze request traces
def analyze_traces [
    --since?: string,            # Filter traces since date (ISO 8601)
    --service?: string,          # Filter by service
    --correlation-id?: string,   # Filter by correlation ID
    --errors-only?: bool,        # Show only error traces
    --slow-only?: bool          # Show only slow requests
] {
    # Load traces
    if not ($CONFIG.trace_log_path | path exists) {
        error make { msg: $"No trace log found at ($CONFIG.trace_log_path)" }
    }
    
    let all_traces = open $CONFIG.trace_log_path 
        | lines 
        | where ($it | str length) > 0
        | each { from json }
    
    # Apply filters
    let filtered_traces = $all_traces | where {
        let trace = $in
        
        # Date filter
        let date_match = if $since != null {
            ($trace.timestamp_iso | into datetime) >= ($since | into datetime)
        } else {
            true
        }
        
        # Service filter
        let service_match = if $service != null {
            $trace.service == $service
        } else {
            true
        }
        
        # Correlation ID filter
        let correlation_match = if $correlation_id != null {
            $trace.correlation_id == $correlation_id
        } else {
            true
        }
        
        # Error filter
        let error_match = if $errors_only {
            $trace.level == "error"
        } else {
            true
        }
        
        # Slow request filter
        let slow_match = if $slow_only {
            $trace.slow_request
        } else {
            true
        }
        
        $date_match and $service_match and $correlation_match and $error_match and $slow_match
    }
    
    if ($filtered_traces | length) == 0 {
        print "No traces found matching the specified criteria"
        return
    }
    
    # Generate analysis
    let total_traces = $filtered_traces | length
    let unique_correlations = $filtered_traces | get correlation_id | uniq | length
    let services = $filtered_traces | get service | uniq
    let error_count = $filtered_traces | where level == "error" | length
    let slow_count = $filtered_traces | where slow_request | length
    let avg_duration = $filtered_traces | get duration_ms | math avg | math round
    let error_rate = $error_count / $total_traces * 100
    
    # Service breakdown
    let service_stats = $filtered_traces 
        | group-by service 
        | transpose service traces
        | each { |group|
            let traces = $group.traces
            {
                service: $group.service,
                request_count: ($traces | length),
                avg_duration: ($traces | get duration_ms | math avg | math round),
                error_count: ($traces | where level == "error" | length),
                slow_count: ($traces | where slow_request | length),
                success_rate: (($traces | where success | length) / ($traces | length) * 100 | math round)
            }
        }
    
    # Hourly distribution
    let hourly_stats = $filtered_traces
        | group-by { |trace| $trace.timestamp_iso | str substring 0..13 }
        | transpose hour traces
        | each { |group|
            {
                hour: $group.hour,
                request_count: ($group.traces | length),
                avg_duration: ($group.traces | get duration_ms | math avg | math round),
                error_count: ($group.traces | where level == "error" | length)
            }
        }
        | sort-by hour
    
    # Display results
    print $"📈 Trace Analysis Results"
    print $"========================"
    print $"Period: ($filtered_traces | get timestamp_iso | math min) to ($filtered_traces | get timestamp_iso | math max)"
    print $""
    print $"📊 Summary:"
    print $"   Total traces: ($total_traces)"
    print $"   Unique request chains: ($unique_correlations)"
    print $"   Services involved: ($services | str join ', ')"
    print $"   Average duration: ($avg_duration)ms"
    print $"   Error rate: ($error_rate | math round)%"
    print $"   Slow requests: ($slow_count) (($slow_count / $total_traces * 100 | math round)%)"
    print $""
    print $"🏗️  Service Breakdown:"
    $service_stats | table
    print $""
    print $"⏰ Hourly Distribution:"
    $hourly_stats | table
    
    # Top errors
    if $error_count > 0 {
        print $""
        print $"❌ Top Errors:"
        let error_summary = $filtered_traces 
            | where level == "error"
            | group-by error
            | transpose error traces
            | each { |group|
                {
                    error: $group.error,
                    count: ($group.traces | length),
                    services: ($group.traces | get service | uniq | str join ', ')
                }
            }
            | sort-by count --reverse
            | first 10
        
        $error_summary | table
    }
    
    {
        summary: {
            total_traces: $total_traces,
            unique_correlations: $unique_correlations,
            services: $services,
            avg_duration_ms: $avg_duration,
            error_rate: $error_rate,
            slow_request_rate: ($slow_count / $total_traces * 100)
        },
        service_stats: $service_stats,
        hourly_stats: $hourly_stats,
        traces: $filtered_traces
    }
}

# Generate performance metrics
def generate_metrics [--save?: bool] {
    let traces = if ($CONFIG.trace_log_path | path exists) {
        open $CONFIG.trace_log_path 
            | lines 
            | where ($it | str length) > 0
            | each { from json }
    } else {
        []
    }
    
    if ($traces | length) == 0 {
        print "No traces available for metrics generation"
        return
    }
    
    let now = date now
    let last_hour = $traces | where {
        ($in.timestamp_iso | into datetime) > ($now - 1hr)
    }
    let last_day = $traces | where {
        ($in.timestamp_iso | into datetime) > ($now - 1day)
    }
    
    let metrics = {
        timestamp: $now,
        period: {
            last_hour: {
                total_requests: ($last_hour | length),
                avg_duration: ($last_hour | get duration_ms | math avg | default 0 | math round),
                error_rate: (if ($last_hour | length) > 0 { ($last_hour | where level == "error" | length) / ($last_hour | length) * 100 } else { 0 }),
                slow_requests: ($last_hour | where slow_request | length)
            },
            last_day: {
                total_requests: ($last_day | length),
                avg_duration: ($last_day | get duration_ms | math avg | default 0 | math round),
                error_rate: (if ($last_day | length) > 0 { ($last_day | where level == "error" | length) / ($last_day | length) * 100 } else { 0 }),
                slow_requests: ($last_day | where slow_request | length)
            }
        },
        services: ($traces | group-by service | transpose service traces | each { |group|
            let service_traces = $group.traces
            {
                service: $group.service,
                total_requests: ($service_traces | length),
                avg_duration: ($service_traces | get duration_ms | math avg | math round),
                error_rate: (($service_traces | where level == "error" | length) / ($service_traces | length) * 100),
                last_request: ($service_traces | get timestamp_iso | math max)
            }
        })
    }
    
    if $save {
        $metrics | to json | save $CONFIG.metrics_path
        print $"Metrics saved to ($CONFIG.metrics_path)"
    }
    
    print $"📊 Performance Metrics"
    print $"====================="
    print $"Generated: ($metrics.timestamp)"
    print $""
    print $"⏰ Last Hour:"
    print $"   Requests: ($metrics.period.last_hour.total_requests)"
    print $"   Avg Duration: ($metrics.period.last_hour.avg_duration)ms"
    print $"   Error Rate: ($metrics.period.last_hour.error_rate | math round)%"
    print $"   Slow Requests: ($metrics.period.last_hour.slow_requests)"
    print $""
    print $"📅 Last Day:"
    print $"   Requests: ($metrics.period.last_day.total_requests)"
    print $"   Avg Duration: ($metrics.period.last_day.avg_duration)ms"
    print $"   Error Rate: ($metrics.period.last_day.error_rate | math round)%"
    print $"   Slow Requests: ($metrics.period.last_day.slow_requests)"
    print $""
    print $"🏗️  Service Metrics:"
    $metrics.services | table
    
    $metrics
}

# ====================================================================
# TESTING FUNCTIONS
# ====================================================================

# Test the tracking system with mock requests
def test_system [] {
    print "🧪 Testing API Request Tracking System"
    print "======================================"
    
    # Test 1: Single request
    print "\n1️⃣  Testing single request tracking..."
    let single_result = track_request --service auth --endpoint "/health" --method GET
    assert ($single_result.correlation_id | ulid validate) "Correlation ID should be valid ULID"
    assert ($single_result.trace_id | ulid validate) "Trace ID should be valid ULID"
    print "   ✅ Single request test passed"
    
    # Test 2: Request chain
    print "\n2️⃣  Testing request chain tracking..."
    let chain_requests = [
        { service: "auth", endpoint: "/validate", method: "POST", body: null, headers: null },
        { service: "user", endpoint: "/profile", method: "GET", body: null, headers: null },
        { service: "order", endpoint: "/history", method: "GET", body: null, headers: null }
    ]
    
    let chain_result = track_request_chain $chain_requests
    assert ($chain_result.correlation_id | ulid validate) "Chain correlation ID should be valid ULID"
    assert ($chain_result.total_requests == 3) "Should have 3 requests in chain"
    print "   ✅ Request chain test passed"
    
    # Test 3: Analysis
    print "\n3️⃣  Testing trace analysis..."
    sleep 1sec  # Ensure traces are written
    let analysis = analyze_traces
    assert ($analysis.summary.total_traces >= 4) "Should have at least 4 traces from tests"
    print "   ✅ Trace analysis test passed"
    
    # Test 4: Metrics
    print "\n4️⃣  Testing metrics generation..."
    let metrics = generate_metrics
    assert ($metrics.period.last_hour.total_requests >= 4) "Should have requests in last hour"
    print "   ✅ Metrics generation test passed"
    
    print "\n🎉 All tests passed successfully!"
}

# ====================================================================
# COMMAND LINE INTERFACE
# ====================================================================

def main [
    command?: string,    # Command to execute (track-request, track-chain, analyze, metrics, test)
    --help              # Show help information
] {
    if $help {
        print "API Request Tracking Template"
        print "============================"
        print ""
        print "Commands:"
        print "  track-request     Track a single API request"
        print "  track-chain       Track a chain of API requests"
        print "  analyze           Analyze request traces"
        print "  metrics           Generate performance metrics"
        print "  test              Run system tests"
        print ""
        print "Examples:"
        print "  nu api-request-tracking.nu track-request --service auth --endpoint /login --method POST"
        print "  nu api-request-tracking.nu analyze --since '2023-08-01' --service auth"
        print "  nu api-request-tracking.nu metrics --save"
        print "  nu api-request-tracking.nu test"
        return
    }
    
    match $command {
        "track-request" => {
            print "Use: track_request --service <service> --endpoint <endpoint> [options]"
        },
        "track-chain" => {
            print "Use: track_request_chain [list of requests]"
        },
        "analyze" => {
            print "Use: analyze_traces [options]"
        },
        "metrics" => {
            print "Use: generate_metrics [--save]"
        },
        "test" => test_system,
        _ => {
            print "Available commands: track-request, track-chain, analyze, metrics, test"
            print "Use --help for more information"
        }
    }
}

# ====================================================================
# EXAMPLE USAGE
# ====================================================================

# Example 1: Track a login request
# let login_result = track_request --service auth --endpoint "/login" --method POST --body '{"username": "user1", "password": "pass123"}'

# Example 2: Track a user workflow
# let workflow = [
#     { service: "auth", endpoint: "/validate", method: "POST", body: null, headers: null },
#     { service: "user", endpoint: "/profile", method: "GET", body: null, headers: null },
#     { service: "order", endpoint: "/create", method: "POST", body: '{"item": "product1", "quantity": 2}', headers: null },
#     { service: "payment", endpoint: "/process", method: "POST", body: null, headers: null },
#     { service: "notification", endpoint: "/send", method: "POST", body: null, headers: null }
# ]
# let workflow_result = track_request_chain $workflow

# Example 3: Analyze recent traces
# let analysis = analyze_traces --since "2023-08-01T00:00:00Z" --errors-only

# Example 4: Generate and save metrics
# let metrics = generate_metrics --save