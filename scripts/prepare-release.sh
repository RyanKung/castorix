#!/bin/bash
# Script to prepare the project for crates.io release

set -e

echo "🚀 Preparing castorix for crates.io release..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Not in project root directory"
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Ensure contracts submodule is initialized
echo "📦 Initializing contracts submodule..."
if [ -f "contracts/.git" ]; then
    echo "✅ Contracts submodule already initialized"
else
    echo "🔧 Initializing contracts submodule..."
    git submodule update --init --recursive contracts || {
        echo "⚠️ Warning: Failed to initialize contracts submodule"
        echo "   This is OK if you're publishing without contract bindings"
    }
fi

# Build the project to generate all required files
echo "🔨 Building project to generate contract bindings..."
cargo build --all-features --release

# Check if generated files exist
if [ -d "src/farcaster/contracts/generated" ]; then
    echo "✅ Generated contract bindings found"
    ls -la src/farcaster/contracts/generated/
else
    echo "⚠️ Warning: No generated contract bindings found"
    echo "   The package will work but without contract interaction features"
fi

# Run all tests to ensure everything works
echo "🧪 Running tests..."
cargo test --all-features

# Check formatting
echo "🎨 Checking code formatting..."
cargo fmt --all -- --check

# Run clippy
echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Check if package is ready for publishing
echo "📋 Checking package readiness..."
cargo package --dry-run

echo "✅ Package is ready for publishing!"
echo ""
echo "To publish to crates.io:"
echo "1. Update version in Cargo.toml"
echo "2. Run: cargo package"
echo "3. Run: cargo publish"
echo ""
echo "📝 Note: The generated contract bindings are included in the package"
echo "   Users won't need to build them from source."
