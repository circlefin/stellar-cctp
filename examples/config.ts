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

import env from "env-var";
import type { Hex } from "viem";
import { privateKeyToAccount } from "viem/accounts";

// Iris Attestation API
export const IRIS_API_URL = env.get("IRIS_API_URL").default("https://iris-api-sandbox.circle.com").asString();

// Remote EVM config
export const EVM_TOKEN_HEX = env.get("EVM_TOKEN_HEX").required().asString();
export const EVM_DOMAIN = env.get("EVM_DOMAIN").default("26").asInt();
export const EVM_CHAIN_ID = env.get("EVM_CHAIN_ID").default("5042002").asInt();
export const EVM_CHAIN_NAME = env.get("EVM_CHAIN_NAME").default("Arc Testnet").asString();
export const EVM_NATIVE_CURRENCY_NAME = env.get("EVM_NATIVE_CURRENCY_NAME").default("ETH").asString();
export const EVM_NATIVE_CURRENCY_SYMBOL = env.get("EVM_NATIVE_CURRENCY_SYMBOL").default("ETH").asString();
export const EVM_NATIVE_CURRENCY_DECIMALS = env.get("EVM_NATIVE_CURRENCY_DECIMALS").default("18").asInt();
export const EVM_PRIVATE_KEY = env.get("EVM_PRIVATE_KEY").required().asString();
export const EVM_ADDRESS = privateKeyToAccount(EVM_PRIVATE_KEY as Hex).address;
export const EVM_RPC_URL = env.get("EVM_RPC_URL").required().asString();
export const EVM_TOKEN_MESSENGER_ADDRESS = env.get("EVM_TOKEN_MESSENGER_ADDRESS").required().asString();
export const EVM_MESSAGE_TRANSMITTER_ADDRESS = env
  .get("EVM_MESSAGE_TRANSMITTER_ADDRESS")
  .required()
  .asString();

// Stellar config
export const STELLAR_RPC_URL = env.get("STELLAR_RPC_URL").required().asString();
export const STELLAR_NETWORK_PASSPHRASE = env.get("STELLAR_NETWORK_PASSPHRASE").required().asString();
export const STELLAR_SECRET_KEY = env.get("STELLAR_SECRET_KEY").required().asString();

// Stellar CCTP contract addresses
export const STELLAR_TOKEN_MESSENGER_MINTER_ADDRESS = env
  .get("STELLAR_TOKEN_MESSENGER_MINTER_ADDRESS")
  .required()
  .asString();
export const STELLAR_MESSAGE_TRANSMITTER_ADDRESS = env
  .get("STELLAR_MESSAGE_TRANSMITTER_ADDRESS")
  .required()
  .asString();
export const STELLAR_USDC_ADDRESS = env.get("STELLAR_USDC_ADDRESS").required().asString();

// CCTP Forwarder (required for EVM→Stellar transfers). This address is set as
// both the mintRecipient and destinationCaller on the EVM burn. Usage of any
// other address for mintRecipient or destinationCaller can lead to stuck funds.
export const STELLAR_CCTP_FORWARDER_ADDRESS = env
  .get("STELLAR_CCTP_FORWARDER_ADDRESS")
  .default("")
  .asString();

// Optional destination caller
export const EVM_DESTINATION_CALLER = env.get("EVM_DESTINATION_CALLER").default("").asString();

// Domain IDs
export const STELLAR_DOMAIN_ID = 27;
