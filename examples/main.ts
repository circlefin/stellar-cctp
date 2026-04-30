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

import "dotenv/config";
import type { Hex } from "viem";
import { minimist } from "zx";
import {
  EVM_DOMAIN,
  EVM_TOKEN_HEX,
  IRIS_API_URL,
  STELLAR_CCTP_FORWARDER_ADDRESS,
  STELLAR_DOMAIN_ID,
} from "./config";
import { depositForBurnEvmWithHook, receiveMessageEvm } from "./evm";
import { depositForBurn, mintAndForward } from "./stellar";
import { buildCctpForwarderHookData, contractStrkeyToBytes32 } from "./stellar-utils";

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

enum CommandName {
  StellarToEvm = "stellar2evm",
  EvmToStellar = "evm2stellar",
}

interface ParsedArgs {
  amount: bigint;
  fastBurn: boolean;
  /** Stellar strkey of the final recipient (for evm2stellar) */
  recipient: string;
}

// ---------------------------------------------------------------------------
// Attestation types
// ---------------------------------------------------------------------------

interface AttestationMessage {
  message: string;
  attestation: string;
  status: string;
}

interface AttestationResponse {
  error?: string;
  messages?: AttestationMessage[];
}

interface BurnFee {
  finalityThreshold: number;
  minimumFee: number;
}

interface BurnFeeSchedule {
  fast: BurnFee;
  slow: BurnFee;
}

// ---------------------------------------------------------------------------
// Fee helpers
// ---------------------------------------------------------------------------

const getMaxFee = (amount: bigint, minimumFeeBps: number): bigint => {
  // minimumFee is in bps — convert to actual fee using integer math:
  // fee = ceil(amount * minimumFeeBps / 10_000)
  const bpsDenominator = 10_000n;
  const numerator = amount * BigInt(minimumFeeBps);
  return (numerator + bpsDenominator - 1n) / bpsDenominator;
};

// ---------------------------------------------------------------------------
// Attestation
// ---------------------------------------------------------------------------

const fetchAttestation = async (txHash: string, domainId: number): Promise<AttestationMessage> => {
  console.log("Fetching attestation...");
  const url = `${IRIS_API_URL}/v2/messages/${domainId}?transactionHash=${txHash}`;

  while (true) {
    try {
      const response = await fetch(url);

      if (!response.ok) {
        if (response.status !== 404) {
          const text = await response.text().catch(() => "");
          console.error(
            `Attestation API error: ${response.status} ${response.statusText}${text ? ` - ${text}` : ""}`,
          );
        }
        await new Promise((r) => setTimeout(r, 5000));
        continue;
      }

      const data = (await response.json()) as AttestationResponse;

      if (data.error || !data.messages || data.messages[0]?.status !== "complete") {
        console.log("Waiting for attestation...");
        await new Promise((r) => setTimeout(r, 5000));
        continue;
      }

      console.log("Attestation retrieved successfully!");
      return data.messages[0];
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      console.error(`Attestation fetch error: ${msg}`);
      await new Promise((r) => setTimeout(r, 5000));
    }
  }
};

