#!/bin/bash

# Clean script for nu_plugin_nw_ulid
# Removes build artifacts and temporary files

set -e

echo "🧹 Cleaning nu_plugin_nw_ulid project..."

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
DEEP_CLEAN=false
REMOVE_DEPS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --deep)
            DEEP_CLEAN=true
            shift
            ;;
        --deps)
            REMOVE_DEPS=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --deep        Deep clean including registry cache"
            echo "  --deps        Remove downloaded dependencies"
            echo "  -h, --help    Show this help message"
            echo ""
            echo "This script removes:"
            echo "  - Build artifacts (target/)"
            echo "  - Temporary files"
            echo "  - Coverage reports"
            echo "  - VS Code workspace cache"
            echo "  - Log files"
            echo ""
            echo "With --deep:"
            echo "  - Cargo registry cache"
            echo "  - Git cache"
            echo ""
            echo "With --deps:"
            echo "  - Downloaded dependencies"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo "📋 Clean Configuration:"
echo "   Deep clean: $DEEP_CLEAN"
echo "   Remove deps: $REMOVE_DEPS"
echo ""

# Calculate sizes before cleaning
if command -v du &> /dev/null; then
    if [[ -d "target" ]]; then
        TARGET_SIZE=$(du -sh target 2>/dev/null | cut -f1)
        echo "📊 Current target/ size: $TARGET_SIZE"
    fi
fi

# Standard cargo clean
echo "🗑️  Running cargo clean..."
cargo clean
echo "✅ Cargo artifacts removed"

# Remove temporary and cache files
echo "🧽 Removing temporary files..."

# Coverage reports
if [[ -f "lcov.info" ]]; then
    rm lcov.info
    echo "   ✅ Coverage report (lcov.info) removed"
fi

if [[ -d "coverage" ]]; then
    rm -rf coverage
    echo "   ✅ HTML coverage reports removed"
fi

# Log files
if [[ -d "logs" ]]; then
    rm -rf logs/*
    echo "   ✅ Log files cleared"
fi

# Temporary directories
if [[ -d "tmp" ]]; then
    rm -rf tmp/*
    echo "   ✅ Temporary files cleared"
fi

# VS Code workspace cache
if [[ -d ".vscode" ]]; then
    find .vscode -name "*.log" -delete 2>/dev/null || true
    echo "   ✅ VS Code cache cleared"
fi

# Rust analyzer cache
if [[ -d ".rust-analyzer" ]]; then
    rm -rf .rust-analyzer
    echo "   ✅ Rust analyzer cache removed"
fi

# MacOS specific files
if [[ "$(uname)" == "Darwin" ]]; then
    find . -name ".DS_Store" -delete 2>/dev/null || true
    echo "   ✅ macOS .DS_Store files removed"
fi

# Editor backup files
find . -name "*~" -delete 2>/dev/null || true
find . -name "*.swp" -delete 2>/dev/null || true
find . -name "*.swo" -delete 2>/dev/null || true
echo "   ✅ Editor backup files removed"

# Deep clean options
if [[ "$DEEP_CLEAN" == "true" ]]; then
    echo ""
    echo "🔥 Performing deep clean..."
    
    # Cargo registry cache
    if [[ -d "$HOME/.cargo/registry" ]]; then
        REGISTRY_SIZE=$(du -sh "$HOME/.cargo/registry" 2>/dev/null | cut -f1)
        echo "   🗂️  Cargo registry size: $REGISTRY_SIZE"
        echo "   ⚠️  Deep clean would remove shared cargo cache"
        echo "   Run manually if needed: rm -rf ~/.cargo/registry"
    fi
    
    # Git cache
    if [[ -d ".git" ]]; then
        git gc --aggressive --prune=now
        echo "   ✅ Git cache optimized"
    fi
    
    # Rust toolchain cache
    if command -v rustup &> /dev/null; then
        echo "   🧹 Cleaning rustup cache..."
        rustup self update-data
        echo "   ✅ Rustup cache updated"
    fi
fi

# Remove dependencies option
if [[ "$REMOVE_DEPS" == "true" ]]; then
    echo ""
    echo "📦 Removing dependencies..."
    
    # This is a more aggressive clean that removes downloaded dependencies
    # but keeps the registry index
    if [[ -d "$HOME/.cargo/registry/cache" ]]; then
        CACHE_SIZE=$(du -sh "$HOME/.cargo/registry/cache" 2>/dev/null | cut -f1)
        echo "   📊 Dependency cache size: $CACHE_SIZE"
        echo "   ⚠️  This will require re-downloading dependencies"
        read -p "   Continue? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$HOME/.cargo/registry/cache"
            echo "   ✅ Dependency cache removed"
        else
            echo "   ⏭️  Skipped dependency removal"
        fi
    fi
fi

# Final size check
echo ""
echo "📊 Clean Summary:"

if [[ -d "target" ]]; then
    NEW_TARGET_SIZE=$(du -sh target 2>/dev/null | cut -f1 || echo "0B")
    echo "   Target directory: $NEW_TARGET_SIZE (was $TARGET_SIZE)"
else
    echo "   Target directory: removed completely"
fi

# Check available space
if command -v df &> /dev/null; then
    AVAILABLE_SPACE=$(df -h . | tail -1 | awk '{print $4}')
    echo "   Available space: $AVAILABLE_SPACE"
fi

echo ""
echo "🎉 Project cleaned successfully!"
echo ""
echo "💡 Next steps:"
echo "   scripts/build.sh      - Rebuild the project"
echo "   scripts/test.sh       - Run tests"
echo "   scripts/setup-dev.sh  - Reset development environment"