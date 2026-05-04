# Stellar CCTP

Official implementation of Circle's Cross-Chain Transfer Protocol (CCTP) smart contracts for Stellar.

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

## Overview

Cross-Chain Transfer Protocol (CCTP) is a permissionless on-chain utility that facilitates the transfer of USDC between blockchain networks. This repository contains the official Stellar implementation of CCTP smart contracts, enabling secure and efficient cross-chain USDC transfers between Stellar and other supported blockchains.

### Key Features

- **Native Cross-Chain Transfers**: Burn USDC on the source chain and mint native USDC on the destination chain
- **Permissionless**: Anyone can transfer USDC cross-chain using CCTP
- **Attestation-Based Security**: Leverages Circle's attestation service for secure message verification
- **Composable**: Integrate cross-chain USDC transfers directly into your smart contracts
- **Soroban Implementation**: Built using Soroban, Stellar's native smart contract platform

## Architecture

CCTP on Stellar consists of three main contracts:

### 1. Message Transmitter V2
- Handles cross-chain message transmission and verification
- Manages attestation validation and message replay prevention
- Ensures message authenticity through signature verification
- Controls attester management and signature thresholds

### 2. Token Messenger Minter V2
- Manages USDC burning on source chains and minting on destination chains
- Controls token pairs between local and remote domains
- Enforces burn limits and transfer controls
- Handles fee collection and decimal conversion between chains
- Manages remote token messenger mappings

### 3. CCTP Forwarder
- Provides hook-based message forwarding for composable integrations
- Enables post-mint token forwarding to specified recipients
- Validates CCTP messages and burn message versions

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) - The project uses the Rust version specified in `rust-toolchain.toml`
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools) (optional, for deployment)
- [Node.js](https://nodejs.org/) (for running scripts)

The Rust toolchain, including `rustfmt` and `clippy`, will be automatically installed based on the `rust-toolchain.toml` file when you run cargo commands.

### Installation

1. **Clone the repository with submodules**

   ```bash
   git clone --recurse-submodules <repository-url>
   cd stellar-cctp
   ```

   If you already cloned without submodules:
   ```bash
   git submodule update --init --recursive
   ```

2. **Set up pre-commit hooks** (recommended)

   ```bash
   git config core.hooksPath .githooks
   ```

## Usage

### Building Contracts

```bash
# Build all contracts
cargo build --target wasm32v1-none --release
```

### Testing

```bash
# Run all unit tests
cargo test --all-features --workspace
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features -- --deny warnings
```

### Test Coverage

This project uses `cargo-llvm-cov` for measuring test coverage:

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Run coverage
cargo cov

# Generate HTML report
cargo cov-html
```

## Project Structure

```
stellar-cctp/
├── contracts/
│   ├── message-transmitter/        # Message transmission and attestation (v1)
│   ├── message-transmitter-v2/     # Message transmission and attestation (v2)
│   ├── token-messenger-minter/     # Token burning and minting (v1)
│   ├── token-messenger-minter-v2/  # Token burning and minting (v2)
│   └── cctp-forwarder/             # Hook-based message forwarding
├── packages/
│   ├── cctp-interfaces/            # Contract interfaces (MessageHandler, Receiver, Relayer)
│   ├── cctp-roles/                 # CCTP-specific roles (Attestable, TokenController, etc.)
│   ├── cctp-utils/                 # Message parsing and construction utilities
│   └── test-contracts/             # Mock contracts for testing
└── scripts/                        # Deployment and admin scripts
    ├── clients/                    # Generated TypeScript bindings
    └── stablecoin-xlm/             # Git submodule
        └── soroban/
            ├── contracts/
            │   └── fiat-token-admin/   # Fiat token admin contract
            └── packages/
                ├── common-roles/       # Shared role implementations
                ├── stablecoin-roles/   # Stablecoin-specific roles
                └── ...                 # Other shared packages
```

## Submodules

This project uses the `stablecoin-xlm` repository as a git submodule to share the `fiat-token-admin` contract and related packages.

### Updating Submodules

```bash
# Update to latest commit
git submodule update --remote stablecoin-xlm
```

## Troubleshooting

### Submodule Issues

**Problem:** Commands from the `scripts` folder fail to execute and exit with an error.

**Solution:**
```bash
git submodule update --init --recursive
```

**Problem:** Submodule directory exists but is empty

**Solution:**
```bash
git submodule sync
git submodule update --init --recursive
```

### Build Issues

**Problem:** `cargo build` fails with package version conflicts

**Solution:**
```bash
cargo clean
rm Cargo.lock
cargo build --target wasm32v1-none --release
```

## Security

Security is our top priority. If you discover a security vulnerability, please follow our [Security Policy](SECURITY.md)

## Resources

- [CCTP Documentation](https://developers.circle.com/cctp)
- [Circle Developer Hub](https://developers.circle.com/)
- [Stellar Documentation](https://developers.stellar.org/)
- [Soroban Documentation](https://soroban.stellar.org/)

## License

This project is licensed under the Apache License, Version 2.0 - see the [LICENSE](LICENSE) file for details.
