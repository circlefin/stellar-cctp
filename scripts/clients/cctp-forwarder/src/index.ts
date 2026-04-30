/**
 * Copyright (c) 2026, Circle Internet Group, Inc. All rights reserved.
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

import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}





/**
 * Errors for the CCTP Forwarder contract.
 */
export const CctpForwarderError = {
  /**
   * Hook data is too short (minimum 87 bytes for shortest strkey)
   */
  7300: {message:"HookDataTooShort"},
  /**
   * The mintRecipient in the message is not this CctpForwarder contract
   */
  7301: {message:"InvalidMintRecipient"},
  /**
   * The recipient in the message is not the configured TokenMessengerMinter
   */
  7302: {message:"InvalidRecipient"},
  /**
   * The message format is invalid
   */
  7303: {message:"InvalidMessageFormat"},
  /**
   * The message version is unsupported
   */
  7304: {message:"UnsupportedMessageVersion"},
  /**
   * The burn message format is invalid
   */
  7305: {message:"InvalidBurnMessageFormat"},
  /**
   * The burn message version is unsupported
   */
  7306: {message:"UnsupportedBurnMessageVersion"},
  /**
   * Failed to parse the forward recipient strkey
   */
  7307: {message:"InvalidForwardRecipient"},
  /**
   * The message transmitter address is not set
   */
  7308: {message:"MessageTransmitterNotSet"},
  /**
   * The token messenger minter address is not set
   */
  7309: {message:"TokenMessengerMinterNotSet"},
  /**
   * The expected message version is not set
   */
  7310: {message:"ExpectedMessageVersionNotSet"},
  /**
   * The expected burn message version is not set
   */
  7311: {message:"ExpectedBurnMessageVersionNotSet"},
  /**
   * The local token could not be resolved
   */
  7312: {message:"LocalTokenNotResolved"},
  /**
   * The hook data version is unsupported
   */
  7313: {message:"InvalidHookVersion"},
  /**
   * No tokens were minted by receive_message
   */
  7314: {message:"NoTokensMinted"},
  /**
   * The message transmitter address has already been set
   */
  7315: {message:"MessageTransmitterAlreadySet"},
  /**
   * The token messenger minter address has already been set
   */
  7316: {message:"TokenMessengerMinterAlreadySet"},
  /**
   * The expected message version has already been set
   */
  7317: {message:"ExpectedMessageVersionAlreadySet"},
  /**
   * The expected burn message version has already been set
   */
  7318: {message:"ExpectedBurnMessageVersionAlreadySet"}
}

/**
 * Storage keys for the CCTP Forwarder contract.
 */
export type CctpForwarderStorageKey = {tag: "MessageTransmitter", values: void} | {tag: "TokenMessengerMinter", values: void} | {tag: "ExpectedMessageVersion", values: void} | {tag: "ExpectedBurnMessageVersion", values: void};


/**
 * Initialization parameters for the CctpForwarder contract.
 */
export interface CctpForwarderContractInitParams {
  /**
 * The admin address (handles upgrades)
 */
admin: string;
  /**
 * The expected burn message version
 */
expected_burn_message_version: u32;
  /**
 * The expected message version
 */
expected_message_version: u32;
  /**
 * The MessageTransmitter contract address
 */
message_transmitter: string;
  /**
 * The contract owner address
 */
owner: string;
  /**
 * The pauser address
 */
pauser: string;
  /**
 * The rescuer address
 */
rescuer: string;
  /**
 * The TokenMessengerMinter contract address
 */
token_messenger_minter: string;
}


/**
 * Error codes for attestation verification
 */
export const AttestationError = {
  /**
   * The attestation length is invalid (must be SIGNATURE_LENGTH * threshold)
   */
  6000: {message:"InvalidAttestationLength"},
  /**
   * Signatures are not in increasing order or a duplicate signature was found
   */
  6001: {message:"InvalidSignatureOrder"},
  /**
   * The recovered signer is not an enabled attester
   */
  6002: {message:"SignerNotAttester"},
  /**
   * Failed to recover public key from signature
   */
  6003: {message:"SignatureRecoveryFailed"},
  /**
   * The signature threshold is invalid
   */
  6004: {message:"InvalidSignatureThreshold"},
  /**
   * Attempted to enable an attester that is already enabled
   */
  6005: {message:"AttesterAlreadyEnabled"},
  /**
   * Attempted to disable an attester that is already disabled
   */
  6006: {message:"AttesterAlreadyDisabled"},
  /**
   * Attempted to get an enabled attester at an index that is out of bounds
   */
  6007: {message:"AttesterIndexOutOfBounds"},
  /**
   * Public key is invalid (all zeros)
   */
  6008: {message:"InvalidAttesterAddress"},
  /**
   * Disabling would leave too few enabled attesters
   */
  6009: {message:"TooFewEnabledAttesters"},
  /**
   * The signature threshold exceeds the number of enabled attesters
   */
  6010: {message:"SignatureThresholdTooHigh"},
  /**
   * The signature threshold is already set
   */
  6011: {message:"SignatureThresholdAlreadySet"},
  /**
   * The signature threshold is not set
   */
  6012: {message:"SignatureThresholdNotSet"},
  /**
   * The signature recovery ID is invalid (must be 0 or 1, or 27/28 in Ethereum encoding)
   */
  6013: {message:"InvalidRecoveryId"}
}




export type AttestableStorageKey = {tag: "SignatureThreshold", values: void} | {tag: "EnabledAttesters", values: void};



/**
 * Error codes for denylist operations
 */
export const DenylistError = {
  /**
   * The account is on the denylist
   */
  6100: {message:"AccountDenylisted"}
}





/**
 * Represents a configuration for a local token needed to perform a swap mint with a SwapMinter.
 */
export interface SwapMinterConfig {
  allow_asset: string;
  swap_minter: string;
}



/**
 * Represents a pair of decimal configurations for local and canonical tokens.
 * 
 * This configuration is used to handle decimal precision differences between
 * tokens on different chains (e.g., Stellar USDC with 7 decimals vs CCTP with 6 decimals).
 */
export interface TokenDecimalConfig {
  /**
 * Number of decimals for the canonical token (e.g., 6 for standard CCTP)
 */
canonical_decimals: u32;
  /**
 * Number of decimals for the local token (e.g., 7 for Stellar USDC)
 */
local_decimals: u32;
}



export const TokenControllerError = {
  /**
   * If a token pair is already linked.
   */
  6300: {message:"TokenPairAlreadyLinked"},
  /**
   * If a token pair is not linked.
   */
  6301: {message:"TokenPairNotLinked"},
  /**
   * If the token decimal config is not set.
   */
  6302: {message:"TokenDecimalConfigNotSet"},
  /**
   * If the burn token is not supported (no burn limit set or limit is zero).
   */
  6303: {message:"BurnTokenNotSupported"},
  /**
   * If the burn amount exceeds the configured limit per message.
   */
  6304: {message:"BurnAmountExceedsLimit"},
  /**
   * If the swap minter config is not set for the token.
   */
  6305: {message:"SwapMinterConfigNotSet"},
  /**
   * If the burn limit per message is invalid (negative).
   */
  6306: {message:"InvalidBurnLimit"},
  /**
   * If local_decimals is less than canonical_decimals.
   */
  6307: {message:"InvalidDecimalScale"},
  /**
   * If the token decimal config is already set.
   */
  6308: {message:"TokenDecimalConfigAlreadySet"},
  /**
   * If the provided local token does not match the stored local token.
   */
  6309: {message:"InvalidLocalToken"}
}




export type TokenControllerStorageKey = {tag: "BurnLimit", values: readonly [string]} | {tag: "RemoteTokenToLocal", values: readonly [readonly [u32, Buffer]]} | {tag: "TokenDecimalConfig", values: readonly [string]} | {tag: "SwapMinterConfig", values: readonly [string]};


/**
 * Errors for the minimum fee controller module.
 */
export const MinFeeControllerError = {
  /**
   * The minimum fee controller has not been set.
   */
  6200: {message:"MinFeeControllerNotSet"},
  /**
   * The provided minimum fee is greater than or equal to MIN_FEE_MULTIPLIER.
   */
  6201: {message:"MinFeeTooHigh"},
  /**
   * The provided amount is too low to compute a minimum fee (must be > 1).
   */
  6202: {message:"AmountTooLow"},
  /**
   * The fee computation overflowed i128.
   */
  6203: {message:"MinFeeComputationOverflow"},
  /**
   * The provided minimum fee is negative.
   */
  6204: {message:"MinFeeNegative"}
}

export type MinFeeControllerStorageKey = {tag: "MinFeeByBurnToken", values: readonly [string]};

export const RemoteTokenMessengerError = {
  /**
   * If a TokenMessenger is already set for the domain.
   */
  6400: {message:"TokenMessengerAlreadySet"},
  /**
   * If no TokenMessenger is set for the domain.
   */
  6401: {message:"NoTokenMessengerSet"},
  /**
   * If the provided TokenMessenger address is zero.
   */
  6402: {message:"ZeroAddress"},
  /**
   * If the remote TokenMessenger is invalid
   */
  6403: {message:"RemoteTokenMessengerNotRegistered"}
}



export type RemoteTokenMessengerStorageKey = {tag: "RemoteTokenMessenger", values: readonly [u32]};



export const ManageableError = {
  7200: {message:"AdminNotSet"},
  7201: {message:"AdminAlreadySet"}
}

/**
 * Storage keys for `Manageable` utility.
 */
export type ManageableStorageKey = {tag: "PendingAdmin", values: void};

/**
 * Errors related to role management operations
 */
export const RoleError = {
  /**
   * The specified role has not been set
   */
  7000: {message:"RoleNotSet"}
}



export const UpgradeableError = {
  /**
   * When migration is attempted but not allowed due to upgrade state.
   */
  1100: {message:"MigrationNotAllowed"}
}



