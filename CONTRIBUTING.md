# Contributing to nu_plugin_nw_ulid

We welcome contributions to nu_plugin_nw_ulid! This document provides guidelines and information for contributors.

**Package**: `nu_plugin_nw_ulid` | **Binary**: `nu_plugin_nw_ulid` | **Organization**: nushell-works

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)

## Code of Conduct

This project adheres to a code of conduct adapted from the [Contributor Covenant](https://www.contributor-covenant.org/). By participating, you are expected to uphold this code.

### Our Pledge

We pledge to make participation in our project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Standards

Examples of behavior that contributes to creating a positive environment include:
- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

## Getting Started

### Prerequisites

Ensure you have the required tools installed:

- **Rust 1.85.0+**: [Install Rust](https://rustup.rs/) (required for Rust edition 2024)
- **Nushell 0.106.1+**: [Install Nushell](https://nushell.sh/book/installation.html)
- **Git**: [Install Git](https://git-scm.com/downloads)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/nu_plugin_nw_ulid.git
   cd nu_plugin_nw_ulid
   ```

## Development Setup

### Environment Setup

1. **Install dependencies:**
   ```bash
   cargo build
   ```

2. **Install development tools:**
   ```bash
   rustup component add clippy rustfmt
   cargo install cargo-audit cargo-deny
   ```

3. **Verify setup:**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

### Plugin Registration

To test your changes:

```bash
# Build the plugin
cargo build --release

# Register with Nushell
plugin add ./target/release/nu_plugin_nw_ulid
plugin use nw_ulid

# Test the plugin
ulid info
```

## Making Changes

### Branch Naming

Use descriptive branch names:
- `feature/add-ulid-generation` - New features
- `fix/parse-error-handling` - Bug fixes
- `docs/update-readme` - Documentation changes
- `refactor/command-structure` - Code refactoring

### Commit Messages

Follow conventional commit format:

```
type(scope): description

Longer explanation if needed

Fixes #issue-number
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks

Example:
```
feat(commands): add ulid generation command

Add comprehensive ULID generation with timestamp and randomness options.
Includes validation and error handling for edge cases.

Fixes #123
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_plugin_version

# Run tests with coverage (requires cargo-llvm-cov)
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

### Test Requirements

- **Comprehensive Coverage**: All new features must have comprehensive tests (aim for >90% coverage)
- **Edge Cases**: Tests should cover both success and error cases, boundary conditions, and edge cases
- **Integration Tests**: Plugin commands must have integration tests with real Nushell execution
- **Property-Based Tests**: ULID operations should include property-based tests for mathematical properties
- **Security Tests**: Security-critical code requires dedicated security tests
- **Performance Tests**: Performance-sensitive code should include benchmark tests
- **Documentation Tests**: All examples in documentation must be tested

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_output);
    }
}
```

## Code Style

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for consistent formatting
- Pass all `cargo clippy` checks with no warnings
- Use meaningful variable and function names
- Add documentation comments for public APIs

### Documentation

- Use `///` for public API documentation
- Include examples in doc comments
- Update README.md for user-facing changes
- Add inline comments for complex logic

Example:
```rust
/// Generates a new ULID with the current timestamp.
///
/// # Examples
///
/// ```
/// let ulid = generate_ulid();
/// assert_eq!(ulid.len(), 26);
/// ```
///
/// # Errors
///
/// Returns an error if the system clock is unavailable.
pub fn generate_ulid() -> Result<String, UlidError> {
    // Implementation
}
```

### Error Handling

- Use `Result<T, E>` for fallible operations
- Create specific error types for different failure modes
- Provide helpful error messages for users
- Handle all error cases appropriately

## Submitting Changes

### Before Submitting

Run the complete validation suite:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Security audit
cargo audit
cargo deny check

# Build release
cargo build --release
```

### Pull Request Process

1. **Create Pull Request:**
   - Use a descriptive title
   - Reference related issues
   - Provide clear description of changes
   - Include testing instructions

2. **PR Template:**
   ```markdown
   ## Description
   Brief description of changes

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update

   ## Testing
   - [ ] Tests added/updated
   - [ ] All tests passing
   - [ ] Manual testing completed

   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Self-review completed
   - [ ] Documentation updated
   - [ ] No new warnings
   ```

3. **Review Process:**
   - Maintainer review required
   - CI checks must pass
   - Address feedback promptly
   - Keep PR updated with main branch

### CI Requirements

All PRs must pass:
- Multi-platform builds (Linux, macOS, Windows)
- Rust toolchain compatibility (stable, beta, MSRV)
- Security scans (audit, CodeQL)
- Code quality checks (clippy, fmt)
- Test suite execution

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible new features
- **PATCH**: Backwards-compatible bug fixes

### Release Workflow

1. **Prepare Release:**
   - Update version in `Cargo.toml`
   - Update `CHANGELOG.md`
   - Update documentation
   - Tag release: `git tag v1.0.0`

2. **Automated Release:**
   - GitHub Actions handles:
     - Multi-platform binary builds
     - Crates.io publication
     - GitHub release creation
     - Documentation deployment

## Development Guidelines

### Architecture Principles

- **Modularity**: Separate concerns into focused modules
- **Testability**: Design for easy testing and mocking
- **Performance**: Optimize for common use cases
- **Security**: Security-first design and regular audits
- **Compatibility**: Maintain Nushell API compatibility

### Adding New Commands

1. **Create command module** in `src/commands/`
2. **Implement `PluginCommand` trait**
3. **Add comprehensive tests**
4. **Update plugin registration**
5. **Document usage and examples**

### Security Considerations

- Validate all user inputs
- Use secure randomness sources
- Avoid exposing sensitive information
- Regular dependency audits
- Follow Rust security best practices

## Phase 5 Contribution Priorities

### Current Focus Areas

As we complete Phase 5 (Documentation & Release), we welcome contributions in these priority areas:

1. **Documentation Improvements**
   - User guide enhancements and examples
   - Tutorial content for common use cases
   - Video tutorials and demos
   - Translation to other languages

2. **Community Tools**
   - Script templates for new use cases
   - Integration examples with popular tools
   - Performance optimization guides
   - Best practices documentation

3. **Quality Assurance**
   - Bug reports and fixes
   - Performance optimization
   - Security improvements
   - Compatibility testing

4. **Ecosystem Integration**
   - Nu scripts for common workflows
   - Integration with popular databases
   - Cloud platform deployment guides
   - CI/CD pipeline examples

### Special Recognition

Contributors helping with Phase 5 completion will receive special recognition in the v1.0 release notes.

## Getting Help

### Communication Channels

- **Issues**: Bug reports and feature requests - [Create Issue](https://github.com/nushell-works/nu_plugin_nw_ulid/issues/new/choose)
- **Discussions**: General questions and ideas - [Join Discussion](https://github.com/nushell-works/nu_plugin_nw_ulid/discussions)
- **Email**: Maintainer contact for sensitive issues

### Resources

- [Nushell Plugin Development](https://nushell.sh/book/plugins.html)
- [ULID Specification](https://github.com/ulid/spec)
- [Rust Documentation](https://doc.rust-lang.org/)
- [nu_plugin_nw_ulid User Guide](docs/USER_GUIDE.md)
- [Developer Guide](docs/DEVELOPER_GUIDE.md)
- [Performance Guide](docs/PERFORMANCE_GUIDE.md)

## Recognition

Contributors will be recognized in:
- Repository contributors list
- Release notes for significant contributions
- Special acknowledgments for major features

Thank you for contributing to nu_plugin_nw_ulid! 🎉
