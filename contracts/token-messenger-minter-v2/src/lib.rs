/*
 * Copyright 2026 Circle Internet Group, Inc. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#![no_std]

use soroban_sdk::{contracterror, contractevent, Address, Bytes, BytesN, Env};

#[cfg(feature = "entry-points")]
mod contract;
pub mod deposit;
pub mod receive;
pub mod storage;
#[cfg(test)]
mod test;
#[cfg(test)]
pub mod test_utils;

/// Errors for the TokenMessengerMinter contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenMessengerMinterError {
    /// The local message transmitter address has not been set.
    LocalMessageTransmitterNotSet = 7100,
    /// The message body version has not been set.
    MessageBodyVersionNotSet = 7101,
    /// Amount must be greater than zero.
    AmountMustBeNonzero = 7102,
    /// Mint recipient must not be zero.
    MintRecipientMustBeNonzero = 7103,
    /// Max fee must be less than amount.
    MaxFeeMustBeLessThanAmount = 7104,
    /// Max fee is less than the calculated minimum fee.
    InsufficientMaxFee = 7105,
    /// No TokenMessenger is registered for the destination domain.
    NoTokenMessengerForDomain = 7106,
    /// Hook data must not be empty for deposit_for_burn_with_hook.
    HookDataEmpty = 7107,
    /// Failed to convert address to bytes32.
    AddressConversionFailed = 7108,
    /// Caller is not the local message transmitter.
    InvalidMessageTransmitter = 7109,
    /// Remote sender is not a registered token messenger for the domain.
    RemoteTokenMessengerNotRegistered = 7110,
    /// Burn message format is invalid.
    InvalidBurnMessageV2Format = 7111,
    /// Burn message version does not match expected version.
    InvalidBurnMessageVersion = 7112,
    /// Fee equals or exceeds the amount.
    FeeEqualsOrExceedsAmount = 7113,
    /// Fee exceeds the max fee specified in the burn message.
    FeeExceedsMaxFee = 7114,
    /// Mint token is not supported (no local token linked for the remote token/domain).
    MintTokenNotSupported = 7115,
    /// Finality threshold is below the minimum required threshold.
    UnsupportedFinalityThreshold = 7116,
    /// Message has expired and must be re-signed.
    MessageExpired = 7117,
    /// Decimal conversion resulted in zero burn amount.
    BurnAmountTooSmall = 7118,
    /// Decimal conversion failed due to overflow or invalid configuration.
    DecimalConversionFailed = 7119,
    /// Token decimal configuration is not set for this token.
    TokenDecimalConfigNotSet = 7120,
    /// Swap minter configuration is not set for this token.
    SwapMinterConfigNotSet = 7121,
    /// Amount in burn message exceeds i128::MAX.
    AmountOverflow = 7122,
    /// Fee recipient role is not set.
    FeeRecipientNotSet = 7123,
    /// Max fee must not be negative.
    MaxFeeMustBeNonNegative = 7124,
}

// ################## EVENTS ##################

/// Emitted when tokens are deposited for burning and cross-chain transfer.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositForBurn {
    /// Address of the token burned on the source domain
    #[topic]
    pub burn_token: Address,
    /// Amount of tokens burned
    pub amount: i128,
    /// Address that deposited the tokens
    #[topic]
    pub depositor: Address,
    /// Address to receive minted tokens on destination domain (as bytes32)
    pub mint_recipient: BytesN<32>,
    /// Destination domain identifier
    pub destination_domain: u32,
    /// Address of TokenMessenger on destination domain (as bytes32)
    pub destination_token_messenger: BytesN<32>,
    /// Authorized caller on destination domain (as bytes32), or zero for any caller
    pub destination_caller: BytesN<32>,
    /// Maximum fee to pay on destination domain
    pub max_fee: i128,
    /// Minimum finality threshold for attestation
    #[topic]
    pub min_finality_threshold: u32,
    /// Optional hook data for execution on destination domain
    pub hook_data: Bytes,
}

/// Emits a DepositForBurn event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `burn_token` - Address of the token burned.
/// * `amount` - Amount of tokens burned.
/// * `depositor` - Address that deposited the tokens.
/// * `mint_recipient` - Address to receive minted tokens on destination domain.
/// * `destination_domain` - Destination domain identifier.
/// * `destination_token_messenger` - Address of TokenMessenger on destination domain.
/// * `destination_caller` - Authorized caller on destination domain.
/// * `max_fee` - Maximum fee to pay on destination domain.
/// * `min_finality_threshold` - Minimum finality threshold for attestation.
/// * `hook_data` - Optional hook data for execution on destination domain.
///
/// # Events
///
/// * topics - `["deposit_for_burn", burn_token: Address, depositor: Address, min_finality_threshold: u32]`
/// * data - `[amount: i128, mint_recipient: BytesN<32>, destination_domain: u32, destination_token_messenger: BytesN<32>, destination_caller: BytesN<32>, max_fee: i128, hook_data: Bytes]`
#[allow(clippy::too_many_arguments)]
pub fn emit_deposit_for_burn(
    e: &Env,
    burn_token: &Address,
    amount: i128,
    depositor: &Address,
    mint_recipient: &BytesN<32>,
    destination_domain: u32,
    destination_token_messenger: &BytesN<32>,
    destination_caller: &BytesN<32>,
    max_fee: i128,
    min_finality_threshold: u32,
    hook_data: &Bytes,
) {
    DepositForBurn {
        burn_token: burn_token.clone(),
        amount,
        depositor: depositor.clone(),
        mint_recipient: mint_recipient.clone(),
        destination_domain,
        destination_token_messenger: destination_token_messenger.clone(),
        destination_caller: destination_caller.clone(),
        max_fee,
        min_finality_threshold,
        hook_data: hook_data.clone(),
    }
    .publish(e);
}

/// Emitted when tokens are minted to a recipient after receiving a cross-chain message.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintAndWithdraw {
    /// Address that received the minted tokens
    #[topic]
    pub mint_recipient: Address,
    /// Amount of tokens received by `mint_recipient`
    pub amount: i128,
    /// Address of the minted token contract
    #[topic]
    pub mint_token: Address,
    /// Fee collected for the mint operation
    pub fee_collected: i128,
}

/// Emits a MintAndWithdraw event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `mint_recipient` - Address that received the minted tokens.
/// * `amount` - Amount of tokens received by `mint_recipient`.
/// * `mint_token` - Address of the minted token contract.
/// * `fee_collected` - Fee collected for the mint operation.
///
/// # Events
///
/// * topics - `["mint_and_withdraw", mint_recipient: Address, mint_token: Address]`
/// * data - `[amount: i128, fee_collected: i128]`
pub fn emit_mint_and_withdraw(
    e: &Env,
    mint_recipient: &Address,
    amount: i128,
    mint_token: &Address,
    fee_collected: i128,
) {
    MintAndWithdraw {
        mint_recipient: mint_recipient.clone(),
        amount,
        mint_token: mint_token.clone(),
        fee_collected,
    }
    .publish(e);
}
