# Castorix Development Rules

## Import Guidelines

### 1. Strict Import Policy (Python-style)

**MANDATORY**: Follow strict import guidelines similar to Python's import standards:

- ❌ **NEVER** use wildcard imports: `use xxx::*;`
- ✅ **ALWAYS** use explicit imports: `use xxx::{Item1, Item2, Item3};`
- ✅ **ONE import per line** for better readability and maintenance

#### Examples:

```rust
// ❌ WRONG - Wildcard import
use std::collections::*;

// ❌ WRONG - Multiple items on one line
use std::collections::{HashMap, HashSet, VecDeque};

// ✅ CORRECT - Explicit imports, one per line
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

// ✅ CORRECT - Multiple items from same module (acceptable for small lists)
use std::collections::{
    HashMap,
    HashSet,
    VecDeque,
};
```

### 2. Environment Variable Access Rules

**STRICT PROHIBITION**: Environment variable access is heavily restricted:

#### Allowed Modules:
- ✅ `src/consts.rs` - **ONLY** for reading configuration
- ✅ `tests/test_consts.rs` - **ONLY** for test environment setup

#### Prohibited Everywhere Else:
- ❌ **NEVER** use `std::env::var()` or `env::var()` in any other module
- ❌ **NEVER** use `std::env::set_var()` or `env::set_var()` in any other module
- ❌ **NEVER** use `std::env::remove_var()` or `env::remove_var()` in any other module

#### Configuration Management:
- ✅ **ALWAYS** use `crate::consts::get_config()` for accessing configuration
- ✅ **ALWAYS** use `tests::test_consts::*` functions for test environment setup

### 3. Test Environment Management

#### Test Environment Setup:
```rust
// ✅ CORRECT - Use test_consts functions
mod test_consts;
use test_consts::{
    setup_local_test_env,
    setup_placeholder_test_env,
    should_skip_rpc_tests,
};

// ❌ WRONG - Direct environment variable access
env::set_var("ETH_OP_RPC_URL", "http://127.0.0.1:8545");
```

#### Available Test Functions:
- `setup_local_test_env()` - For local Anvil testing
- `setup_placeholder_test_env()` - For configuration validation
- `setup_demo_test_env()` - For demo API testing
- `should_skip_rpc_tests()` - Check if RPC tests should be skipped

### 4. Error Handling in Tests

#### Strict Test Validation:
- ❌ **NEVER** use `println!("   ⚠️ ...")` without `panic!`
- ✅ **ALWAYS** use `panic!` for test failures
- ✅ **ALWAYS** validate output content with assertions

#### Examples:

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

### 5. Module Organization

#### Library Structure:
- `src/lib.rs` - Main library entry point
- `src/core/` - Core library functionality
- `src/cli/` - Command-line interface
- `src/farcaster/` - Farcaster protocol implementation
- `tests/` - Integration tests

#### Public API:
- ✅ **ALWAYS** re-export types through `pub use` in module `mod.rs`
- ✅ **ALWAYS** use explicit re-exports, never wildcard re-exports

### 6. Code Quality Standards

#### Compilation:
- ✅ **ALWAYS** ensure `cargo check` passes without errors
- ✅ **ALWAYS** fix all warnings before committing
- ✅ **ALWAYS** run tests before committing

#### Documentation:
- ✅ **ALWAYS** document public APIs with `///` comments
- ✅ **ALWAYS** include examples in documentation when appropriate

## Enforcement

These rules are enforced through:
1. Code review process
2. Automated linting (where possible)
3. Manual verification during development

## Violations

Violations of these rules will result in:
1. Immediate code review rejection
2. Required fixes before merge
3. Documentation of violations for team learning

---

**Remember**: These rules ensure code maintainability, security, and consistency across the project.
