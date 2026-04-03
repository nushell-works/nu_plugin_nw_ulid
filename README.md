# nu_plugin_nw_ulid

[![CI](https://github.com/nushell-works/nu_plugin_nw_ulid/workflows/CI/badge.svg)](https://github.com/nushell-works/nu_plugin_nw_ulid/actions)
[![Security](https://github.com/nushell-works/nu_plugin_nw_ulid/workflows/Security/badge.svg)](https://github.com/nushell-works/nu_plugin_nw_ulid/actions)
[![License](https://img.shields.io/badge/license-BSD--3--Clause-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/nu_plugin_nw_ulid.svg)](https://crates.io/crates/nu_plugin_nw_ulid)

Production-grade ULID (Universally Unique Lexicographically Sortable Identifier) utilities plugin for Nushell. Generate, parse, validate & manipulate ULIDs with cryptographically secure operations, enterprise-grade security, streaming support, and seamless integration with Nushell's structured data model.

## Features

- **🔒 Cryptographically Secure**: Uses secure randomness for ULID generation with comprehensive security validation
- **⚡ High Performance**: Optimized for bulk operations and streaming data with benchmarked performance
- **🔧 Complete ULID Support**: 23 production commands for generating, parsing, validating, and manipulating ULIDs
- **🏢 Enterprise Grade**: Security audit (A- rating), 90% test coverage, comprehensive testing, and quality assurance
- **🌊 Streaming Support**: Memory-efficient processing of large datasets with configurable batch sizes
- **🐚 Nushell Native**: Full integration with Nushell's structured data and pipeline model
- **🎯 Security First**: Built-in security advice and warnings for sensitive operations
- **🚀 Production Ready**: Zero clippy warnings, memory safety, and competitive performance vs reference implementations

## Installation

### Via Cargo

```bash
cargo install nu_plugin_nw_ulid
```

### From Source

```bash
git clone https://github.com/nushell-works/nu_plugin_nw_ulid.git
cd nu_plugin_nw_ulid
cargo install --path .
```

### Register Plugin

After installation, register the plugin with Nushell:

```bash
plugin add ~/.cargo/bin/nu_plugin_nw_ulid
plugin use nw_ulid
```

## Quick Start

```nushell
# Check plugin information
ulid info

# Generate ULIDs
ulid generate                              # Generate single ULID
ulid generate --count 5                    # Generate multiple ULIDs
ulid generate --timestamp 1692000000000    # Generate with custom timestamp

# Validate and parse ULIDs
ulid validate "01AN4Z07BY79KA1307SR9X4MV3"  # Validate ULID format
ulid parse "01AN4Z07BY79KA1307SR9X4MV3"     # Parse into components

# Sort and inspect ULIDs
echo ["01BN4Z07BY79KA1307SR9X4MV3", "01AN4Z07BY79KA1307SR9X4MV3"] | ulid sort
ulid inspect "01AN4Z07BY79KA1307SR9X4MV3"   # Detailed ULID analysis

# Stream processing for large datasets
echo ["01AN4Z07BY79KA1307SR9X4MV3", "invalid"] | ulid stream validate
ulid generate-stream --count 1000 --batch-size 100

# Security context checking
ulid security-advice                        # Get security recommendations
```

## Production Commands (23 Commands Available)

### Core ULID Operations
- `ulid generate [--count] [--timestamp] [--format]` - Generate ULIDs with options
- `ulid validate <ulid>` - Validate ULID format and integrity
- `ulid parse <ulid>` - Parse ULID into timestamp and randomness components
- `ulid security-advice` - Get security recommendations for ULID usage

### Analysis & Sorting
- `ulid sort [--reverse] [--natural]` - Sort data by ULID timestamp order
- `ulid inspect <ulid>` - Extract detailed metadata and statistics from ULIDs

### Streaming Operations (High Performance)
- `ulid stream <operation> [--batch-size] [--parallel] [--continue-on-error]` - Stream-process large datasets
- `ulid generate-stream [--count] [--batch-size] [--timestamp]` - Generate continuous ULID streams

### Time Operations
- `ulid time now [--format]` - Current timestamp in various formats
- `ulid time parse <timestamp>` - Parse timestamps into components
- `ulid time millis [timestamp]` - Convert to milliseconds (ULID format)

### Encoding Operations
- `ulid encode base32 <data>` - Encode using Crockford Base32 (ULID standard)
- `ulid decode base32 <data> [--text]` - Decode Crockford Base32
- `ulid encode hex <data> [--uppercase]` - Hexadecimal encoding
- `ulid decode hex <data> [--text]` - Hexadecimal decoding

### Cryptographic Operations
- `ulid hash sha256 <data>` - SHA-256 hashing
- `ulid hash sha512 <data>` - SHA-512 hashing
- `ulid hash blake3 <data> [--length]` - BLAKE3 hashing with variable length
- `ulid hash random [--length]` - Cryptographically secure random bytes

### Legacy UUID Support
- `ulid uuid generate` - Generate UUID v4 (compatibility)
- `ulid uuid validate <uuid>` - Validate UUID format
- `ulid uuid parse <uuid>` - Parse UUID into components

### Plugin Information
- `ulid info` - Display plugin metadata and diagnostics

### Example Usage

```nushell
# Plugin information
> ulid info
╭─────────────┬────────────────────────────────────────────────────────────────╮
│ name        │ nu_plugin_nw_ulid                                                 │
│ version     │ 0.1.0                                                          │
│ description │ Professional ULID (Universally Unique Lexicographically        │
│             │ Sortable Identifier) utilities plugin for Nushell              │
│ authors     │ John Ky <newhoggy@gmail.com>                                   │
│ license     │ BSD-3-Clause                                                   │
│ repository  │ https://github.com/nushell-works/nu_plugin_nw_ulid                │
╰─────────────┴────────────────────────────────────────────────────────────────╯

# Generate and work with ULIDs
> ulid generate
01K2W41TWG3FKYYSK430SR8KW6

> ulid validate "01K2W41TWG3FKYYSK430SR8KW6"
true

> ulid parse "01K2W41TWG3FKYYSK430SR8KW6"
╭────────────┬────────────────────────────╮
│ ulid       │ 01K2W41TWG3FKYYSK430SR8KW6 │
│ timestamp  │ {record 3 fields}          │
│ randomness │ {record 1 field}           │
│ valid      │ true                       │
╰────────────┴────────────────────────────╯

# Bulk operations
> ulid generate --count 3
╭───┬────────────────────────────╮
│ 0 │ 01K2W41TWG3FKYYSK430SR8KW7 │
│ 1 │ 01K2W41TWG3FKYYSK430SR8KW8 │
│ 2 │ 01K2W41TWG3FKYYSK430SR8KW9 │
╰───┴────────────────────────────╯

# Stream processing for large datasets
> echo ["01K2W41TWG3FKYYSK430SR8KW6", "invalid"] | ulid stream validate
╭───┬─────────────────────────────┬───────╮
│ # │ ulid                        │ valid │
├───┼─────────────────────────────┼───────┤
│ 0 │ 01K2W41TWG3FKYYSK430SR8KW6  │ true  │
│ 1 │ invalid                     │ false │
╰───┴─────────────────────────────┴───────╯
```

## What are ULIDs?

ULIDs (Universally Unique Lexicographically Sortable Identifiers) are 128-bit identifiers that are:

- **Sortable**: Lexicographically sortable by generation time
- **Compact**: 26 character string representation using Crockford Base32
- **URL Safe**: No special characters
- **Case Insensitive**: All uppercase for consistency
- **Monotonic**: Within the same millisecond, values are monotonically increasing

### ULID Structure

```
01ARZ3NDEKTSV4RRFFQ69G5FAV
|----------|-------------|
  Timestamp    Randomness
   (48-bit)     (80-bit)
```

## Performance & Security

### Production Performance

Based on benchmarking against reference implementations:

- **ULID Generation**: ~40ns per operation (release mode)
- **ULID Validation**: ~12ns per operation
- **Bulk Operations**: Efficient batch processing with configurable sizes
- **Memory Usage**: Optimized allocation patterns with streaming support
- **Concurrent Operations**: Thread-safe with parallel processing support

### Security Features

- **A- Security Rating**: Comprehensive security audit completed
- **Cryptographic Randomness**: Uses secure system entropy for ULID generation
- **Security Advice**: Built-in security advice for ULID usage
- **Input Validation**: Comprehensive attack resistance testing
- **Memory Safety**: Rust's memory guarantees prevent buffer overflows
- **Zero Information Leakage**: Sanitized error messages

### Enterprise Ready

- **90% Test Coverage**: Comprehensive test suite with security and performance testing
- **Zero Clippy Warnings**: Production-quality code standards
- **Cross-Platform**: Supports Linux, macOS, Windows
- **Streaming Support**: Memory-efficient processing of large datasets
- **Quality Assurance**: Automated security scanning and dependency auditing

## Development

### Prerequisites

- Rust 1.89.0 or later (required for Nushell 0.109.1)
- Nushell 0.109.1 or later

### Building

```bash
git clone https://github.com/nushell-works/nu_plugin_nw_ulid.git
cd nu_plugin_nw_ulid
cargo build --release
```

### Testing

```bash
cargo test
cargo clippy
cargo fmt --check
```

### Security

```bash
cargo audit
cargo deny check
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Process

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run the full test suite
5. Submit a pull request

### Code Standards

- All code must pass `cargo clippy` with no warnings
- Code must be formatted with `cargo fmt`
- Tests required for all new functionality
- Security scans must pass (`cargo audit`, `cargo deny check`)

## Architecture

This plugin follows enterprise-grade development practices:

- **Modular Design**: Separate modules for each command type
- **Comprehensive Testing**: Unit, integration, and property-based tests
- **Security First**: Regular security audits and dependency scanning
- **Performance Optimized**: Benchmarked and optimized for common use cases
- **Cross-Platform**: Supports Linux, macOS, and Windows

## License

Licensed under the BSD 3-Clause License. See [LICENSE](LICENSE) for details.

## Acknowledgments

- [ULID Specification](https://github.com/ulid/spec) - The ULID specification
- [Nushell](https://nushell.sh) - The amazing shell this plugin extends
- Rust Community - For the incredible ecosystem and tooling

## Documentation & Support

### Documentation
- 📚 [API Documentation](docs/scripting/api.md) - Complete API reference
- 🚀 [Scripting Guide](docs/scripting/README.md) - Automation patterns and helpers
- 🔒 [Security Audit](docs/security/audit-report.md) - Comprehensive security assessment
- ⚡ [Performance Report](docs/quality/phase4-completion-report.md) - Benchmarks and optimization details
- 🏗️ [Architecture Guide](docs/architecture/plugin-design.md) - Technical design documentation

### Community & Support
- 🐛 [Issue Tracker](https://github.com/nushell-works/nu_plugin_nw_ulid/issues) - Bug reports and feature requests
- 💬 [Discussions](https://github.com/nushell-works/nu_plugin_nw_ulid/discussions) - Community discussions
- 🤝 [Contributing](CONTRIBUTING.md) - Contribution guidelines
- 📦 [Crates.io](https://crates.io/crates/nu_plugin_nw_ulid) - Package repository

---

**Built with ❤️ for the Nushell community**
