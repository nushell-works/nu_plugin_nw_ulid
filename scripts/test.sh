#!/bin/bash

# Test execution script
# Runs comprehensive test suite with coverage reporting

set -e

echo "🧪 Running nu_plugin_ulid test suite..."

# Source cargo environment if it exists
if [[ -f "$HOME/.cargo/env" ]]; then
    source "$HOME/.cargo/env"
fi

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Error: This script must be run from the project root directory"
    exit 1
fi

# Parse command line arguments
COVERAGE=false
VERBOSE=false
FILTER=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --coverage)
            COVERAGE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --filter)
            FILTER="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --coverage    Generate code coverage report"
            echo "  --verbose     Show detailed test output"
            echo "  --filter STR  Run only tests matching filter"
            echo "  -h, --help    Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo "📋 Test Configuration:"
echo "   Coverage: $COVERAGE"
echo "   Verbose: $VERBOSE"
if [[ -n "$FILTER" ]]; then
    echo "   Filter: $FILTER"
fi
echo ""

# Build test arguments
TEST_ARGS=("test" "--all-features" "--workspace")

if [[ "$VERBOSE" == "true" ]]; then
    TEST_ARGS+=("--" "--nocapture")
fi

if [[ -n "$FILTER" ]]; then
    if [[ "$VERBOSE" == "true" ]]; then
        TEST_ARGS[-1]="--nocapture"
        TEST_ARGS+=("$FILTER")
    else
        TEST_ARGS+=("--" "$FILTER")
    fi
fi

# Run tests with or without coverage
if [[ "$COVERAGE" == "true" ]]; then
    echo "🔍 Running tests with coverage analysis..."
    
    if command -v cargo-llvm-cov &> /dev/null; then
        # Generate coverage report
        cargo llvm-cov clean
        cargo llvm-cov "${TEST_ARGS[@]}" --lcov --output-path lcov.info
        
        # Generate HTML report if possible
        if command -v genhtml &> /dev/null; then
            echo "📊 Generating HTML coverage report..."
            genhtml lcov.info --output-directory coverage --title "nu_plugin_ulid Coverage Report"
            echo "✅ HTML coverage report generated at: coverage/index.html"
        fi
        
        # Show coverage summary
        echo ""
        echo "📈 Coverage Summary:"
        cargo llvm-cov report --lcov --output-path - | grep -E "^(SF|LF|LH)" | \
        awk '
        /^SF:/ { file = substr($0, 4) }
        /^LF:/ { lines = substr($0, 4) }
        /^LH:/ { 
            hit = substr($0, 4)
            if (lines > 0) {
                pct = (hit / lines) * 100
                printf "   %s: %.1f%% (%d/%d lines)\n", file, pct, hit, lines
            }
        }'
        
        echo "✅ Tests completed with coverage analysis"
    else
        echo "⚠️  cargo-llvm-cov not installed, running tests without coverage"
        cargo "${TEST_ARGS[@]}"
        echo "✅ Tests completed"
    fi
else
    echo "🚀 Running tests..."
    cargo "${TEST_ARGS[@]}"
    echo "✅ Tests completed"
fi

# Run doc tests separately
echo ""
echo "📚 Running documentation tests..."
if cargo test --doc --all-features; then
    echo "✅ Documentation tests passed"
else
    echo "❌ Documentation tests failed"
    exit 1
fi

# Test examples if they exist
if [[ -d "examples" ]] && [[ -n "$(ls -A examples 2>/dev/null)" ]]; then
    echo ""
    echo "📝 Running example tests..."
    if cargo test --examples --all-features; then
        echo "✅ Example tests passed"
    else
        echo "❌ Example tests failed"
        exit 1
    fi
fi

# Benchmark tests if available
if grep -q "\[\[bench\]\]" Cargo.toml 2>/dev/null; then
    echo ""
    echo "⚡ Running benchmark tests..."
    if cargo test --benches --all-features; then
        echo "✅ Benchmark tests passed"
    else
        echo "❌ Benchmark tests failed"
        exit 1
    fi
fi

echo ""
echo "🎉 All tests completed successfully!"

if [[ "$COVERAGE" == "true" ]]; then
    echo ""
    echo "📊 Coverage files generated:"
    echo "   - lcov.info (for CI/CD)"
    if [[ -d "coverage" ]]; then
        echo "   - coverage/index.html (human-readable)"
    fi
fi

echo ""
echo "💡 Test commands:"
echo "   scripts/test.sh --coverage  - Run with coverage"
echo "   scripts/test.sh --verbose   - Show detailed output"
echo "   scripts/test.sh --filter X  - Run specific tests"