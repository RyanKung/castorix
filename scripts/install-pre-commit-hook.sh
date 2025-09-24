#!/bin/bash

# Install pre-commit hook for castorix project
# This script sets up the pre-commit hook that runs cargo fmt and clippy before commits

set -e

echo "🔧 Installing pre-commit hook for castorix..."

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "❌ Not in a git repository. Please run this script from the project root."
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Copy the pre-commit hook
cp scripts/pre-commit-hook.sh .git/hooks/pre-commit

# Make it executable
chmod +x .git/hooks/pre-commit

echo "✅ Pre-commit hook installed successfully!"
echo ""
echo "📋 The hook will now run the following checks before each commit:"
echo "   - cargo +nightly fmt (format code)"
echo "   - cargo clippy --fix (auto-fix clippy issues)"
echo "   - Check for multi-import statements"
echo "   - Check for TODO/FIXME comments"
echo "   - Run quick unit tests"
echo ""
echo "💡 To disable the hook temporarily, run:"
echo "   git commit --no-verify -m \"your message\""
echo ""
echo "💡 To uninstall the hook, run:"
echo "   rm .git/hooks/pre-commit"
