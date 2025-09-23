# 🔐 Castorix — Farcaster Power Toolkit

<div align="center">
  <img src="logo.png" alt="Castorix Logo" width="200" height="200">
</div>

[![License: GPL-2.0](https://img.shields.io/badge/License-GPL--2.0-blue.svg)](https://opensource.org/licenses/GPL-2.0)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Farcaster](https://img.shields.io/badge/Farcaster-Protocol-purple.svg)](https://farcaster.xyz)
[![Snapchain](https://img.shields.io/badge/Snapchain-Ready-green.svg)](https://github.com/farcasterxyz/snapchain)

Castorix is a Rust command-line interface and library for Farcaster builders. It keeps your custody wallets encrypted, generates Basenames/ENS username proofs, registers Ed25519 signers, pulls Hub data, and stays in sync with Snapchain — all from one toolchain.

## 🌟 Feature Highlights
- 🔐 **Encrypted key vault** — interactive flows keep ECDSA custody wallets under `~/.castorix/keys`
- 🏷️ **Basename & ENS proofs** — resolve domains, audit Base subdomains, and mint Farcaster-ready username proofs
- 📡 **Hub power tools** — fetch user graphs, storage stats, custody addresses, and push proof submissions
- ✍️ **Signer management** — generate Ed25519 keys, register/unregister with dry-run previews, and export safely
- 🚨 **Spam intelligence** — optional labels from the `merkle-team/labels` dataset bundled as a submodule
- 🧩 **All-in-one workspace** — Farcaster contract bindings, helper binaries, and a Snapchain node live in the repo

## 🗂️ Repository Layout
```
.
├── src/                  # CLI entry points, Farcaster client, key managers
├── tests/                # Integration tests (many expect a local Anvil node)
├── examples/             # Example binaries and demos
├── contracts/            # Solidity contracts, scripts, Foundry config
├── snapchain/            # Snapchain Rust node (see snapchain/README.md)
├── labels/labels/        # Spam label dataset for hub spam tooling
└── README.md
```

## 🧰 Prerequisites
- 🦀 Rust 1.70 or newer (`rustup` makes this painless)
- 🧱 `cargo` and `git`
- 🌐 An Ethereum JSON-RPC endpoint (`ETH_RPC_URL`) for ENS lookups
- 🛰️ A Farcaster Hub endpoint (`FARCASTER_HUB_URL`), e.g. Neynar's public hub
- 🛠️ Optional: Foundry's `anvil` for local Optimism forks (`cargo install --locked foundry-cli`)
- 🗃️ Optional: `git submodule update --init --recursive` to pull spam labels for `castorix hub spam*`

## 🚀 Installation
```bash
git clone https://github.com/RyanKung/castorix.git
cd castorix
git submodule update --init --recursive  # required for spam tooling

cp env.example .env                      # customize to match your environment
cargo build                              # build the CLI and library

# Optional: install a global binary
cargo install --path .
```

During development call commands with `cargo run -- <subcommand>`. After installing globally, just run `castorix <subcommand>`.

## ⚙️ Configuration
`env.example` lists the knobs Castorix understands. Common ones:

- `ETH_RPC_URL` — mainnet RPC for ENS queries
- `ETH_BASE_RPC_URL` — Base RPC for `.base.eth` lookups
- `ETH_OP_RPC_URL` — Optimism RPC when touching on-chain Farcaster contracts
- `FARCASTER_HUB_URL` — Hub REST endpoint

Copy `env.example` to `.env` so `dotenv` can load values automatically. Signing commands either need:

1. an encrypted key loaded via `castorix key load <name>`, or
2. a `PRIVATE_KEY` environment variable for legacy mode.

Encrypted ECDSA keys, custody wallets, and Ed25519 signers live under `~/.castorix/`.

## 🧭 CLI Quick Tour
Prefix examples with `cargo run --` while developing. They assume the binary name is `castorix` once installed.

### 🔑 Key management (ECDSA wallets)
- `castorix key generate-encrypted` — interactive flow, stores a new wallet under `~/.castorix/keys`
- `castorix key import` — encrypt an existing hex key without leaking it to shell history
- `castorix key list` — show aliases, addresses, and creation dates
- `castorix key load <name>` — decrypt into the current session
- `castorix key info` — inspect the loaded wallet
- `castorix key sign "Message"` / `verify` — quick signature helpers
- `castorix key rename` / `update-alias` / `delete`
- `castorix key generate` — legacy plain-text key generator (use carefully)

### 🛡️ Custody key management (FID specific)
- `castorix custody list`
- `castorix custody import <fid>` — store the custody private key encrypted per FID
- `castorix custody from-mnemonic <fid>` — derive from a recovery phrase
- `castorix custody delete <fid>` — remove the encrypted file

Custody wallets live in `~/.castorix/custody/` and power signer registration workflows.

### 🌐 ENS & Basenames
- `castorix ens resolve vitalik.eth`
- `castorix ens domains 0x...` / `all-domains`
- `castorix ens base-subdomains 0x...` — best-effort Base reverse lookups
- `castorix ens check-base-subdomain name.base.eth`
- `castorix ens query-base-contract name.base.eth`
- `castorix ens verify mydomain.eth`
- `castorix ens create mydomain.eth 12345 --wallet-name <key>` — writes `proof_<domain>_<fid>.json`
- `castorix ens verify-proof ./proof.json`

### 📡 Farcaster Hub tooling
- `castorix hub user <fid>` / `profile <fid> [--all]`
- `castorix hub followers <fid> [--limit N]` / `following`
- `castorix hub eth-addresses <fid>` / `ens-domains <fid>` / `custody-address <fid>`
- `castorix hub info` / `stats <fid>`
- `castorix hub spam <fid> [more]` / `spam-stat`
- `castorix hub submit-proof ./proof.json <fid> [--wallet-name <key>]`
- `castorix hub submit-proof-eip712 ./proof.json --wallet-name <key>`

`hub cast` and `hub verify-eth` currently emit “not implemented” messages while the protobuf workflow is rebuilt.

### ✍️ Signer management (Ed25519)
- `castorix signers list`
- `castorix signers info <fid>`
- `castorix signers register <fid> [--wallet <custody>] [--payment-wallet <key>] [--dry-run]`
- `castorix signers unregister <fid> [--wallet <custody>] [--payment-wallet <key>] [--dry-run]`
- `castorix signers export <index|pubkey>`
- `castorix signers delete <index|pubkey>`

`--dry-run` previews the Key Gateway transaction and still stores the generated signer encrypted under `~/.castorix/ed25519/`.

### 🧪 Miscellaneous helpers
- `cargo start-node` / `cargo stop-node` — spin up or tear down an Optimism-forking Anvil instance

## ✅ Running Tests

### Unit Tests
Unit tests don't require external dependencies and can be run directly:

```bash
cargo test --lib                     # Run library unit tests only
cargo test --bin castorix            # Run binary unit tests only
```

### Integration Tests
**Important**: Integration tests require a local Anvil node running on `http://127.0.0.1:8545`. You must start the node before running integration tests:

```bash
# Start local Anvil node (required for integration tests)
cargo start-node                     # launches an Anvil fork (requires foundry)

# Run all tests (unit + integration)
RUNNING_TESTS=1 cargo test

# Or run only integration tests
RUNNING_TESTS=1 cargo test --test farcaster_integration_test

# Stop the node when done
cargo stop-node
```

### Test Types
- **Unit tests** (`cargo test --lib`): Test individual modules and functions
- **Integration tests** (`cargo test --test *`): Test end-to-end workflows with blockchain interactions
- **Binary tests** (`cargo test --bin castorix`): Test CLI functionality

Some integration tests lean on external RPCs or datasets; skip them if prerequisites aren't ready.

## 🪐 Snapchain crate
The `snapchain/` directory contains a Rust implementation of the Snapchain data layer. Check `snapchain/README.md` for build docs. Castorix CLI doesn’t require it unless you’re hacking on the node itself.

## 🛣️ Known Limitations & Roadmap
- 📝 `castorix hub cast` and `hub verify-eth` are placeholders until the protobuf migration lands
- 🔑 Username proof submission requires hub-side Ed25519 signer support
- 🗃️ Spam tooling expects `labels/labels/spam.jsonl` — run `git submodule update --init --recursive`
- ⛽ Many commands touch mainnet services — mind gas costs and RPC rate limits

## 🤝 Contributing
We love patches! Start with [contracts/CONTRIBUTING.md](contracts/CONTRIBUTING.md) and open an issue or discussion before large changes.

## 📄 License
Castorix ships under the GPL-2.0 License. See [LICENSE](LICENSE) for the legalese.
