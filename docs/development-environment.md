# Development Environment

This document describes the development environment setup and requirements for the `nu_plugin_nw_ulid` project.

## Verified Environment

**Platform**: macOS (Darwin 24.6.0, ARM64)  
**Date Verified**: 2025-08-17

## Required Tools

### Core Development Tools

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | 1.89.0 | Core language and toolchain |
| **Cargo** | 1.89.0 | Package manager and build tool |
| **Nushell** | 0.106.1 | Target shell environment |
| **Git** | 2.39.5+ | Version control |

### Development Quality Tools

| Tool | Version | Purpose |
|------|---------|---------|
| **clippy** | 0.1.89 | Rust linter and suggestions |
| **rustfmt** | 1.8.0 | Code formatting |

## Installation Instructions

### 1. Install Rust Toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

### 2. Install Nushell

```bash
cargo install nu
```

### 3. Verify Installation

```bash
rustc --version    # Should show: rustc 1.89.0+
cargo --version    # Should show: cargo 1.89.0+
nu --version       # Should show: 0.106.1+
clippy --version   # Should show: clippy 0.1.89+
rustfmt --version  # Should show: rustfmt 1.8.0+
```

## Git Configuration

For this project, commits should use:
```bash
git config user.email newhoggy@gmail.com
```

## Environment Validation

The development environment has been validated with:
- ✅ Rust toolchain installation and functionality
- ✅ Nushell installation and execution
- ✅ Development tools (clippy, rustfmt) availability
- ✅ Git configuration for project commits

## Next Steps

With the development environment properly configured, you can proceed to:
1. Project infrastructure setup (Phase 1.1)
2. Initial Cargo project creation
3. CI/CD pipeline configuration
4. Demo plugin implementation

## Troubleshooting

### Common Issues

**Rust not found after installation**:
```bash
source ~/.cargo/env
# Or add to shell profile: echo 'source ~/.cargo/env' >> ~/.zshrc
```

**Nushell installation fails**:
- Ensure Rust is properly installed first
- Alternative: `brew install nushell` (on macOS)

**Clippy/rustfmt not available**:
```bash
rustup component add clippy rustfmt
```

## Development Workflow

1. **Code Quality**: All code must pass `cargo clippy` and `cargo fmt`
2. **Testing**: Comprehensive test coverage required
3. **Documentation**: Code changes require documentation updates
4. **Security**: Regular `cargo audit` runs for dependency security