# Castorix

A comprehensive Rust library for interacting with Farcaster protocol, providing type-safe contract bindings, ENS resolution, and message handling capabilities.

## Overview

Castorix is a production-ready Rust SDK for the Farcaster protocol, featuring:

- 🔒 **Type-safe Farcaster contract bindings** using official ABI definitions
- 🌐 **ENS resolution** for Ethereum and Base domains
- 📨 **Message handling** with protobuf serialization
- 🔑 **Key management** with encryption and secure storage
- ⚡ **High-performance** async/await support
- 🛡️ **Memory safety** with Rust's ownership system

## Features

### Contract Integration

- **Official ABI bindings** generated from Farcaster's official contract repository
- **All major contracts supported**: IdRegistry, KeyRegistry, StorageRegistry, IdGateway, KeyGateway, Bundler, RecoveryProxy, SignedKeyRequestValidator
- **Type-safe contract calls** with automatic parameter encoding/decoding
- **Comprehensive error handling** with detailed error messages

### ENS Resolution

- **Multi-chain support**: Ethereum Mainnet and Base
- **Subdomain resolution**: Handles complex ENS subdomain structures
- **Caching**: Efficient caching for improved performance
- **Fallback mechanisms**: Graceful handling of resolution failures

### Message Handling

- **Protobuf serialization**: Full support for Farcaster message types
- **Username proofs**: Generation and verification of username proofs
- **Message validation**: Built-in validation for all message types

### Key Management

- **Encrypted storage**: Secure storage of private keys
- **Multiple key types**: Support for Ed25519 and Ethereum keys
- **Environment-based configuration**: Flexible configuration management
- **Password protection**: Optional password-based encryption

## Installation

Add Castorix to your `Cargo.toml`:

```toml
[dependencies]
castorix = "0.1.0"
```

## Quick Start

### Basic Contract Interaction

```rust
use castorix::farcaster::contracts::{
    id_registry_abi::IdRegistryAbi,
    types::ContractAddresses,
};
use ethers::providers::{Provider, Http};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize provider
    let provider = Provider::<Http>::try_from("https://optimism-mainnet.g.alchemy.com/v2/your_api_key")?;
    
    // Get contract addresses
    let addresses = ContractAddresses::default();
    
    // Create ABI-based contract instance
    let id_registry = IdRegistryAbi::new(provider, addresses.id_registry)?;
    
    // Get current ID counter
    match id_registry.id_counter().await? {
        castorix::farcaster::contracts::types::ContractResult::Success(counter) => {
            println!("Current ID counter: {}", counter);
        },
        castorix::farcaster::contracts::types::ContractResult::Error(e) => {
            println!("Error: {}", e);
        }
    }
    
    Ok(())
}
```

### ENS Resolution

```rust
use castorix::ens_proof::EnsProof;
use castorix::consts::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    let config = Config::load()?;
    
    // Create ENS proof resolver
    let ens_proof = EnsProof::new(&config)?;
    
    // Resolve ENS name
    let address = ens_proof.resolve_name("vitalik.eth").await?;
    println!("Resolved address: {:?}", address);
    
    Ok(())
}
```

### Key Management

```rust
use castorix::key_manager::KeyManager;
use castorix::consts::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load()?;
    
    // Create key manager
    let key_manager = KeyManager::from_env(&config)?;
    
    // Generate new Ed25519 key
    let (public_key, _private_key) = key_manager.generate_ed25519_keypair()?;
    println!("Generated public key: {:?}", public_key);
    
    Ok(())
}
```

## Configuration

Create a `.env` file in your project root:

```bash
# Ethereum RPC endpoint for ENS resolution and blockchain interactions
ETH_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your_api_key_here

# Base chain RPC endpoint for Base subdomain resolution
ETH_BASE_RPC_URL=https://mainnet.base.org

# Optimism RPC endpoint for Farcaster contract interactions
ETH_OP_RPC_URL=https://optimism-mainnet.g.alchemy.com/v2/your_api_key_here

# Farcaster Hub URL for submitting messages and proofs
FARCASTER_HUB_URL=https://hub-api.neynar.com
```

## Dependencies

### Core Dependencies

- **ethers**: Ethereum library for contract interactions and blockchain communication
- **tokio**: Async runtime for high-performance I/O operations
- **anyhow**: Error handling with context-aware error messages
- **serde**: Serialization/deserialization framework
- **chrono**: Date and time handling
- **reqwest**: HTTP client for API interactions

### Cryptographic Dependencies

- **ed25519-dalek**: Ed25519 signature scheme implementation
- **k256**: Secp256k1 elliptic curve cryptography
- **sha2**: SHA-256 hashing algorithm
- **blake3**: BLAKE3 cryptographic hash function
- **argon2**: Password hashing and key derivation

