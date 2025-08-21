#!/bin/bash

# Build script for nu_plugin_nw_ulid
# Builds optimized release version with comprehensive checks

set -e

echo "🔨 Building nu_plugin_nw_ulid..."

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
MODE="release"
VERBOSE=false
CHECK_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            MODE="debug"
            shift
            ;;
        --release)
            MODE="release"
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --check)
            CHECK_ONLY=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --debug       Build debug version (default: release)"
            echo "  --release     Build release version"
            echo "  --verbose     Show detailed build output"
            echo "  --check       Only check compilation, don't build artifacts"
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

echo "📋 Build Configuration:"
echo "   Mode: $MODE"
echo "   Verbose: $VERBOSE"
echo "   Check only: $CHECK_ONLY"
echo ""

# Build command arguments
BUILD_ARGS=()

if [[ "$CHECK_ONLY" == "true" ]]; then
    BUILD_ARGS+=("check")
else
    BUILD_ARGS+=("build")
fi

BUILD_ARGS+=("--all-features")

if [[ "$MODE" == "release" ]]; then
    BUILD_ARGS+=("--release")
fi

if [[ "$VERBOSE" == "true" ]]; then
    BUILD_ARGS+=("--verbose")
fi

# Pre-build checks
echo "🔍 Running pre-build checks..."

# Check code formatting
echo "  📝 Checking code formatting..."
if ! cargo fmt --check; then
    echo "❌ Code formatting issues found. Run: cargo fmt"
    exit 1
fi

# Quick clippy check
echo "  🔍 Running quick linting..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Linting issues found. Fix clippy warnings."
    exit 1
fi

echo "✅ Pre-build checks passed"
echo ""

# Build the project
echo "🚀 Building project..."
echo "Command: cargo ${BUILD_ARGS[*]}"
echo ""

if cargo "${BUILD_ARGS[@]}"; then
    echo ""
    echo "✅ Build completed successfully!"
else
    echo ""
    echo "❌ Build failed"
    exit 1
fi

# Post-build information
if [[ "$CHECK_ONLY" == "false" ]]; then
    echo ""
    echo "📦 Build Information:"
    
    if [[ "$MODE" == "release" ]]; then
        BINARY_PATH="target/release/nu_plugin_nw_ulid"
        if [[ -f "$BINARY_PATH" ]]; then
            SIZE=$(ls -lh "$BINARY_PATH" | awk '{print $5}')
            echo "   Binary: $BINARY_PATH"
            echo "   Size: $SIZE"
            
            # Show binary info if available
            if command -v file &> /dev/null; then
                FILE_INFO=$(file "$BINARY_PATH")
                echo "   Type: $FILE_INFO"
            fi
            
            # Show stripped status
            if command -v strip &> /dev/null; then
                if strip --help 2>&1 | grep -q "\-\-only\-keep\-debug"; then
                    echo "   Stripped: $(strip --only-keep-debug --dry-run "$BINARY_PATH" 2>&1 | grep -q 'File format not recognized' && echo 'Yes' || echo 'No')"
                fi
            fi
        fi
    else
        BINARY_PATH="target/debug/nu_plugin_nw_ulid"
        if [[ -f "$BINARY_PATH" ]]; then
            SIZE=$(ls -lh "$BINARY_PATH" | awk '{print $5}')
            echo "   Binary: $BINARY_PATH"
            echo "   Size: $SIZE"
        fi
    fi
    
    # Show compilation time if available
    if [[ -f "target/.rustc_info.json" ]]; then
        echo "   Target: $(jq -r '.host' target/.rustc_info.json 2>/dev/null || echo 'Unknown')"
    fi
fi

echo ""
echo "🎉 Build process completed!"

if [[ "$CHECK_ONLY" == "false" && "$MODE" == "release" ]]; then
    echo ""
    echo "💡 Next steps:"
    echo "   - Test the plugin: scripts/test.sh"
    echo "   - Install plugin: scripts/install-plugin.sh"
    echo "   - Run security audit: cargo audit"
    echo "   - Check dependencies: cargo deny check"
fi