# Makefile for nu_plugin_ulid
# Provides convenient shortcuts for common development tasks

.PHONY: help setup build test check check-all clean install format lint audit coverage watch docs

# Default target
help: ## Show this help message
	@echo "nu_plugin_ulid Development Commands"
	@echo "=================================="
	@echo ""
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "Quick Start:"
	@echo "  make setup    - Set up development environment"
	@echo "  make check    - Run quick quality checks"
	@echo "  make test     - Run all tests"
	@echo "  make build    - Build release version"

# Development Environment
setup: ## Set up development environment
	@echo "🚀 Setting up development environment..."
	@./scripts/setup-dev.sh

# Building
build: ## Build release version
	@echo "🔨 Building release version..."
	@./scripts/build.sh --release

build-debug: ## Build debug version
	@echo "🔨 Building debug version..."
	@./scripts/build.sh --debug

build-check: ## Check compilation without building
	@echo "🔍 Checking compilation..."
	@./scripts/build.sh --check

# Testing
test: ## Run all tests
	@echo "🧪 Running all tests..."
	@./scripts/test.sh

test-coverage: ## Run tests with coverage
	@echo "🧪 Running tests with coverage..."
	@./scripts/test.sh --coverage

test-verbose: ## Run tests with verbose output
	@echo "🧪 Running tests (verbose)..."
	@./scripts/test.sh --verbose

test-integration: ## Run integration tests with Nushell
	@echo "🧪 Running integration tests..."
	@if command -v nu >/dev/null 2>&1; then \
		./scripts/test-integration.sh; \
	else \
		echo "❌ Nushell not found. Please install Nushell first."; \
		echo "Install with: cargo install nu --version 0.106.1"; \
		exit 1; \
	fi

# Quality Checks
check: ## Run quick quality checks
	@echo "🔍 Running quick quality checks..."
	@./scripts/check.sh

check-all: ## Run comprehensive quality checks
	@echo "🔍 Running comprehensive quality checks..."
	@./scripts/check-all.sh

format: ## Format code
	@echo "🎨 Formatting code..."
	@cargo fmt --all

format-check: ## Check code formatting
	@echo "🎨 Checking code formatting..."
	@cargo fmt --all -- --check

lint: ## Run clippy linter
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

# Security
audit: ## Run security audit
	@echo "🔐 Running security audit..."
	@cargo audit

deny: ## Run supply chain security checks
	@echo "🔗 Running supply chain security checks..."
	@cargo deny check

# Installation
install: ## Install plugin locally
	@echo "🔌 Installing plugin..."
	@./scripts/install-plugin.sh

install-force: ## Force reinstall plugin
	@echo "🔌 Force installing plugin..."
	@./scripts/install-plugin.sh --force

# Maintenance
clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	@./scripts/clean.sh

clean-deep: ## Deep clean including caches
	@echo "🧹 Deep cleaning..."
	@./scripts/clean.sh --deep

clean-deps: ## Clean downloaded dependencies
	@echo "🧹 Cleaning dependencies..."
	@./scripts/clean.sh --deps

# Development Workflow
watch: ## Watch for changes and run checks
	@echo "👀 Watching for changes..."
	@cargo watch -x check -x test

watch-test: ## Watch for changes and run tests
	@echo "👀 Watching for changes (tests only)..."
	@cargo watch -x test

watch-clippy: ## Watch for changes and run clippy
	@echo "👀 Watching for changes (clippy)..."
	@cargo watch -x clippy

# Documentation
docs: ## Build documentation
	@echo "📚 Building documentation..."
	@cargo doc --no-deps --all-features --open

docs-check: ## Check documentation builds
	@echo "📚 Checking documentation..."
	@cargo doc --no-deps --all-features

# Pre-commit
pre-commit-install: ## Install pre-commit hooks
	@echo "🪝 Installing pre-commit hooks..."
	@pre-commit install

pre-commit-run: ## Run pre-commit hooks
	@echo "🪝 Running pre-commit hooks..."
	@pre-commit run --all-files

# Release Preparation
release-check: ## Check if ready for release
	@echo "🚀 Checking release readiness..."
	@./scripts/check-all.sh
	@echo "✅ Release checks completed"

release-dry-run: ## Simulate release build
	@echo "🎭 Simulating release..."
	@cargo build --release
	@cargo test --release
	@echo "✅ Release simulation completed"

# Benchmarks (if implemented)
bench: ## Run benchmarks
	@echo "⚡ Running benchmarks..."
	@cargo bench

# Development Shortcuts
dev: check test ## Quick development check (format + lint + test)

ci: check-all test-coverage audit deny ## Full CI simulation

quick: format lint test ## Quick development cycle

# Git Helpers
commit-check: ## Check if ready to commit
	@echo "🔍 Pre-commit validation..."
	@make format-check
	@make lint
	@make test
	@echo "✅ Ready to commit"

# Plugin-specific targets
plugin-info: ## Show plugin information
	@echo "🔌 Plugin information..."
	@if [ -f "target/release/nu_plugin_ulid" ]; then \
		echo "Binary: target/release/nu_plugin_ulid"; \
		ls -lh target/release/nu_plugin_ulid; \
	else \
		echo "Plugin not built. Run 'make build' first."; \
	fi

plugin-test: ## Test plugin with Nushell
	@echo "🧪 Testing plugin with Nushell..."
	@if command -v nu >/dev/null 2>&1; then \
		if [ -f "target/release/nu_plugin_ulid" ]; then \
			echo "Testing plugin registration..."; \
			nu -c "plugin add target/release/nu_plugin_ulid; plugin use ulid; ulid info"; \
		else \
			echo "Plugin not built. Run 'make build' first."; \
		fi; \
	else \
		echo "Nushell not found. Please install Nushell first."; \
	fi

# Utility targets
deps-update: ## Update dependencies
	@echo "📦 Updating dependencies..."
	@cargo update

deps-outdated: ## Check for outdated dependencies
	@echo "📦 Checking for outdated dependencies..."
	@cargo outdated

deps-unused: ## Check for unused dependencies
	@echo "📦 Checking for unused dependencies..."
	@cargo +nightly udeps

tree: ## Show dependency tree
	@echo "🌳 Dependency tree..."
	@cargo tree

# Project Information
info: ## Show project information
	@echo "nu_plugin_ulid Project Information"
	@echo "================================="
	@echo "Version: $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')"
	@echo "Rust Version: $(shell rustc --version)"
	@echo "Cargo Version: $(shell cargo --version)"
	@echo "Project Size: $(shell du -sh . 2>/dev/null | cut -f1)"
	@if [ -d "target" ]; then echo "Build Artifacts: $(shell du -sh target 2>/dev/null | cut -f1)"; fi
	@echo "License: BSD-3-Clause"
	@echo "Repository: https://github.com/nushell-works/nu_plugin_ulid"