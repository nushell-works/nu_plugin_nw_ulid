#!/bin/bash

# Quick quality check script
# Runs essential quality checks for development

set -e

echo "🔍 Running quick quality checks..."

# Source cargo environment if it exists
if [[ -f "$HOME/.cargo/env" ]]; then
    source "$HOME/.cargo/env"
fi

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Error: This script must be run from the project root directory"
    exit 1
fi

echo "📋 Quick Check Summary:"
echo "   - Code formatting"
echo "   - Linting (clippy)"
echo "   - Unit tests"
echo "   - Build verification"
echo ""

# Check code formatting
echo "🎨 Checking code formatting..."
if cargo fmt --check; then
    echo "✅ Code formatting is correct"
else
    echo "❌ Code formatting issues found. Run: cargo fmt"
    exit 1
fi

# Run clippy
echo "🔍 Running clippy..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo "✅ Clippy checks passed"
else
    echo "❌ Clippy issues found. Fix the warnings above."
    exit 1
fi

# Run tests
echo "🧪 Running tests..."
if cargo test; then
    echo "✅ All tests passed"
else
    echo "❌ Tests failed"
    exit 1
fi

# Build check
echo "🔨 Verifying build..."
if cargo build; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

echo ""
echo "🎉 Quick quality checks completed successfully!"
echo ""
echo "💡 For comprehensive checks, run: scripts/check-all.sh"