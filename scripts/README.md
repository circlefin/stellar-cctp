# Stellar CCTP Scripts

TypeScript CLI tools for building, deploying, and operating the Stellar CCTP contracts (MessageTransmitter, TokenMessengerMinter, CctpForwarder) and their stablecoin dependencies (FiatTokenAdmin, USDC, AllowAsset).

## Prerequisites

- Node.js >= 20
- Yarn
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) (for generating contract bindings)
- Rust + `wasm32v1-none` target (for building contracts)

## Setup

```bash
yarn install
```

### Build contracts and generate bindings

```bash
yarn build:contracts
yarn generate:bindings
```

## Configuration

All CLI scripts load configuration from a `.env` file via `--env-file=.env`. Copy the template and fill in the values for your target network:

```bash
cp .env.example .env
```

For multiple networks, create separate env files (e.g., `.env.local`, `.env.testnet`) and point scripts at the appropriate one.

See `.env.example` for all available variables. Key concepts:

**Asset code lookup** — asset issuer keys are stored as `EOA_{CODE}_ISSUER_SECRET` / `EOA_{CODE}_ISSUER_PUBLIC` env vars, and contract IDs as `ASSET_{CODE}_CONTRACT_ID`. CLI scripts resolve these automatically from the env file.

**EOA aliases** — named accounts with keypairs stored as `EOA_{ALIAS}_SECRET` and `EOA_{ALIAS}_PUBLIC` in the env file. Scripts resolve accounts by alias (e.g., `requireEoaKeypair("cctp_deployer")` reads `EOA_CCTP_DEPLOYER_SECRET`).

**Account funding** — new accounts need XLM to exist on the Stellar network. The scripts support two funding methods, resolved automatically from the env file:

| Method | Env var | Description |
|---|---|---|
| **Funder EOA** | `EOA_FUNDER_SECRET` | Funds accounts via direct `createAccount` transfer from a pre-funded keypair. Used on networks without Friendbot (e.g., private networks). |
| **Friendbot** | `FRIENDBOT_URL` | Funds accounts via the Friendbot HTTP endpoint. Available on public testnet and local Quickstart nodes. |

When both are set, **funder takes precedence** over friendbot.

`FUNDER_BALANCE_THRESHOLD` (default: `1`) controls the minimum XLM balance for newly created accounts.

**Deterministic deploys** — each contract supports an optional `*_SALT` env var (e.g., `MESSAGE_TRANSMITTER_SALT`). When set to a 64-character hex string, the deploy uses that salt for `createCustomContract`, producing a deterministic contract address. When empty, a random salt is generated.

## CLI Scripts

### Local Environment Setup

Starts a local Stellar Quickstart node, generates all role accounts, deploys all contracts (MessageTransmitter, TokenMessengerMinter, CctpForwarder, FiatTokenAdmin, USDC, AllowAsset), configures them, and writes everything to `.env.local`. On subsequent runs with an existing env file, reuses accounts and updates URLs.

```bash
yarn setup:local-env
yarn setup:local-env --from-env .env.test-blank
yarn setup:local-env --mint-asset-code EURC --allow-asset-code EURCALLOW
```

| Parameter | Required | Description |
|---|---|---|
| `--from-env` | No | Path to an env file to read from and write to. Created if it doesn't exist (default: `.env.local`) |
| `--mint-asset-code` | No | Asset code for the mint asset (default: `USDC`) |
| `--allow-asset-code` | No | Asset code for the allow asset (default: `USDCAllCCTP`) |
| `--debug` | No | Enable debug output (default: `false`) |

## Submodules

This project uses the `stablecoin-xlm` repository as a git submodule to share the `fiat-token-admin` contract and related packages.

### Updating Submodules

```bash
# Update to latest commit
git submodule update --remote stablecoin-xlm
```

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

## Project Structure

```
scripts/
├── cli/                                    # CLI entry points
│   └── setup/
│       └── local-env.ts                    # Automated local environment setup
├── deploy/                                 # Core deployment logic
│   ├── deploy-message-transmitter-v2.ts
│   ├── deploy-token-messenger-minter-v2.ts
│   └── deploy-cctp-forwarder.ts
├── setup/                                  # Core setup logic
│   └── link-remote-resources.ts            # Batch link remote messengers + token pairs
├── sac/                                    # Stellar Asset Contract utilities
│   └── contract.ts
├── clients/                                # Generated contract bindings (do not edit)
│   ├── message-transmitter-v2/
│   ├── token-messenger-minter-v2/
│   └── cctp-forwarder/
├── stablecoin-xlm/                         # Git submodule
├── .env.example                            # Environment variable template (local dev defaults)
└── package.json
```

Stablecoin scripts (deploy FiatTokenAdmin, deploy SACs, configure minter, set SAC admin, create EOA, create trustline) are provided by the `stablecoin-common` dependency — see [stablecoin-xlm/soroban/scripts/README.md](../stablecoin-xlm/soroban/scripts/README.md) for details.
