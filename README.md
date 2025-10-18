# 🔐 Castorix — Farcaster Power Toolkit

<div align="center">
  <img src="logo.png" alt="Castorix Logo" width="200" height="200">
</div>

[![License: GPL-2.0](https://img.shields.io/badge/License-GPL--2.0-blue.svg)](https://opensource.org/licenses/GPL-2.0)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Farcaster](https://img.shields.io/badge/Farcaster-Protocol-purple.svg)](https://farcaster.xyz)
[![Snapchain](https://img.shields.io/badge/Snapchain-Ready-green.svg)](https://github.com/farcasterxyz/snapchain)

Castorix is a Rust command-line interface and library for Farcaster builders. It keeps your custody wallets encrypted, generates Basenames/ENS username proofs, registers Ed25519 signers, pulls Hub data, and stays in sync with Snapchain — all from one toolchain.

## 🚀 Quick Start

Get up and running in 5 minutes:

```bash
# 1. Clone and setup
git clone https://github.com/RyanKung/castorix.git
cd castorix
cp env.example .env

# 2. Edit .env with your API keys
# ETH_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your-key
# FARCASTER_HUB_URL=https://hub-api.neynar.com

# 3. Build and install
cargo build
cargo install --path .

# 4. Create your first encrypted wallet
castorix key generate-encrypted my-wallet "My Main Wallet"

# 5. Create an ENS username proof
castorix ens create yourname.eth 12345 --wallet-name my-wallet

# 6. Submit to Farcaster Hub
castorix hub submit-proof ./proof_yourname_eth_12345.json 12345 --wallet-name my-wallet
```

> **Need help?** Check out the [examples](examples/) directory or jump to the [CLI Quick Tour](#-cli-quick-tour).

## 🌟 Feature Highlights
- 🔐 **Encrypted key vault** — interactive flows keep ECDSA custody wallets under `~/.castorix/keys`
- 🏷️ **Basename & ENS proofs** — resolve domains, audit Base subdomains, and mint Farcaster-ready username proofs
- 📡 **Hub power tools** — fetch user graphs, storage stats, custody addresses, casts, and push proof submissions
- ✍️ **Signer management** — generate Ed25519 keys, register/unregister with dry-run previews, and export safely
- 🚨 **Spam intelligence** — optional labels from the `merkle-team/labels` dataset bundled as a submodule
- 🤖 **MCP Server** — expose Farcaster query tools to AI assistants (22 tools for Claude Desktop and more)
- 🧩 **All-in-one workspace** — Farcaster contract bindings, helper binaries, and a Snapchain node live in the repo

## 🗂️ Repository Layout
```
.
├── src/                  # CLI entry points, Farcaster client, key managers
│   ├── cli/              # Command-line interface and handlers
│   ├── farcaster/        # Farcaster protocol integration
│   ├── ens_proof/        # ENS domain proof generation
│   ├── key_manager.rs    # Encrypted key management
│   └── main.rs           # Application entry point
├── tests/                # Integration tests (many expect a local Anvil node)
├── examples/             # Example binaries and demos
│   ├── basic_key_management.rs
│   ├── ens_proof_creation.rs
│   └── README.md
├── contracts/            # Solidity contracts, scripts, Foundry config
│   ├── src/              # Smart contract source code
│   ├── script/           # Deployment scripts
│   └── test/             # Contract tests
├── snapchain/            # Snapchain Rust node (see snapchain/README.md)
├── labels/labels/        # Spam label dataset for hub spam tooling
├── generated_abis/       # Generated contract ABIs
├── proto/                # Protocol buffer definitions
├── env.example           # Environment configuration template
└── README.md
```

## 🧰 Prerequisites

### Required
- 🦀 **Rust 1.70+** — Install via [rustup](https://rustup.rs/)
- 🧱 **Cargo** — Comes with Rust
- 🌐 **Ethereum RPC endpoint** — Get free API keys from [Alchemy](https://www.alchemy.com/) or [Infura](https://infura.io/)
- 🛰️ **Farcaster Hub endpoint** — Use Neynar's public hub (free tier available)

### Optional but Recommended
- 🛠️ **Foundry's Anvil** — For local development (`cargo install --locked foundry-cli`)
- 🗃️ **Git submodules** — For spam detection features (`git submodule update --init --recursive`)
- 📦 **Protocol Buffers compiler** — For advanced features (`brew install protobuf`)

### System Requirements
- **Memory**: 8GB+ RAM recommended
- **Storage**: 1GB+ free space
- **Network**: Stable internet connection for RPC calls

## 🚀 Installation

### 1. Clone the Repository
```bash
git clone https://github.com/RyanKung/castorix.git
cd castorix
```

### 2. Initialize Submodules (Optional)
```bash
git submodule update --init --recursive  # required for spam detection features
```

### 3. Configure Environment
```bash
cp env.example .env                      # copy configuration template
# Edit .env with your API keys and endpoints
```

### 4. Build the Project
```bash
cargo build                              # build the CLI and library
```

### 5. Install Globally (Optional)
```bash
cargo install --path .                   # install castorix command globally
```

### Development vs Production
- **Development**: Use `cargo run -- <subcommand>` to run commands
- **Production**: After global install, use `castorix <subcommand>` directly

### Verify Installation
```bash
cargo run -- --help                      # or castorix --help if installed globally
```

## ⚙️ Configuration

### Environment Variables
Copy `env.example` to `.env` and customize the values:

```bash
cp env.example .env
# Edit .env with your configuration
```

### Required Configuration

#### Ethereum RPC Endpoints
```bash
# Main Ethereum network (required for ENS operations)
ETH_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your-api-key

# Base chain (required for .base.eth domains)
ETH_BASE_RPC_URL=https://mainnet.base.org

# Optimism (required for Farcaster contracts)
ETH_OP_RPC_URL=https://mainnet.optimism.io
```

#### Farcaster Hub
```bash
# Farcaster Hub endpoint (default: Neynar public hub)
FARCASTER_HUB_URL=https://hub-api.neynar.com
```

### Key Management Options

Castorix supports two key management modes:

#### 1. Encrypted Key Management (Recommended)
```bash
# Generate a new encrypted key
castorix key generate-encrypted my-wallet "My Main Wallet"

# Load the key for operations
castorix key load my-wallet
```

#### 2. Legacy Mode (Not Recommended)
```bash
# Set private key in environment (insecure)
PRIVATE_KEY=0x1234567890abcdef...
```

### Storage Locations
- **Encrypted keys**: `~/.castorix/keys/`
- **Custody wallets**: `~/.castorix/custody/`
- **Ed25519 signers**: `~/.castorix/ed25519/`
- **Configuration**: `~/.castorix/config/`

## 🧭 CLI Quick Tour

> **Note**: During development, prefix commands with `cargo run --`. After global installation, use `castorix` directly.

### Getting Help
```bash
castorix --help                    # Show all available commands
castorix key --help               # Show key management commands
castorix ens --help               # Show ENS-related commands
castorix hub --help               # Show Farcaster Hub commands
```

### 🔑 Key Management (ECDSA Wallets)

#### Secure Encrypted Keys (Recommended)
```bash
# Generate a new encrypted wallet
castorix key generate-encrypted my-wallet "My Main Wallet"

# Import an existing private key securely
castorix key import backup-key "Backup Wallet" 0x1234...

# List all your encrypted keys
castorix key list

# Load a key for operations
castorix key load my-wallet

# Get information about the loaded key
castorix key info
```

#### Key Operations
```bash
# Sign a message with the loaded key
castorix key sign "Hello, Farcaster!"

# Verify a signature
castorix key verify <signature> "Hello, Farcaster!"

# Rename a key
castorix key rename old-name new-name

# Update key alias/description
castorix key update-alias my-wallet "Updated Description"

# Delete a key (permanent!)
castorix key delete my-wallet
```

#### Legacy Mode (Not Recommended)
```bash
# Generate plain-text key (insecure)
castorix key generate
```

### 🛡️ Custody Key Management (FID Specific)

Custody wallets are used for managing Farcaster identities and signer registration.

```bash
# List all stored custody wallets
castorix custody list

# Import custody private key for a specific FID
castorix custody import 12345

# Derive custody key from recovery mnemonic
castorix custody from-mnemonic 12345

# Delete custody wallet for a FID
castorix custody delete 12345
```

> **Storage**: Custody wallets are encrypted and stored in `~/.castorix/custody/`

### 🌐 ENS & Basenames

#### Domain Resolution & Verification
```bash
# Resolve an ENS domain to address
castorix ens resolve vitalik.eth

# Get all ENS domains for an address
castorix ens domains 0x1234567890abcdef...

# Get all domains (comprehensive lookup)
castorix ens all-domains 0x1234567890abcdef...

# Verify domain ownership
castorix ens verify mydomain.eth
```

#### Base Chain Domains
```bash
# Get Base subdomains for an address
castorix ens base-subdomains 0x1234567890abcdef...

# Check specific Base subdomain
castorix ens check-base-subdomain name.base.eth

# Query Base contract directly
castorix ens query-base-contract name.base.eth
```

#### Username Proof Creation
```bash
# Create a username proof for Farcaster
castorix ens create mydomain.eth 12345 --wallet-name my-wallet

# Verify a proof file
castorix ens verify-proof ./proof_mydomain_eth_12345.json
```

> **Output**: Proof files are saved as `proof_<domain>_<fid>.json`

### 📡 Farcaster Hub Integration

#### User Data & Profiles
```bash
# Get basic user information
castorix hub user 12345

# Get detailed user profile
castorix hub profile 12345

# Get profile with all metadata
castorix hub profile 12345 --all
```

#### Social Graph
```bash
# Get user's followers
castorix hub followers 12345

# Get followers with limit
castorix hub followers 12345 --limit 50

# Get users that this FID follows
castorix hub following 12345
```

#### Address & Domain Information
```bash
# Get Ethereum addresses for a FID
castorix hub eth-addresses 12345

# Get ENS domains for a FID
castorix hub ens-domains 12345

# Get custody address for a FID
castorix hub custody-address 12345
```

#### Hub Status & Statistics
```bash
# Get hub information
castorix hub info

# Get storage statistics for a FID
castorix hub stats 12345
```

#### Spam Detection
```bash
# Check if FID is marked as spam
castorix hub spam 12345

# Get spam statistics
castorix hub spam-stat
```

#### User Content (Casts)
```bash
# Get recent casts by FID
castorix hub casts 12345

# Get specific number of casts
castorix hub casts 12345 --limit 10

# Get all casts (may take time)
castorix hub casts 12345 --limit 0

# View full JSON structure
castorix hub casts 12345 --limit 5 --json
```

**Displays:**
- ⏰ Timestamp (formatted UTC)
- 🔗 Cast hash (unique ID)
- 🔑 Signer (Ed25519 public key)
- 📝 Text content
- 📎 Number of embeds
- 👥 Number of mentions

#### Proof Submission
```bash
# Submit username proof to hub
castorix hub submit-proof ./proof.json 12345 --wallet-name my-wallet

# Submit EIP-712 signed proof
castorix hub submit-proof-eip712 ./proof.json --wallet-name my-wallet
```

> **Note**: `hub cast` and `hub verify-eth` commands are currently under development.

### ✍️ Signer Management (Ed25519)

Ed25519 signers are used for signing Farcaster messages and content.

#### Signer Information
```bash
# List all signers for a FID
castorix signers list

# Get detailed signer information
castorix signers info 12345
```

#### Signer Registration
```bash
# Register a new signer (with dry-run preview)
castorix signers register 12345 --wallet my-custody --payment-wallet my-key --dry-run

# Register signer (live transaction)
castorix signers register 12345 --wallet my-custody --payment-wallet my-key

# Unregister a signer
castorix signers unregister 12345 --wallet my-custody --payment-wallet my-key --dry-run
```

#### Signer Management
```bash
# Export signer by index or public key
castorix signers export 0
castorix signers export 0x1234...

# Delete signer
castorix signers delete 0
castorix signers delete 0x1234...
```

> **Dry Run**: Use `--dry-run` to preview transactions without executing them. Generated signers are encrypted and stored in `~/.castorix/ed25519/`.

### 🌐 REST API Server (HTTP Integration)

Castorix includes a traditional RESTful HTTP API server for web and application integrations.

#### Starting the API Server
```bash
# Start on default port (3000)
castorix api serve

# Start on custom port
castorix api serve --port 8080

# Start on specific host
castorix api serve --host 127.0.0.1 --port 3000
```

#### Available Endpoints

**Health Check:**
- `GET /health` - Server status

**Hub Endpoints:**
- `GET /api/hub/users/:fid` - Get user information ✅
- `GET /api/hub/users/:fid/profile` - Get detailed profile
- `GET /api/hub/users/:fid/stats` - Get user statistics
- `GET /api/hub/users/:fid/followers` - Get followers list
- `GET /api/hub/users/:fid/following` - Get following list
- `GET /api/hub/users/:fid/addresses` - Get ETH addresses
- `GET /api/hub/users/:fid/ens` - Get ENS domains
- `GET /api/hub/users/:fid/custody` - Get custody address
- `GET /api/hub/users/:fid/casts` - Get user casts
- `GET /api/hub/spam/:fid` - Check spam status ✅
- `GET /api/hub/info` - Get hub information

**ENS Endpoints** (requires ETH_RPC_URL):
- `GET /api/ens/resolve/:domain` - Resolve ENS domain
- `GET /api/ens/verify/:domain/:address` - Verify ownership

**Contract Endpoints** (requires ETH_OP_RPC_URL):
- `GET /api/contract/fid/price` - Get FID registration price ✅
- `GET /api/contract/storage/price/:units` - Get storage price ✅
- `GET /api/contract/address/:address/fid` - Check address FID ✅

#### Example Usage

```bash
# Check server health
curl http://localhost:3000/health

# Get user information
curl http://localhost:3000/api/hub/users/3

# Check spam status
curl http://localhost:3000/api/hub/spam/12345

# Get FID registration price
curl http://localhost:3000/api/contract/fid/price
```

> **Documentation**: See [API_SERVER.md](API_SERVER.md) for complete API documentation with examples in JavaScript and Python.

### 🤖 MCP Server (AI Assistant Integration)

Castorix includes a Model Context Protocol server that exposes Farcaster query capabilities to AI assistants.

#### Starting the MCP Server
```bash
# Start MCP server (communicates via stdio)
castorix mcp serve
```

#### Available Tools (22 total)

**Hub Queries (12)**
- `hub_get_user` - Get user information by FID
- `hub_get_profile` - Get detailed profile
- `hub_get_stats` - Get user statistics
- `hub_get_followers` - Get followers list
- `hub_get_following` - Get following list
- `hub_get_eth_addresses` - Get verified Ethereum addresses
- `hub_get_custody_address` - Get custody address
- `hub_get_info` - Get Hub sync status
- `hub_get_ens_domains` - Get verified ENS domains
- `hub_check_spam` - Check spam status
- `hub_get_spam_stats` - Get spam statistics
- `hub_get_casts` - Get user posts/casts

**ENS Tools (3)**
- `ens_resolve_domain` - Resolve ENS domain to address
- `ens_check_base_subdomain` - Check Base subdomain
- `ens_verify_ownership` - Verify domain ownership

**Contract Queries (4)**
- `fid_get_price` - Get FID registration cost
- `storage_get_price` - Get storage rental price
- `fid_check_address` - Check if address has FID
- `storage_check_units` - Check storage units

**Signer & Custody (3)**
- `signers_list_local` - List local Ed25519 keys
- `signers_get_info` - Get signer info
- `custody_list_local` - List custody keys

#### Claude Desktop Integration

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "castorix": {
      "command": "/path/to/castorix",
      "args": ["mcp", "serve"],
      "env": {
        "FARCASTER_HUB_URL": "https://hub-api.neynar.com",
        "ETH_RPC_URL": "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY",
        "ETH_BASE_RPC_URL": "https://mainnet.base.org",
        "ETH_OP_RPC_URL": "https://mainnet.optimism.io"
      }
    }
  }
}
```

#### Example Queries (in Claude)
- "Tell me about FID 3"
- "Show me @dwr's latest 10 casts"
- "What Ethereum addresses does FID 3 have?"
- "Resolve vitalik.eth"
- "Is FID 12345 spam?"
- "How much does FID registration cost?"

> **Note**: The MCP server communicates via JSON-RPC 2.0 over stdio and is compatible with any MCP-compatible AI assistant.

### 🧪 Development Helpers

#### Local Development Environment
```bash
# Start local Anvil node (Optimism fork)
cargo start-node

# Stop local Anvil node
cargo stop-node

# Check node status
curl http://127.0.0.1:8545
```

#### Running Examples
```bash
# Run basic key management example
cargo run --example basic_key_management

# Run ENS proof creation example
cargo run --example ens_proof_creation
```

## ✅ Running Tests

Most integration tests require a local Optimism fork and specific environment setup.

### Prerequisites for Testing
```bash
# Install Foundry (required for Anvil)
cargo install --locked foundry-cli

# Initialize git submodules for spam detection tests
git submodule update --init --recursive
```

### Running Tests
```bash
# Start local Anvil node (Optimism fork)
cargo start-node

# Run tests with test environment flag
RUNNING_TESTS=1 cargo test

# Stop the local node
cargo stop-node
```

### Test Categories
- **Unit tests**: `cargo test` (no external dependencies)
- **Integration tests**: `RUNNING_TESTS=1 cargo test` (requires Anvil)
- **External API tests**: May fail without proper API keys

> **Note**: Some tests require external RPC endpoints and may be skipped if prerequisites aren't available.

## 🪐 Snapchain Integration

The `snapchain/` directory contains a complete Rust implementation of Farcaster's Snapchain data layer.

### What is Snapchain?
Snapchain is a decentralized data storage layer for the Farcaster protocol, providing:
- **High Throughput**: 10,000+ transactions per second
- **Data Availability**: Real-time access to user data
- **Canonical Implementation**: Reference implementation for the protocol

### Running a Snapchain Node
```bash
# Navigate to snapchain directory
cd snapchain

# Follow the setup guide
cat README.md

# Start a node (requires Docker)
./snapchain.sh start
```

> **Note**: The Castorix CLI doesn't require Snapchain unless you're running your own node or contributing to the protocol implementation.

## 🔧 Troubleshooting

### Common Issues

#### "No private key found in environment variables"
```bash
# Solution 1: Use encrypted key management (recommended)
castorix key generate-encrypted my-wallet "My Wallet"
castorix key load my-wallet

# Solution 2: Set PRIVATE_KEY environment variable (legacy)
export PRIVATE_KEY=0x1234...
```

#### "Failed to connect to RPC endpoint"
```bash
# Check your .env file
cat .env | grep ETH_RPC_URL

# Verify API key is valid
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  $ETH_RPC_URL
```

#### "Permission denied" errors
```bash
# Ensure proper permissions for castorix directory
chmod 700 ~/.castorix/
chmod 600 ~/.castorix/keys/*
```

#### ENS domain not resolving
```bash
# Check if domain exists
castorix ens resolve your-domain.eth

# Verify you own the domain
castorix ens verify your-domain.eth
```

#### Tests failing
```bash
# Ensure Anvil is running
cargo start-node

# Check if ports are available
lsof -i :8545

# Run with verbose output
RUST_LOG=debug RUNNING_TESTS=1 cargo test
```

### Getting Help
- 📖 Check the [examples](examples/) directory for usage patterns
- 🐛 Report issues on [GitHub Issues](https://github.com/RyanKung/castorix/issues)
- 💬 Join discussions in [GitHub Discussions](https://github.com/RyanKung/castorix/discussions)

## 🛣️ Known Limitations & Roadmap

### Current Limitations
- 📝 **Hub Casting**: `castorix hub cast` and `hub verify-eth` commands are under development
- 🔑 **Proof Submission**: Username proof submission requires hub-side Ed25519 signer support
- 🗃️ **Spam Detection**: Requires `git submodule update --init --recursive` for `labels/labels/spam.jsonl`
- ⛽ **Gas Costs**: Many operations interact with mainnet contracts — monitor gas costs
- 🌐 **Rate Limits**: Respect RPC provider rate limits for production usage

### Upcoming Features
- 🚀 **Enhanced Hub Integration**: Full protobuf support for casting and verification
- 🔐 **Hardware Wallet Support**: Integration with Ledger and other hardware wallets
- 📱 **Mobile Support**: CLI optimizations for mobile development workflows
- 🎯 **Advanced Proof Types**: Support for additional proof formats and verification methods
- 📊 **Analytics Dashboard**: Built-in analytics for Farcaster data insights

### Contributing
We welcome contributions! Please see [contracts/CONTRIBUTING.md](contracts/CONTRIBUTING.md) for guidelines.

## 🤝 Contributing

We welcome contributions from developers of all skill levels!

### Getting Started
1. **Read the guidelines**: Start with [contracts/CONTRIBUTING.md](contracts/CONTRIBUTING.md)
2. **Open an issue**: Discuss large changes before implementing
3. **Fork and branch**: Create a feature branch from `main`
4. **Test thoroughly**: Ensure all tests pass
5. **Submit PR**: Include a clear description of changes

### Areas for Contribution
- 🔧 **Bug fixes**: Report and fix issues
- 📚 **Documentation**: Improve guides and examples
- 🚀 **Features**: Add new functionality
- 🧪 **Tests**: Improve test coverage
- 🎨 **UI/UX**: Enhance CLI experience

### Code Style
- Follow Rust conventions and `cargo fmt`
- Add documentation for public APIs
- Include tests for new features
- Update README for significant changes

## 📄 License
Castorix ships under the GPL-2.0 License. See [LICENSE](LICENSE) for the legalese.
