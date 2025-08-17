#!/bin/bash

# DevContainer setup script
# This script runs during container creation to set up the development environment

set -e

echo "🚀 Setting up nu_plugin_ulid development environment..."

# Update package lists
echo "📦 Updating package lists..."
sudo apt-get update

# Install additional development tools
echo "🔧 Installing development tools..."
sudo apt-get install -y \
    git \
    curl \
    wget \
    jq \
    htop \
    tree \
    fd-find \
    ripgrep \
    bat \
    exa \
    zoxide

# Install Nushell
echo "🐚 Installing Nushell..."
if ! command -v nu &> /dev/null; then
    echo "Installing Nushell via cargo..."
    cargo install nu --locked
    echo "✅ Nushell installed successfully"
else
    echo "✅ Nushell already installed"
fi

# Install additional Rust tools
echo "🦀 Installing Rust development tools..."
rustup component add clippy rustfmt
cargo install --locked \
    cargo-audit \
    cargo-deny \
    cargo-watch \
    cargo-expand \
    cargo-edit \
    cargo-outdated \
    cargo-udeps \
    cargo-llvm-cov

# Install pre-commit hooks framework
echo "🪝 Installing pre-commit..."
pip3 install pre-commit

# Configure git (if in codespace)
if [[ -n "${CODESPACES}" ]]; then
    echo "🌐 Configuring git for Codespaces..."
    git config --global user.name "${GITHUB_USER}"
    git config --global user.email "${GITHUB_USER}@users.noreply.github.com"
fi

# Create useful shell aliases
echo "🔗 Setting up shell aliases..."
cat >> ~/.bashrc << 'EOF'

# nu_plugin_ulid development aliases
alias ll='exa -la'
alias la='exa -la'
alias lt='exa --tree'
alias cat='bat'
alias find='fd'
alias grep='rg'

# Rust development shortcuts
alias cb='cargo build'
alias ct='cargo test'
alias cc='cargo check'
alias cf='cargo fmt'
alias cl='cargo clippy'
alias cw='cargo watch -x check -x test'
alias audit='cargo audit'
alias deny='cargo deny check'

# Plugin development
alias plugin-build='cargo build --release'
alias plugin-test='cargo test && cargo clippy'
alias plugin-install='cargo install --path .'
alias plugin-clean='cargo clean'

# Quick quality checks
alias quick-check='scripts/check.sh'
alias full-check='scripts/check-all.sh'
EOF

echo "✅ DevContainer setup completed successfully!"
echo ""
echo "🎉 Development environment is ready!"
echo "   - Rust $(rustc --version)"
echo "   - Cargo $(cargo --version)"
echo "   - Nushell will be available after setup"
echo "   - All development tools installed"
echo ""
echo "💡 Try running: scripts/setup-dev.sh"