const fetchBurnFee = async (sourceDomain: number, destDomain: number): Promise<BurnFeeSchedule> => {
  console.log("Fetching fees...");

  const response = await fetch(`${IRIS_API_URL}/v2/burn/usdc/fees/${sourceDomain}/${destDomain}`);
  if (!response.ok) {
    console.warn(`Failed to fetch burn fees: ${response.statusText}, using defaults`);
    return {
      fast: { finalityThreshold: 1000, minimumFee: 1 },
      slow: { finalityThreshold: 2000, minimumFee: 1 },
    };
  }

  const fees: BurnFee[] = await response.json();
  return {
    fast: fees.find((f) => f.finalityThreshold === 1000) ?? { finalityThreshold: 1000, minimumFee: 1 },
    slow: fees.find((f) => f.finalityThreshold === 2000) ?? { finalityThreshold: 2000, minimumFee: 1 },
  };
};

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const main = async (): Promise<void> => {
  const commandName = process.argv.slice(2)[0] as CommandName;

  const rawArgs = minimist(process.argv.slice(3), {
    string: ["amount", "fastBurn", "recipient"],
  });

  const amountStr = rawArgs.amount;
  if (!amountStr || !/^\d+$/.test(amountStr)) {
    console.error("--amount is required and must be a non-negative integer");
    process.exit(1);
  }

  const args: ParsedArgs = {
    amount: BigInt(amountStr),
    fastBurn: rawArgs.fastBurn === "true",
    recipient: rawArgs.recipient ?? "",
  };

  if (!Object.values(CommandName).includes(commandName)) {
    console.error(`Command must be one of: ${Object.values(CommandName).join(", ")}`);
    process.exit(1);
  }

  console.log(`Direction: ${commandName}`);
  console.log(
    `Args: amount=${args.amount.toString()}, fastBurn=${args.fastBurn}, recipient=${args.recipient || "(default)"}`,
  );

  if (commandName === CommandName.StellarToEvm) {
    // -----------------------------------------------------------------------
    // Stellar → EVM
    // -----------------------------------------------------------------------
    const burnFee = await fetchBurnFee(STELLAR_DOMAIN_ID, EVM_DOMAIN);
    const { minimumFee, finalityThreshold: minFinalityThreshold } = args.fastBurn
      ? burnFee.fast
      : burnFee.slow;
    const maxFee = getMaxFee(args.amount, minimumFee);

    const depositTxHash = await depositForBurn(args.amount, maxFee, minFinalityThreshold);
    console.log(`DepositForBurn Tx: ${depositTxHash}`);

    const attestation = await fetchAttestation(depositTxHash, STELLAR_DOMAIN_ID);

    const receiveTxHash = await receiveMessageEvm(attestation.message as Hex, attestation.attestation as Hex);
    console.log(`ReceiveMessage Tx: ${receiveTxHash}`);
  } else if (commandName === CommandName.EvmToStellar) {
    // -----------------------------------------------------------------------
    // EVM → Stellar (via CCTP Forwarder)
    //
    // The CCTP Forwarder is required for EVM → Stellar transfers. On the EVM
    // side, depositForBurnWithHook is called with both mintRecipient and
    // destinationCaller set to the CCTP Forwarder contract address. Usage of
    // any other address for mintRecipient or destinationCaller can lead to
    // stuck funds. The final recipient is encoded in hookData.
    //
    // On the Stellar side, mint_and_forward is called on the forwarder,
    // which calls receive_message on MessageTransmitter, mints tokens,
    // and forwards them to the recipient.
    // -----------------------------------------------------------------------
    if (!STELLAR_CCTP_FORWARDER_ADDRESS) {
      console.error("STELLAR_CCTP_FORWARDER_ADDRESS is required for EVM→Stellar transfers");
      process.exit(1);
    }

    const recipient = args.recipient;
    if (!recipient) {
      console.error(
        "--recipient is required for EVM→Stellar transfers (Stellar strkey: G..., C..., or M...)",
      );
      process.exit(1);
    }

    const burnFee = await fetchBurnFee(EVM_DOMAIN, STELLAR_DOMAIN_ID);
    const { minimumFee, finalityThreshold: minFinalityThreshold } = args.fastBurn
      ? burnFee.fast
      : burnFee.slow;
    const maxFee = getMaxFee(args.amount, minimumFee);

    console.log(`Using CCTP Forwarder: ${STELLAR_CCTP_FORWARDER_ADDRESS}`);
    console.log(`Forward recipient: ${recipient}`);
    const cctpForwarderBytes32 = contractStrkeyToBytes32(STELLAR_CCTP_FORWARDER_ADDRESS);
    const hookData = buildCctpForwarderHookData(recipient);

    const depositTxHash = await depositForBurnEvmWithHook(
      args.amount,
      EVM_TOKEN_HEX as Hex,
      cctpForwarderBytes32,
      cctpForwarderBytes32,
      maxFee,
      minFinalityThreshold,
      hookData,
    );
    console.log(`DepositForBurnWithHook Tx: ${depositTxHash}`);

    const attestation = await fetchAttestation(depositTxHash, EVM_DOMAIN);

    const receiveTxHash = await mintAndForward(
      STELLAR_CCTP_FORWARDER_ADDRESS,
      attestation.message,
      attestation.attestation,
    );
    console.log(`MintAndForward Tx: ${receiveTxHash}`);
  }

  console.log("USDC transfer completed!");
};

main().catch((error) => {
  console.error("Transfer failed:", error.message ?? error);
  if (error.cause) console.error("Cause:", error.cause);
  process.exit(1);
});
