# Makefile for Castorix project

.PHONY: help test test-mock test-real test-anvil clean-anvil build check

# Default target
help:
	@echo "Available targets:"
	@echo "  test        - Run all tests (mock mode)"
	@echo "  test-real   - Run tests with real blockchain (requires Anvil)"
	@echo "  test-anvil  - Start Anvil and run tests"
	@echo "  test-mock   - Run tests in mock mode only"
	@echo "  build       - Build the project"
	@echo "  check       - Check the project"
	@echo "  clean-anvil - Stop any running Anvil processes"
	@echo "  help        - Show this help message"

# Build the project
build:
	@echo "🔨 Building project..."
	cargo build

# Check the project
check:
	@echo "🔍 Checking project..."
	cargo check

# Run tests in mock mode (default)
test: test-mock

# Run tests in mock mode only
test-mock:
	@echo "🧪 Running tests in mock mode..."
	cargo test

# Start Anvil and run tests with real blockchain
test-anvil:
	@echo "🚀 Starting test environment with Anvil..."
	./scripts/test-with-anvil.sh

# Run tests with real blockchain (assumes Anvil is already running)
test-real:
	@echo "🧪 Running tests with real blockchain..."
	RUNNING_TESTS=1 cargo test -- --ignored

# Stop any running Anvil processes
clean-anvil:
	@echo "🧹 Cleaning up Anvil processes..."
	@if [ -f /tmp/anvil-test-8545.pid ]; then \
		ANVIL_PID=$$(cat /tmp/anvil-test-8545.pid); \
		if kill -0 $$ANVIL_PID 2>/dev/null; then \
			echo "   Stopping Anvil (PID: $$ANVIL_PID)"; \
			kill $$ANVIL_PID; \
		fi; \
		rm -f /tmp/anvil-test-8545.pid; \
	fi
	@echo "✅ Cleanup complete"

# Run specific test suite
test-simple:
	@echo "🧪 Running simple tests..."
	cargo test simple_tests

test-onchain:
	@echo "🧪 Running onchain tests..."
	cargo test onchain_tests

# Development helpers
dev-setup:
	@echo "⚙️  Setting up development environment..."
	@if ! command -v anvil &> /dev/null; then \
		echo "Installing Foundry..."; \
		curl -L https://foundry.paradigm.xyz | bash; \
		foundryup; \
	fi
	@echo "✅ Development environment ready"

# Run tests with verbose output
test-verbose:
	@echo "🧪 Running tests with verbose output..."
	cargo test -- --nocapture

# Run tests and show coverage (if tarpaulin is installed)
test-coverage:
	@echo "🧪 Running tests with coverage..."
	@if command -v cargo-tarpaulin &> /dev/null; then \
		cargo tarpaulin --out Html; \
		echo "📊 Coverage report generated in tarpaulin-report.html"; \
	else \
		echo "❌ cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"; \
	fi
