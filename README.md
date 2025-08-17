# nu_plugin_ulid

[![CI](https://github.com/nushell-works/nu_plugin_ulid/workflows/CI/badge.svg)](https://github.com/nushell-works/nu_plugin_ulid/actions)
[![Security](https://github.com/nushell-works/nu_plugin_ulid/workflows/Security/badge.svg)](https://github.com/nushell-works/nu_plugin_ulid/actions)
[![License](https://img.shields.io/badge/license-BSD--3--Clause-blue.svg)](LICENSE)

Professional ULID (Universally Unique Lexicographically Sortable Identifier) utilities plugin for Nushell. Generate, parse, validate & manipulate ULIDs with cryptographically secure operations, bulk processing, and seamless integration with Nushell's structured data model.

## Features

- **🔒 Cryptographically Secure**: Uses secure randomness for ULID generation
- **⚡ High Performance**: Optimized for bulk operations and streaming data
- **🔧 Complete ULID Support**: Generate, parse, validate, and manipulate ULIDs
- **🏢 Enterprise Grade**: Comprehensive testing, security scanning, and quality assurance
- **🐚 Nushell Native**: Full integration with Nushell's structured data and pipeline model

## Installation

### Via Cargo

```bash
cargo install nu_plugin_ulid
```

### From Source

```bash
git clone https://github.com/nushell-works/nu_plugin_ulid.git
cd nu_plugin_ulid
cargo install --path .
```

### Register Plugin

After installation, register the plugin with Nushell:

```bash
plugin add ~/.cargo/bin/nu_plugin_ulid
plugin use ulid
```

## Quick Start

```nushell
# Check plugin information
ulid info

# Generate UUIDs (demo functionality)
ulid uuid generate

# Work with timestamps
ulid time now
ulid time now --format millis

# Encode/decode operations
"hello world" | ulid encode base32
"CSQPYRK1E8QDC4AKF31QH2E6" | ulid decode base32 --text

# Cryptographic operations
"hello world" | ulid hash sha256
ulid hash random --length 16
```

## Current Commands (Phase 1.2 Demo)

### Plugin Information
- `ulid info` - Display plugin metadata and diagnostics

### UUID Operations (Foundation)
- `ulid uuid generate` - Generate UUID v4
- `ulid uuid validate <uuid>` - Validate UUID format
- `ulid uuid parse <uuid>` - Parse UUID into components

### Time Operations (ULID Timestamp Component)
- `ulid time now [--format]` - Current timestamp in various formats
- `ulid time parse <timestamp>` - Parse timestamps into components
- `ulid time millis [timestamp]` - Convert to milliseconds (ULID format)

### Encoding Operations (ULID Base32 Foundation)
- `ulid encode base32 <data>` - Encode using Crockford Base32
- `ulid decode base32 <data>` - Decode Crockford Base32
- `ulid encode hex <data>` - Hexadecimal encoding
- `ulid decode hex <data>` - Hexadecimal decoding

### Cryptographic Operations (Security Patterns)
- `ulid hash sha256 <data>` - SHA-256 hashing
- `ulid hash sha512 <data>` - SHA-512 hashing
- `ulid hash blake3 <data>` - BLAKE3 hashing with variable length
- `ulid hash random [--length]` - Cryptographically secure random bytes

### Example Usage

```nushell
# Plugin information
> ulid info
╭─────────────┬─────────────────────────────────────────────────╮
│ name        │ nu_plugin_ulid                                  │
│ version     │ 0.1.0                                           │
│ description │ Professional ULID utilities plugin for Nushell  │
│ authors     │ John Ky <newhoggy@gmail.com>                    │
│ license     │ BSD-3-Clause                                    │
│ repository  │ https://github.com/nushell-works/nu_plugin_ulid │
╰─────────────┴─────────────────────────────────────────────────╯

# Time operations
> ulid time now --format millis
1702742400000

# Encoding operations
> "hello world" | ulid encode base32
CSQPYRK1E8QDC4AKF31QH2E6V4

> "CSQPYRK1E8QDC4AKF31QH2E6V4" | ulid decode base32 --text
hello world

# Cryptographic operations
> "hello world" | ulid hash sha256
b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
```

## Coming Soon (Phase 3)

The current demo commands establish patterns for the production ULID implementation:

- `ulid generate` - Generate ULIDs with various options
- `ulid parse` - Parse ULIDs into timestamp and randomness components
- `ulid validate` - Validate ULID format and integrity
- `ulid sort` - Sort data by ULID lexicographic order

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
|----------|------------|
  Timestamp    Randomness
   (48-bit)     (80-bit)
```

## Development

### Prerequisites

- Rust 1.81.0 or later
- Nushell 0.95.0 or later

### Building

```bash
git clone https://github.com/nushell-works/nu_plugin_ulid.git
cd nu_plugin_ulid
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

## Support

- 📚 [Documentation](https://docs.rs/nu_plugin_ulid)
- 🐛 [Issue Tracker](https://github.com/nushell-works/nu_plugin_ulid/issues)
- 💬 [Discussions](https://github.com/nushell-works/nu_plugin_ulid/discussions)

---

**Built with ❤️ for the Nushell community**
