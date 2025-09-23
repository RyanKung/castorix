# Release Guide for Castorix

This document explains how to prepare and publish castorix to crates.io.

## 🚀 Quick Release Process

### 1. Pre-Release Preparation

```bash
# Run the automated release preparation script
./scripts/prepare-release.sh
```

This script will:
- Clean previous builds
- Initialize contracts submodule
- Generate contract bindings
- Run all tests
- Check formatting and linting
- Verify package readiness

### 2. Manual Release Steps

```bash
# Update version in Cargo.toml (e.g., 0.1.0 -> 0.1.1)
# Then package and publish
cargo package
cargo publish
```

## 📦 Package Contents

### What's Included in the Published Package

The published package includes:
- ✅ All source code (`src/**/*.rs`)
- ✅ Pre-generated contract bindings (`src/farcaster/contracts/generated/**/*.rs`)
- ✅ Protobuf definitions (`proto/**/*.proto`)
- ✅ Documentation (`README.md`)
- ✅ License (`LICENSE`)
- ✅ Build script (`build.rs`)

### What's Excluded from the Published Package

The following development-only files are excluded:
- ❌ Contract source code (`contracts/**/*`)
- ❌ Generated ABIs (`generated_abis/**/*`)
- ❌ Build artifacts (`target/**/*`)
- ❌ CI/CD workflows (`.github/**/*`)
- ❌ Test files (`tests/**/*`)
- ❌ Test data (`test_data/**/*`)

## 🔧 Build System

### Development Environment
- Uses `build.rs` to generate contract bindings from source
- Requires `contracts` submodule and Foundry installation
- Generates files in `src/farcaster/contracts/generated/`

### Published Package
- Includes pre-generated contract bindings
- Users don't need Foundry or contract sources
- `build.rs` detects environment and skips generation if contracts unavailable

## 🎯 Key Benefits

1. **Zero Build Dependencies**: Users don't need Foundry or Solidity tools
2. **Fast Installation**: No contract compilation during `cargo build`
3. **Reliable Builds**: Pre-generated bindings ensure consistent results
4. **Small Package Size**: Only essential files included

## ⚠️ Important Notes

### Version Management
- Always update version in `Cargo.toml` before releasing
- Follow semantic versioning (semver)
- Update CHANGELOG.md for significant changes

### Contract Updates
When Farcaster contracts change:
1. Update contracts submodule
2. Regenerate bindings: `cargo build --all-features`
3. Commit the updated generated files
4. Update version and release

### Testing Before Release
Always run the full test suite:
```bash
cargo test --all-features
cargo test --test "*"  # Integration tests
```

## 🐛 Troubleshooting

### "Generated files not found" Error
- Ensure `contracts` submodule is initialized
- Run `cargo build --all-features` to generate bindings
- Check that `src/farcaster/contracts/generated/` exists

### "Package too large" Error
- Check `exclude` list in `Cargo.toml`
- Ensure large files are properly excluded
- Use `cargo package --list` to inspect package contents

### "Build failed" in CI
- Ensure contracts submodule is initialized in CI
- Check that all dependencies are available
- Verify build script works in clean environment