export const MerkleDistributorError = {
  /**
   * The merkle root is not set.
   */
  1300: {message:"RootNotSet"},
  /**
   * The provided index was already claimed.
   */
  1301: {message:"IndexAlreadyClaimed"},
  /**
   * The proof is invalid.
   */
  1302: {message:"InvalidProof"}
}

/**
 * Storage keys for the data associated with `MerkleDistributor`
 */
export type MerkleDistributorStorageKey = {tag: "Root", values: void} | {tag: "Claimed", values: readonly [u32]};

export type Rounding = {tag: "Floor", values: void} | {tag: "Ceil", values: void};

export const SorobanFixedPointError = {
  /**
   * The operation failed because the denominator is 0.
   */
  1500: {message:"ZeroDenominator"},
  /**
   * The operation failed because a phantom overflow occurred.
   */
  1501: {message:"PhantomOverflow"},
  /**
   * The operation failed because the result does not fit in Self.
   */
  1502: {message:"ResultOverflow"}
}

export const CryptoError = {
  /**
   * The merkle proof length is out of bounds.
   */
  1400: {message:"MerkleProofOutOfBounds"},
  /**
   * The index of the leaf is out of bounds.
   */
  1401: {message:"MerkleIndexOutOfBounds"},
  /**
   * No data in hasher state.
   */
  1402: {message:"HasherEmptyState"}
}



export const PausableError = {
  /**
   * The operation failed because the contract is paused.
   */
  1000: {message:"EnforcedPause"},
  /**
   * The operation failed because the contract is not paused.
   */
  1001: {message:"ExpectedPause"}
}

/**
 * Storage key for the pausable state
 */
export type PausableStorageKey = {tag: "Paused", values: void};


/**
 * Storage key for a simple role (single address per role).
 */
export interface RoleKey {
  role: string;
}

export const RoleTransferError = {
  2200: {message:"NoPendingTransfer"},
  2201: {message:"InvalidLiveUntilLedger"},
  2202: {message:"InvalidPendingAccount"}
}





export const AccessControlError = {
  2000: {message:"Unauthorized"},
  2001: {message:"AdminNotSet"},
  2002: {message:"IndexOutOfBounds"},
  2003: {message:"AdminRoleNotFound"},
  2004: {message:"RoleCountIsNotZero"},
  2005: {message:"RoleNotFound"},
  2006: {message:"AdminAlreadySet"},
  2007: {message:"RoleNotHeld"},
  2008: {message:"RoleIsEmpty"}
}




/**
 * Storage key for enumeration of accounts per role.
 */
export interface RoleAccountKey {
  index: u32;
  role: string;
}

/**
 * Storage keys for the data associated with the access control
 */
export type AccessControlStorageKey = {tag: "RoleAccounts", values: readonly [RoleAccountKey]} | {tag: "HasRole", values: readonly [string, string]} | {tag: "RoleAccountsCount", values: readonly [string]} | {tag: "RoleAdmin", values: readonly [string]} | {tag: "Admin", values: void} | {tag: "PendingAdmin", values: void};

export const OwnableError = {
  2100: {message:"OwnerNotSet"},
  2101: {message:"TransferInProgress"},
  2102: {message:"OwnerAlreadySet"}
}




/**
 * Storage keys for `Ownable` utility.
 */
export type OwnableStorageKey = {tag: "Owner", values: void} | {tag: "PendingOwner", values: void};

