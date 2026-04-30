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
 * Errors for the TokenMessengerMinter contract.
 */
export const TokenMessengerMinterError = {
  /**
   * The local message transmitter address has not been set.
   */
  7100: {message:"LocalMessageTransmitterNotSet"},
  /**
   * The message body version has not been set.
   */
  7101: {message:"MessageBodyVersionNotSet"},
  /**
   * Amount must be greater than zero.
   */
  7102: {message:"AmountMustBeNonzero"},
  /**
   * Mint recipient must not be zero.
   */
  7103: {message:"MintRecipientMustBeNonzero"},
  /**
   * Max fee must be less than amount.
   */
  7104: {message:"MaxFeeMustBeLessThanAmount"},
  /**
   * Max fee is less than the calculated minimum fee.
   */
  7105: {message:"InsufficientMaxFee"},
  /**
   * No TokenMessenger is registered for the destination domain.
   */
  7106: {message:"NoTokenMessengerForDomain"},
  /**
   * Hook data must not be empty for deposit_for_burn_with_hook.
   */
  7107: {message:"HookDataEmpty"},
  /**
   * Failed to convert address to bytes32.
   */
  7108: {message:"AddressConversionFailed"},
  /**
   * Caller is not the local message transmitter.
   */
  7109: {message:"InvalidMessageTransmitter"},
  /**
   * Remote sender is not a registered token messenger for the domain.
   */
  7110: {message:"RemoteTokenMessengerNotRegistered"},
  /**
   * Burn message format is invalid.
   */
  7111: {message:"InvalidBurnMessageV2Format"},
  /**
   * Burn message version does not match expected version.
   */
  7112: {message:"InvalidBurnMessageVersion"},
  /**
   * Fee equals or exceeds the amount.
   */
  7113: {message:"FeeEqualsOrExceedsAmount"},
  /**
   * Fee exceeds the max fee specified in the burn message.
   */
  7114: {message:"FeeExceedsMaxFee"},
  /**
   * Mint token is not supported (no local token linked for the remote token/domain).
   */
  7115: {message:"MintTokenNotSupported"},
  /**
   * Finality threshold is below the minimum required threshold.
   */
  7116: {message:"UnsupportedFinalityThreshold"},
  /**
   * Message has expired and must be re-signed.
   */
  7117: {message:"MessageExpired"},
  /**
   * Decimal conversion resulted in zero burn amount.
   */
  7118: {message:"BurnAmountTooSmall"},
  /**
   * Decimal conversion failed due to overflow or invalid configuration.
   */
  7119: {message:"DecimalConversionFailed"},
  /**
   * Token decimal configuration is not set for this token.
   */
  7120: {message:"TokenDecimalConfigNotSet"},
  /**
   * Swap minter configuration is not set for this token.
   */
  7121: {message:"SwapMinterConfigNotSet"},
  /**
   * Amount in burn message exceeds i128::MAX.
   */
  7122: {message:"AmountOverflow"},
  /**
   * Fee recipient role is not set.
   */
  7123: {message:"FeeRecipientNotSet"},
  /**
   * Max fee must not be negative.
   */
  7124: {message:"MaxFeeMustBeNonNegative"}
}

/**
 * Storage keys for the TokenMessengerMinter contract.
 */
export type TokenMessengerMinterStorageKey = {tag: "LocalMessageTransmitter", values: void} | {tag: "MessageBodyVersion", values: void};


