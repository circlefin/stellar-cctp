/**
 * Copyright 2026 Circle Internet Group, Inc. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import {
  Address,
  Contract,
  Keypair,
  nativeToScVal,
  rpc,
  TransactionBuilder,
  xdr,
} from "@stellar/stellar-sdk";
import { isAddress, pad } from "viem";
import {
  EVM_ADDRESS,
  EVM_DESTINATION_CALLER,
  EVM_DOMAIN,
  STELLAR_MESSAGE_TRANSMITTER_ADDRESS,
  STELLAR_NETWORK_PASSPHRASE,
  STELLAR_RPC_URL,
  STELLAR_SECRET_KEY,
  STELLAR_TOKEN_MESSENGER_MINTER_ADDRESS,
  STELLAR_USDC_ADDRESS,
} from "./config";

const TIMEOUT_SECONDS = 120;

// ---------------------------------------------------------------------------
// Soroban helpers
// ---------------------------------------------------------------------------

const getServer = (): rpc.Server => new rpc.Server(STELLAR_RPC_URL);

const getKeypair = (): Keypair => Keypair.fromSecret(STELLAR_SECRET_KEY);

/**
 * Assembles, simulates, signs, and submits a Soroban transaction.
 */
const submitSorobanTx = async (
  server: rpc.Server,
  keypair: Keypair,
  contractId: string,
  method: string,
  args: xdr.ScVal[],
): Promise<string> => {
  const account = await server.getAccount(keypair.publicKey());
  const contract = new Contract(contractId);

  const tx = new TransactionBuilder(account, {
    fee: "10000000",
    networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(TIMEOUT_SECONDS)
    .build();

  const simulated = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(simulated)) {
    throw new Error(`Simulation failed: ${JSON.stringify(simulated, null, 2)}`);
  }
  const prepared = rpc.assembleTransaction(tx, simulated).build();
  prepared.sign(keypair);

  const sendResult = await server.sendTransaction(prepared);
  if (sendResult.status === "ERROR") {
    throw new Error(`Send failed: ${JSON.stringify(sendResult)}`);
  }

  // Poll for completion
  const pollStart = Date.now();
  const TX_POLL_TIMEOUT_MS = 2 * 60 * 1000;
  let getResult = await server.getTransaction(sendResult.hash);
  while (getResult.status === "NOT_FOUND") {
    if (Date.now() - pollStart > TX_POLL_TIMEOUT_MS) {
      throw new Error(`Transaction ${sendResult.hash} not confirmed after ${TX_POLL_TIMEOUT_MS / 1000}s`);
    }
    await new Promise((r) => setTimeout(r, 2000));
    getResult = await server.getTransaction(sendResult.hash);
  }

  if (getResult.status !== "SUCCESS") {
    throw new Error(`Transaction failed: ${JSON.stringify(getResult)}`);
  }

  return sendResult.hash;
};

// ---------------------------------------------------------------------------
// Address conversion helpers
// ---------------------------------------------------------------------------

/**
 * Converts an EVM hex address (20 bytes) to a left-zero-padded 32-byte ScVal
 * (BytesN<32>) for use as a CCTP mint recipient or destination caller.
 */
const evmAddressToBytes32ScVal = (evmAddress: string): xdr.ScVal => {
  const normalized = (evmAddress.startsWith("0x") ? evmAddress : `0x${evmAddress}`) as `0x${string}`;
  if (!isAddress(normalized)) {
    throw new Error(`Invalid EVM address: ${evmAddress}`);
  }
  const padded = pad(normalized);
  return xdr.ScVal.scvBytes(Buffer.from(padded.slice(2), "hex"));
};

/**
 * Converts a hex string (with optional 0x prefix) to a Soroban Bytes ScVal.
 */
const hexToScValBytes = (hex: string): xdr.ScVal => {
  const cleaned = hex.startsWith("0x") ? hex.slice(2) : hex;
  return xdr.ScVal.scvBytes(Buffer.from(cleaned, "hex"));
};

// ---------------------------------------------------------------------------
// Stellar → EVM: deposit_for_burn on Stellar
// ---------------------------------------------------------------------------

/**
 * Approves the TokenMessengerMinter contract to spend USDC, then calls
 * deposit_for_burn to burn USDC on Stellar for minting on a remote EVM chain.
 *
 * @param amount - Amount in Stellar USDC subunits (7 decimals, e.g. 10_000_000 = 1 USDC)
 * @param maxFee - Maximum fee in canonical decimals (6 decimals)
 * @param minFinalityThreshold - Finality threshold (1000 = fast, 2000 = standard)
 * @param hookData - Optional hook data hex string for depositForBurnWithHook
 */
export const depositForBurn = async (
  amount: bigint,
  maxFee: bigint,
  minFinalityThreshold: number,
  hookData?: string,
): Promise<string> => {
  const server = getServer();
  const keypair = getKeypair();

  // Step 1: Approve TMM to spend USDC
  console.log("Approving USDC spend on Stellar...");
  const latestLedger = await server.getLatestLedger();
  const expirationLedger = latestLedger.sequence + 100_000;

  await submitSorobanTx(server, keypair, STELLAR_USDC_ADDRESS, "approve", [
    // from
    new Address(keypair.publicKey()).toScVal(),
    // spender
    new Address(STELLAR_TOKEN_MESSENGER_MINTER_ADDRESS).toScVal(),
    // amount (i128)
    nativeToScVal(amount, { type: "i128" }),
    // expiration_ledger (u32)
    nativeToScVal(expirationLedger, { type: "u32" }),
  ]);
  console.log("  USDC approved.");

  // Step 2: Build mint recipient (EVM address → bytes32)
  const mintRecipient = evmAddressToBytes32ScVal(EVM_ADDRESS);

  // Destination caller
  const destinationCaller = EVM_DESTINATION_CALLER
    ? evmAddressToBytes32ScVal(EVM_DESTINATION_CALLER)
    : xdr.ScVal.scvBytes(Buffer.alloc(32));

  // Step 3: deposit_for_burn or deposit_for_burn_with_hook
  const method = hookData ? "deposit_for_burn_with_hook" : "deposit_for_burn";
  const args: xdr.ScVal[] = [
    // caller
    new Address(keypair.publicKey()).toScVal(),
    // amount (i128)
    nativeToScVal(amount, { type: "i128" }),
    // destination_domain (u32)
    nativeToScVal(EVM_DOMAIN, { type: "u32" }),
    // mint_recipient (bytes)
    mintRecipient,
    // burn_token (Address — the USDC contract)
    new Address(STELLAR_USDC_ADDRESS).toScVal(),
    // destination_caller (bytes)
    destinationCaller,
    // max_fee (i128)
    nativeToScVal(maxFee, { type: "i128" }),
    // min_finality_threshold (u32)
    nativeToScVal(minFinalityThreshold, { type: "u32" }),
  ];

  if (hookData) {
    args.push(hexToScValBytes(hookData));
  }

  console.log(`Executing ${method} on Stellar...`);
  const txHash = await submitSorobanTx(server, keypair, STELLAR_TOKEN_MESSENGER_MINTER_ADDRESS, method, args);

  console.log(`  ${method} Tx: ${txHash}`);
  return txHash;
};

// ---------------------------------------------------------------------------
// EVM → Stellar: receive_message on Stellar
// ---------------------------------------------------------------------------

/**
 * Calls receive_message on the Stellar MessageTransmitter contract to complete
 * an EVM → Stellar transfer (direct, without forwarder). The message and
 * attestation are obtained from Circle's Iris attestation API.
 *
 * @param messageHex - The CCTP message bytes (0x-prefixed hex)
 * @param attestationHex - The attestation signature bytes (0x-prefixed hex)
 */
export const receiveMessage = async (messageHex: string, attestationHex: string): Promise<string> => {
  const server = getServer();
  const keypair = getKeypair();

  console.log("Executing receive_message on Stellar...");
  const txHash = await submitSorobanTx(
    server,
    keypair,
    STELLAR_MESSAGE_TRANSMITTER_ADDRESS,
    "receive_message",
    [
      // caller
      new Address(keypair.publicKey()).toScVal(),
      // message (Bytes)
      hexToScValBytes(messageHex),
      // attestation (Bytes)
      hexToScValBytes(attestationHex),
    ],
  );

  console.log(`  receive_message Tx: ${txHash}`);
  return txHash;
};

// ---------------------------------------------------------------------------
// EVM → Stellar: mint_and_forward via CCTP Forwarder
// ---------------------------------------------------------------------------

/**
 * Calls mint_and_forward on the Stellar CCTP Forwarder contract to complete
 * an EVM → Stellar transfer that was initiated with depositForBurnWithHook.
 *
 * The forwarder verifies the message, mints tokens, and forwards them to the
 * recipient encoded in the hook data.
 *
 * @param cctpForwarderAddress - Stellar contract address (C…) of the CCTP Forwarder
 * @param messageHex - The CCTP message bytes (0x-prefixed hex)
 * @param attestationHex - The attestation signature bytes (0x-prefixed hex)
 */
export const mintAndForward = async (
  cctpForwarderAddress: string,
  messageHex: string,
  attestationHex: string,
): Promise<string> => {
  const server = getServer();
  const keypair = getKeypair();

  console.log("Executing mint_and_forward on Stellar CCTP Forwarder...");
  const txHash = await submitSorobanTx(server, keypair, cctpForwarderAddress, "mint_and_forward", [
    // message (Bytes)
    hexToScValBytes(messageHex),
    // attestation (Bytes)
    hexToScValBytes(attestationHex),
  ]);

  console.log(`  mint_and_forward Tx: ${txHash}`);
  return txHash;
};
