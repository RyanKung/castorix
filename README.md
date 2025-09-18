# Castorix

Castorix is a Rust command-line interface and library for interacting with the Farcaster protocol. It helps you manage encrypted Ethereum custody keys, produce ENS username proofs, inspect Hub data, and keep track of Ed25519 signer keys from a single toolchain.

## Feature Highlights
- Encrypted Ethereum key storage with interactive CLI flows that keep secrets under `~/.castorix/keys`
- ENS tooling to resolve domains, inspect Base subdomains, and generate Farcaster-compatible username proofs
- Farcaster Hub client for querying users, followers, storage stats, spam labels, and submitting username proofs
- Custody and Ed25519 signer key workflows, including dry-run support before talking to Key Gateway contracts
- Optional spam label inspection using the `merkle-team/labels` dataset tracked via the `labels/` submodule
- Additional crates for contract bindings, helper binaries, and a Snapchain reference implementation bundled with the repository

## Repository Layout
```
.
├── src/                  # CLI entry points, Farcaster client, key managers
├── tests/                # Integration tests (many expect a local Anvil node)
├── examples/             # Example binaries
├── contracts/            # Farcaster Solidity contracts & tooling
├── snapchain/            # Snapchain Rust node (see snapchain/README.md)
├── labels/labels/        # Spam label dataset used by hub spam commands
└── README.md
```

## Prerequisites
- Rust 1.70 or newer (`rustup` recommended)
- `cargo` and `git`
- An Ethereum JSON-RPC endpoint (`ETH_RPC_URL`) for ENS lookups
- A Farcaster Hub endpoint (`FARCASTER_HUB_URL`), e.g. Neynar's public hub
- Optional: Foundry's `anvil` for local Optimism forks (`cargo install --locked foundry-cli`)
- Optional: `git submodule update --init --recursive` to download the spam labels used by `castorix hub spam*`

## Installation
```bash
git clone https://github.com/RyanKung/castorix.git
cd castorix
git submodule update --init --recursive  # required for spam tooling

cp env.example .env                      # customise to match your environment
cargo build                              # build the CLI and library

# Optional: install a global binary
cargo install --path .
```

During development you can invoke commands with `cargo run -- <subcommand>`. After installing globally, use `castorix <subcommand>`.

## Configuration
`env.example` lists the environment variables the CLI understands. The most commonly used ones are:

- `ETH_RPC_URL` – mainnet RPC used for ENS queries
- `ETH_BASE_RPC_URL` – Base RPC for `.base.eth` lookups
- `ETH_OP_RPC_URL` – Optimism RPC when talking to Farcaster contracts
- `FARCASTER_HUB_URL` – Hub REST API endpoint

Copy `env.example` to `.env` and adjust values so `dotenv` can load them automatically. Commands that sign messages need an ECDSA key. You can either:

1. Load an encrypted key (`castorix key load <name>`) before running signing commands, or
2. Set a `PRIVATE_KEY` environment variable for legacy mode.

Encrypted ECDSA keys, custody wallets, and Ed25519 signer keys are stored beneath `~/.castorix/`.

## CLI Quick Tour
While developing, prefix commands with `cargo run --`. The examples below assume the binary is installed as `castorix`.

### Key management (ECDSA wallets)
- `castorix key generate-encrypted` — interactive; creates a new key stored under `~/.castorix/keys`
- `castorix key import` — interactive; encrypts an existing hex private key
- `castorix key list` — shows stored keys with aliases and creation dates
- `castorix key load <name>` — decrypts a key into the current session
- `castorix key info` — prints details about the loaded wallet
- `castorix key sign "Message"` — signs a message with the loaded wallet
- `castorix key verify "Message" <signature>` — verifies a signature against the loaded wallet
- `castorix key rename <old> <new>` / `castorix key update-alias <name> "Alias"`
- `castorix key delete <name>` — securely removes an encrypted key file
- `castorix key generate` — legacy helper that prints the raw private key (use with caution)

### Custody key management (FID specific wallets)
- `castorix custody list`
- `castorix custody import <fid>` — prompts for a private key and stores it encrypted per FID
- `castorix custody from-mnemonic <fid>` — derives a custody wallet from a recovery phrase
- `castorix custody delete <fid>`

Custody files live in `~/.castorix/custody/` and must exist before registering signers.

### ENS utilities
- `castorix ens resolve vitalik.eth`
- `castorix ens domains 0x...` / `castorix ens all-domains 0x...`
- `castorix ens base-subdomains 0x...` (best-effort; Base reverse lookups are limited)
- `castorix ens check-base-subdomain name.base.eth`
- `castorix ens query-base-contract name.base.eth`
- `castorix ens verify mydomain.eth` — ensures the loaded wallet controls the name
- `castorix ens create mydomain.eth 12345 --wallet-name <key>` — writes `proof_<domain>_<fid>.json`
- `castorix ens verify-proof ./proof.json`

### Farcaster Hub tooling
- `castorix hub user <fid>` / `castorix hub profile <fid> [--all]`
- `castorix hub followers <fid> [--limit N]` / `castorix hub following <fid> [--limit N]`
- `castorix hub eth-addresses <fid>` / `castorix hub ens-domains <fid>` / `castorix hub custody-address <fid>`
- `castorix hub info` / `castorix hub stats <fid>`
- `castorix hub spam <fid> [more fids...]` / `castorix hub spam-stat`
- `castorix hub submit-proof ./proof.json <fid> [--wallet-name <key>]`
- `castorix hub submit-proof-eip712 ./proof.json --wallet-name <key>`

`hub cast` and `hub verify-eth` currently emit “not implemented” messages while the protobuf workflow is being rebuilt.

### Signer management (Ed25519)
- `castorix signers list`
- `castorix signers info <fid>`
- `castorix signers register <fid> [--wallet <custody>] [--payment-wallet <key>] [--dry-run]`
- `castorix signers unregister <fid> [--wallet <custody>] [--payment-wallet <key>] [--dry-run]`
- `castorix signers export <index|pubkey>`
- `castorix signers delete <index|pubkey>`

`--dry-run` previews the Key Gateway transaction without submitting it but still encrypts the generated signer under `~/.castorix/ed25519/`.

### Miscellaneous
- `cargo start-node` / `cargo stop-node` manage an Optimism-forking Anvil instance for local testing

## Running Tests
Most integration tests expect a local Optimism fork on `http://127.0.0.1:8545` and the environment variable `RUNNING_TESTS=1`. A typical workflow:

```bash
cargo start-node                     # launches an Anvil fork (requires foundry)
RUNNING_TESTS=1 cargo test
cargo stop-node
```

Some tests rely on network access or external datasets; skip them if the prerequisites are unavailable.

## Snapchain crate
The `snapchain/` directory contains a Rust implementation of the Snapchain data layer. Consult `snapchain/README.md` for build and deployment instructions. The CLI does not require this component unless you are working on the Snapchain node itself.

## Known Limitations / Roadmap
- Casting (`castorix hub cast`) and Ethereum verification submissions are placeholders until the protobuf migration is complete
- Username proof submission requires hub support for Ed25519 signer registration
- The spam tooling needs `labels/labels/spam.jsonl`; run `git submodule update --init --recursive` after cloning
- Many CLI commands interact with mainnet services—understand gas costs and rate limits before using production endpoints

## Contributing
Contributions are welcome. Start with [contracts/CONTRIBUTING.md](contracts/CONTRIBUTING.md) and open an issue or discussion before large changes.

## License
Castorix is distributed under the GPL-2.0 License. See [LICENSE](LICENSE) for details.
