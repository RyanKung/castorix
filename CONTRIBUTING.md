# Contributing to Castorix

Thank you for your interest in contributing to Castorix! This document provides guidelines for contributing to the project.

## Development Environment Setup

### Prerequisites
- Rust 1.70+ 
- Cargo
- Git

### Initial Setup
```bash
git clone <repository-url>
cd castorix
cargo build
```

## Development Guidelines

### 1. Import Standards (CRITICAL)

We follow strict import guidelines inspired by Python's import standards:

#### ❌ FORBIDDEN:
```rust
// Wildcard imports
use std::collections::*;

// Multiple imports on one line (for readability)
use std::{collections::HashMap, io::Result};
```

#### ✅ REQUIRED:
```rust
// One import per line, explicit imports only
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Result;

// Multiple items from same module (acceptable for small lists)
use std::collections::{
    HashMap,
    HashSet,
    VecDeque,
};
```

### 2. Environment Variable Access (SECURITY CRITICAL)

Environment variable access is **STRICTLY PROHIBITED** except in specific modules:

#### ✅ ALLOWED ONLY IN:
- `src/consts.rs` - For reading configuration
- `tests/test_consts.rs` - For test environment setup

#### ❌ FORBIDDEN EVERYWHERE ELSE:
```rust
// NEVER do this in any other module
env::var("SOME_VAR")
env::set_var("SOME_VAR", "value")
env::remove_var("SOME_VAR")
```

#### ✅ USE INSTEAD:
```rust
// For configuration access
use crate::consts;
let config = consts::get_config();
let rpc_url = config.eth_rpc_url();

// For test environment setup
use test_consts::{setup_local_test_env, setup_placeholder_test_env};
setup_local_test_env();
```

### 3. Test Development

#### Test Environment Setup:
```rust
mod test_consts;
use test_consts::{
    setup_local_test_env,
    setup_placeholder_test_env,
    should_skip_rpc_tests,
};

#[tokio::test]
async fn test_something() {
    if should_skip_rpc_tests() {
        println!("Skipping RPC tests");
        return;
    }
    
    setup_local_test_env();
    // ... test code ...
}
```

#### Test Validation:
```rust
// ❌ WRONG - Warning without panic
if !output.contains("expected") {
    println!("   ⚠️ Output unexpected");
}

// ✅ CORRECT - Direct panic on failure
if !output.contains("expected") {
    panic!("Output unexpected: {}", output);
}
```

### 4. Code Quality Standards

#### Before Committing:
1. Run `cargo check` - Must pass without errors
2. Run `cargo test` - All tests must pass
3. Fix all warnings - Zero warnings policy
4. Follow import guidelines - No wildcard imports

#### Code Style:
- Use `rustfmt` for formatting
- Follow Rust naming conventions
- Document public APIs with `///` comments
- Include examples in documentation when helpful

### 5. Pull Request Process

#### Before Submitting:
1. **Import Check**: Ensure no `use xxx::*;` imports
2. **Environment Check**: Verify no unauthorized `env::` usage
3. **Test Check**: All tests pass, no warnings
4. **Documentation**: Update docs if adding new features

#### PR Description:
- Describe what changes were made
- Explain why the changes were necessary
- List any breaking changes
- Include test results

#### Review Process:
- All PRs require review
- Automated checks must pass
- Manual review for rule compliance
- Security review for environment variable usage

### 6. Architecture Guidelines

#### Module Organization:
- `src/lib.rs` - Library entry point
- `src/core/` - Core functionality (reusable)
- `src/cli/` - Command-line interface
- `src/farcaster/` - Farcaster protocol
- `tests/` - Integration tests

#### Public API Design:
- Prefer composition over inheritance
- Use explicit types over generic types when possible
- Provide clear error messages
- Follow Rust ownership patterns

### 7. Security Considerations

#### Environment Variables:
- **NEVER** read environment variables directly
- **ALWAYS** use `consts::get_config()` for configuration
- **NEVER** hardcode sensitive values in code

#### Key Management:
- Use encrypted storage for private keys
- Never log private keys or sensitive data
- Follow secure coding practices for cryptographic operations

## Getting Help

### Resources:
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Project documentation in `docs/`

### Questions:
- Open an issue for questions about the codebase
- Use GitHub Discussions for general questions
- Join our community chat (if available)

## Reporting Issues

### Bug Reports:
- Use the issue template
- Include steps to reproduce
- Provide system information
- Include relevant logs

### Security Issues:
- **DO NOT** open public issues for security vulnerabilities
- Contact maintainers privately
- Follow responsible disclosure practices

## Recognition

Contributors will be recognized in:
- CONTRIBUTORS.md file
- Release notes (for significant contributions)
- Project documentation

---

Thank you for contributing to Castorix! Your efforts help make the project better for everyone.
