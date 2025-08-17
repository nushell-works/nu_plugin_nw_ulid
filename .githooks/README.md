# Git Hooks Configuration

This project uses git hooks to ensure code quality and consistency.

## Pre-commit Hook

The pre-commit hook runs automatically before each commit and performs the following checks:

1. **Code Formatting**: `cargo fmt --all --check`
   - Ensures all Rust code follows consistent formatting standards
   - Fails if code is not properly formatted
   - Run `cargo fmt --all` to auto-format code before committing

2. **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
   - Runs Clippy to catch common mistakes and improve code quality
   - Treats all warnings as errors to maintain high code quality
   - Fix any clippy warnings before committing

3. **Testing**: `cargo test --all`
   - Runs all unit tests to ensure functionality is not broken
   - Ensures all tests pass before allowing commit

## Setup

The hooks are automatically configured when you clone this repository. If you need to reinstall them:

```bash
# Make sure the pre-commit hook is executable
chmod +x .git/hooks/pre-commit
```

## Manual Execution

You can run the pre-commit checks manually at any time:

```bash
# Run the pre-commit hook manually
./.git/hooks/pre-commit

# Or run individual commands:
cargo fmt --all --check    # Check formatting
cargo clippy --all-targets --all-features -- -D warnings  # Run linting
cargo test --all          # Run tests
```

## Bypassing Hooks (Not Recommended)

In exceptional cases, you can bypass the pre-commit hook:

```bash
git commit --no-verify -m "your commit message"
```

However, this is strongly discouraged as it can introduce formatting issues and break the build.

## Benefits

- **Consistent Code Style**: Automatic formatting ensures all code follows the same style
- **Early Error Detection**: Clippy catches potential issues before they reach the main branch
- **Reliable Builds**: Tests ensure functionality remains intact
- **Improved Code Quality**: Prevents common mistakes from being committed
- **Team Productivity**: Reduces time spent in code review on formatting and style issues