### Protocol Dependencies

- **protobuf**: Protocol Buffers for message serialization
- **hex**: Hexadecimal encoding/decoding
- **base64**: Base64 encoding/decoding
- **bs58**: Base58 encoding for Bitcoin-style addresses

### Development Dependencies

- **foundry**: Solidity development framework for contract compilation
- **forge**: Solidity compiler and testing framework

## Architecture

### Project Structure

```
src/
├── farcaster/
│   ├── contracts/
│   │   ├── generated/          # Auto-generated ABI bindings
│   │   ├── id_registry_abi.rs  # Type-safe contract wrappers
│   │   ├── types.rs            # Contract types and addresses
│   │   └── client.rs           # Unified contract client
│   └── mod.rs
├── ens_proof/                  # ENS resolution and proof generation
├── key_manager.rs             # Key management and encryption
├── consts.rs                  # Configuration management
└── lib.rs                     # Library root
```

### Contract Integration Flow

1. **ABI Generation**: Solidity contracts are compiled using Foundry
2. **Rust Binding Generation**: ethers-rs abigen generates type-safe bindings
3. **Wrapper Creation**: High-level wrappers provide convenient APIs
4. **Client Integration**: Unified client manages all contract interactions

## Examples

### Contract Examples

- [`test_abi_implementation.rs`](examples/test_abi_implementation.rs): Demonstrates ABI-based contract calls
- [`contract_interface_test.rs`](examples/contract_interface_test.rs): Tests different contract interfaces
- [`onchain_data_reader.rs`](examples/onchain_data_reader.rs): Reads on-chain contract data

### Configuration Examples

- [`config_example.rs`](examples/config_example.rs): Shows how to use the configuration system

## Development

### Prerequisites

- Rust 1.70+
- Foundry (for Solidity compilation)
- Git (for submodule management)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/your-org/castorix.git
cd castorix

# Initialize and update submodules
git submodule update --init --recursive

# Build the project
cargo build

# Run tests
cargo test

# Run examples
cargo run --example test_abi_implementation
```

### Contract Development

The project includes the official Farcaster contracts as a git submodule. When contracts are updated:

```bash
# Update the submodule
git submodule update --remote contracts

# Rebuild to regenerate bindings
cargo build
```

## Testing

```bash
# Run all tests
cargo test

# Run contract tests
cargo test --package castorix --lib farcaster::contracts::tests

# Run on-chain tests (requires RPC access)
cargo test --package castorix --lib farcaster::contracts::onchain_tests
```

## Error Handling

Castorix provides comprehensive error handling with detailed context:

```rust
use anyhow::Result;

async fn example() -> Result<()> {
    match id_registry.owner_of(1).await? {
        ContractResult::Success(owner) => println!("Owner: {:?}", owner),
        ContractResult::Error(e) => println!("Contract error: {}", e),
    }
    Ok(())
}
```

## Performance

- **Async/await**: Non-blocking I/O operations
- **Connection pooling**: Efficient HTTP connection management
- **Caching**: Built-in caching for ENS resolution and contract calls
- **Batch operations**: Support for batch contract calls

## Security

- **Memory safety**: Rust's ownership system prevents memory leaks
- **Encrypted storage**: Private keys are encrypted at rest
- **Secure defaults**: Safe configuration defaults
- **Input validation**: Comprehensive input validation

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust naming conventions
- Add tests for new features
- Update documentation
- Ensure all tests pass
- Use `cargo fmt` and `cargo clippy`

## License

This project is licensed under the GNU General Public License v2.0 - see the [LICENSE](LICENSE) file for details.

**Important:** GPLv2 is a copyleft license that requires derivative works to also be licensed under GPLv2. If you plan to use this library in a proprietary application, please contact the maintainers for alternative licensing options.

## Acknowledgments

- [Farcaster Protocol](https://farcaster.xyz/) for the amazing decentralized social protocol
- [ethers-rs](https://github.com/gakonst/ethers-rs) for excellent Ethereum tooling
- [OpenZeppelin](https://openzeppelin.com/) for secure smart contract libraries

## Support

- 📖 [Documentation](https://docs.rs/castorix)
- 🐛 [Issue Tracker](https://github.com/your-org/castorix/issues)
- 💬 [Discussions](https://github.com/your-org/castorix/discussions)
- 📧 [Email Support](mailto:support@your-org.com)

## Changelog

### v0.1.0 (Current)

- ✅ Official Farcaster contract ABI bindings
- ✅ ENS resolution for Ethereum and Base
- ✅ Encrypted key management
- ✅ Message handling with protobuf
- ✅ Comprehensive error handling
- ✅ Async/await support
- ✅ Configuration management
- ✅ Extensive test coverage

---

**Castorix** - Empowering Rust developers to build on Farcaster with confidence and type safety.
