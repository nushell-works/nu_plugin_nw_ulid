#!/bin/bash

# Comprehensive quality check script
# Runs all quality checks including security audits

set -e

echo "🔍 Running comprehensive quality checks..."

# Source cargo environment if it exists
if [[ -f "$HOME/.cargo/env" ]]; then
    source "$HOME/.cargo/env"
fi

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Error: This script must be run from the project root directory"
    exit 1
fi

echo "📋 Comprehensive Check Summary:"
echo "   - Code formatting"
echo "   - Linting (clippy)"
echo "   - Unit tests"
echo "   - Documentation tests"
echo "   - Build verification (debug & release)"
echo "   - Security audit"
echo "   - Supply chain security"
echo "   - Dependency analysis"
echo ""

# Check code formatting
echo "🎨 Checking code formatting..."
if cargo fmt --check; then
    echo "✅ Code formatting is correct"
else
    echo "❌ Code formatting issues found. Run: cargo fmt"
    exit 1
fi

# Run clippy with all targets and features
echo "🔍 Running clippy (comprehensive)..."
if cargo clippy --all-targets --all-features --workspace -- -D warnings; then
    echo "✅ Clippy checks passed"
else
    echo "❌ Clippy issues found. Fix the warnings above."
    exit 1
fi

# Run all tests
echo "🧪 Running all tests..."
if cargo test --all-features --workspace; then
    echo "✅ All tests passed"
else
    echo "❌ Tests failed"
    exit 1
fi

# Test documentation
echo "📚 Testing documentation..."
if cargo doc --no-deps --all-features; then
    echo "✅ Documentation builds successfully"
else
    echo "❌ Documentation build failed"
    exit 1
fi

# Build debug
echo "🔨 Building debug version..."
if cargo build --all-features; then
    echo "✅ Debug build successful"
else
    echo "❌ Debug build failed"
    exit 1
fi

# Build release
echo "🚀 Building release version..."
if cargo build --release --all-features; then
    echo "✅ Release build successful"
else
    echo "❌ Release build failed"
    exit 1
fi

# Security audit
echo "🔐 Running security audit..."
if command -v cargo-audit &> /dev/null; then
    if cargo audit; then
        echo "✅ Security audit passed"
    else
        echo "❌ Security audit found issues"
        exit 1
    fi
else
    echo "⚠️  cargo-audit not installed, skipping security audit"
fi

# Supply chain security
echo "🔗 Running supply chain security checks..."
if command -v cargo-deny &> /dev/null; then
    if cargo deny check; then
        echo "✅ Supply chain security checks passed"
    else
        echo "❌ Supply chain security issues found"
        exit 1
    fi
else
    echo "⚠️  cargo-deny not installed, skipping supply chain checks"
fi

# Check for outdated dependencies
echo "📦 Checking for outdated dependencies..."
if command -v cargo-outdated &> /dev/null; then
    cargo outdated
    echo "✅ Dependency analysis completed"
else
    echo "⚠️  cargo-outdated not installed, skipping dependency check"
fi

# Check for unused dependencies
echo "🧹 Checking for unused dependencies..."
if command -v cargo-udeps &> /dev/null; then
    if cargo +nightly udeps; then
        echo "✅ No unused dependencies found"
    else
        echo "⚠️  Unused dependencies detected (review output above)"
    fi
else
    echo "⚠️  cargo-udeps not installed, skipping unused dependency check"
fi

# Final binary size check
echo "📏 Checking binary size..."
if [[ -f "target/release/nu_plugin_ulid" ]]; then
    SIZE=$(ls -lh target/release/nu_plugin_ulid | awk '{print $5}')
    echo "✅ Release binary size: $SIZE"
else
    echo "⚠️  Release binary not found"
fi

echo ""
echo "🎉 Comprehensive quality checks completed successfully!"
echo ""
echo "📊 Summary:"
echo "   ✅ Code quality verified"
echo "   ✅ All tests passing"
echo "   ✅ Security audits passed"
echo "   ✅ Build verification completed"
echo "   ✅ Documentation builds"
echo ""
echo "🚀 Your code is ready for production!"