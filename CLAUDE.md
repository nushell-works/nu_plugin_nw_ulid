# CLAUDE.md

## Nushell dependency versioning

When bumping nushell crate versions, these four things must stay in sync:

1. `nu-plugin` in Cargo.toml
2. `nu-protocol` in Cargo.toml
3. `nu-test-support` in Cargo.toml (dev-dependencies)
4. `cargo install nu --version X` in `.github/workflows/ci.yml`

Rust treats types from different nu-protocol versions as distinct, so a mismatch causes compilation errors. A mismatch with the CI nushell binary causes integration test failures.