export interface Client {
  /**
   * Construct and simulate a pause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pause: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a paused transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  paused: (options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a unpause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unpause: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade: ({new_wasm_hash, operator}: {new_wasm_hash: Buffer, operator: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_admin: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_owner transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_owner: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_pauser transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pauser: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_rescuer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_rescuer: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a accept_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_admin: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a rescue_sep41 transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  rescue_sep41: ({token_contract, to, amount}: {token_contract: string, to: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a update_pauser transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_pauser: ({new_pauser}: {new_pauser: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a transfer_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  transfer_admin: ({new_admin, expires_in_ledgers}: {new_admin: string, expires_in_ledgers: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a update_rescuer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_rescuer: ({new_rescuer}: {new_rescuer: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a accept_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_ownership: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a mint_and_forward transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Mints tokens via CCTP and forwards them to the forward recipient.
   * 
   * This function calls `receive_message` on the MessageTransmitter to mint tokens
   * to this contract, then forwards them to the recipient specified in the hook data.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * * `message` - The CCTP message bytes.
   * * `attestation` - The attestation bytes for the message.
   * 
   * # Errors
   * 
   * * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
   * * [`CctpForwarderError::InvalidMessageFormat`] – The message format is invalid.
   * * [`CctpForwarderError::UnsupportedMessageVersion`] – The message version is not supported.
   * * [`CctpForwarderError::InvalidBurnMessageFormat`] – The burn message format is invalid.
   * * [`CctpForwarderError::UnsupportedBurnMessageVersion`] – The burn message version is not supported.
   * * [`CctpForwarderError::InvalidMintRecipient`] – The mintRecipient is not this contract.
   * * [`CctpForwarderError::InvalidRecipient`] – The recipient is not the TokenMessengerMinte
   */
  mint_and_forward: ({message, attestation}: {message: Buffer, attestation: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_pending_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pending_admin: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_pending_owner transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pending_owner: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  transfer_ownership: ({new_owner, expires_in_ledgers}: {new_owner: string, expires_in_ledgers: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_message_transmitter transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the configured MessageTransmitter address.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * 
   * # Returns
   * 
   * The MessageTransmitter contract address.
   */
  get_message_transmitter: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_token_messenger_minter transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the configured TokenMessengerMinter address.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * 
   * # Returns
   * 
   * The TokenMessengerMinter contract address.
   */
  get_token_messenger_minter: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_expected_message_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the expected message version.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * 
   * # Returns
   * 
   * The expected message version.
   */
  get_expected_message_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_expected_burn_msg_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the expected burn message version.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * 
   * # Returns
   * 
   * The expected burn message version.
   */
  get_expected_burn_msg_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {params}: {params: CctpForwarderContractInitParams},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({params}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAABQAAADdFbWl0dGVkIHdoZW4gYSBtaW50IGFuZCBmb3J3YXJkIG9wZXJhdGlvbiBpcyBjb21wbGV0ZWQuAAAAAAAAAAAOTWludEFuZEZvcndhcmQAAAAAAAEAAAAQbWludF9hbmRfZm9yd2FyZAAAAAMAAAAuVGhlIGFkZHJlc3MgdGhhdCByZWNlaXZlZCB0aGUgZm9yd2FyZGVkIHRva2VucwAAAAAAEWZvcndhcmRfcmVjaXBpZW50AAAAAAAAFAAAAAAAAAAkVGhlIHRva2VuIGFkZHJlc3MgdGhhdCB3YXMgZm9yd2FyZGVkAAAABXRva2VuAAAAAAAAEwAAAAAAAAAeVGhlIGFtb3VudCBvZiB0b2tlbnMgZm9yd2FyZGVkAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAI=",
        "AAAABAAAACdFcnJvcnMgZm9yIHRoZSBDQ1RQIEZvcndhcmRlciBjb250cmFjdC4AAAAAAAAAABJDY3RwRm9yd2FyZGVyRXJyb3IAAAAAABMAAAA9SG9vayBkYXRhIGlzIHRvbyBzaG9ydCAobWluaW11bSA4NyBieXRlcyBmb3Igc2hvcnRlc3Qgc3Rya2V5KQAAAAAAABBIb29rRGF0YVRvb1Nob3J0AAAchAAAAENUaGUgbWludFJlY2lwaWVudCBpbiB0aGUgbWVzc2FnZSBpcyBub3QgdGhpcyBDY3RwRm9yd2FyZGVyIGNvbnRyYWN0AAAAABRJbnZhbGlkTWludFJlY2lwaWVudAAAHIUAAABHVGhlIHJlY2lwaWVudCBpbiB0aGUgbWVzc2FnZSBpcyBub3QgdGhlIGNvbmZpZ3VyZWQgVG9rZW5NZXNzZW5nZXJNaW50ZXIAAAAAEEludmFsaWRSZWNpcGllbnQAAByGAAAAHVRoZSBtZXNzYWdlIGZvcm1hdCBpcyBpbnZhbGlkAAAAAAAAFEludmFsaWRNZXNzYWdlRm9ybWF0AAAchwAAACJUaGUgbWVzc2FnZSB2ZXJzaW9uIGlzIHVuc3VwcG9ydGVkAAAAAAAZVW5zdXBwb3J0ZWRNZXNzYWdlVmVyc2lvbgAAAAAAHIgAAAAiVGhlIGJ1cm4gbWVzc2FnZSBmb3JtYXQgaXMgaW52YWxpZAAAAAAAGEludmFsaWRCdXJuTWVzc2FnZUZvcm1hdAAAHIkAAAAnVGhlIGJ1cm4gbWVzc2FnZSB2ZXJzaW9uIGlzIHVuc3VwcG9ydGVkAAAAAB1VbnN1cHBvcnRlZEJ1cm5NZXNzYWdlVmVyc2lvbgAAAAAAHIoAAAAsRmFpbGVkIHRvIHBhcnNlIHRoZSBmb3J3YXJkIHJlY2lwaWVudCBzdHJrZXkAAAAXSW52YWxpZEZvcndhcmRSZWNpcGllbnQAAAAciwAAACpUaGUgbWVzc2FnZSB0cmFuc21pdHRlciBhZGRyZXNzIGlzIG5vdCBzZXQAAAAAABhNZXNzYWdlVHJhbnNtaXR0ZXJOb3RTZXQAAByMAAAALVRoZSB0b2tlbiBtZXNzZW5nZXIgbWludGVyIGFkZHJlc3MgaXMgbm90IHNldAAAAAAAABpUb2tlbk1lc3Nlbmdlck1pbnRlck5vdFNldAAAAAAcjQAAACdUaGUgZXhwZWN0ZWQgbWVzc2FnZSB2ZXJzaW9uIGlzIG5vdCBzZXQAAAAAHEV4cGVjdGVkTWVzc2FnZVZlcnNpb25Ob3RTZXQAAByOAAAALFRoZSBleHBlY3RlZCBidXJuIG1lc3NhZ2UgdmVyc2lvbiBpcyBub3Qgc2V0AAAAIEV4cGVjdGVkQnVybk1lc3NhZ2VWZXJzaW9uTm90U2V0AAAcjwAAACVUaGUgbG9jYWwgdG9rZW4gY291bGQgbm90IGJlIHJlc29sdmVkAAAAAAAAFUxvY2FsVG9rZW5Ob3RSZXNvbHZlZAAAAAAAHJAAAAAkVGhlIGhvb2sgZGF0YSB2ZXJzaW9uIGlzIHVuc3VwcG9ydGVkAAAAEkludmFsaWRIb29rVmVyc2lvbgAAAAAckQAAAChObyB0b2tlbnMgd2VyZSBtaW50ZWQgYnkgcmVjZWl2ZV9tZXNzYWdlAAAADk5vVG9rZW5zTWludGVkAAAAABySAAAANFRoZSBtZXNzYWdlIHRyYW5zbWl0dGVyIGFkZHJlc3MgaGFzIGFscmVhZHkgYmVlbiBzZXQAAAAcTWVzc2FnZVRyYW5zbWl0dGVyQWxyZWFkeVNldAAAHJMAAAA3VGhlIHRva2VuIG1lc3NlbmdlciBtaW50ZXIgYWRkcmVzcyBoYXMgYWxyZWFkeSBiZWVuIHNldAAAAAAeVG9rZW5NZXNzZW5nZXJNaW50ZXJBbHJlYWR5U2V0AAAAAByUAAAAMVRoZSBleHBlY3RlZCBtZXNzYWdlIHZlcnNpb24gaGFzIGFscmVhZHkgYmVlbiBzZXQAAAAAAAAgRXhwZWN0ZWRNZXNzYWdlVmVyc2lvbkFscmVhZHlTZXQAAByVAAAANlRoZSBleHBlY3RlZCBidXJuIG1lc3NhZ2UgdmVyc2lvbiBoYXMgYWxyZWFkeSBiZWVuIHNldAAAAAAAJEV4cGVjdGVkQnVybk1lc3NhZ2VWZXJzaW9uQWxyZWFkeVNldAAAHJY=",
        "AAAAAgAAAC1TdG9yYWdlIGtleXMgZm9yIHRoZSBDQ1RQIEZvcndhcmRlciBjb250cmFjdC4AAAAAAAAAAAAAF0NjdHBGb3J3YXJkZXJTdG9yYWdlS2V5AAAAAAQAAAAAAAAALlRoZSBhZGRyZXNzIG9mIHRoZSBNZXNzYWdlVHJhbnNtaXR0ZXIgY29udHJhY3QAAAAAABJNZXNzYWdlVHJhbnNtaXR0ZXIAAAAAAAAAAAAwVGhlIGFkZHJlc3Mgb2YgdGhlIFRva2VuTWVzc2VuZ2VyTWludGVyIGNvbnRyYWN0AAAAFFRva2VuTWVzc2VuZ2VyTWludGVyAAAAAAAAABxUaGUgZXhwZWN0ZWQgbWVzc2FnZSB2ZXJzaW9uAAAAFkV4cGVjdGVkTWVzc2FnZVZlcnNpb24AAAAAAAAAAAAhVGhlIGV4cGVjdGVkIGJ1cm4gbWVzc2FnZSB2ZXJzaW9uAAAAAAAAGkV4cGVjdGVkQnVybk1lc3NhZ2VWZXJzaW9uAAA=",
        "AAAAAAAAAAAAAAAFcGF1c2UAAAAAAAAAAAAAAA==",
        "AAAAAAAAAAAAAAAGcGF1c2VkAAAAAAAAAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAHdW5wYXVzZQAAAAAAAAAAAA==",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAACAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAAAAAAACG9wZXJhdG9yAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAJZ2V0X2FkbWluAAAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAAJZ2V0X293bmVyAAAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAAKZ2V0X3BhdXNlcgAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAALZ2V0X3Jlc2N1ZXIAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAAMYWNjZXB0X2FkbWluAAAAAAAAAAA=",
        "AAAAAAAAAAAAAAAMcmVzY3VlX3NlcDQxAAAAAwAAAAAAAAAOdG9rZW5fY29udHJhY3QAAAAAABMAAAAAAAAAAnRvAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAA",
        "AAAAAAAAAOtJbml0aWFsaXplcyB0aGUgQ2N0cEZvcndhcmRlciBjb250cmFjdC4KCiMgQXJndW1lbnRzCgoqIGBlbnZgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoqIGBwYXJhbXNgIC0gSW5pdGlhbGl6YXRpb24gcGFyYW1ldGVycyBpbmNsdWRpbmcgcm9sZXMgYW5kIGNvbnRyYWN0IGFkZHJlc3Nlcy4KCiMgRXZlbnRzCgpFbWl0cyByb2xlIGNvbmZpZ3VyYXRpb24gZXZlbnRzIGZvciBhbGwgcm9sZXMuAAAAAA1fX2NvbnN0cnVjdG9yAAAAAAAAAQAAAAAAAAAGcGFyYW1zAAAAAAfQAAAAH0NjdHBGb3J3YXJkZXJDb250cmFjdEluaXRQYXJhbXMAAAAAAA==",
        "AAAAAAAAAAAAAAANdXBkYXRlX3BhdXNlcgAAAAAAAAEAAAAAAAAACm5ld19wYXVzZXIAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAOdHJhbnNmZXJfYWRtaW4AAAAAAAIAAAAAAAAACW5ld19hZG1pbgAAAAAAABMAAAAAAAAAEmV4cGlyZXNfaW5fbGVkZ2VycwAAAAAABAAAAAA=",
        "AAAAAAAAAAAAAAAOdXBkYXRlX3Jlc2N1ZXIAAAAAAAEAAAAAAAAAC25ld19yZXNjdWVyAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAQYWNjZXB0X293bmVyc2hpcAAAAAAAAAAA",
        "AAAAAAAABABNaW50cyB0b2tlbnMgdmlhIENDVFAgYW5kIGZvcndhcmRzIHRoZW0gdG8gdGhlIGZvcndhcmQgcmVjaXBpZW50LgoKVGhpcyBmdW5jdGlvbiBjYWxscyBgcmVjZWl2ZV9tZXNzYWdlYCBvbiB0aGUgTWVzc2FnZVRyYW5zbWl0dGVyIHRvIG1pbnQgdG9rZW5zCnRvIHRoaXMgY29udHJhY3QsIHRoZW4gZm9yd2FyZHMgdGhlbSB0byB0aGUgcmVjaXBpZW50IHNwZWNpZmllZCBpbiB0aGUgaG9vayBkYXRhLgoKIyBBcmd1bWVudHMKCiogYGVgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoqIGBtZXNzYWdlYCAtIFRoZSBDQ1RQIG1lc3NhZ2UgYnl0ZXMuCiogYGF0dGVzdGF0aW9uYCAtIFRoZSBhdHRlc3RhdGlvbiBieXRlcyBmb3IgdGhlIG1lc3NhZ2UuCgojIEVycm9ycwoKKiBgSG9zdEVycm9yOiBFcnJvcihDb250cmFjdCwgIzEwMDApYCDigJMgQ29udHJhY3QgaXMgcGF1c2VkIChgRW5mb3JjZWRQYXVzZWRgKS4KKiBbYENjdHBGb3J3YXJkZXJFcnJvcjo6SW52YWxpZE1lc3NhZ2VGb3JtYXRgXSDigJMgVGhlIG1lc3NhZ2UgZm9ybWF0IGlzIGludmFsaWQuCiogW2BDY3RwRm9yd2FyZGVyRXJyb3I6OlVuc3VwcG9ydGVkTWVzc2FnZVZlcnNpb25gXSDigJMgVGhlIG1lc3NhZ2UgdmVyc2lvbiBpcyBub3Qgc3VwcG9ydGVkLgoqIFtgQ2N0cEZvcndhcmRlckVycm9yOjpJbnZhbGlkQnVybk1lc3NhZ2VGb3JtYXRgXSDigJMgVGhlIGJ1cm4gbWVzc2FnZSBmb3JtYXQgaXMgaW52YWxpZC4KKiBbYENjdHBGb3J3YXJkZXJFcnJvcjo6VW5zdXBwb3J0ZWRCdXJuTWVzc2FnZVZlcnNpb25gXSDigJMgVGhlIGJ1cm4gbWVzc2FnZSB2ZXJzaW9uIGlzIG5vdCBzdXBwb3J0ZWQuCiogW2BDY3RwRm9yd2FyZGVyRXJyb3I6OkludmFsaWRNaW50UmVjaXBpZW50YF0g4oCTIFRoZSBtaW50UmVjaXBpZW50IGlzIG5vdCB0aGlzIGNvbnRyYWN0LgoqIFtgQ2N0cEZvcndhcmRlckVycm9yOjpJbnZhbGlkUmVjaXBpZW50YF0g4oCTIFRoZSByZWNpcGllbnQgaXMgbm90IHRoZSBUb2tlbk1lc3Nlbmdlck1pbnRlAAAAEG1pbnRfYW5kX2ZvcndhcmQAAAACAAAAAAAAAAdtZXNzYWdlAAAAAA4AAAAAAAAAC2F0dGVzdGF0aW9uAAAAAA4AAAAA",
        "AAAAAQAAADlJbml0aWFsaXphdGlvbiBwYXJhbWV0ZXJzIGZvciB0aGUgQ2N0cEZvcndhcmRlciBjb250cmFjdC4AAAAAAAAAAAAAH0NjdHBGb3J3YXJkZXJDb250cmFjdEluaXRQYXJhbXMAAAAACAAAACRUaGUgYWRtaW4gYWRkcmVzcyAoaGFuZGxlcyB1cGdyYWRlcykAAAAFYWRtaW4AAAAAAAATAAAAIVRoZSBleHBlY3RlZCBidXJuIG1lc3NhZ2UgdmVyc2lvbgAAAAAAAB1leHBlY3RlZF9idXJuX21lc3NhZ2VfdmVyc2lvbgAAAAAAAAQAAAAcVGhlIGV4cGVjdGVkIG1lc3NhZ2UgdmVyc2lvbgAAABhleHBlY3RlZF9tZXNzYWdlX3ZlcnNpb24AAAAEAAAAJ1RoZSBNZXNzYWdlVHJhbnNtaXR0ZXIgY29udHJhY3QgYWRkcmVzcwAAAAATbWVzc2FnZV90cmFuc21pdHRlcgAAAAATAAAAGlRoZSBjb250cmFjdCBvd25lciBhZGRyZXNzAAAAAAAFb3duZXIAAAAAAAATAAAAElRoZSBwYXVzZXIgYWRkcmVzcwAAAAAABnBhdXNlcgAAAAAAEwAAABNUaGUgcmVzY3VlciBhZGRyZXNzAAAAAAdyZXNjdWVyAAAAABMAAAApVGhlIFRva2VuTWVzc2VuZ2VyTWludGVyIGNvbnRyYWN0IGFkZHJlc3MAAAAAAAAWdG9rZW5fbWVzc2VuZ2VyX21pbnRlcgAAAAAAEw==",
        "AAAAAAAAAAAAAAARZ2V0X3BlbmRpbmdfYWRtaW4AAAAAAAAAAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAARZ2V0X3BlbmRpbmdfb3duZXIAAAAAAAAAAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAASdHJhbnNmZXJfb3duZXJzaGlwAAAAAAACAAAAAAAAAAluZXdfb3duZXIAAAAAAAATAAAAAAAAABJleHBpcmVzX2luX2xlZGdlcnMAAAAAAAQAAAAA",
        "AAAAAAAAAKBSZXR1cm5zIHRoZSBjb25maWd1cmVkIE1lc3NhZ2VUcmFuc21pdHRlciBhZGRyZXNzLgoKIyBBcmd1bWVudHMKCiogYGVgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoKIyBSZXR1cm5zCgpUaGUgTWVzc2FnZVRyYW5zbWl0dGVyIGNvbnRyYWN0IGFkZHJlc3MuAAAAF2dldF9tZXNzYWdlX3RyYW5zbWl0dGVyAAAAAAAAAAABAAAAEw==",
        "AAAAAAAAAKRSZXR1cm5zIHRoZSBjb25maWd1cmVkIFRva2VuTWVzc2VuZ2VyTWludGVyIGFkZHJlc3MuCgojIEFyZ3VtZW50cwoKKiBgZWAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCgojIFJldHVybnMKClRoZSBUb2tlbk1lc3Nlbmdlck1pbnRlciBjb250cmFjdCBhZGRyZXNzLgAAABpnZXRfdG9rZW5fbWVzc2VuZ2VyX21pbnRlcgAAAAAAAAAAAAEAAAAT",
        "AAAAAAAAAIhSZXR1cm5zIHRoZSBleHBlY3RlZCBtZXNzYWdlIHZlcnNpb24uCgojIEFyZ3VtZW50cwoKKiBgZWAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCgojIFJldHVybnMKClRoZSBleHBlY3RlZCBtZXNzYWdlIHZlcnNpb24uAAAAHGdldF9leHBlY3RlZF9tZXNzYWdlX3ZlcnNpb24AAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAJJSZXR1cm5zIHRoZSBleHBlY3RlZCBidXJuIG1lc3NhZ2UgdmVyc2lvbi4KCiMgQXJndW1lbnRzCgoqIGBlYCAtIEFjY2VzcyB0byB0aGUgU29yb2JhbiBlbnZpcm9ubWVudC4KCiMgUmV0dXJucwoKVGhlIGV4cGVjdGVkIGJ1cm4gbWVzc2FnZSB2ZXJzaW9uLgAAAAAAHWdldF9leHBlY3RlZF9idXJuX21zZ192ZXJzaW9uAAAAAAAAAAAAAAEAAAAE",
        "AAAABQAAAAAAAAAAAAAAD0F0dGVzdGVyRW5hYmxlZAAAAAABAAAAEGF0dGVzdGVyX2VuYWJsZWQAAAABAAAAAAAAAAhhdHRlc3RlcgAAA+4AAAAUAAAAAQAAAAI=",
        "AAAABAAAAChFcnJvciBjb2RlcyBmb3IgYXR0ZXN0YXRpb24gdmVyaWZpY2F0aW9uAAAAAAAAABBBdHRlc3RhdGlvbkVycm9yAAAADgAAAEhUaGUgYXR0ZXN0YXRpb24gbGVuZ3RoIGlzIGludmFsaWQgKG11c3QgYmUgU0lHTkFUVVJFX0xFTkdUSCAqIHRocmVzaG9sZCkAAAAYSW52YWxpZEF0dGVzdGF0aW9uTGVuZ3RoAAAXcAAAAElTaWduYXR1cmVzIGFyZSBub3QgaW4gaW5jcmVhc2luZyBvcmRlciBvciBhIGR1cGxpY2F0ZSBzaWduYXR1cmUgd2FzIGZvdW5kAAAAAAAAFUludmFsaWRTaWduYXR1cmVPcmRlcgAAAAAAF3EAAAAvVGhlIHJlY292ZXJlZCBzaWduZXIgaXMgbm90IGFuIGVuYWJsZWQgYXR0ZXN0ZXIAAAAAEVNpZ25lck5vdEF0dGVzdGVyAAAAAAAXcgAAACtGYWlsZWQgdG8gcmVjb3ZlciBwdWJsaWMga2V5IGZyb20gc2lnbmF0dXJlAAAAABdTaWduYXR1cmVSZWNvdmVyeUZhaWxlZAAAABdzAAAAIlRoZSBzaWduYXR1cmUgdGhyZXNob2xkIGlzIGludmFsaWQAAAAAABlJbnZhbGlkU2lnbmF0dXJlVGhyZXNob2xkAAAAAAAXdAAAADdBdHRlbXB0ZWQgdG8gZW5hYmxlIGFuIGF0dGVzdGVyIHRoYXQgaXMgYWxyZWFkeSBlbmFibGVkAAAAABZBdHRlc3RlckFscmVhZHlFbmFibGVkAAAAABd1AAAAOUF0dGVtcHRlZCB0byBkaXNhYmxlIGFuIGF0dGVzdGVyIHRoYXQgaXMgYWxyZWFkeSBkaXNhYmxlZAAAAAAAABdBdHRlc3RlckFscmVhZHlEaXNhYmxlZAAAABd2AAAARkF0dGVtcHRlZCB0byBnZXQgYW4gZW5hYmxlZCBhdHRlc3RlciBhdCBhbiBpbmRleCB0aGF0IGlzIG91dCBvZiBib3VuZHMAAAAAABhBdHRlc3RlckluZGV4T3V0T2ZCb3VuZHMAABd3AAAAIVB1YmxpYyBrZXkgaXMgaW52YWxpZCAoYWxsIHplcm9zKQAAAAAAABZJbnZhbGlkQXR0ZXN0ZXJBZGRyZXNzAAAAABd4AAAAL0Rpc2FibGluZyB3b3VsZCBsZWF2ZSB0b28gZmV3IGVuYWJsZWQgYXR0ZXN0ZXJzAAAAABZUb29GZXdFbmFibGVkQXR0ZXN0ZXJzAAAAABd5AAAAP1RoZSBzaWduYXR1cmUgdGhyZXNob2xkIGV4Y2VlZHMgdGhlIG51bWJlciBvZiBlbmFibGVkIGF0dGVzdGVycwAAAAAZU2lnbmF0dXJlVGhyZXNob2xkVG9vSGlnaAAAAAAAF3oAAAAmVGhlIHNpZ25hdHVyZSB0aHJlc2hvbGQgaXMgYWxyZWFkeSBzZXQAAAAAABxTaWduYXR1cmVUaHJlc2hvbGRBbHJlYWR5U2V0AAAXewAAACJUaGUgc2lnbmF0dXJlIHRocmVzaG9sZCBpcyBub3Qgc2V0AAAAAAAYU2lnbmF0dXJlVGhyZXNob2xkTm90U2V0AAAXfAAAAFRUaGUgc2lnbmF0dXJlIHJlY292ZXJ5IElEIGlzIGludmFsaWQgKG11c3QgYmUgMCBvciAxLCBvciAyNy8yOCBpbiBFdGhlcmV1bSBlbmNvZGluZykAAAARSW52YWxpZFJlY292ZXJ5SWQAAAAAABd9",
        "AAAABQAAAAAAAAAAAAAAEEF0dGVzdGVyRGlzYWJsZWQAAAABAAAAEWF0dGVzdGVyX2Rpc2FibGVkAAAAAAAAAQAAAAAAAAAIYXR0ZXN0ZXIAAAPuAAAAFAAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAFkF0dGVzdGVyTWFuYWdlclVwZGF0ZWQAAAAAAAEAAAAYYXR0ZXN0ZXJfbWFuYWdlcl91cGRhdGVkAAAAAgAAAAAAAAAZcHJldmlvdXNfYXR0ZXN0ZXJfbWFuYWdlcgAAAAAAA+gAAAATAAAAAQAAAAAAAAAUbmV3X2F0dGVzdGVyX21hbmFnZXIAAAATAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAAGVNpZ25hdHVyZVRocmVzaG9sZFVwZGF0ZWQAAAAAAAABAAAAG3NpZ25hdHVyZV90aHJlc2hvbGRfdXBkYXRlZAAAAAACAAAAAAAAABdvbGRfc2lnbmF0dXJlX3RocmVzaG9sZAAAAAAEAAAAAAAAAAAAAAAXbmV3X3NpZ25hdHVyZV90aHJlc2hvbGQAAAAABAAAAAAAAAAC",
        "AAAAAgAAAAAAAAAAAAAAFEF0dGVzdGFibGVTdG9yYWdlS2V5AAAAAgAAAAAAAAAAAAAAElNpZ25hdHVyZVRocmVzaG9sZAAAAAAAAAAAAAAAAAAQRW5hYmxlZEF0dGVzdGVycw==",
        "AAAABQAAADFFbWl0dGVkIHdoZW4gYW4gYWRkcmVzcyBpcyBhZGRlZCB0byB0aGUgZGVueWxpc3QuAAAAAAAAAAAAAApEZW55bGlzdGVkAAAAAAABAAAACmRlbnlsaXN0ZWQAAAAAAAEAAAAAAAAAB2FjY291bnQAAAAAEwAAAAEAAAAC",
        "AAAABQAAADVFbWl0dGVkIHdoZW4gYW4gYWRkcmVzcyBpcyByZW1vdmVkIGZyb20gdGhlIGRlbnlsaXN0LgAAAAAAAAAAAAAMVW5EZW55bGlzdGVkAAAAAQAAAA11bl9kZW55bGlzdGVkAAAAAAAAAQAAAAAAAAAHYWNjb3VudAAAAAATAAAAAQAAAAI=",
        "AAAABAAAACNFcnJvciBjb2RlcyBmb3IgZGVueWxpc3Qgb3BlcmF0aW9ucwAAAAAAAAAADURlbnlsaXN0RXJyb3IAAAAAAAABAAAAHlRoZSBhY2NvdW50IGlzIG9uIHRoZSBkZW55bGlzdAAAAAAAEUFjY291bnREZW55bGlzdGVkAAAAAAAX1A==",
        "AAAABQAAACdFbWl0dGVkIHdoZW4gdGhlIGRlbnlsaXN0ZXIgaXMgdXBkYXRlZC4AAAAAAAAAABFEZW55bGlzdGVyQ2hhbmdlZAAAAAAAAAEAAAASZGVueWxpc3Rlcl9jaGFuZ2VkAAAAAAACAAAAAAAAAA5vbGRfZGVueWxpc3RlcgAAAAAD6AAAABMAAAABAAAAAAAAAA5uZXdfZGVueWxpc3RlcgAAAAAAEwAAAAEAAAAC",
        "AAAABQAAADJFbWl0dGVkIHdoZW4gdGhlIGZlZSByZWNpcGllbnQgYWRkcmVzcyBpcyB1cGRhdGVkLgAAAAAAAAAAAA9GZWVSZWNpcGllbnRTZXQAAAAAAQAAABFmZWVfcmVjaXBpZW50X3NldAAAAAAAAAEAAAAAAAAADWZlZV9yZWNpcGllbnQAAAAAAAATAAAAAAAAAAI=",
        "AAAABQAAADRFbWl0dGVkIHdoZW4gYSB0b2tlbiBwYWlyIGlzIGxpbmtlZCBiZXR3ZWVuIGRvbWFpbnMuAAAAAAAAAA9Ub2tlblBhaXJMaW5rZWQAAAAAAQAAABF0b2tlbl9wYWlyX2xpbmtlZAAAAAAAAAMAAAAAAAAAC2xvY2FsX3Rva2VuAAAAABMAAAAAAAAAAAAAAA1yZW1vdGVfZG9tYWluAAAAAAAABAAAAAAAAAAAAAAADHJlbW90ZV90b2tlbgAAA+4AAAAgAAAAAAAAAAI=",
        "AAAAAQAAAF1SZXByZXNlbnRzIGEgY29uZmlndXJhdGlvbiBmb3IgYSBsb2NhbCB0b2tlbiBuZWVkZWQgdG8gcGVyZm9ybSBhIHN3YXAgbWludCB3aXRoIGEgU3dhcE1pbnRlci4AAAAAAAAAAAAAEFN3YXBNaW50ZXJDb25maWcAAAACAAAAAAAAAAthbGxvd19hc3NldAAAAAATAAAAAAAAAAtzd2FwX21pbnRlcgAAAAAT",
        "AAAABQAAADZFbWl0dGVkIHdoZW4gYSB0b2tlbiBwYWlyIGlzIHVubGlua2VkIGJldHdlZW4gZG9tYWlucy4AAAAAAAAAAAARVG9rZW5QYWlyVW5saW5rZWQAAAAAAAABAAAAE3Rva2VuX3BhaXJfdW5saW5rZWQAAAAAAwAAAAAAAAALbG9jYWxfdG9rZW4AAAAAEwAAAAAAAAAAAAAADXJlbW90ZV9kb21haW4AAAAAAAAEAAAAAAAAAAAAAAAMcmVtb3RlX3Rva2VuAAAD7gAAACAAAAAAAAAAAg==",
        "AAAAAQAAAPBSZXByZXNlbnRzIGEgcGFpciBvZiBkZWNpbWFsIGNvbmZpZ3VyYXRpb25zIGZvciBsb2NhbCBhbmQgY2Fub25pY2FsIHRva2Vucy4KClRoaXMgY29uZmlndXJhdGlvbiBpcyB1c2VkIHRvIGhhbmRsZSBkZWNpbWFsIHByZWNpc2lvbiBkaWZmZXJlbmNlcyBiZXR3ZWVuCnRva2VucyBvbiBkaWZmZXJlbnQgY2hhaW5zIChlLmcuLCBTdGVsbGFyIFVTREMgd2l0aCA3IGRlY2ltYWxzIHZzIENDVFAgd2l0aCA2IGRlY2ltYWxzKS4AAAAAAAAAElRva2VuRGVjaW1hbENvbmZpZwAAAAAAAgAAAEZOdW1iZXIgb2YgZGVjaW1hbHMgZm9yIHRoZSBjYW5vbmljYWwgdG9rZW4gKGUuZy4sIDYgZm9yIHN0YW5kYXJkIENDVFApAAAAAAASY2Fub25pY2FsX2RlY2ltYWxzAAAAAAAEAAAAQU51bWJlciBvZiBkZWNpbWFscyBmb3IgdGhlIGxvY2FsIHRva2VuIChlLmcuLCA3IGZvciBTdGVsbGFyIFVTREMpAAAAAAAADmxvY2FsX2RlY2ltYWxzAAAAAAAE",
        "AAAABQAAACtFbWl0dGVkIHdoZW4gYSBuZXcgdG9rZW4gY29udHJvbGxlciBpcyBzZXQuAAAAAAAAAAASU2V0VG9rZW5Db250cm9sbGVyAAAAAAABAAAAFHNldF90b2tlbl9jb250cm9sbGVyAAAAAQAAAAAAAAAQdG9rZW5fY29udHJvbGxlcgAAABMAAAAAAAAAAg==",
        "AAAABQAAADVFbWl0dGVkIHdoZW4gYSBzd2FwIG1pbnRlciBjb25maWcgaXMgc2V0IGZvciBhIHRva2VuLgAAAAAAAAAAAAATU3dhcE1pbnRlckNvbmZpZ1NldAAAAAABAAAAFnN3YXBfbWludGVyX2NvbmZpZ19zZXQAAAAAAAIAAAAAAAAAC2xvY2FsX3Rva2VuAAAAABMAAAABAAAAAAAAABJzd2FwX21pbnRlcl9jb25maWcAAAAAB9AAAAAQU3dhcE1pbnRlckNvbmZpZwAAAAAAAAAC",
        "AAAABAAAAAAAAAAAAAAAFFRva2VuQ29udHJvbGxlckVycm9yAAAACgAAACJJZiBhIHRva2VuIHBhaXIgaXMgYWxyZWFkeSBsaW5rZWQuAAAAAAAWVG9rZW5QYWlyQWxyZWFkeUxpbmtlZAAAAAAYnAAAAB5JZiBhIHRva2VuIHBhaXIgaXMgbm90IGxpbmtlZC4AAAAAABJUb2tlblBhaXJOb3RMaW5rZWQAAAAAGJ0AAAAnSWYgdGhlIHRva2VuIGRlY2ltYWwgY29uZmlnIGlzIG5vdCBzZXQuAAAAABhUb2tlbkRlY2ltYWxDb25maWdOb3RTZXQAABieAAAASElmIHRoZSBidXJuIHRva2VuIGlzIG5vdCBzdXBwb3J0ZWQgKG5vIGJ1cm4gbGltaXQgc2V0IG9yIGxpbWl0IGlzIHplcm8pLgAAABVCdXJuVG9rZW5Ob3RTdXBwb3J0ZWQAAAAAABifAAAAPElmIHRoZSBidXJuIGFtb3VudCBleGNlZWRzIHRoZSBjb25maWd1cmVkIGxpbWl0IHBlciBtZXNzYWdlLgAAABZCdXJuQW1vdW50RXhjZWVkc0xpbWl0AAAAABigAAAAM0lmIHRoZSBzd2FwIG1pbnRlciBjb25maWcgaXMgbm90IHNldCBmb3IgdGhlIHRva2VuLgAAAAAWU3dhcE1pbnRlckNvbmZpZ05vdFNldAAAAAAYoQAAADRJZiB0aGUgYnVybiBsaW1pdCBwZXIgbWVzc2FnZSBpcyBpbnZhbGlkIChuZWdhdGl2ZSkuAAAAEEludmFsaWRCdXJuTGltaXQAABiiAAAAMklmIGxvY2FsX2RlY2ltYWxzIGlzIGxlc3MgdGhhbiBjYW5vbmljYWxfZGVjaW1hbHMuAAAAAAATSW52YWxpZERlY2ltYWxTY2FsZQAAABijAAAAK0lmIHRoZSB0b2tlbiBkZWNpbWFsIGNvbmZpZyBpcyBhbHJlYWR5IHNldC4AAAAAHFRva2VuRGVjaW1hbENvbmZpZ0FscmVhZHlTZXQAABikAAAAQklmIHRoZSBwcm92aWRlZCBsb2NhbCB0b2tlbiBkb2VzIG5vdCBtYXRjaCB0aGUgc3RvcmVkIGxvY2FsIHRva2VuLgAAAAAAEUludmFsaWRMb2NhbFRva2VuAAAAAAAYpQ==",
        "AAAABQAAAD9FbWl0dGVkIHdoZW4gYSBidXJuIGxpbWl0IHBlciBtZXNzYWdlIGlzIHNldCBmb3IgYSBsb2NhbCB0b2tlbi4AAAAAAAAAABZTZXRCdXJuTGltaXRQZXJNZXNzYWdlAAAAAAABAAAAGnNldF9idXJuX2xpbWl0X3Blcl9tZXNzYWdlAAAAAAACAAAAAAAAAAtsb2NhbF90b2tlbgAAAAATAAAAAQAAAAAAAAAWYnVybl9saW1pdF9wZXJfbWVzc2FnZQAAAAAACwAAAAAAAAAC",
        "AAAABQAAAD9FbWl0dGVkIHdoZW4gYSBzd2FwIG1pbnRlciBjb25maWcgaXMgcmVtb3ZlZCBmb3IgYSBsb2NhbCB0b2tlbi4AAAAAAAAAABdTd2FwTWludGVyQ29uZmlnUmVtb3ZlZAAAAAABAAAAGnN3YXBfbWludGVyX2NvbmZpZ19yZW1vdmVkAAAAAAACAAAAAAAAAAtsb2NhbF90b2tlbgAAAAATAAAAAQAAAAAAAAASc3dhcF9taW50ZXJfY29uZmlnAAAAAAfQAAAAEFN3YXBNaW50ZXJDb25maWcAAAAAAAAAAg==",
        "AAAABQAAAEVFbWl0dGVkIHdoZW4gYSBsb2NhbCB0b2tlbiBkZWNpbWFsIGNvbmZpZyBpcyBhZGRlZCBmb3IgYSBsb2NhbCB0b2tlbi4AAAAAAAAAAAAAF1Rva2VuRGVjaW1hbENvbmZpZ0FkZGVkAAAAAAEAAAAadG9rZW5fZGVjaW1hbF9jb25maWdfYWRkZWQAAAAAAAIAAAAAAAAAC2xvY2FsX3Rva2VuAAAAABMAAAABAAAAAAAAABR0b2tlbl9kZWNpbWFsX2NvbmZpZwAAB9AAAAASVG9rZW5EZWNpbWFsQ29uZmlnAAAAAAAAAAAAAg==",
        "AAAAAgAAAAAAAAAAAAAAGVRva2VuQ29udHJvbGxlclN0b3JhZ2VLZXkAAAAAAAAEAAAAAQAAAAAAAAAJQnVybkxpbWl0AAAAAAAAAQAAABMAAAABAAAAAAAAABJSZW1vdGVUb2tlblRvTG9jYWwAAAAAAAEAAAPtAAAAAgAAAAQAAAPuAAAAIAAAAAEAAAAAAAAAElRva2VuRGVjaW1hbENvbmZpZwAAAAAAAQAAABMAAAABAAAAAAAAABBTd2FwTWludGVyQ29uZmlnAAAAAQAAABM=",
        "AAAABQAAAEJFbWl0dGVkIHdoZW4gdGhlIG1pbmltdW0gZmVlIGlzIHVwZGF0ZWQgZm9yIGEgc3BlY2lmaWMgYnVybiB0b2tlbi4AAAAAAAAAAAAJTWluRmVlU2V0AAAAAAAAAQAAAAttaW5fZmVlX3NldAAAAAACAAAAAAAAAApidXJuX3Rva2VuAAAAAAATAAAAAQAAAAAAAAAHbWluX2ZlZQAAAAALAAAAAAAAAAI=",
        "AAAABQAAADhFbWl0dGVkIHdoZW4gdGhlIG1pbmltdW0gZmVlIGNvbnRyb2xsZXIgcm9sZSBpcyB1cGRhdGVkLgAAAAAAAAATTWluRmVlQ29udHJvbGxlclNldAAAAAABAAAAFm1pbl9mZWVfY29udHJvbGxlcl9zZXQAAAAAAAEAAAAAAAAAFm5ld19taW5fZmVlX2NvbnRyb2xsZXIAAAAAABMAAAABAAAAAg==",
        "AAAABAAAAC1FcnJvcnMgZm9yIHRoZSBtaW5pbXVtIGZlZSBjb250cm9sbGVyIG1vZHVsZS4AAAAAAAAAAAAAFU1pbkZlZUNvbnRyb2xsZXJFcnJvcgAAAAAAAAUAAAAsVGhlIG1pbmltdW0gZmVlIGNvbnRyb2xsZXIgaGFzIG5vdCBiZWVuIHNldC4AAAAWTWluRmVlQ29udHJvbGxlck5vdFNldAAAAAAYOAAAAEhUaGUgcHJvdmlkZWQgbWluaW11bSBmZWUgaXMgZ3JlYXRlciB0aGFuIG9yIGVxdWFsIHRvIE1JTl9GRUVfTVVMVElQTElFUi4AAAANTWluRmVlVG9vSGlnaAAAAAAAGDkAAABGVGhlIHByb3ZpZGVkIGFtb3VudCBpcyB0b28gbG93IHRvIGNvbXB1dGUgYSBtaW5pbXVtIGZlZSAobXVzdCBiZSA+IDEpLgAAAAAADEFtb3VudFRvb0xvdwAAGDoAAAAkVGhlIGZlZSBjb21wdXRhdGlvbiBvdmVyZmxvd2VkIGkxMjguAAAAGU1pbkZlZUNvbXB1dGF0aW9uT3ZlcmZsb3cAAAAAABg7AAAAJVRoZSBwcm92aWRlZCBtaW5pbXVtIGZlZSBpcyBuZWdhdGl2ZS4AAAAAAAAOTWluRmVlTmVnYXRpdmUAAAAAGDw=",
        "AAAAAgAAAAAAAAAAAAAAGk1pbkZlZUNvbnRyb2xsZXJTdG9yYWdlS2V5AAAAAAABAAAAAQAAAAAAAAARTWluRmVlQnlCdXJuVG9rZW4AAAAAAAABAAAAEw==",
        "AAAABAAAAAAAAAAAAAAAGVJlbW90ZVRva2VuTWVzc2VuZ2VyRXJyb3IAAAAAAAAEAAAAMklmIGEgVG9rZW5NZXNzZW5nZXIgaXMgYWxyZWFkeSBzZXQgZm9yIHRoZSBkb21haW4uAAAAAAAYVG9rZW5NZXNzZW5nZXJBbHJlYWR5U2V0AAAZAAAAACtJZiBubyBUb2tlbk1lc3NlbmdlciBpcyBzZXQgZm9yIHRoZSBkb21haW4uAAAAABNOb1Rva2VuTWVzc2VuZ2VyU2V0AAAAGQEAAAAvSWYgdGhlIHByb3ZpZGVkIFRva2VuTWVzc2VuZ2VyIGFkZHJlc3MgaXMgemVyby4AAAAAC1plcm9BZGRyZXNzAAAAGQIAAAAnSWYgdGhlIHJlbW90ZSBUb2tlbk1lc3NlbmdlciBpcyBpbnZhbGlkAAAAACFSZW1vdGVUb2tlbk1lc3Nlbmdlck5vdFJlZ2lzdGVyZWQAAAAAABkD",
        "AAAABQAAAC5FbWl0dGVkIHdoZW4gYSByZW1vdGUgVG9rZW5NZXNzZW5nZXIgaXMgYWRkZWQuAAAAAAAAAAAAGVJlbW90ZVRva2VuTWVzc2VuZ2VyQWRkZWQAAAAAAAABAAAAHHJlbW90ZV90b2tlbl9tZXNzZW5nZXJfYWRkZWQAAAACAAAAAAAAAAZkb21haW4AAAAAAAQAAAAAAAAAAAAAAA90b2tlbl9tZXNzZW5nZXIAAAAD7gAAACAAAAAAAAAAAg==",
        "AAAABQAAADBFbWl0dGVkIHdoZW4gYSByZW1vdGUgVG9rZW5NZXNzZW5nZXIgaXMgcmVtb3ZlZC4AAAAAAAAAG1JlbW90ZVRva2VuTWVzc2VuZ2VyUmVtb3ZlZAAAAAABAAAAHnJlbW90ZV90b2tlbl9tZXNzZW5nZXJfcmVtb3ZlZAAAAAAAAgAAAAAAAAAGZG9tYWluAAAAAAAEAAAAAAAAAAAAAAAPdG9rZW5fbWVzc2VuZ2VyAAAAA+4AAAAgAAAAAAAAAAI=",
        "AAAAAgAAAAAAAAAAAAAAHlJlbW90ZVRva2VuTWVzc2VuZ2VyU3RvcmFnZUtleQAAAAAAAQAAAAEAAAAAAAAAFFJlbW90ZVRva2VuTWVzc2VuZ2VyAAAAAQAAAAQ=",
        "AAAABQAAADJFdmVudCBlbWl0dGVkIHdoZW4gYW4gYWRtaW4gdHJhbnNmZXIgaXMgY29tcGxldGVkLgAAAAAAAAAAAAxBZG1pbkNoYW5nZWQAAAABAAAADWFkbWluX2NoYW5nZWQAAAAAAAACAAAAAAAAAAlvbGRfYWRtaW4AAAAAAAPoAAAAEwAAAAAAAAAAAAAACW5ld19hZG1pbgAAAAAAABMAAAAAAAAAAg==",
        "AAAABQAAADJFdmVudCBlbWl0dGVkIHdoZW4gYW4gYWRtaW4gdHJhbnNmZXIgaXMgaW5pdGlhdGVkLgAAAAAAAAAAABJBZG1pbkNoYW5nZVN0YXJ0ZWQAAAAAAAEAAAAUYWRtaW5fY2hhbmdlX3N0YXJ0ZWQAAAACAAAAAAAAAAlvbGRfYWRtaW4AAAAAAAPoAAAAEwAAAAAAAAAAAAAACW5ld19hZG1pbgAAAAAAABMAAAAAAAAAAg==",
        "AAAABAAAAAAAAAAAAAAAD01hbmFnZWFibGVFcnJvcgAAAAACAAAAAAAAAAtBZG1pbk5vdFNldAAAABwgAAAAAAAAAA9BZG1pbkFscmVhZHlTZXQAAAAcIQ==",
        "AAAAAgAAACZTdG9yYWdlIGtleXMgZm9yIGBNYW5hZ2VhYmxlYCB1dGlsaXR5LgAAAAAAAAAAABRNYW5hZ2VhYmxlU3RvcmFnZUtleQAAAAEAAAAAAAAAAAAAAAxQZW5kaW5nQWRtaW4=",
        "AAAABAAAACxFcnJvcnMgcmVsYXRlZCB0byByb2xlIG1hbmFnZW1lbnQgb3BlcmF0aW9ucwAAAAAAAAAJUm9sZUVycm9yAAAAAAAAAQAAACNUaGUgc3BlY2lmaWVkIHJvbGUgaGFzIG5vdCBiZWVuIHNldAAAAAAKUm9sZU5vdFNldAAAAAAbWA==",
        "AAAABQAAAClFdmVudCBlbWl0dGVkIHdoZW4gdGhlIHBhdXNlciBpcyBjaGFuZ2VkLgAAAAAAAAAAAAANUGF1c2VyQ2hhbmdlZAAAAAAAAAEAAAAOcGF1c2VyX2NoYW5nZWQAAAAAAAEAAAAAAAAAC25ld19hZGRyZXNzAAAAABMAAAAAAAAAAg==",
        "AAAABQAAADJFdmVudCBlbWl0dGVkIHdoZW4gdGhlIHJlc2N1ZXIgYWRkcmVzcyBpcyBjaGFuZ2VkLgAAAAAAAAAAAA5SZXNjdWVyQ2hhbmdlZAAAAAAAAQAAAA9yZXNjdWVyX2NoYW5nZWQAAAAAAQAAAAAAAAALbmV3X3Jlc2N1ZXIAAAAAEwAAAAAAAAAC",
        "AAAABAAAAAAAAAAAAAAAEFVwZ3JhZGVhYmxlRXJyb3IAAAABAAAAQVdoZW4gbWlncmF0aW9uIGlzIGF0dGVtcHRlZCBidXQgbm90IGFsbG93ZWQgZHVlIHRvIHVwZ3JhZGUgc3RhdGUuAAAAAAAAE01pZ3JhdGlvbk5vdEFsbG93ZWQAAAAETA==",
        "AAAABQAAACpFdmVudCBlbWl0dGVkIHdoZW4gdGhlIG1lcmtsZSByb290IGlzIHNldC4AAAAAAAAAAAAHU2V0Um9vdAAAAAABAAAACHNldF9yb290AAAAAQAAAAAAAAAEcm9vdAAAAA4AAAAAAAAAAg==",
        "AAAABQAAACdFdmVudCBlbWl0dGVkIHdoZW4gYW4gaW5kZXggaXMgY2xhaW1lZC4AAAAAAAAAAApTZXRDbGFpbWVkAAAAAAABAAAAC3NldF9jbGFpbWVkAAAAAAEAAAAAAAAABWluZGV4AAAAAAAAAAAAAAAAAAAC",
        "AAAABAAAAAAAAAAAAAAAFk1lcmtsZURpc3RyaWJ1dG9yRXJyb3IAAAAAAAMAAAAbVGhlIG1lcmtsZSByb290IGlzIG5vdCBzZXQuAAAAAApSb290Tm90U2V0AAAAAAUUAAAAJ1RoZSBwcm92aWRlZCBpbmRleCB3YXMgYWxyZWFkeSBjbGFpbWVkLgAAAAATSW5kZXhBbHJlYWR5Q2xhaW1lZAAAAAUVAAAAFVRoZSBwcm9vZiBpcyBpbnZhbGlkLgAAAAAAAAxJbnZhbGlkUHJvb2YAAAUW",
        "AAAAAgAAAD1TdG9yYWdlIGtleXMgZm9yIHRoZSBkYXRhIGFzc29jaWF0ZWQgd2l0aCBgTWVya2xlRGlzdHJpYnV0b3JgAAAAAAAAAAAAABtNZXJrbGVEaXN0cmlidXRvclN0b3JhZ2VLZXkAAAAAAgAAAAAAAAAoVGhlIE1lcmtsZSByb290IG9mIHRoZSBkaXN0cmlidXRpb24gdHJlZQAAAARSb290AAAAAQAAACNNYXBzIGFuIGluZGV4IHRvIGl0cyBjbGFpbWVkIHN0YXR1cwAAAAAHQ2xhaW1lZAAAAAABAAAABA==",
        "AAAAAgAAAAAAAAAAAAAACFJvdW5kaW5nAAAAAgAAAAAAAAAAAAAABUZsb29yAAAAAAAAAAAAAAAAAAAEQ2VpbA==",
        "AAAABAAAAAAAAAAAAAAAFlNvcm9iYW5GaXhlZFBvaW50RXJyb3IAAAAAAAMAAAAyVGhlIG9wZXJhdGlvbiBmYWlsZWQgYmVjYXVzZSB0aGUgZGVub21pbmF0b3IgaXMgMC4AAAAAAA9aZXJvRGVub21pbmF0b3IAAAAF3AAAADlUaGUgb3BlcmF0aW9uIGZhaWxlZCBiZWNhdXNlIGEgcGhhbnRvbSBvdmVyZmxvdyBvY2N1cnJlZC4AAAAAAAAPUGhhbnRvbU92ZXJmbG93AAAABd0AAAA9VGhlIG9wZXJhdGlvbiBmYWlsZWQgYmVjYXVzZSB0aGUgcmVzdWx0IGRvZXMgbm90IGZpdCBpbiBTZWxmLgAAAAAAAA5SZXN1bHRPdmVyZmxvdwAAAAAF3g==",
        "AAAABAAAAAAAAAAAAAAAC0NyeXB0b0Vycm9yAAAAAAMAAAApVGhlIG1lcmtsZSBwcm9vZiBsZW5ndGggaXMgb3V0IG9mIGJvdW5kcy4AAAAAAAAWTWVya2xlUHJvb2ZPdXRPZkJvdW5kcwAAAAAFeAAAACdUaGUgaW5kZXggb2YgdGhlIGxlYWYgaXMgb3V0IG9mIGJvdW5kcy4AAAAAFk1lcmtsZUluZGV4T3V0T2ZCb3VuZHMAAAAABXkAAAAYTm8gZGF0YSBpbiBoYXNoZXIgc3RhdGUuAAAAEEhhc2hlckVtcHR5U3RhdGUAAAV6",
        "AAAABQAAACpFdmVudCBlbWl0dGVkIHdoZW4gdGhlIGNvbnRyYWN0IGlzIHBhdXNlZC4AAAAAAAAAAAAGUGF1c2VkAAAAAAABAAAABnBhdXNlZAAAAAAAAAAAAAI=",
        "AAAABQAAACxFdmVudCBlbWl0dGVkIHdoZW4gdGhlIGNvbnRyYWN0IGlzIHVucGF1c2VkLgAAAAAAAAAIVW5wYXVzZWQAAAABAAAACHVucGF1c2VkAAAAAAAAAAI=",
        "AAAABAAAAAAAAAAAAAAADVBhdXNhYmxlRXJyb3IAAAAAAAACAAAANFRoZSBvcGVyYXRpb24gZmFpbGVkIGJlY2F1c2UgdGhlIGNvbnRyYWN0IGlzIHBhdXNlZC4AAAANRW5mb3JjZWRQYXVzZQAAAAAAA+gAAAA4VGhlIG9wZXJhdGlvbiBmYWlsZWQgYmVjYXVzZSB0aGUgY29udHJhY3QgaXMgbm90IHBhdXNlZC4AAAANRXhwZWN0ZWRQYXVzZQAAAAAAA+k=",
        "AAAAAgAAACJTdG9yYWdlIGtleSBmb3IgdGhlIHBhdXNhYmxlIHN0YXRlAAAAAAAAAAAAElBhdXNhYmxlU3RvcmFnZUtleQAAAAAAAQAAAAAAAAAySW5kaWNhdGVzIHdoZXRoZXIgdGhlIGNvbnRyYWN0IGlzIGluIHBhdXNlZCBzdGF0ZS4AAAAAAAZQYXVzZWQAAA==",
        "AAAAAQAAADhTdG9yYWdlIGtleSBmb3IgYSBzaW1wbGUgcm9sZSAoc2luZ2xlIGFkZHJlc3MgcGVyIHJvbGUpLgAAAAAAAAAHUm9sZUtleQAAAAABAAAAAAAAAARyb2xlAAAAEQ==",
        "AAAABAAAABtFcnJvcnMgZm9yIHJvbGUgb3BlcmF0aW9ucy4AAAAAAAAAAAlSb2xlRXJyb3IAAAAAAAABAAAAGlRoZSByb2xlIGhhcyBub3QgYmVlbiBzZXQuAAAAAAAKUm9sZU5vdFNldAAAAAAbWA==",
        "AAAABAAAAAAAAAAAAAAAEVJvbGVUcmFuc2ZlckVycm9yAAAAAAAAAwAAAAAAAAARTm9QZW5kaW5nVHJhbnNmZXIAAAAAAAiYAAAAAAAAABZJbnZhbGlkTGl2ZVVudGlsTGVkZ2VyAAAAAAiZAAAAAAAAABVJbnZhbGlkUGVuZGluZ0FjY291bnQAAAAAAAia",
        "AAAABQAAACVFdmVudCBlbWl0dGVkIHdoZW4gYSByb2xlIGlzIGdyYW50ZWQuAAAAAAAAAAAAAAtSb2xlR3JhbnRlZAAAAAABAAAADHJvbGVfZ3JhbnRlZAAAAAMAAAAAAAAABHJvbGUAAAARAAAAAQAAAAAAAAAHYWNjb3VudAAAAAATAAAAAQAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAI=",
        "AAAABQAAACVFdmVudCBlbWl0dGVkIHdoZW4gYSByb2xlIGlzIHJldm9rZWQuAAAAAAAAAAAAAAtSb2xlUmV2b2tlZAAAAAABAAAADHJvbGVfcmV2b2tlZAAAAAMAAAAAAAAABHJvbGUAAAARAAAAAQAAAAAAAAAHYWNjb3VudAAAAAATAAAAAQAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAI=",
        "AAAABQAAAC9FdmVudCBlbWl0dGVkIHdoZW4gdGhlIGFkbWluIHJvbGUgaXMgcmVub3VuY2VkLgAAAAAAAAAADkFkbWluUmVub3VuY2VkAAAAAAABAAAAD2FkbWluX3Jlbm91bmNlZAAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAAAg==",
        "AAAABQAAACtFdmVudCBlbWl0dGVkIHdoZW4gYSByb2xlIGFkbWluIGlzIGNoYW5nZWQuAAAAAAAAAAAQUm9sZUFkbWluQ2hhbmdlZAAAAAEAAAAScm9sZV9hZG1pbl9jaGFuZ2VkAAAAAAADAAAAAAAAAARyb2xlAAAAEQAAAAEAAAAAAAAAE3ByZXZpb3VzX2FkbWluX3JvbGUAAAAAEQAAAAAAAAAAAAAADm5ld19hZG1pbl9yb2xlAAAAAAARAAAAAAAAAAI=",
        "AAAABAAAAAAAAAAAAAAAEkFjY2Vzc0NvbnRyb2xFcnJvcgAAAAAACQAAAAAAAAAMVW5hdXRob3JpemVkAAAH0AAAAAAAAAALQWRtaW5Ob3RTZXQAAAAH0QAAAAAAAAAQSW5kZXhPdXRPZkJvdW5kcwAAB9IAAAAAAAAAEUFkbWluUm9sZU5vdEZvdW5kAAAAAAAH0wAAAAAAAAASUm9sZUNvdW50SXNOb3RaZXJvAAAAAAfUAAAAAAAAAAxSb2xlTm90Rm91bmQAAAfVAAAAAAAAAA9BZG1pbkFscmVhZHlTZXQAAAAH1gAAAAAAAAALUm9sZU5vdEhlbGQAAAAH1wAAAAAAAAALUm9sZUlzRW1wdHkAAAAH2A==",
        "AAAABQAAADJFdmVudCBlbWl0dGVkIHdoZW4gYW4gYWRtaW4gdHJhbnNmZXIgaXMgY29tcGxldGVkLgAAAAAAAAAAABZBZG1pblRyYW5zZmVyQ29tcGxldGVkAAAAAAABAAAAGGFkbWluX3RyYW5zZmVyX2NvbXBsZXRlZAAAAAIAAAAAAAAACW5ld19hZG1pbgAAAAAAABMAAAABAAAAAAAAAA5wcmV2aW91c19hZG1pbgAAAAAAEwAAAAAAAAAC",
        "AAAABQAAADJFdmVudCBlbWl0dGVkIHdoZW4gYW4gYWRtaW4gdHJhbnNmZXIgaXMgaW5pdGlhdGVkLgAAAAAAAAAAABZBZG1pblRyYW5zZmVySW5pdGlhdGVkAAAAAAABAAAAGGFkbWluX3RyYW5zZmVyX2luaXRpYXRlZAAAAAMAAAAAAAAADWN1cnJlbnRfYWRtaW4AAAAAAAATAAAAAQAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAAAAAAAAAAAEWxpdmVfdW50aWxfbGVkZ2VyAAAAAAAABAAAAAAAAAAC",
        "AAAAAQAAADFTdG9yYWdlIGtleSBmb3IgZW51bWVyYXRpb24gb2YgYWNjb3VudHMgcGVyIHJvbGUuAAAAAAAAAAAAAA5Sb2xlQWNjb3VudEtleQAAAAAAAgAAAAAAAAAFaW5kZXgAAAAAAAAEAAAAAAAAAARyb2xlAAAAEQ==",
        "AAAAAgAAADxTdG9yYWdlIGtleXMgZm9yIHRoZSBkYXRhIGFzc29jaWF0ZWQgd2l0aCB0aGUgYWNjZXNzIGNvbnRyb2wAAAAAAAAAF0FjY2Vzc0NvbnRyb2xTdG9yYWdlS2V5AAAAAAYAAAABAAAAAAAAAAxSb2xlQWNjb3VudHMAAAABAAAH0AAAAA5Sb2xlQWNjb3VudEtleQAAAAAAAQAAAAAAAAAHSGFzUm9sZQAAAAACAAAAEwAAABEAAAABAAAAAAAAABFSb2xlQWNjb3VudHNDb3VudAAAAAAAAAEAAAARAAAAAQAAAAAAAAAJUm9sZUFkbWluAAAAAAAAAQAAABEAAAAAAAAAAAAAAAVBZG1pbgAAAAAAAAAAAAAAAAAADFBlbmRpbmdBZG1pbg==",
        "AAAABAAAAAAAAAAAAAAADE93bmFibGVFcnJvcgAAAAMAAAAAAAAAC093bmVyTm90U2V0AAAACDQAAAAAAAAAElRyYW5zZmVySW5Qcm9ncmVzcwAAAAAINQAAAAAAAAAPT3duZXJBbHJlYWR5U2V0AAAACDY=",
        "AAAABQAAADZFdmVudCBlbWl0dGVkIHdoZW4gYW4gb3duZXJzaGlwIHRyYW5zZmVyIGlzIGluaXRpYXRlZC4AAAAAAAAAAAART3duZXJzaGlwVHJhbnNmZXIAAAAAAAABAAAAEm93bmVyc2hpcF90cmFuc2ZlcgAAAAAAAwAAAAAAAAAJb2xkX293bmVyAAAAAAAAEwAAAAAAAAAAAAAACW5ld19vd25lcgAAAAAAABMAAAAAAAAAAAAAABFsaXZlX3VudGlsX2xlZGdlcgAAAAAAAAQAAAAAAAAAAg==",
        "AAAABQAAACpFdmVudCBlbWl0dGVkIHdoZW4gb3duZXJzaGlwIGlzIHJlbm91bmNlZC4AAAAAAAAAAAAST3duZXJzaGlwUmVub3VuY2VkAAAAAAABAAAAE293bmVyc2hpcF9yZW5vdW5jZWQAAAAAAQAAAAAAAAAJb2xkX293bmVyAAAAAAAAEwAAAAAAAAAC",
        "AAAABQAAADZFdmVudCBlbWl0dGVkIHdoZW4gYW4gb3duZXJzaGlwIHRyYW5zZmVyIGlzIGNvbXBsZXRlZC4AAAAAAAAAAAAaT3duZXJzaGlwVHJhbnNmZXJDb21wbGV0ZWQAAAAAAAEAAAAcb3duZXJzaGlwX3RyYW5zZmVyX2NvbXBsZXRlZAAAAAEAAAAAAAAACW5ld19vd25lcgAAAAAAABMAAAAAAAAAAg==",
        "AAAAAgAAACNTdG9yYWdlIGtleXMgZm9yIGBPd25hYmxlYCB1dGlsaXR5LgAAAAAAAAAAEU93bmFibGVTdG9yYWdlS2V5AAAAAAAAAgAAAAAAAAAAAAAABU93bmVyAAAAAAAAAAAAAAAAAAAMUGVuZGluZ093bmVy" ]),
      options
    )
  }
  public readonly fromJSON = {
    pause: this.txFromJSON<null>,
        paused: this.txFromJSON<boolean>,
        unpause: this.txFromJSON<null>,
        upgrade: this.txFromJSON<null>,
        get_admin: this.txFromJSON<Option<string>>,
        get_owner: this.txFromJSON<Option<string>>,
        get_pauser: this.txFromJSON<Option<string>>,
        get_rescuer: this.txFromJSON<Option<string>>,
        accept_admin: this.txFromJSON<null>,
        rescue_sep41: this.txFromJSON<null>,
        update_pauser: this.txFromJSON<null>,
        transfer_admin: this.txFromJSON<null>,
        update_rescuer: this.txFromJSON<null>,
        accept_ownership: this.txFromJSON<null>,
        mint_and_forward: this.txFromJSON<null>,
        get_pending_admin: this.txFromJSON<Option<string>>,
        get_pending_owner: this.txFromJSON<Option<string>>,
        transfer_ownership: this.txFromJSON<null>,
        get_message_transmitter: this.txFromJSON<string>,
        get_token_messenger_minter: this.txFromJSON<string>,
        get_expected_message_version: this.txFromJSON<u32>,
        get_expected_burn_msg_version: this.txFromJSON<u32>
  }
}