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

import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { deployContractInstance, fundAccount, uploadWasm } from "@circlefin/stellar-stablecoin-scripts/common/index.js";
import { type Keypair, rpc } from "@stellar/stellar-sdk";
import { okAsync, type ResultAsync } from "neverthrow";
import { hexToBuffer } from "../cli/utils.js";
import { Client as TokenMessengerMinterV2Client } from "../clients/token-messenger-minter-v2/src/index.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DEFAULT_WASM_PATH = path.resolve(__dirname, "../../target/wasm32v1-none/release/token_messenger_minter_v2.wasm");

export interface DeployTokenMessengerMinterConfig {
  rpcUrl: string;
  networkPassphrase: string;
  deployer: Keypair;
  owner: string;
  pauser: string;
  rescuer: string;
  tokenController: string;
  admin: string;
  feeRecipient: string;
  minFeeController: string;
  denylister: string;
  messageTransmitter: string;
  messageBodyVersion: number;
  remoteDomains: number[];
  remoteTokenMessengers: Buffer[];
  wasmHash?: string;
  wasmPath?: string;
  friendbotUrl?: string;
  allowHttp?: boolean;
  /** Deployment salt. If provided, enables deterministic contract addresses. */
  salt?: Buffer;
}

export interface DeployTokenMessengerMinterResult {
  contractId: string;
  wasmHash: string;
}

export const deployTokenMessengerMinter = (
  config: DeployTokenMessengerMinterConfig,
): ResultAsync<DeployTokenMessengerMinterResult, Error> => {
  const {
    rpcUrl,
    networkPassphrase,
    deployer,
    owner,
    pauser,
    rescuer,
    tokenController,
    admin,
    feeRecipient,
    minFeeController,
    denylister,
    messageTransmitter,
    messageBodyVersion,
    remoteDomains,
    remoteTokenMessengers,
    wasmPath = DEFAULT_WASM_PATH,
    allowHttp = true,
  } = config;

  const server = new rpc.Server(rpcUrl, { allowHttp });

  const fundStep = config.friendbotUrl
    ? fundAccount(config.friendbotUrl, deployer.publicKey())
    : okAsync<void, Error>(undefined);

  return fundStep
    .andThen(() => resolveWasmHash(config, server, deployer, wasmPath, networkPassphrase))
    .andThen((wasmHash) => {
      const params = {
        owner,
        pauser,
        rescuer,
        token_controller: tokenController,
        admin,
        fee_recipient: feeRecipient,
        min_fee_controller: minFeeController,
        denylister,
        message_transmitter: messageTransmitter,
        message_body_version: messageBodyVersion,
        remote_domains: remoteDomains,
        remote_token_messengers: remoteTokenMessengers,
      };

      return deployContractInstance({ wasmHash, server, networkPassphrase, deployer, salt: config.salt }, (opts) =>
        TokenMessengerMinterV2Client.deploy({ params }, opts),
      ).map((contractId) => ({
        contractId,
        wasmHash: wasmHash.toString("hex"),
      }));
    });
};

const resolveWasmHash = (
  config: DeployTokenMessengerMinterConfig,
  server: rpc.Server,
  deployer: Keypair,
  wasmPath: string,
  networkPassphrase: string,
): ResultAsync<Buffer, Error> => {
  if (config.wasmHash) {
    return okAsync(hexToBuffer(config.wasmHash));
  }
  return uploadWasm(server, deployer, wasmPath, networkPassphrase);
};
