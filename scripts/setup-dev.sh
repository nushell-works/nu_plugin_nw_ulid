#!/bin/bash

# Development environment setup script
# Sets up the development environment for nu_plugin_ulid

set -e

echo "🚀 Setting up nu_plugin_ulid development environment..."

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -f "src/lib.rs" ]]; then
    echo "❌ Error: This script must be run from the project root directory"
    exit 1
fi

# Source cargo environment if it exists
if [[ -f "$HOME/.cargo/env" ]]; then
    source "$HOME/.cargo/env"
fi

# Check Rust installation
echo "🦀 Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust $(rustc --version)"
echo "✅ Cargo $(cargo --version)"

# Install required Rust components
echo "🔧 Installing Rust components..."
rustup component add clippy rustfmt

# Install development tools
echo "📦 Installing development tools..."
TOOLS=(
    cargo-audit
    cargo-deny
    cargo-watch
    cargo-expand
    cargo-edit
    cargo-outdated
    cargo-udeps
    cargo-llvm-cov
)

for tool in "${TOOLS[@]}"; do
    if ! cargo install --list | grep -q "^$tool "; then
        echo "Installing $tool..."
        cargo install "$tool" --locked
    else
        echo "✅ $tool already installed"
    fi
done

# Install/check Nushell
echo "🐚 Checking Nushell installation..."
if ! command -v nu &> /dev/null; then
    echo "📥 Installing Nushell..."
    cargo install nu --locked
    echo "✅ Nushell installed"
else
    echo "✅ Nushell $(nu --version)"
fi

# Set up pre-commit hooks
echo "🪝 Setting up pre-commit hooks..."
if command -v pre-commit &> /dev/null; then
    if [[ -f ".pre-commit-config.yaml" ]]; then
        pre-commit install
        echo "✅ Pre-commit hooks installed"
    else
        echo "⚠️  No .pre-commit-config.yaml found, skipping pre-commit setup"
    fi
else
    echo "⚠️  pre-commit not found, skipping hook setup"
fi

# Build the project
echo "🔨 Building project..."
cargo build
echo "✅ Project built successfully"

# Run tests
echo "🧪 Running tests..."
cargo test
echo "✅ All tests passed"

# Check code quality
echo "🔍 Running code quality checks..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Clippy checks passed"

cargo fmt --check
echo "✅ Code formatting is correct"

# Security audit
echo "🔐 Running security audit..."
if command -v cargo-audit &> /dev/null; then
    cargo audit
    echo "✅ Security audit passed"
fi

if command -v cargo-deny &> /dev/null; then
    cargo deny check
    echo "✅ Supply chain security checks passed"
fi

# Create useful directories
echo "📁 Creating development directories..."
mkdir -p logs tmp .vscode

# Set up .vscode settings for the project
if [[ ! -f ".vscode/settings.json" ]]; then
    echo "⚙️  Creating VS Code settings..."
    mkdir -p .vscode
    cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.check.extraArgs": ["--all-targets", "--all-features"],
    "rust-analyzer.cargo.allFeatures": true,
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.imports.granularity.group": "module",
    "rust-analyzer.completion.addCallParenthesis": false,
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
        "source.fixAll": "explicit"
    },
    "files.watcherExclude": {
        "**/target/**": true
    },
    "rust-analyzer.linkedProjects": ["./Cargo.toml"]
}
EOF
    echo "✅ VS Code settings created"
fi

# Set up launch configuration for debugging
if [[ ! -f ".vscode/launch.json" ]]; then
    echo "🐛 Creating debug configuration..."
    cat > .vscode/launch.json << 'EOF'
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug unit tests",
            "cargo": {
                "args": ["test", "--no-run", "--lib"],
                "filter": {
                    "name": "nu_plugin_ulid",
                    "kind": "lib"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug plugin binary",
            "cargo": {
                "args": ["build", "--bin=nu_plugin_ulid"],
                "filter": {
                    "name": "nu_plugin_ulid",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
EOF
    echo "✅ Debug configuration created"
fi

echo ""
echo "🎉 Development environment setup completed successfully!"
echo ""
echo "📋 Summary:"
echo "   ✅ Rust toolchain ready"
echo "   ✅ Development tools installed"
echo "   ✅ Nushell available"
echo "   ✅ Project builds and tests pass"
echo "   ✅ Code quality checks pass"
echo "   ✅ Security audits pass"
echo "   ✅ VS Code configuration ready"
echo ""
echo "🚀 You're ready to start developing!"
echo ""
echo "💡 Useful commands:"
echo "   scripts/check.sh        - Quick quality check"
echo "   scripts/check-all.sh    - Full comprehensive check"
echo "   scripts/test.sh         - Run all tests"
echo "   scripts/build.sh        - Build release version"
echo "   cargo watch -x check    - Watch for changes"
echo ""
echo "🔧 Plugin development:"
echo "   cargo build --release   - Build plugin"
echo "   ./scripts/install-plugin.sh - Install plugin to Nushell"