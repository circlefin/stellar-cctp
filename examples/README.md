# CCTP on Stellar — Example Scripts

This directory provides standalone example scripts for transferring USDC between EVM chains and Stellar using [Cross-Chain Transfer Protocol (CCTP)](https://developers.circle.com/crosschain-transfers).

## How It Works

1. **Burn**: USDC is burned on the source chain
2. **Attest**: Circle's attestation service signs the burn event
3. **Mint**: Native USDC is minted on the destination chain

### EVM → Stellar (via CCTP Forwarder)

When transferring from an EVM chain to Stellar, the script uses the **CCTP Forwarder** pattern:

- `depositForBurnWithHook` is called on the EVM `TokenMessengerV2`
- Both `mintRecipient` and `destinationCaller` must be set to the Stellar **CCTP Forwarder** contract address. Usage of any other address for `mintRecipient` or `destinationCaller` can lead to stuck funds
- `hookData` encodes the final Stellar recipient address (G…, C…, or M… strkey)
- Circle attests the burn
- `mint_and_forward` is called on the Stellar CCTP Forwarder contract, which calls `receive_message` on `MessageTransmitter`, mints tokens, and forwards them to the recipient encoded in the hook data

### Stellar → EVM

For Stellar to EVM transfers, the standard flow is used:

- `deposit_for_burn` is called on the Stellar `TokenMessengerMinterV2`
- Circle attests the burn
- `receiveMessage` is called on the EVM `MessageTransmitterV2`

---

## Getting Started

### Prerequisites

- **Node.js** v22+
- A funded **Stellar testnet** account (with XLM and USDC)
- A funded **EVM testnet** wallet (with ETH/gas and USDC)
- CCTP contracts deployed on both chains

### 1. Install Dependencies

```bash
npm install
```

### 2. Configure Environment

```bash
cp .env.example .env
```

Edit `.env` with your credentials and contract addresses:

```bash
# Your Stellar wallet
STELLAR_SECRET_KEY=S...

# Your EVM wallet
EVM_PRIVATE_KEY=0x...
```

### 3. Get Test Tokens

Get testnet tokens from the [Circle Faucet](https://faucet.circle.com/).

---

## Usage

### Stellar → EVM

**Fast Transfer** (lower finality threshold, higher fee):

```bash
npm run bridge stellar2evm -- --amount 10000000 --fastBurn true
```

**Standard Transfer**:

```bash
npm run bridge stellar2evm -- --amount 10000000
```

> **Note:** Amounts are in Stellar USDC subunits (7 decimals). `10000000` = 1 USDC.

### EVM → Stellar (via CCTP Forwarder)

Requires `--recipient` with the Stellar destination address (G…, C…, or M… strkey).

> **Important:** G-address and M-address recipients must have an established [USDC trustline](https://developers.circle.com/stablecoins/quickstart-setup-usdc-trustline-stellar) before receiving funds. Transfers to accounts without a USDC trustline will fail.

**Fast Transfer**:

```bash
npm run bridge evm2stellar -- --amount 1000000 --recipient G... --fastBurn true
```

**Standard Transfer**:

```bash
npm run bridge evm2stellar -- --amount 1000000 --recipient G...
```

> **Note:** Amounts are in EVM USDC subunits (6 decimals). `1000000` = 1 USDC.

---

## File Overview

| File               | Description                                                                                        |
| ------------------ | -------------------------------------------------------------------------------------------------- |
| `main.ts`          | CLI entry point — parses args, orchestrates the burn→attest→mint flow                              |
| `evm.ts`           | EVM chain interactions (approve, depositForBurn, receiveMessage) using viem                        |
| `stellar.ts`       | Stellar chain interactions (approve, deposit_for_burn, receive_message) using @stellar/stellar-sdk |
| `stellar-utils.ts` | Stellar address encoding utilities and CCTP Forwarder hook data construction                       |
| `config.ts`        | Environment variable parsing and validation                                                        |
| `.env.example`     | Template for required configuration                                                                |

## Domain IDs

For supported chains and domain IDs, see [CCTP Supported Chains and Domains](https://developers.circle.com/cctp/concepts/supported-chains-and-domains#supported-blockchains-and-domains).
