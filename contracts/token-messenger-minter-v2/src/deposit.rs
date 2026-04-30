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

//! Deposit and burn operations for TokenMessengerMinter.
//!
//! This module contains the implementation logic for `deposit_for_burn` and
//! `deposit_for_burn_with_hook` operations.

use cctp_interfaces::RelayerClient;
use cctp_roles::{min_fee_controller, remote_token_messenger, token_controller};
use cctp_utils::bytes::positive_i128_to_u256;
use cctp_utils::{normalize_for_burn, to_canonical_amount, BurnMessageV2, TokenDecimalPair};
use soroban_sdk::{panic_with_error, token::TokenClient, Address, Bytes, BytesN, Env};
use stellar_utils::{address_to_bytes32, is_zero_bytes};

use crate::storage;
use crate::TokenMessengerMinterError;

/// Represents a normalized amount with its canonical conversion and decimal configuration.
pub struct NormalizedAmount {
    /// Amount in local decimals after dust removal
    pub local_amount: i128,
    /// Amount converted to canonical decimals
    pub canonical_amount: i128,
    /// The decimal pair used for conversion
    pub decimal_pair: TokenDecimalPair,
}

/// Represents the calculated burn amounts for deposit operations.
pub struct BurnAmounts {
    /// Amount to burn locally (normalized, dust removed)
    pub local_burn_amount: i128,
    /// Amount for the burn message (in canonical decimals)
    pub canonical_burn_amount: i128,
    /// Max fee for the burn message (in canonical decimals)
    pub canonical_max_fee: i128,
}

/// Internal implementation for deposit_for_burn operations.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `amount` - Amount of tokens to burn in local decimals (must be non-zero).
/// * `caller` - The address of the caller depositing tokens.
/// * `destination_domain` - Destination domain to receive message on.
/// * `mint_recipient` - Address of mint recipient on destination domain (as bytes32).
/// * `burn_token` - Token to burn.
/// * `destination_caller` - Authorized caller on the destination domain (as bytes32).
/// * `max_fee` - Maximum fee to pay on the destination domain in local decimals.
/// * `min_finality_threshold` - The minimum finality at which the burn message will be attested.
/// * `hook_data` - Optional hook data for the destination domain.
///
/// # Errors
///
/// * [`TokenMessengerMinterError::AmountMustBeNonzero`] – If amount is zero or negative.
/// * [`TokenMessengerMinterError::MintRecipientMustBeNonzero`] – If mint_recipient is zero.
/// * [`TokenMessengerMinterError::MaxFeeMustBeNonNegative`] – If max_fee is negative.
/// * [`TokenMessengerMinterError::MaxFeeMustBeLessThanAmount`] – If canonical max_fee >= canonical amount.
/// * [`TokenMessengerMinterError::InsufficientMaxFee`] – If canonical max_fee < min fee for canonical amount.
/// * [`TokenMessengerMinterError::NoTokenMessengerForDomain`] – If no token messenger registered.
/// * [`TokenMessengerMinterError::AddressConversionFailed`] – If address conversion fails.
#[allow(clippy::too_many_arguments)]
pub fn deposit_for_burn_impl(
    e: &Env,
    caller: &Address,
    amount: i128,
    destination_domain: u32,
    mint_recipient: &BytesN<32>,
    burn_token: &Address,
    destination_caller: &BytesN<32>,
    max_fee: i128,
    min_finality_threshold: u32,
    hook_data: &Bytes,
) {
    if amount <= 0 {
        panic_with_error!(e, TokenMessengerMinterError::AmountMustBeNonzero);
    }
    if is_zero_bytes(mint_recipient) {
        panic_with_error!(e, TokenMessengerMinterError::MintRecipientMustBeNonzero);
    }
    if max_fee < 0 {
        panic_with_error!(e, TokenMessengerMinterError::MaxFeeMustBeNonNegative);
    }

    // Handle decimal conversion
    // For Stellar USDC (7 decimals) vs other chains (6 decimals), we normalize the amount
    // to remove dust before burning, leaving the dust in the user's account.
    let BurnAmounts {
        local_burn_amount,
        canonical_burn_amount,
        canonical_max_fee,
    } = calculate_burn_amounts(e, burn_token, amount, max_fee);

    // Validate using canonical amounts
    if canonical_max_fee >= canonical_burn_amount {
        panic_with_error!(e, TokenMessengerMinterError::MaxFeeMustBeLessThanAmount);
    }

    // Verify minimum fee requirement if min_fee is configured
    let min_fee = min_fee_controller::get_min_fee(e, burn_token);
    if min_fee > 0 {
        let canonical_min_fee_amount =
            min_fee_controller::get_min_fee_amount(e, burn_token, canonical_burn_amount);
        if canonical_max_fee < canonical_min_fee_amount {
            panic_with_error!(e, TokenMessengerMinterError::InsufficientMaxFee);
        }
    }

    let destination_token_messenger =
        remote_token_messenger::get_remote_token_messenger(e, destination_domain).unwrap_or_else(
            || panic_with_error!(e, TokenMessengerMinterError::NoTokenMessengerForDomain),
        );

    // Verify burn token is supported and amount is within limits (using local burn amount)
    token_controller::enforce_within_burn_limit(e, burn_token, local_burn_amount);

    let burn_token_bytes32 = address_to_bytes32(burn_token).unwrap_or_else(|| {
        panic_with_error!(e, TokenMessengerMinterError::AddressConversionFailed)
    });
    let message_sender = address_to_bytes32(caller).unwrap_or_else(|| {
        panic_with_error!(e, TokenMessengerMinterError::AddressConversionFailed)
    });

    // Transfer tokens from caller to this contract, then burn
    // Only burn the normalized amount - dust remains with the user
    deposit_and_burn(e, caller, burn_token, local_burn_amount);

    let message_body_version = storage::get_message_body_version(e);

    let canonical_burn_amount_u256 = positive_i128_to_u256(e, canonical_burn_amount)
        .unwrap_or_else(|_| {
            panic_with_error!(e, TokenMessengerMinterError::DecimalConversionFailed)
        });
    let canonical_max_fee_u256 = positive_i128_to_u256(e, canonical_max_fee).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::DecimalConversionFailed)
    });

    let burn_message = BurnMessageV2::format_for_relay(
        e,
        message_body_version,
        burn_token_bytes32,
        mint_recipient.clone(),
        canonical_burn_amount_u256,
        message_sender,
        canonical_max_fee_u256,
        hook_data.clone(),
    );

    let local_message_transmitter = storage::get_local_message_transmitter(e);
    let message_transmitter_client = RelayerClient::new(e, &local_message_transmitter);

    message_transmitter_client.send_message(
        &e.current_contract_address(),
        &destination_domain,
        &destination_token_messenger,
        destination_caller,
        &min_finality_threshold,
        &burn_message,
    );

    // Emit event with canonical amounts for cross-chain consistency
    crate::emit_deposit_for_burn(
        e,
        burn_token,
        canonical_burn_amount,
        caller,
        mint_recipient,
        destination_domain,
        &destination_token_messenger,
        destination_caller,
        canonical_max_fee,
        min_finality_threshold,
        hook_data,
    );
}