export interface TokenMessengerMinterV2ContractInitParams {
  admin: string;
  denylister: string;
  fee_recipient: string;
  message_body_version: u32;
  message_transmitter: string;
  min_fee_controller: string;
  owner: string;
  pauser: string;
  remote_domains: Array<u32>;
  remote_token_messengers: Array<Buffer>;
  rescuer: string;
  token_controller: string;
}



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
   * Construct and simulate a denylist transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  denylist: ({account}: {account: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

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
   * Construct and simulate a get_min_fee transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_min_fee: ({burn_token}: {burn_token: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_rescuer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_rescuer: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a set_min_fee transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_min_fee: ({burn_token, min_fee}: {burn_token: string, min_fee: i128}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a un_denylist transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  un_denylist: ({account}: {account: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a accept_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_admin: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a rescue_sep41 transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  rescue_sep41: ({token_contract, to, amount}: {token_contract: string, to: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a is_denylisted transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_denylisted: ({account}: {account: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a update_pauser transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_pauser: ({new_pauser}: {new_pauser: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_denylister transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_denylister: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a transfer_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  transfer_admin: ({new_admin, expires_in_ledgers}: {new_admin: string, expires_in_ledgers: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a update_rescuer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_rescuer: ({new_rescuer}: {new_rescuer: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_local_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_local_token: ({remote_domain, remote_token}: {remote_domain: u32, remote_token: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a link_token_pair transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  link_token_pair: ({local_token, remote_domain, remote_token}: {local_token: string, remote_domain: u32, remote_token: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a accept_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_ownership: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a deposit_for_burn transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Deposits and burns tokens from sender to be minted on destination domain.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * * `caller` - The address of the caller depositing tokens.
   * * `amount` - Amount of tokens to burn (must be non-zero).
   * * `destination_domain` - Destination domain to receive message on.
   * * `mint_recipient` - Address of mint recipient on destination domain (as bytes32).
   * * `burn_token` - Token to burn `amount` of, on local domain.
   * * `destination_caller` - Authorized caller on the destination domain (as bytes32).
   * If zero, any address can broadcast the message.
   * * `max_fee` - Maximum fee to pay on the destination domain, in units of burn_token.
   * * `min_finality_threshold` - The minimum finality at which the burn message will be attested.
   * 
   * # Errors
   * 
   * * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
   * * `HostError: Error(Auth, InvalidAction)` – `caller` authorization fails.
   * * [`DenylistError::AccountDenylisted`] – If caller is on the denylist.
   * * [`TokenMesseng
   */
  deposit_for_burn: ({caller, amount, destination_domain, mint_recipient, burn_token, destination_caller, max_fee, min_finality_threshold}: {caller: string, amount: i128, destination_domain: u32, mint_recipient: Buffer, burn_token: string, destination_caller: Buffer, max_fee: i128, min_finality_threshold: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_fee_recipient transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_fee_recipient: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_pending_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pending_admin: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_pending_owner transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pending_owner: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a set_fee_recipient transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_fee_recipient: ({new_fee_recipient}: {new_fee_recipient: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a unlink_token_pair transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unlink_token_pair: ({local_token, remote_domain, remote_token}: {local_token: string, remote_domain: u32, remote_token: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a update_denylister transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_denylister: ({denylister}: {denylister: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_min_fee_amount transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_min_fee_amount: ({burn_token, amount}: {burn_token: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  transfer_ownership: ({new_owner, expires_in_ledgers}: {new_owner: string, expires_in_ledgers: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_token_controller transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_token_controller: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a set_token_controller transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_token_controller: ({new_token_controller}: {new_token_controller: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_min_fee_controller transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_min_fee_controller: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_swap_minter_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_swap_minter_config: ({local_token}: {local_token: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<SwapMinterConfig>>>

  /**
   * Construct and simulate a set_min_fee_controller transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_min_fee_controller: ({new_min_fee_controller}: {new_min_fee_controller: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_swap_minter_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_swap_minter_config: ({local_token, swap_minter, allow_asset}: {local_token: string, swap_minter: string, allow_asset: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_message_body_version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the message body version used for burn messages.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * 
   * # Returns
   * 
   * The message body version.
   * 
   * # Errors
   * 
   * * [`TokenMessengerMinterError::MessageBodyVersionNotSet`] – If not set.
   */
  get_message_body_version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_token_decimal_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_token_decimal_config: ({local_token}: {local_token: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<TokenDecimalConfig>>>

  /**
   * Construct and simulate a set_token_decimal_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_token_decimal_config: ({local_token, local_decimals, canonical_decimals}: {local_token: string, local_decimals: u32, canonical_decimals: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a remove_swap_minter_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_swap_minter_config: ({local_token}: {local_token: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a add_remote_token_messenger transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_remote_token_messenger: ({domain, token_messenger}: {domain: u32, token_messenger: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a deposit_for_burn_with_hook transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Deposits and burns tokens from sender to be minted on destination domain,
   * with optional hook data for execution on the destination chain.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * * `caller` - The address of the caller depositing tokens.
   * * `amount` - Amount of tokens to burn (must be non-zero).
   * * `destination_domain` - Destination domain to receive message on.
   * * `mint_recipient` - Address of mint recipient on destination domain (as bytes32).
   * * `burn_token` - Token to burn `amount` of, on local domain.
   * * `destination_caller` - Authorized caller on the destination domain (as bytes32).
   * If zero, any address can broadcast the message.
   * * `max_fee` - Maximum fee to pay on the destination domain, in units of burn_token.
   * * `min_finality_threshold` - The minimum finality at which the burn message will be attested.
   * * `hook_data` - Hook data to append to burn message for interpretation on destination domain.
   * 
   * # Errors
   * 
   * * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
   * * `Host
   */
  deposit_for_burn_with_hook: ({caller, amount, destination_domain, mint_recipient, burn_token, destination_caller, max_fee, min_finality_threshold, hook_data}: {caller: string, amount: i128, destination_domain: u32, mint_recipient: Buffer, burn_token: string, destination_caller: Buffer, max_fee: i128, min_finality_threshold: u32, hook_data: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_remote_token_messenger transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_remote_token_messenger: ({domain}: {domain: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Option<Buffer>>>

  /**
   * Construct and simulate a get_local_message_transmitter transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the address of the local MessageTransmitter contract.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * 
   * # Returns
   * 
   * The address of the local MessageTransmitter contract.
   * 
   * # Errors
   * 
   * * [`TokenMessengerMinterError::LocalMessageTransmitterNotSet`] – If not set.
   */
  get_local_message_transmitter: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a handle_recv_finalized_message transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Handles an incoming finalized message received by the local MessageTransmitter,
   * and takes the appropriate action. For a burn message, mints the associated token
   * to the requested recipient on the local domain.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * * `source_domain` - The domain where the message originated from.
   * * `sender` - The sender of the message (remote TokenMessenger).
   * * `finality_threshold_executed` - The level of finality at which the message was attested to.
   * * `message_body` - The message body bytes (burn message).
   * 
   * # Returns
   * 
   * `true` if successful.
   * 
   * # Errors
   * 
   * * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
   * * `HostError: Error(Auth, InvalidAction)` – Authorization from the local MessageTransmitter fails.
   * * [`RemoteTokenMessengerError::RemoteTokenMessengerNotRegistered`] – Sender is not a registered remote token messenger.
   * * [`TokenMessengerMinterError::InvalidBurnMessageFormat`] – Burn message format is invalid.
   * * [`TokenMessengerMinterError::In
   */
  handle_recv_finalized_message: ({source_domain, sender, finality_threshold_executed, message_body}: {source_domain: u32, sender: Buffer, finality_threshold_executed: u32, message_body: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a remove_remote_token_messenger transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_remote_token_messenger: ({domain}: {domain: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_max_burn_amount_per_message transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_max_burn_amount_per_message: ({local_token}: {local_token: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<i128>>>

  /**
   * Construct and simulate a handle_recv_unfinalized_message transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Handles an incoming unfinalized message received by the local MessageTransmitter,
   * and takes the appropriate action. For a burn message, mints the associated token
   * to the requested recipient on the local domain, less fees.
   * Fees are separately minted to the currently set `fee_recipient` address.
   * 
   * # Arguments
   * 
   * * `e` - Access to the Soroban environment.
   * * `source_domain` - The domain where the message originated from.
   * * `sender` - The sender of the message (remote TokenMessenger).
   * * `finality_threshold_executed` - The level of finality at which the message was attested to.
   * * `message_body` - The message body bytes (burn message).
   * 
   * # Returns
   * 
   * `true` if successful.
   * 
   * # Errors
   * 
   * * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
   * * `HostError: Error(Auth, InvalidAction)` – Authorization from the local MessageTransmitter fails.
   * * [`RemoteTokenMessengerError::RemoteTokenMessengerNotRegistered`] – Sender is not a registered remote token messenger.
   * * [`TokenMessengerMinterError::Unsupporte
   */
  handle_recv_unfinalized_message: ({source_domain, sender, finality_threshold_executed, message_body}: {source_domain: u32, sender: Buffer, finality_threshold_executed: u32, message_body: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a set_max_burn_amount_per_message transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_max_burn_amount_per_message: ({local_token, burn_limit_per_message}: {local_token: string, burn_limit_per_message: i128}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {params}: {params: TokenMessengerMinterV2ContractInitParams},
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
      new ContractSpec([ "AAAABQAAAEdFbWl0dGVkIHdoZW4gdG9rZW5zIGFyZSBkZXBvc2l0ZWQgZm9yIGJ1cm5pbmcgYW5kIGNyb3NzLWNoYWluIHRyYW5zZmVyLgAAAAAAAAAADkRlcG9zaXRGb3JCdXJuAAAAAAABAAAAEGRlcG9zaXRfZm9yX2J1cm4AAAAKAAAAMEFkZHJlc3Mgb2YgdGhlIHRva2VuIGJ1cm5lZCBvbiB0aGUgc291cmNlIGRvbWFpbgAAAApidXJuX3Rva2VuAAAAAAATAAAAAQAAABdBbW91bnQgb2YgdG9rZW5zIGJ1cm5lZAAAAAAGYW1vdW50AAAAAAALAAAAAAAAACFBZGRyZXNzIHRoYXQgZGVwb3NpdGVkIHRoZSB0b2tlbnMAAAAAAAAJZGVwb3NpdG9yAAAAAAAAEwAAAAEAAABDQWRkcmVzcyB0byByZWNlaXZlIG1pbnRlZCB0b2tlbnMgb24gZGVzdGluYXRpb24gZG9tYWluIChhcyBieXRlczMyKQAAAAAObWludF9yZWNpcGllbnQAAAAAA+4AAAAgAAAAAAAAAB1EZXN0aW5hdGlvbiBkb21haW4gaWRlbnRpZmllcgAAAAAAABJkZXN0aW5hdGlvbl9kb21haW4AAAAAAAQAAAAAAAAAPEFkZHJlc3Mgb2YgVG9rZW5NZXNzZW5nZXIgb24gZGVzdGluYXRpb24gZG9tYWluIChhcyBieXRlczMyKQAAABtkZXN0aW5hdGlvbl90b2tlbl9tZXNzZW5nZXIAAAAD7gAAACAAAAAAAAAATEF1dGhvcml6ZWQgY2FsbGVyIG9uIGRlc3RpbmF0aW9uIGRvbWFpbiAoYXMgYnl0ZXMzMiksIG9yIHplcm8gZm9yIGFueSBjYWxsZXIAAAASZGVzdGluYXRpb25fY2FsbGVyAAAAAAPuAAAAIAAAAAAAAAAoTWF4aW11bSBmZWUgdG8gcGF5IG9uIGRlc3RpbmF0aW9uIGRvbWFpbgAAAAdtYXhfZmVlAAAAAAsAAAAAAAAAKk1pbmltdW0gZmluYWxpdHkgdGhyZXNob2xkIGZvciBhdHRlc3RhdGlvbgAAAAAAFm1pbl9maW5hbGl0eV90aHJlc2hvbGQAAAAAAAQAAAABAAAANk9wdGlvbmFsIGhvb2sgZGF0YSBmb3IgZXhlY3V0aW9uIG9uIGRlc3RpbmF0aW9uIGRvbWFpbgAAAAAACWhvb2tfZGF0YQAAAAAAAA4AAAAAAAAAAg==",
        "AAAABQAAAFRFbWl0dGVkIHdoZW4gdG9rZW5zIGFyZSBtaW50ZWQgdG8gYSByZWNpcGllbnQgYWZ0ZXIgcmVjZWl2aW5nIGEgY3Jvc3MtY2hhaW4gbWVzc2FnZS4AAAAAAAAAD01pbnRBbmRXaXRoZHJhdwAAAAABAAAAEW1pbnRfYW5kX3dpdGhkcmF3AAAAAAAABAAAACdBZGRyZXNzIHRoYXQgcmVjZWl2ZWQgdGhlIG1pbnRlZCB0b2tlbnMAAAAADm1pbnRfcmVjaXBpZW50AAAAAAATAAAAAQAAAC1BbW91bnQgb2YgdG9rZW5zIHJlY2VpdmVkIGJ5IGBtaW50X3JlY2lwaWVudGAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAACRBZGRyZXNzIG9mIHRoZSBtaW50ZWQgdG9rZW4gY29udHJhY3QAAAAKbWludF90b2tlbgAAAAAAEwAAAAEAAAAkRmVlIGNvbGxlY3RlZCBmb3IgdGhlIG1pbnQgb3BlcmF0aW9uAAAADWZlZV9jb2xsZWN0ZWQAAAAAAAALAAAAAAAAAAI=",
        "AAAABAAAAC1FcnJvcnMgZm9yIHRoZSBUb2tlbk1lc3Nlbmdlck1pbnRlciBjb250cmFjdC4AAAAAAAAAAAAAGVRva2VuTWVzc2VuZ2VyTWludGVyRXJyb3IAAAAAAAAZAAAAN1RoZSBsb2NhbCBtZXNzYWdlIHRyYW5zbWl0dGVyIGFkZHJlc3MgaGFzIG5vdCBiZWVuIHNldC4AAAAAHUxvY2FsTWVzc2FnZVRyYW5zbWl0dGVyTm90U2V0AAAAAAAbvAAAACpUaGUgbWVzc2FnZSBib2R5IHZlcnNpb24gaGFzIG5vdCBiZWVuIHNldC4AAAAAABhNZXNzYWdlQm9keVZlcnNpb25Ob3RTZXQAABu9AAAAIUFtb3VudCBtdXN0IGJlIGdyZWF0ZXIgdGhhbiB6ZXJvLgAAAAAAABNBbW91bnRNdXN0QmVOb256ZXJvAAAAG74AAAAgTWludCByZWNpcGllbnQgbXVzdCBub3QgYmUgemVyby4AAAAaTWludFJlY2lwaWVudE11c3RCZU5vbnplcm8AAAAAG78AAAAhTWF4IGZlZSBtdXN0IGJlIGxlc3MgdGhhbiBhbW91bnQuAAAAAAAAGk1heEZlZU11c3RCZUxlc3NUaGFuQW1vdW50AAAAABvAAAAAME1heCBmZWUgaXMgbGVzcyB0aGFuIHRoZSBjYWxjdWxhdGVkIG1pbmltdW0gZmVlLgAAABJJbnN1ZmZpY2llbnRNYXhGZWUAAAAAG8EAAAA7Tm8gVG9rZW5NZXNzZW5nZXIgaXMgcmVnaXN0ZXJlZCBmb3IgdGhlIGRlc3RpbmF0aW9uIGRvbWFpbi4AAAAAGU5vVG9rZW5NZXNzZW5nZXJGb3JEb21haW4AAAAAABvCAAAAO0hvb2sgZGF0YSBtdXN0IG5vdCBiZSBlbXB0eSBmb3IgZGVwb3NpdF9mb3JfYnVybl93aXRoX2hvb2suAAAAAA1Ib29rRGF0YUVtcHR5AAAAAAAbwwAAACVGYWlsZWQgdG8gY29udmVydCBhZGRyZXNzIHRvIGJ5dGVzMzIuAAAAAAAAF0FkZHJlc3NDb252ZXJzaW9uRmFpbGVkAAAAG8QAAAAsQ2FsbGVyIGlzIG5vdCB0aGUgbG9jYWwgbWVzc2FnZSB0cmFuc21pdHRlci4AAAAZSW52YWxpZE1lc3NhZ2VUcmFuc21pdHRlcgAAAAAAG8UAAABBUmVtb3RlIHNlbmRlciBpcyBub3QgYSByZWdpc3RlcmVkIHRva2VuIG1lc3NlbmdlciBmb3IgdGhlIGRvbWFpbi4AAAAAAAAhUmVtb3RlVG9rZW5NZXNzZW5nZXJOb3RSZWdpc3RlcmVkAAAAAAAbxgAAAB9CdXJuIG1lc3NhZ2UgZm9ybWF0IGlzIGludmFsaWQuAAAAABpJbnZhbGlkQnVybk1lc3NhZ2VWMkZvcm1hdAAAAAAbxwAAADVCdXJuIG1lc3NhZ2UgdmVyc2lvbiBkb2VzIG5vdCBtYXRjaCBleHBlY3RlZCB2ZXJzaW9uLgAAAAAAABlJbnZhbGlkQnVybk1lc3NhZ2VWZXJzaW9uAAAAAAAbyAAAACFGZWUgZXF1YWxzIG9yIGV4Y2VlZHMgdGhlIGFtb3VudC4AAAAAAAAYRmVlRXF1YWxzT3JFeGNlZWRzQW1vdW50AAAbyQAAADZGZWUgZXhjZWVkcyB0aGUgbWF4IGZlZSBzcGVjaWZpZWQgaW4gdGhlIGJ1cm4gbWVzc2FnZS4AAAAAABBGZWVFeGNlZWRzTWF4RmVlAAAbygAAAFBNaW50IHRva2VuIGlzIG5vdCBzdXBwb3J0ZWQgKG5vIGxvY2FsIHRva2VuIGxpbmtlZCBmb3IgdGhlIHJlbW90ZSB0b2tlbi9kb21haW4pLgAAABVNaW50VG9rZW5Ob3RTdXBwb3J0ZWQAAAAAABvLAAAAO0ZpbmFsaXR5IHRocmVzaG9sZCBpcyBiZWxvdyB0aGUgbWluaW11bSByZXF1aXJlZCB0aHJlc2hvbGQuAAAAABxVbnN1cHBvcnRlZEZpbmFsaXR5VGhyZXNob2xkAAAbzAAAACpNZXNzYWdlIGhhcyBleHBpcmVkIGFuZCBtdXN0IGJlIHJlLXNpZ25lZC4AAAAAAA5NZXNzYWdlRXhwaXJlZAAAAAAbzQAAADBEZWNpbWFsIGNvbnZlcnNpb24gcmVzdWx0ZWQgaW4gemVybyBidXJuIGFtb3VudC4AAAASQnVybkFtb3VudFRvb1NtYWxsAAAAABvOAAAAQ0RlY2ltYWwgY29udmVyc2lvbiBmYWlsZWQgZHVlIHRvIG92ZXJmbG93IG9yIGludmFsaWQgY29uZmlndXJhdGlvbi4AAAAAF0RlY2ltYWxDb252ZXJzaW9uRmFpbGVkAAAAG88AAAA2VG9rZW4gZGVjaW1hbCBjb25maWd1cmF0aW9uIGlzIG5vdCBzZXQgZm9yIHRoaXMgdG9rZW4uAAAAAAAYVG9rZW5EZWNpbWFsQ29uZmlnTm90U2V0AAAb0AAAADRTd2FwIG1pbnRlciBjb25maWd1cmF0aW9uIGlzIG5vdCBzZXQgZm9yIHRoaXMgdG9rZW4uAAAAFlN3YXBNaW50ZXJDb25maWdOb3RTZXQAAAAAG9EAAAApQW1vdW50IGluIGJ1cm4gbWVzc2FnZSBleGNlZWRzIGkxMjg6Ok1BWC4AAAAAAAAOQW1vdW50T3ZlcmZsb3cAAAAAG9IAAAAeRmVlIHJlY2lwaWVudCByb2xlIGlzIG5vdCBzZXQuAAAAAAASRmVlUmVjaXBpZW50Tm90U2V0AAAAABvTAAAAHU1heCBmZWUgbXVzdCBub3QgYmUgbmVnYXRpdmUuAAAAAAAAF01heEZlZU11c3RCZU5vbk5lZ2F0aXZlAAAAG9Q=",
        "AAAAAgAAADNTdG9yYWdlIGtleXMgZm9yIHRoZSBUb2tlbk1lc3Nlbmdlck1pbnRlciBjb250cmFjdC4AAAAAAAAAAB5Ub2tlbk1lc3Nlbmdlck1pbnRlclN0b3JhZ2VLZXkAAAAAAAIAAAAAAAAANFRoZSBhZGRyZXNzIG9mIHRoZSBsb2NhbCBNZXNzYWdlVHJhbnNtaXR0ZXIgY29udHJhY3QAAAAXTG9jYWxNZXNzYWdlVHJhbnNtaXR0ZXIAAAAAAAAAACZUaGUgdmVyc2lvbiBvZiB0aGUgYnVybiBtZXNzYWdlIGZvcm1hdAAAAAAAEk1lc3NhZ2VCb2R5VmVyc2lvbgAA",
        "AAAAAAAAAAAAAAAFcGF1c2UAAAAAAAAAAAAAAA==",
        "AAAAAAAAAAAAAAAGcGF1c2VkAAAAAAAAAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAHdW5wYXVzZQAAAAAAAAAAAA==",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAACAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAAAAAAACG9wZXJhdG9yAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAIZGVueWxpc3QAAAABAAAAAAAAAAdhY2NvdW50AAAAABMAAAAA",
        "AAAAAAAAAAAAAAAJZ2V0X2FkbWluAAAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAAJZ2V0X293bmVyAAAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAAKZ2V0X3BhdXNlcgAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAALZ2V0X21pbl9mZWUAAAAAAQAAAAAAAAAKYnVybl90b2tlbgAAAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAALZ2V0X3Jlc2N1ZXIAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAALc2V0X21pbl9mZWUAAAAAAgAAAAAAAAAKYnVybl90b2tlbgAAAAAAEwAAAAAAAAAHbWluX2ZlZQAAAAALAAAAAA==",
        "AAAAAAAAAAAAAAALdW5fZGVueWxpc3QAAAAAAQAAAAAAAAAHYWNjb3VudAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAMYWNjZXB0X2FkbWluAAAAAAAAAAA=",
        "AAAAAAAAAAAAAAAMcmVzY3VlX3NlcDQxAAAAAwAAAAAAAAAOdG9rZW5fY29udHJhY3QAAAAAABMAAAAAAAAAAnRvAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAA",
        "AAAAAAAABABJbml0aWFsaXplcyB0aGUgVG9rZW5NZXNzZW5nZXJNaW50ZXJWMiBjb250cmFjdC4KCiMgQXJndW1lbnRzCgoqIGBlbnZgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoqIGBwYXJhbXNgIC0gSW5pdGlhbGl6YXRpb24gcGFyYW1ldGVycyBpbmNsdWRpbmc6Ci0gYG93bmVyYDogVGhlIGNvbnRyYWN0IG93bmVyIGFkZHJlc3MuCi0gYHBhdXNlcmA6IFRoZSBhZGRyZXNzIGF1dGhvcml6ZWQgdG8gcGF1c2UvdW5wYXVzZSB0aGUgY29udHJhY3QuCi0gYHJlc2N1ZXJgOiBUaGUgYWRkcmVzcyBhdXRob3JpemVkIHRvIHJlc2N1ZSB0b2tlbnMuCi0gYHRva2VuX2NvbnRyb2xsZXJgOiBUaGUgYWRkcmVzcyBhdXRob3JpemVkIHRvIG1hbmFnZSB0b2tlbiBjb25maWd1cmF0aW9ucy4KLSBgYWRtaW5gOiBUaGUgY29udHJhY3QgYWRtaW4gYWRkcmVzcy4KLSBgZmVlX3JlY2lwaWVudGA6IFRoZSBhZGRyZXNzIHRoYXQgcmVjZWl2ZXMgZmVlcy4KLSBgbWluX2ZlZV9jb250cm9sbGVyYDogVGhlIGFkZHJlc3MgdGhhdCBjb250cm9scyBtaW4gZmVlIGNvbmZpZ3VyYXRpb24uCi0gYGRlbnlsaXN0ZXJgOiBUaGUgYWRkcmVzcyBhdXRob3JpemVkIHRvIG1hbmFnZSB0aGUgZGVueWxpc3QuCi0gYG1lc3NhZ2VfdHJhbnNtaXR0ZXJgOiBUaGUgbG9jYWwgTWVzc2FnZVRyYW5zbWl0dGVyIGNvbnRyYWN0IGFkZHJlc3MuCi0gYG1lc3NhZ2VfYm9keV92ZXJzaW9uYDogVGhlIHZlcnNpb24gZm9yIGJ1cm4gbWVzc2FnZSBib2RpZXMuCi0gYHJlbW90ZV9kb21haW5zYDogTGlzdCBvZiByZW1vdGUgZG9tYWluIGlkZW50aWZpZXJzLgotIGByZW1vdGVfdG9rZW5fbWVzc2VuZ2Vyc2A6IENvcnJlc3BvbmRpbmcgcmVtb3RlIHRva2VuIG1lc3NlbmdlciBhZGRyZXNzZXMuCgojIEVycm9ycwoKKiBQYW5pY3MgaWYgYHJlbW90ZV9kb21haW5zYCBhbmQgYHJlbW90ZV90b2tlbl9tZXNzZW5nZXJzYCBoYXZlIGRpZmZlcmVudCBsZW5ndGhzLgoqIFtgUmVtb3RlVG9rZW5NZXNzZW5nZXJFcnJvcjo6WmVyb0FkZHJlc3NgAAAADV9fY29uc3RydWN0b3IAAAAAAAABAAAAAAAAAAZwYXJhbXMAAAAAB9AAAAAoVG9rZW5NZXNzZW5nZXJNaW50ZXJWMkNvbnRyYWN0SW5pdFBhcmFtcwAAAAA=",
        "AAAAAAAAAAAAAAANaXNfZGVueWxpc3RlZAAAAAAAAAEAAAAAAAAAB2FjY291bnQAAAAAEwAAAAEAAAAB",
        "AAAAAAAAAAAAAAANdXBkYXRlX3BhdXNlcgAAAAAAAAEAAAAAAAAACm5ld19wYXVzZXIAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAOZ2V0X2RlbnlsaXN0ZXIAAAAAAAAAAAABAAAD6AAAABM=",
        "AAAAAAAAAAAAAAAOdHJhbnNmZXJfYWRtaW4AAAAAAAIAAAAAAAAACW5ld19hZG1pbgAAAAAAABMAAAAAAAAAEmV4cGlyZXNfaW5fbGVkZ2VycwAAAAAABAAAAAA=",
        "AAAAAAAAAAAAAAAOdXBkYXRlX3Jlc2N1ZXIAAAAAAAEAAAAAAAAAC25ld19yZXNjdWVyAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAPZ2V0X2xvY2FsX3Rva2VuAAAAAAIAAAAAAAAADXJlbW90ZV9kb21haW4AAAAAAAAEAAAAAAAAAAxyZW1vdGVfdG9rZW4AAAPuAAAAIAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAAPbGlua190b2tlbl9wYWlyAAAAAAMAAAAAAAAAC2xvY2FsX3Rva2VuAAAAABMAAAAAAAAADXJlbW90ZV9kb21haW4AAAAAAAAEAAAAAAAAAAxyZW1vdGVfdG9rZW4AAAPuAAAAIAAAAAA=",
        "AAAAAQAAAAAAAAAAAAAAKFRva2VuTWVzc2VuZ2VyTWludGVyVjJDb250cmFjdEluaXRQYXJhbXMAAAAMAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAACmRlbnlsaXN0ZXIAAAAAABMAAAAAAAAADWZlZV9yZWNpcGllbnQAAAAAAAATAAAAAAAAABRtZXNzYWdlX2JvZHlfdmVyc2lvbgAAAAQAAAAAAAAAE21lc3NhZ2VfdHJhbnNtaXR0ZXIAAAAAEwAAAAAAAAASbWluX2ZlZV9jb250cm9sbGVyAAAAAAATAAAAAAAAAAVvd25lcgAAAAAAABMAAAAAAAAABnBhdXNlcgAAAAAAEwAAAAAAAAAOcmVtb3RlX2RvbWFpbnMAAAAAA+oAAAAEAAAAAAAAABdyZW1vdGVfdG9rZW5fbWVzc2VuZ2VycwAAAAPqAAAD7gAAACAAAAAAAAAAB3Jlc2N1ZXIAAAAAEwAAAAAAAAAQdG9rZW5fY29udHJvbGxlcgAAABM=",
        "AAAAAAAAAAAAAAAQYWNjZXB0X293bmVyc2hpcAAAAAAAAAAA",
        "AAAAAAAABABEZXBvc2l0cyBhbmQgYnVybnMgdG9rZW5zIGZyb20gc2VuZGVyIHRvIGJlIG1pbnRlZCBvbiBkZXN0aW5hdGlvbiBkb21haW4uCgojIEFyZ3VtZW50cwoKKiBgZWAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCiogYGNhbGxlcmAgLSBUaGUgYWRkcmVzcyBvZiB0aGUgY2FsbGVyIGRlcG9zaXRpbmcgdG9rZW5zLgoqIGBhbW91bnRgIC0gQW1vdW50IG9mIHRva2VucyB0byBidXJuIChtdXN0IGJlIG5vbi16ZXJvKS4KKiBgZGVzdGluYXRpb25fZG9tYWluYCAtIERlc3RpbmF0aW9uIGRvbWFpbiB0byByZWNlaXZlIG1lc3NhZ2Ugb24uCiogYG1pbnRfcmVjaXBpZW50YCAtIEFkZHJlc3Mgb2YgbWludCByZWNpcGllbnQgb24gZGVzdGluYXRpb24gZG9tYWluIChhcyBieXRlczMyKS4KKiBgYnVybl90b2tlbmAgLSBUb2tlbiB0byBidXJuIGBhbW91bnRgIG9mLCBvbiBsb2NhbCBkb21haW4uCiogYGRlc3RpbmF0aW9uX2NhbGxlcmAgLSBBdXRob3JpemVkIGNhbGxlciBvbiB0aGUgZGVzdGluYXRpb24gZG9tYWluIChhcyBieXRlczMyKS4KSWYgemVybywgYW55IGFkZHJlc3MgY2FuIGJyb2FkY2FzdCB0aGUgbWVzc2FnZS4KKiBgbWF4X2ZlZWAgLSBNYXhpbXVtIGZlZSB0byBwYXkgb24gdGhlIGRlc3RpbmF0aW9uIGRvbWFpbiwgaW4gdW5pdHMgb2YgYnVybl90b2tlbi4KKiBgbWluX2ZpbmFsaXR5X3RocmVzaG9sZGAgLSBUaGUgbWluaW11bSBmaW5hbGl0eSBhdCB3aGljaCB0aGUgYnVybiBtZXNzYWdlIHdpbGwgYmUgYXR0ZXN0ZWQuCgojIEVycm9ycwoKKiBgSG9zdEVycm9yOiBFcnJvcihDb250cmFjdCwgIzEwMDApYCDigJMgQ29udHJhY3QgaXMgcGF1c2VkIChgRW5mb3JjZWRQYXVzZWRgKS4KKiBgSG9zdEVycm9yOiBFcnJvcihBdXRoLCBJbnZhbGlkQWN0aW9uKWAg4oCTIGBjYWxsZXJgIGF1dGhvcml6YXRpb24gZmFpbHMuCiogW2BEZW55bGlzdEVycm9yOjpBY2NvdW50RGVueWxpc3RlZGBdIOKAkyBJZiBjYWxsZXIgaXMgb24gdGhlIGRlbnlsaXN0LgoqIFtgVG9rZW5NZXNzZW5nAAAAEGRlcG9zaXRfZm9yX2J1cm4AAAAIAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAASZGVzdGluYXRpb25fZG9tYWluAAAAAAAEAAAAAAAAAA5taW50X3JlY2lwaWVudAAAAAAD7gAAACAAAAAAAAAACmJ1cm5fdG9rZW4AAAAAABMAAAAAAAAAEmRlc3RpbmF0aW9uX2NhbGxlcgAAAAAD7gAAACAAAAAAAAAAB21heF9mZWUAAAAACwAAAAAAAAAWbWluX2ZpbmFsaXR5X3RocmVzaG9sZAAAAAAABAAAAAA=",
        "AAAAAAAAAAAAAAARZ2V0X2ZlZV9yZWNpcGllbnQAAAAAAAAAAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAARZ2V0X3BlbmRpbmdfYWRtaW4AAAAAAAAAAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAARZ2V0X3BlbmRpbmdfb3duZXIAAAAAAAAAAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAARc2V0X2ZlZV9yZWNpcGllbnQAAAAAAAABAAAAAAAAABFuZXdfZmVlX3JlY2lwaWVudAAAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAARdW5saW5rX3Rva2VuX3BhaXIAAAAAAAADAAAAAAAAAAtsb2NhbF90b2tlbgAAAAATAAAAAAAAAA1yZW1vdGVfZG9tYWluAAAAAAAABAAAAAAAAAAMcmVtb3RlX3Rva2VuAAAD7gAAACAAAAAA",
        "AAAAAAAAAAAAAAARdXBkYXRlX2RlbnlsaXN0ZXIAAAAAAAABAAAAAAAAAApkZW55bGlzdGVyAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAASZ2V0X21pbl9mZWVfYW1vdW50AAAAAAACAAAAAAAAAApidXJuX3Rva2VuAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAACw==",
        "AAAAAAAAAAAAAAASdHJhbnNmZXJfb3duZXJzaGlwAAAAAAACAAAAAAAAAAluZXdfb3duZXIAAAAAAAATAAAAAAAAABJleHBpcmVzX2luX2xlZGdlcnMAAAAAAAQAAAAA",
        "AAAAAAAAAAAAAAAUZ2V0X3Rva2VuX2NvbnRyb2xsZXIAAAAAAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAAUc2V0X3Rva2VuX2NvbnRyb2xsZXIAAAABAAAAAAAAABRuZXdfdG9rZW5fY29udHJvbGxlcgAAABMAAAAA",
        "AAAAAAAAAAAAAAAWZ2V0X21pbl9mZWVfY29udHJvbGxlcgAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAAAAAAAAAAAAAWZ2V0X3N3YXBfbWludGVyX2NvbmZpZwAAAAAAAQAAAAAAAAALbG9jYWxfdG9rZW4AAAAAEwAAAAEAAAPoAAAH0AAAABBTd2FwTWludGVyQ29uZmln",
        "AAAAAAAAAAAAAAAWc2V0X21pbl9mZWVfY29udHJvbGxlcgAAAAAAAQAAAAAAAAAWbmV3X21pbl9mZWVfY29udHJvbGxlcgAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAWc2V0X3N3YXBfbWludGVyX2NvbmZpZwAAAAAAAwAAAAAAAAALbG9jYWxfdG9rZW4AAAAAEwAAAAAAAAALc3dhcF9taW50ZXIAAAAAEwAAAAAAAAALYWxsb3dfYXNzZXQAAAAAEwAAAAA=",
        "AAAAAAAAAOxSZXR1cm5zIHRoZSBtZXNzYWdlIGJvZHkgdmVyc2lvbiB1c2VkIGZvciBidXJuIG1lc3NhZ2VzLgoKIyBBcmd1bWVudHMKCiogYGVgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoKIyBSZXR1cm5zCgpUaGUgbWVzc2FnZSBib2R5IHZlcnNpb24uCgojIEVycm9ycwoKKiBbYFRva2VuTWVzc2VuZ2VyTWludGVyRXJyb3I6Ok1lc3NhZ2VCb2R5VmVyc2lvbk5vdFNldGBdIOKAkyBJZiBub3Qgc2V0LgAAABhnZXRfbWVzc2FnZV9ib2R5X3ZlcnNpb24AAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAAYZ2V0X3Rva2VuX2RlY2ltYWxfY29uZmlnAAAAAQAAAAAAAAALbG9jYWxfdG9rZW4AAAAAEwAAAAEAAAPoAAAH0AAAABJUb2tlbkRlY2ltYWxDb25maWcAAA==",
        "AAAAAAAAAAAAAAAYc2V0X3Rva2VuX2RlY2ltYWxfY29uZmlnAAAAAwAAAAAAAAALbG9jYWxfdG9rZW4AAAAAEwAAAAAAAAAObG9jYWxfZGVjaW1hbHMAAAAAAAQAAAAAAAAAEmNhbm9uaWNhbF9kZWNpbWFscwAAAAAABAAAAAA=",
        "AAAAAAAAAAAAAAAZcmVtb3ZlX3N3YXBfbWludGVyX2NvbmZpZwAAAAAAAAEAAAAAAAAAC2xvY2FsX3Rva2VuAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAaYWRkX3JlbW90ZV90b2tlbl9tZXNzZW5nZXIAAAAAAAIAAAAAAAAABmRvbWFpbgAAAAAABAAAAAAAAAAPdG9rZW5fbWVzc2VuZ2VyAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAABABEZXBvc2l0cyBhbmQgYnVybnMgdG9rZW5zIGZyb20gc2VuZGVyIHRvIGJlIG1pbnRlZCBvbiBkZXN0aW5hdGlvbiBkb21haW4sCndpdGggb3B0aW9uYWwgaG9vayBkYXRhIGZvciBleGVjdXRpb24gb24gdGhlIGRlc3RpbmF0aW9uIGNoYWluLgoKIyBBcmd1bWVudHMKCiogYGVgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoqIGBjYWxsZXJgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIGNhbGxlciBkZXBvc2l0aW5nIHRva2Vucy4KKiBgYW1vdW50YCAtIEFtb3VudCBvZiB0b2tlbnMgdG8gYnVybiAobXVzdCBiZSBub24temVybykuCiogYGRlc3RpbmF0aW9uX2RvbWFpbmAgLSBEZXN0aW5hdGlvbiBkb21haW4gdG8gcmVjZWl2ZSBtZXNzYWdlIG9uLgoqIGBtaW50X3JlY2lwaWVudGAgLSBBZGRyZXNzIG9mIG1pbnQgcmVjaXBpZW50IG9uIGRlc3RpbmF0aW9uIGRvbWFpbiAoYXMgYnl0ZXMzMikuCiogYGJ1cm5fdG9rZW5gIC0gVG9rZW4gdG8gYnVybiBgYW1vdW50YCBvZiwgb24gbG9jYWwgZG9tYWluLgoqIGBkZXN0aW5hdGlvbl9jYWxsZXJgIC0gQXV0aG9yaXplZCBjYWxsZXIgb24gdGhlIGRlc3RpbmF0aW9uIGRvbWFpbiAoYXMgYnl0ZXMzMikuCklmIHplcm8sIGFueSBhZGRyZXNzIGNhbiBicm9hZGNhc3QgdGhlIG1lc3NhZ2UuCiogYG1heF9mZWVgIC0gTWF4aW11bSBmZWUgdG8gcGF5IG9uIHRoZSBkZXN0aW5hdGlvbiBkb21haW4sIGluIHVuaXRzIG9mIGJ1cm5fdG9rZW4uCiogYG1pbl9maW5hbGl0eV90aHJlc2hvbGRgIC0gVGhlIG1pbmltdW0gZmluYWxpdHkgYXQgd2hpY2ggdGhlIGJ1cm4gbWVzc2FnZSB3aWxsIGJlIGF0dGVzdGVkLgoqIGBob29rX2RhdGFgIC0gSG9vayBkYXRhIHRvIGFwcGVuZCB0byBidXJuIG1lc3NhZ2UgZm9yIGludGVycHJldGF0aW9uIG9uIGRlc3RpbmF0aW9uIGRvbWFpbi4KCiMgRXJyb3JzCgoqIGBIb3N0RXJyb3I6IEVycm9yKENvbnRyYWN0LCAjMTAwMClgIOKAkyBDb250cmFjdCBpcyBwYXVzZWQgKGBFbmZvcmNlZFBhdXNlZGApLgoqIGBIb3N0AAAAGmRlcG9zaXRfZm9yX2J1cm5fd2l0aF9ob29rAAAAAAAJAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAASZGVzdGluYXRpb25fZG9tYWluAAAAAAAEAAAAAAAAAA5taW50X3JlY2lwaWVudAAAAAAD7gAAACAAAAAAAAAACmJ1cm5fdG9rZW4AAAAAABMAAAAAAAAAEmRlc3RpbmF0aW9uX2NhbGxlcgAAAAAD7gAAACAAAAAAAAAAB21heF9mZWUAAAAACwAAAAAAAAAWbWluX2ZpbmFsaXR5X3RocmVzaG9sZAAAAAAABAAAAAAAAAAJaG9va19kYXRhAAAAAAAADgAAAAA=",
        "AAAAAAAAAAAAAAAaZ2V0X3JlbW90ZV90b2tlbl9tZXNzZW5nZXIAAAAAAAEAAAAAAAAABmRvbWFpbgAAAAAABAAAAAEAAAPoAAAD7gAAACA=",
        "AAAAAAAAARJSZXR1cm5zIHRoZSBhZGRyZXNzIG9mIHRoZSBsb2NhbCBNZXNzYWdlVHJhbnNtaXR0ZXIgY29udHJhY3QuCgojIEFyZ3VtZW50cwoKKiBgZWAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCgojIFJldHVybnMKClRoZSBhZGRyZXNzIG9mIHRoZSBsb2NhbCBNZXNzYWdlVHJhbnNtaXR0ZXIgY29udHJhY3QuCgojIEVycm9ycwoKKiBbYFRva2VuTWVzc2VuZ2VyTWludGVyRXJyb3I6OkxvY2FsTWVzc2FnZVRyYW5zbWl0dGVyTm90U2V0YF0g4oCTIElmIG5vdCBzZXQuAAAAAAAdZ2V0X2xvY2FsX21lc3NhZ2VfdHJhbnNtaXR0ZXIAAAAAAAAAAAAAAQAAABM=",
        "AAAAAAAABABIYW5kbGVzIGFuIGluY29taW5nIGZpbmFsaXplZCBtZXNzYWdlIHJlY2VpdmVkIGJ5IHRoZSBsb2NhbCBNZXNzYWdlVHJhbnNtaXR0ZXIsCmFuZCB0YWtlcyB0aGUgYXBwcm9wcmlhdGUgYWN0aW9uLiBGb3IgYSBidXJuIG1lc3NhZ2UsIG1pbnRzIHRoZSBhc3NvY2lhdGVkIHRva2VuCnRvIHRoZSByZXF1ZXN0ZWQgcmVjaXBpZW50IG9uIHRoZSBsb2NhbCBkb21haW4uCgojIEFyZ3VtZW50cwoKKiBgZWAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCiogYHNvdXJjZV9kb21haW5gIC0gVGhlIGRvbWFpbiB3aGVyZSB0aGUgbWVzc2FnZSBvcmlnaW5hdGVkIGZyb20uCiogYHNlbmRlcmAgLSBUaGUgc2VuZGVyIG9mIHRoZSBtZXNzYWdlIChyZW1vdGUgVG9rZW5NZXNzZW5nZXIpLgoqIGBmaW5hbGl0eV90aHJlc2hvbGRfZXhlY3V0ZWRgIC0gVGhlIGxldmVsIG9mIGZpbmFsaXR5IGF0IHdoaWNoIHRoZSBtZXNzYWdlIHdhcyBhdHRlc3RlZCB0by4KKiBgbWVzc2FnZV9ib2R5YCAtIFRoZSBtZXNzYWdlIGJvZHkgYnl0ZXMgKGJ1cm4gbWVzc2FnZSkuCgojIFJldHVybnMKCmB0cnVlYCBpZiBzdWNjZXNzZnVsLgoKIyBFcnJvcnMKCiogYEhvc3RFcnJvcjogRXJyb3IoQ29udHJhY3QsICMxMDAwKWAg4oCTIENvbnRyYWN0IGlzIHBhdXNlZCAoYEVuZm9yY2VkUGF1c2VkYCkuCiogYEhvc3RFcnJvcjogRXJyb3IoQXV0aCwgSW52YWxpZEFjdGlvbilgIOKAkyBBdXRob3JpemF0aW9uIGZyb20gdGhlIGxvY2FsIE1lc3NhZ2VUcmFuc21pdHRlciBmYWlscy4KKiBbYFJlbW90ZVRva2VuTWVzc2VuZ2VyRXJyb3I6OlJlbW90ZVRva2VuTWVzc2VuZ2VyTm90UmVnaXN0ZXJlZGBdIOKAkyBTZW5kZXIgaXMgbm90IGEgcmVnaXN0ZXJlZCByZW1vdGUgdG9rZW4gbWVzc2VuZ2VyLgoqIFtgVG9rZW5NZXNzZW5nZXJNaW50ZXJFcnJvcjo6SW52YWxpZEJ1cm5NZXNzYWdlRm9ybWF0YF0g4oCTIEJ1cm4gbWVzc2FnZSBmb3JtYXQgaXMgaW52YWxpZC4KKiBbYFRva2VuTWVzc2VuZ2VyTWludGVyRXJyb3I6OkluAAAAHWhhbmRsZV9yZWN2X2ZpbmFsaXplZF9tZXNzYWdlAAAAAAAABAAAAAAAAAANc291cmNlX2RvbWFpbgAAAAAAAAQAAAAAAAAABnNlbmRlcgAAAAAD7gAAACAAAAAAAAAAG2ZpbmFsaXR5X3RocmVzaG9sZF9leGVjdXRlZAAAAAAEAAAAAAAAAAxtZXNzYWdlX2JvZHkAAAAOAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAdcmVtb3ZlX3JlbW90ZV90b2tlbl9tZXNzZW5nZXIAAAAAAAABAAAAAAAAAAZkb21haW4AAAAAAAQAAAAA",
        "AAAAAAAAAAAAAAAfZ2V0X21heF9idXJuX2Ftb3VudF9wZXJfbWVzc2FnZQAAAAABAAAAAAAAAAtsb2NhbF90b2tlbgAAAAATAAAAAQAAA+gAAAAL",
        "AAAAAAAABABIYW5kbGVzIGFuIGluY29taW5nIHVuZmluYWxpemVkIG1lc3NhZ2UgcmVjZWl2ZWQgYnkgdGhlIGxvY2FsIE1lc3NhZ2VUcmFuc21pdHRlciwKYW5kIHRha2VzIHRoZSBhcHByb3ByaWF0ZSBhY3Rpb24uIEZvciBhIGJ1cm4gbWVzc2FnZSwgbWludHMgdGhlIGFzc29jaWF0ZWQgdG9rZW4KdG8gdGhlIHJlcXVlc3RlZCByZWNpcGllbnQgb24gdGhlIGxvY2FsIGRvbWFpbiwgbGVzcyBmZWVzLgpGZWVzIGFyZSBzZXBhcmF0ZWx5IG1pbnRlZCB0byB0aGUgY3VycmVudGx5IHNldCBgZmVlX3JlY2lwaWVudGAgYWRkcmVzcy4KCiMgQXJndW1lbnRzCgoqIGBlYCAtIEFjY2VzcyB0byB0aGUgU29yb2JhbiBlbnZpcm9ubWVudC4KKiBgc291cmNlX2RvbWFpbmAgLSBUaGUgZG9tYWluIHdoZXJlIHRoZSBtZXNzYWdlIG9yaWdpbmF0ZWQgZnJvbS4KKiBgc2VuZGVyYCAtIFRoZSBzZW5kZXIgb2YgdGhlIG1lc3NhZ2UgKHJlbW90ZSBUb2tlbk1lc3NlbmdlcikuCiogYGZpbmFsaXR5X3RocmVzaG9sZF9leGVjdXRlZGAgLSBUaGUgbGV2ZWwgb2YgZmluYWxpdHkgYXQgd2hpY2ggdGhlIG1lc3NhZ2Ugd2FzIGF0dGVzdGVkIHRvLgoqIGBtZXNzYWdlX2JvZHlgIC0gVGhlIG1lc3NhZ2UgYm9keSBieXRlcyAoYnVybiBtZXNzYWdlKS4KCiMgUmV0dXJucwoKYHRydWVgIGlmIHN1Y2Nlc3NmdWwuCgojIEVycm9ycwoKKiBgSG9zdEVycm9yOiBFcnJvcihDb250cmFjdCwgIzEwMDApYCDigJMgQ29udHJhY3QgaXMgcGF1c2VkIChgRW5mb3JjZWRQYXVzZWRgKS4KKiBgSG9zdEVycm9yOiBFcnJvcihBdXRoLCBJbnZhbGlkQWN0aW9uKWAg4oCTIEF1dGhvcml6YXRpb24gZnJvbSB0aGUgbG9jYWwgTWVzc2FnZVRyYW5zbWl0dGVyIGZhaWxzLgoqIFtgUmVtb3RlVG9rZW5NZXNzZW5nZXJFcnJvcjo6UmVtb3RlVG9rZW5NZXNzZW5nZXJOb3RSZWdpc3RlcmVkYF0g4oCTIFNlbmRlciBpcyBub3QgYSByZWdpc3RlcmVkIHJlbW90ZSB0b2tlbiBtZXNzZW5nZXIuCiogW2BUb2tlbk1lc3Nlbmdlck1pbnRlckVycm9yOjpVbnN1cHBvcnRlAAAAH2hhbmRsZV9yZWN2X3VuZmluYWxpemVkX21lc3NhZ2UAAAAABAAAAAAAAAANc291cmNlX2RvbWFpbgAAAAAAAAQAAAAAAAAABnNlbmRlcgAAAAAD7gAAACAAAAAAAAAAG2ZpbmFsaXR5X3RocmVzaG9sZF9leGVjdXRlZAAAAAAEAAAAAAAAAAxtZXNzYWdlX2JvZHkAAAAOAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAfc2V0X21heF9idXJuX2Ftb3VudF9wZXJfbWVzc2FnZQAAAAACAAAAAAAAAAtsb2NhbF90b2tlbgAAAAATAAAAAAAAABZidXJuX2xpbWl0X3Blcl9tZXNzYWdlAAAAAAALAAAAAA==",
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
        denylist: this.txFromJSON<null>,
        get_admin: this.txFromJSON<Option<string>>,
        get_owner: this.txFromJSON<Option<string>>,
        get_pauser: this.txFromJSON<Option<string>>,
        get_min_fee: this.txFromJSON<i128>,
        get_rescuer: this.txFromJSON<Option<string>>,
        set_min_fee: this.txFromJSON<null>,
        un_denylist: this.txFromJSON<null>,
        accept_admin: this.txFromJSON<null>,
        rescue_sep41: this.txFromJSON<null>,
        is_denylisted: this.txFromJSON<boolean>,
        update_pauser: this.txFromJSON<null>,
        get_denylister: this.txFromJSON<Option<string>>,
        transfer_admin: this.txFromJSON<null>,
        update_rescuer: this.txFromJSON<null>,
        get_local_token: this.txFromJSON<Option<string>>,
        link_token_pair: this.txFromJSON<null>,
        accept_ownership: this.txFromJSON<null>,
        deposit_for_burn: this.txFromJSON<null>,
        get_fee_recipient: this.txFromJSON<Option<string>>,
        get_pending_admin: this.txFromJSON<Option<string>>,
        get_pending_owner: this.txFromJSON<Option<string>>,
        set_fee_recipient: this.txFromJSON<null>,
        unlink_token_pair: this.txFromJSON<null>,
        update_denylister: this.txFromJSON<null>,
        get_min_fee_amount: this.txFromJSON<i128>,
        transfer_ownership: this.txFromJSON<null>,
        get_token_controller: this.txFromJSON<Option<string>>,
        set_token_controller: this.txFromJSON<null>,
        get_min_fee_controller: this.txFromJSON<Option<string>>,
        get_swap_minter_config: this.txFromJSON<Option<SwapMinterConfig>>,
        set_min_fee_controller: this.txFromJSON<null>,
        set_swap_minter_config: this.txFromJSON<null>,
        get_message_body_version: this.txFromJSON<u32>,
        get_token_decimal_config: this.txFromJSON<Option<TokenDecimalConfig>>,
        set_token_decimal_config: this.txFromJSON<null>,
        remove_swap_minter_config: this.txFromJSON<null>,
        add_remote_token_messenger: this.txFromJSON<null>,
        deposit_for_burn_with_hook: this.txFromJSON<null>,
        get_remote_token_messenger: this.txFromJSON<Option<Buffer>>,
        get_local_message_transmitter: this.txFromJSON<string>,
        handle_recv_finalized_message: this.txFromJSON<boolean>,
        remove_remote_token_messenger: this.txFromJSON<null>,
        get_max_burn_amount_per_message: this.txFromJSON<Option<i128>>,
        handle_recv_unfinalized_message: this.txFromJSON<boolean>,
        set_max_burn_amount_per_message: this.txFromJSON<null>
  }
}