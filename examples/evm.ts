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
  type Chain,
  createPublicClient,
  createWalletClient,
  defineChain,
  encodeFunctionData,
  type Hex,
  type HttpTransport,
  http,
  type PublicClient,
  type WalletClient,
} from "viem";
import { type PrivateKeyAccount, privateKeyToAccount } from "viem/accounts";
import {
  EVM_CHAIN_ID,
  EVM_CHAIN_NAME,
  EVM_MESSAGE_TRANSMITTER_ADDRESS,
  EVM_NATIVE_CURRENCY_DECIMALS,
  EVM_NATIVE_CURRENCY_NAME,
  EVM_NATIVE_CURRENCY_SYMBOL,
  EVM_PRIVATE_KEY,
  EVM_RPC_URL,
  EVM_TOKEN_HEX,
  EVM_TOKEN_MESSENGER_ADDRESS,
  STELLAR_DOMAIN_ID,
} from "./config";

// ABI fragments
const ERC20_APPROVE_ABI = [
  {
    type: "function",
    name: "approve",
    stateMutability: "nonpayable",
    inputs: [
      { name: "spender", type: "address" },
      { name: "amount", type: "uint256" },
    ],
    outputs: [{ name: "", type: "bool" }],
  },
] as const;

const TOKEN_MESSENGER_V2_ABI = [
  {
    type: "function",
    name: "depositForBurnWithHook",
    stateMutability: "nonpayable",
    inputs: [
      { name: "amount", type: "uint256" },
      { name: "destinationDomain", type: "uint32" },
      { name: "mintRecipient", type: "bytes32" },
      { name: "burnToken", type: "address" },
      { name: "destinationCaller", type: "bytes32" },
      { name: "maxFee", type: "uint256" },
      { name: "minFinalityThreshold", type: "uint32" },
      { name: "hookData", type: "bytes" },
    ],
    outputs: [],
  },
] as const;

const MESSAGE_TRANSMITTER_V2_ABI = [
  {
    type: "function",
    name: "receiveMessage",
    stateMutability: "nonpayable",
    inputs: [
      { name: "message", type: "bytes" },
      { name: "attestation", type: "bytes" },
    ],
    outputs: [{ name: "", type: "bool" }],
  },
] as const;

// Define the EVM chain from env config (defaults to Arc Testnet)
const evmChain = defineChain({
  id: EVM_CHAIN_ID,
  name: EVM_CHAIN_NAME,
  nativeCurrency: {
    name: EVM_NATIVE_CURRENCY_NAME,
    symbol: EVM_NATIVE_CURRENCY_SYMBOL,
    decimals: EVM_NATIVE_CURRENCY_DECIMALS,
  },
  rpcUrls: {
    default: { http: [EVM_RPC_URL] },
  },
});

interface EvmClients {
  walletClient: WalletClient<HttpTransport, Chain, PrivateKeyAccount>;
  publicClient: PublicClient<HttpTransport, Chain>;
  account: PrivateKeyAccount;
}

const getClients = (): EvmClients => {
  const account = privateKeyToAccount(EVM_PRIVATE_KEY as Hex);
  const transport = http(EVM_RPC_URL);

  const walletClient = createWalletClient({
    chain: evmChain,
    transport,
    account,
  });

  const publicClient = createPublicClient({
    chain: evmChain,
    transport,
  });

  return { walletClient, publicClient, account };
};

const approve = async (amount: bigint): Promise<void> => {
  const { walletClient, publicClient } = getClients();

  console.log("Approving USDC spend on EVM...");
  const hash = await walletClient.sendTransaction({
    to: EVM_TOKEN_HEX as Hex,
    data: encodeFunctionData({
      abi: ERC20_APPROVE_ABI,
      functionName: "approve",
      args: [EVM_TOKEN_MESSENGER_ADDRESS as Hex, amount],
    }),
  });

  const receipt = await publicClient.waitForTransactionReceipt({ hash });
  if (receipt.status !== "success") {
    throw new Error(`EVM approve failed: ${hash}`);
  }
  console.log(`  Approved: ${hash}`);
};

/**
 * Burns USDC on EVM for minting on Stellar via the CCTP Forwarder (with hook).
 *
 * Both {@link mintRecipient} and {@link destinationCaller} must be set to the
 * CCTP Forwarder contract address. Usage of any other address for
 * mintRecipient or destinationCaller can lead to stuck funds.
 * The final Stellar recipient is encoded in {@link hookData}.
 */
export const depositForBurnEvmWithHook = async (
  amount: bigint,
  burnToken: Hex,
  mintRecipient: Hex,
  destinationCaller: Hex,
  maxFee: bigint,
  minFinalityThreshold: number,
  hookData: Hex,
): Promise<string> => {
  const { walletClient, publicClient } = getClients();

  await approve(amount);

  console.log("Depositing for burn on EVM (with hook)...");
  const hash = await walletClient.sendTransaction({
    to: EVM_TOKEN_MESSENGER_ADDRESS as Hex,
    data: encodeFunctionData({
      abi: TOKEN_MESSENGER_V2_ABI,
      functionName: "depositForBurnWithHook",
      args: [
        amount,
        STELLAR_DOMAIN_ID,
        mintRecipient,
        burnToken,
        destinationCaller,
        maxFee,
        minFinalityThreshold,
        hookData,
      ],
    }),
  });

  const receipt = await publicClient.waitForTransactionReceipt({ hash });
  if (receipt.status !== "success") {
    throw new Error(`EVM depositForBurnWithHook failed: ${hash}`);
  }
  console.log(`  Burn Tx (with hook): ${hash}`);
  return hash;
};

/**
 * Receives a CCTP message on EVM, completing a Stellar → EVM transfer.
 */
export const receiveMessageEvm = async (message: Hex, attestation: Hex): Promise<string> => {
  const { walletClient, publicClient } = getClients();

  console.log("Receiving message on EVM...");
  const hash = await walletClient.sendTransaction({
    to: EVM_MESSAGE_TRANSMITTER_ADDRESS as Hex,
    data: encodeFunctionData({
      abi: MESSAGE_TRANSMITTER_V2_ABI,
      functionName: "receiveMessage",
      args: [message, attestation],
    }),
  });

  const receipt = await publicClient.waitForTransactionReceipt({ hash });
  if (receipt.status !== "success") {
    throw new Error(`EVM receiveMessage failed: ${hash}`);
  }
  console.log(`  Receive Tx: ${hash}`);
  return hash;
};