/// Normalizes a local amount (stripping dust) and converts it to canonical decimals.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `burn_token` - Address of the token.
/// * `amount` - Amount in local decimal format.
///
/// # Returns
///
/// A [`NormalizedAmount`] containing the normalized local amount, canonical amount,
/// and the decimal pair used for conversion.
///
/// # Errors
///
/// * [`TokenMessengerMinterError::TokenDecimalConfigNotSet`] – If no decimal config exists.
/// * [`TokenMessengerMinterError::DecimalConversionFailed`] – If conversion overflows or
///   decimal configuration is invalid.
pub fn to_canonical_amount_normalized(
    e: &Env,
    burn_token: &Address,
    amount: i128,
) -> NormalizedAmount {
    let config = token_controller::get_token_decimal_config(e, burn_token).unwrap_or_else(|| {
        panic_with_error!(e, TokenMessengerMinterError::TokenDecimalConfigNotSet)
    });

    let decimal_pair = TokenDecimalPair {
        local_decimals: config.local_decimals,
        canonical_decimals: config.canonical_decimals,
    };

    let local_amount = normalize_for_burn(amount, decimal_pair).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::DecimalConversionFailed)
    });

    let canonical_amount = to_canonical_amount(local_amount, decimal_pair).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::DecimalConversionFailed)
    });

    NormalizedAmount {
        local_amount,
        canonical_amount,
        decimal_pair,
    }
}

/// Calculates the local and canonical burn amounts based on decimal configuration.
///
/// - Normalizes the local amount to remove dust (rounds down to nearest canonical-compatible value)
/// - Converts to canonical decimal format for the burn message
/// - Converts max_fee to canonical decimal format
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `burn_token` - Address of the token being burned.
/// * `amount` - Original amount in local decimal format.
/// * `max_fee` - Maximum fee in local decimal format.
///
/// # Returns
///
/// A `BurnAmounts` struct containing:
/// - `local_burn_amount`: Amount to burn locally (normalized, dust removed)
/// - `canonical_burn_amount`: Amount for the burn message (in canonical decimals)
/// - `canonical_max_fee`: Max fee for the burn message (in canonical decimals)
///
/// # Errors
///
/// * [`TokenMessengerMinterError::TokenDecimalConfigNotSet`] – If no decimal config exists.
/// * [`TokenMessengerMinterError::BurnAmountTooSmall`] – If normalized amount is zero.
/// * [`TokenMessengerMinterError::DecimalConversionFailed`] – If conversion overflows.
fn calculate_burn_amounts(
    e: &Env,
    burn_token: &Address,
    amount: i128,
    max_fee: i128,
) -> BurnAmounts {
    let normalized = to_canonical_amount_normalized(e, burn_token, amount);

    if normalized.local_amount <= 0 {
        panic_with_error!(e, TokenMessengerMinterError::BurnAmountTooSmall);
    }

    let canonical_max_fee =
        to_canonical_amount(max_fee, normalized.decimal_pair).unwrap_or_else(|_| {
            panic_with_error!(e, TokenMessengerMinterError::DecimalConversionFailed)
        });

    BurnAmounts {
        local_burn_amount: normalized.local_amount,
        canonical_burn_amount: normalized.canonical_amount,
        canonical_max_fee,
    }
}

/// Transfers tokens from the depositor to this contract and burns them.
///
/// Uses `transfer_from` which requires the caller to have previously approved
/// this contract to spend tokens on their behalf via `token.approve()`.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `burn_token` - Address of the token to burn.
/// * `from` - Address depositing the tokens.
/// * `amount` - Amount of tokens to transfer and burn.
fn deposit_and_burn(e: &Env, from: &Address, burn_token: &Address, amount: i128) {
    let token_client = TokenClient::new(e, burn_token);
    let contract_address = e.current_contract_address();

    token_client.transfer_from(&contract_address, from, &contract_address, &amount);
    token_client.burn(&contract_address, &amount);
}
