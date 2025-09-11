# Castorix

A comprehensive Rust library and CLI tool for interacting with the Farcaster protocol, providing type-safe contract bindings, ENS resolution, key management, and message handling capabilities.

## Overview

Castorix is a production-ready Rust SDK for the Farcaster protocol, featuring:

- 🔒 **Type-safe Farcaster contract bindings** using official ABI definitions
- 🌐 **ENS resolution** for Ethereum and Base domains
- 📨 **Message handling** with protobuf serialization
- 🔑 **Encrypted key management** with secure storage
- 🖥️ **CLI interface** for easy interaction with Farcaster
- ⚡ **High-performance** async/await support
- 🛡️ **Memory safety** with Rust's ownership system

## Features

### Contract Integration

- **Official ABI bindings** generated from Farcaster's official contract repository
- **All major contracts supported**: IdRegistry, KeyRegistry, StorageRegistry, IdGateway, KeyGateway, Bundler, RecoveryProxy, SignedKeyRequestValidator
- **Type-safe contract calls** with automatic parameter encoding/decoding
- **Comprehensive error handling** with detailed error messages
- **Network verification** with automatic contract address validation

### ENS Resolution

- **Multi-chain support**: Ethereum Mainnet and Base
- **Subdomain resolution**: Handles complex ENS subdomain structures
- **Username proof generation**: Create and verify Farcaster username proofs
- **Fallback mechanisms**: Graceful handling of resolution failures

### Key Management

- **Encrypted storage**: Secure storage of private keys using AES-GCM
- **Multiple key types**: Support for Ed25519 and Ethereum ECDSA keys
- **Password protection**: Optional password-based encryption
- **Environment-based configuration**: Flexible configuration management
- **Key derivation**: Support for BIP39 mnemonic phrases

### CLI Interface

- **Interactive commands**: Easy-to-use CLI for all major operations
- **Key management**: Generate, import, export, and manage cryptographic keys
- **Hub interaction**: Submit messages and interact with Farcaster Hub
- **ENS operations**: Resolve names and generate proofs
- **Contract queries**: Query Farcaster contracts directly

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/your-org/castorix.git
cd castorix

# Initialize and update submodules
git submodule update --init --recursive

# Build the project
cargo build --release

# Install the CLI tool
cargo install --path .
```

### Dependencies

- Rust 1.70+
- Git (for submodule management)
- Foundry (for contract compilation, optional)

## Quick Start

### CLI Usage

```bash
# Initialize configuration
castorix init

# Generate a new Ed25519 key pair
castorix key generate --type ed25519

# Resolve an ENS name
castorix ens resolve vitalik.eth

# Query Farcaster contract data
castorix hub get-fid --address 0x...
```

### Library Usage

#### Basic Contract Interaction

```rust
use castorix::farcaster::contracts::FarcasterContractClient;
use castorix::consts::get_config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration
    let config = get_config();
    
    // Create contract client with default addresses
    let client = FarcasterContractClient::new_with_default_addresses(
        config.eth_op_rpc_url().to_string()
    )?;
    
    // Get network information
    let network_info = client.get_network_info().await?;
    println!("Chain ID: {}", network_info.chain_id);
    println!("Block number: {}", network_info.block_number);
    
    // Verify contracts are accessible
    let verification = client.verify_contracts().await?;
    println!("ID Registry verified: {}", verification.id_registry);
    
    Ok(())
}
```

#### ENS Resolution

```rust
use castorix::ens_proof::EnsProof;
use castorix::consts::get_config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration
    let config = get_config();
    
    // Create ENS proof resolver
    let ens_proof = EnsProof::new(&config)?;
    
    // Resolve ENS name
    let address = ens_proof.resolve_name("vitalik.eth").await?;
    println!("Resolved address: {:?}", address);
    
    // Generate username proof
    let proof = ens_proof.generate_username_proof("vitalik.eth").await?;
    println!("Username proof: {:?}", proof);
    
    Ok(())
}
```

#### Key Management

```rust
use castorix::key_manager::KeyManager;
use castorix::consts::get_config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration
    let config = get_config();
    
    // Create key manager
    let key_manager = KeyManager::from_env(&config)?;
    
    // Generate new Ed25519 key pair
    let (public_key, private_key) = key_manager.generate_ed25519_keypair()?;
    println!("Generated public key: {:?}", public_key);
    
    // Store encrypted key
    key_manager.store_encrypted_ed25519_key(&private_key, "my_key_id")?;
    
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

