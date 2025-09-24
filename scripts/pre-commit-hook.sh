#!/bin/bash

# Pre-commit hook for castorix project
# This hook runs cargo fmt and cargo clippy --fix before allowing commits
#
# Testing Strategy:
# - Pre-commit: Only runs unit tests (fast, no external dependencies)
# - Local development: Use 'make test-local' for integration tests with Anvil nodes
# - CI: GitHub Actions manages Anvil nodes and runs all tests

set -e

echo "🔧 Running pre-commit checks..."

# Check if we're in a Rust project
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Not in a Rust project directory. Skipping pre-commit checks."
    exit 0
fi

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust toolchain."
    exit 1
fi

# Check if nightly rustfmt is available
if ! cargo +nightly fmt --version &> /dev/null; then
    echo "❌ Nightly rustfmt not available. Please install nightly toolchain with rustfmt component."
    echo "   Run: rustup toolchain install nightly --component rustfmt"
    exit 1
fi

echo "📝 Running cargo +nightly fmt..."
cargo +nightly fmt

echo "🔍 Running cargo clippy --fix..."
cargo clippy --fix --allow-dirty --allow-staged

echo "🔍 Checking for multi-import statements..."
if grep -r "^[[:space:]]*use.*,.*;" src/ tests/ --include="*.rs"; then
    echo "❌ Found multi-import use statements. Please use one import per line."
    echo "Example: use module::{item1, item2}; should be:"
    echo "use module::item1;"
    echo "use module::item2;"
    exit 1
fi

echo "🔍 Checking for TODO/FIXME comments..."
if grep -r "TODO\|FIXME\|XXX\|HACK" src/ --include="*.rs"; then
    echo "⚠️ Found TODO/FIXME comments in source code:"
    grep -r "TODO\|FIXME\|XXX\|HACK" src/ --include="*.rs" || true
    echo "Please review and address these comments before committing."
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "🧪 Running quick unit tests..."
cargo test --lib --quiet

echo "🔍 Checking for integration test dependencies..."
if grep -r "anvil\|Anvil" tests/ --include="*.rs" > /dev/null 2>&1; then
    echo "ℹ️  Integration tests detected (require Anvil nodes)"
    echo "   Use 'make test-local' to run integration tests locally"
    echo "   CI will run integration tests with pre-started nodes"
fi

echo "✅ Pre-commit checks passed!"
echo "📋 Summary:"
echo "   - Code formatted with nightly rustfmt"
echo "   - Clippy auto-fixes applied"
echo "   - Import formatting validated"
echo "   - Unit tests passed"
echo "   - Integration tests skipped (require Anvil nodes)"
echo "   - Ready to commit"

exit 0