# Optional: Custom key storage directory
KEY_STORAGE_PATH=~/.castorix/keys
```

## Architecture

### Project Structure

```
src/
├── cli/                      # Command-line interface
│   ├── commands.rs          # CLI command definitions
│   ├── handlers/            # Command handlers
│   └── types.rs             # CLI types
├── farcaster/               # Farcaster protocol integration
│   ├── contracts/           # Contract bindings and clients
│   │   ├── generated/       # Auto-generated ABI bindings
│   │   ├── client.rs        # Unified contract client
│   │   ├── types.rs         # Contract types and addresses
│   │   └── *_tests.rs       # Contract tests
│   └── mod.rs
├── ens_proof/               # ENS resolution and proof generation
├── message/                 # Message handling and protobuf
├── key_manager.rs           # Key management
├── encrypted_*_manager.rs   # Encrypted key storage
├── farcaster_client.rs      # Farcaster Hub client
├── consts.rs                # Configuration management
└── lib.rs                   # Library root
```

### Contract Integration Flow

1. **ABI Generation**: Solidity contracts are compiled using Foundry
2. **Rust Binding Generation**: ethers-rs abigen generates type-safe bindings
3. **Client Integration**: Unified client manages all contract interactions
4. **Type Safety**: All contract calls are type-checked at compile time

## Examples

### Contract Examples

- [`simple_contract_test.rs`](examples/simple_contract_test.rs): Basic contract connectivity test
- [`contract_interface_test.rs`](examples/contract_interface_test.rs): Tests different contract interfaces
- [`onchain_data_reader.rs`](examples/onchain_data_reader.rs): Reads on-chain contract data
- [`farcaster_contracts_demo.rs`](examples/farcaster_contracts_demo.rs): Comprehensive contract demo

### Configuration Examples

- [`config_example.rs`](examples/config_example.rs): Shows how to use the configuration system

## Development

### Prerequisites

- Rust 1.70+
- Git (for submodule management)
- Foundry (for contract compilation)

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
cargo run --example simple_contract_test
```

### Contract Development

The project includes the official Farcaster contracts as git submodules:

```bash
# Update the submodules
git submodule update --remote contracts
git submodule update --remote snapchain

# Rebuild to regenerate bindings
cargo build
```

## Testing

```bash
# Run all tests
cargo test

# Run contract tests
cargo test --lib farcaster::contracts

# Run tests with Anvil (local blockchain)
make test-anvil

# Run mock tests (no blockchain required)
make test-mock
```

### Testing Strategy

- **Unit tests**: Test individual components in isolation
- **Integration tests**: Test contract interactions with mock blockchain
- **On-chain tests**: Test with real blockchain (requires RPC access)
- **Hybrid testing**: Automatically fall back to mock when blockchain unavailable

## CLI Commands

```bash
# Key management
castorix key generate --type ed25519
castorix key list
castorix key export --id <key_id>

# ENS operations
castorix ens resolve <name>
castorix ens generate-proof <name>

# Hub operations
castorix hub get-fid --address <address>
castorix hub submit-message --file <message.json>

# Contract operations
castorix contract verify
castorix contract get-network-info
```

## Error Handling

Castorix provides comprehensive error handling with detailed context:

```rust
use anyhow::Result;

async fn example() -> Result<()> {
    match client.get_network_info().await {
        Ok(info) => println!("Chain ID: {}", info.chain_id),
        Err(e) => println!("Network error: {}", e),
    }
    Ok(())
}
```

## Performance

- **Async/await**: Non-blocking I/O operations
- **Connection pooling**: Efficient HTTP connection management
- **Hybrid testing**: Automatic fallback between real and mock environments
- **Batch operations**: Support for batch contract calls

## Security

- **Memory safety**: Rust's ownership system prevents memory leaks
- **Encrypted storage**: Private keys are encrypted using AES-GCM
- **Secure defaults**: Safe configuration defaults
- **Input validation**: Comprehensive input validation
- **Password protection**: Optional password-based key encryption

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
- Follow the existing code structure

## License

This project is licensed under the GNU General Public License v2.0 - see the [LICENSE](LICENSE) file for details.

**Important:** GPLv2 is a copyleft license that requires derivative works to also be licensed under GPLv2. If you plan to use this library in a proprietary application, please contact the maintainers for alternative licensing options.

## Acknowledgments

- [Farcaster Protocol](https://farcaster.xyz/) for the amazing decentralized social protocol
- [ethers-rs](https://github.com/gakonst/ethers-rs) for excellent Ethereum tooling
- [OpenZeppelin](https://openzeppelin.com/) for secure smart contract libraries
- [Foundry](https://book.getfoundry.sh/) for Solidity development tools

## Support

- 📖 [Documentation](https://docs.rs/castorix)
- 🐛 [Issue Tracker](https://github.com/your-org/castorix/issues)
- 💬 [Discussions](https://github.com/your-org/castorix/discussions)

## Changelog

### v0.1.0 (Current)

- ✅ Official Farcaster contract ABI bindings
- ✅ ENS resolution for Ethereum and Base
- ✅ Encrypted key management with AES-GCM
- ✅ CLI interface for easy interaction
- ✅ Message handling with protobuf
- ✅ Comprehensive error handling
- ✅ Async/await support
- ✅ Configuration management
- ✅ Hybrid testing (real + mock environments)
- ✅ Extensive test coverage

---

**Castorix** - Empowering Rust developers to build on Farcaster with confidence and type safety.