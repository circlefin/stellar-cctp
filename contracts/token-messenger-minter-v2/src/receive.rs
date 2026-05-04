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

//! Receive and mint operations for TokenMessengerMinter.
//!
//! This module contains the implementation logic for `handle_recv_finalized_message`
//! and `handle_recv_unfinalized_message` operations.

use cctp_roles::{fee_recipient, token_controller};
use cctp_utils::{
    to_local_amount, u256_to_positive_i128, u256_to_u32, BurnMessageV2, TokenDecimalPair,
};
use common_roles::simple_role;
use soroban_sdk::{
    address_payload::AddressPayload, panic_with_error, token::TokenClient, Address, Bytes, BytesN,
    Env,
};
use stablecoin_interfaces::SwapMinterClient;

use crate::storage;
use crate::TokenMessengerMinterError;

/// Represents the converted local amounts from canonical decimal format.
pub struct LocalAmounts {
    /// Amount in local decimal format
    pub amount: i128,
    /// Max fee in local decimal format
    pub max_fee: i128,
    /// Fee in local decimal format
    pub fee: i128,
}

/// Represents a parsed burn message with extracted fields.
struct ParsedBurnMessageV2 {
    /// The address of the token burned on the source domain (as bytes32)
    pub burn_token: BytesN<32>,
    /// The address to receive minted tokens on this domain
    pub mint_recipient: Address,
    /// The amount of tokens in the burn message (canonical decimals)
    pub canonical_amount: i128,
    /// The max fee specified by the sender (canonical decimals)
    pub canonical_max_fee: i128,
    /// The fee executed (to be sent to fee_recipient, canonical decimals)
    pub canonical_fee: i128,
}

/// Internal implementation for handling received burn messages.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `remote_domain` - The domain where the message originated from.
/// * `message_body` - The burn message bytes.
///
/// # Returns
///
/// `true` if successful.
///
/// # Errors
///
/// * [`TokenMessengerMinterError::InvalidBurnMessageFormat`] – Burn message is malformed.
/// * [`TokenMessengerMinterError::InvalidBurnMessageVersion`] – Version mismatch.
/// * [`TokenMessengerMinterError::MessageExpired`] – Message has expired.
/// * [`TokenMessengerMinterError::FeeEqualsOrExceedsAmount`] – Fee equals or exceeds amount.
/// * [`TokenMessengerMinterError::FeeExceedsMaxFee`] – Fee exceeds max fee.
/// * [`TokenMessengerMinterError::MintTokenNotSupported`] – No local token is linked.
/// * [`TokenMessengerMinterError::DecimalConversionFailed`] – If decimal conversion overflows.
pub fn handle_receive_message_impl(e: &Env, remote_domain: u32, message_body: &Bytes) -> bool {
    // Parse and validate message structure (format, version, expiration)
    // Amounts returned are in canonical decimals from the source chain
    let ParsedBurnMessageV2 {
        burn_token,
        mint_recipient,
        canonical_amount,
        canonical_max_fee,
        canonical_fee,
    } = parse_burn_message(e, message_body);

    // Get the local token for the remote token/domain pair
    let mint_token = token_controller::get_local_token(e, remote_domain, &burn_token)
        .unwrap_or_else(|| panic_with_error!(e, TokenMessengerMinterError::MintTokenNotSupported));

    // Convert canonical amounts to local decimal format if configured
    // For example, 6 decimal USDC from other chains -> 7 decimal Stellar USDC
    let LocalAmounts {
        amount: local_amount,
        max_fee: local_max_fee,
        fee: local_fee,
    } = convert_to_local_amounts(
        e,
        &mint_token,
        canonical_amount,
        canonical_max_fee,
        canonical_fee,
    );

    if local_fee != 0 && local_fee >= local_amount {
        panic_with_error!(e, TokenMessengerMinterError::FeeEqualsOrExceedsAmount);
    }
    if local_fee > local_max_fee {
        panic_with_error!(e, TokenMessengerMinterError::FeeExceedsMaxFee);
    }

    // Calculate amounts: mint (amount - fee) to recipient, fee to fee_recipient
    let local_amount_to_mint = local_amount.checked_sub(local_fee).unwrap_or_else(|| {
        panic_with_error!(e, TokenMessengerMinterError::DecimalConversionFailed)
    });

    // Calculate canonical amount to mint (amount - fee) for event emission
    let canonical_amount_to_mint = canonical_amount
        .checked_sub(canonical_fee)
        .unwrap_or_else(|| panic_with_error!(e, TokenMessengerMinterError::AmountOverflow));

    mint_and_withdraw(
        e,
        &mint_token,
        &mint_recipient,
        local_amount_to_mint,
        local_fee,
    );

    // Emit event with canonical amounts for cross-chain consistency
    crate::emit_mint_and_withdraw(
        e,
        &mint_recipient,
        canonical_amount_to_mint,
        &mint_token,
        canonical_fee,
    );

    true
}

/// Converts canonical amounts to local decimal format based on token configuration.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `mint_token` - Address of the local token to mint.
/// * `canonical_amount` - Amount in canonical decimal format (from burn message).
/// * `canonical_max_fee` - Max fee in canonical decimal format (from burn message).
/// * `canonical_fee` - Fee in canonical decimal format (from burn message).
///
/// # Returns
///
/// A `LocalAmounts` struct containing the amount, max_fee, and fee converted to local decimal format.
///
/// # Errors
///
/// * [`TokenMessengerMinterError::TokenDecimalConfigNotSet`] – If no decimal config exists.
/// * [`TokenMessengerMinterError::DecimalConversionFailed`] – If conversion overflows.
fn convert_to_local_amounts(
    e: &Env,
    mint_token: &Address,
    canonical_amount: i128,
    canonical_max_fee: i128,
    canonical_fee: i128,
) -> LocalAmounts {
    // Require explicit config even if local and canonical decimals are the same to prevent accidental missed conversions
    let config = token_controller::get_token_decimal_config(e, mint_token).unwrap_or_else(|| {
        panic_with_error!(e, TokenMessengerMinterError::TokenDecimalConfigNotSet)
    });

    let decimal_pair = TokenDecimalPair {
        local_decimals: config.local_decimals,
        canonical_decimals: config.canonical_decimals,
    };

    // Convert canonical amount to local format (e.g., 123456 -> 1234560 for 6->7 decimals)
    let amount = to_local_amount(canonical_amount, decimal_pair).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::DecimalConversionFailed)
    });

    // Convert canonical max_fee to local format
    let max_fee = to_local_amount(canonical_max_fee, decimal_pair).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::DecimalConversionFailed)
    });

    // Convert canonical fee to local format
    let fee = to_local_amount(canonical_fee, decimal_pair).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::DecimalConversionFailed)
    });

    LocalAmounts {
        amount,
        max_fee,
        fee,
    }
}

/// Parses and validates the structure of a BurnMessage, extracting relevant fields.
///
/// This function validates the message format, version, and expiration, but does NOT
/// validate fee constraints. Fee validation is done after decimal conversion in the
/// calling function to ensure validation uses local amounts.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `message_body` - The serialized burn message bytes.
///
/// # Returns
///
/// A `ParsedBurnMessage` containing the validated structure and extracted fields.
/// Amounts are in canonical decimals from the source chain.
///
/// # Errors
///
/// * [`TokenMessengerMinterError::InvalidBurnMessageFormat`] – Burn message is malformed.
/// * [`TokenMessengerMinterError::InvalidBurnMessageVersion`] – Version mismatch.
/// * [`TokenMessengerMinterError::MessageExpired`] – Message has expired.
/// * [`TokenMessengerMinterError::AmountOverflow`] – Amount, max_fee, or fee exceeds i128::MAX.
fn parse_burn_message(e: &Env, message_body: &Bytes) -> ParsedBurnMessageV2 {
    // Validate message format
    BurnMessageV2::validate_format(message_body).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::InvalidBurnMessageV2Format)
    });

    // Validate version
    let message_version = BurnMessageV2::get_version(message_body).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::InvalidBurnMessageV2Format)
    });
    let expected_version = storage::get_message_body_version(e);
    if message_version != expected_version {
        panic_with_error!(e, TokenMessengerMinterError::InvalidBurnMessageVersion);
    }

    // Enforce message expiration.
    let expiration_block_u256 = BurnMessageV2::get_expiration_block(e, message_body)
        .unwrap_or_else(|_| {
            panic_with_error!(e, TokenMessengerMinterError::InvalidBurnMessageV2Format)
        });

    let expiration_block = u256_to_u32(&expiration_block_u256).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::InvalidBurnMessageV2Format)
    });
    if expiration_block != 0 && expiration_block <= e.ledger().sequence() {
        panic_with_error!(e, TokenMessengerMinterError::MessageExpired);
    }

    // Extract fields (amounts are in canonical decimals)
    let burn_token = BurnMessageV2::get_burn_token(message_body).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::InvalidBurnMessageV2Format)
    });

    let mint_recipient_bytes32 =
        BurnMessageV2::get_mint_recipient(message_body).unwrap_or_else(|_| {
            panic_with_error!(e, TokenMessengerMinterError::InvalidBurnMessageV2Format)
        });

    let canonical_amount_u256 = BurnMessageV2::get_amount(e, message_body).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::InvalidBurnMessageV2Format)
    });

    let canonical_max_fee_u256 = BurnMessageV2::get_max_fee(e, message_body).unwrap_or_else(|_| {
        panic_with_error!(e, TokenMessengerMinterError::InvalidBurnMessageV2Format)
    });

    let canonical_fee_u256 =
        BurnMessageV2::get_fee_executed(e, message_body).unwrap_or_else(|_| {
            panic_with_error!(e, TokenMessengerMinterError::InvalidBurnMessageV2Format)
        });

    let canonical_amount = u256_to_positive_i128(&canonical_amount_u256)
        .unwrap_or_else(|_| panic_with_error!(e, TokenMessengerMinterError::AmountOverflow));
    let canonical_max_fee = u256_to_positive_i128(&canonical_max_fee_u256)
        .unwrap_or_else(|_| panic_with_error!(e, TokenMessengerMinterError::AmountOverflow));
    let canonical_fee = u256_to_positive_i128(&canonical_fee_u256)
        .unwrap_or_else(|_| panic_with_error!(e, TokenMessengerMinterError::AmountOverflow));

    // The mint_recipient is expected to be a Stellar contract address.
    let mint_recipient_payload = AddressPayload::ContractIdHash(mint_recipient_bytes32.clone());
    let mint_recipient = Address::from_payload(e, mint_recipient_payload);

    ParsedBurnMessageV2 {
        burn_token,
        mint_recipient,
        canonical_amount,
        canonical_max_fee,
        canonical_fee,
    }
}

/// Mints tokens to the recipient and optionally to the fee recipient via the SwapMinter.
///
/// This function retrieves the SwapMinter configuration for the token, approves the allowance
/// asset for the swap operation, and calls `swap_mint` on the SwapMinter contract.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `mint_token` - The address of the local token (used to look up the SwapMinter config).
/// * `mint_recipient` - The address to receive the minted tokens.
/// * `amount` - The amount of tokens to mint to the recipient.
/// * `fee` - The fee to mint to the fee recipient (if > 0).
///
/// # Errors
///
/// * [`TokenMessengerMinterError::SwapMinterConfigNotSet`] – If no SwapMinter config exists for the token.
/// * [`TokenMessengerMinterError::FeeRecipientNotSet`] – If fee > 0 and the fee recipient role is not set.
fn mint_and_withdraw(
    e: &Env,
    mint_token: &Address,
    mint_recipient: &Address,
    amount: i128,
    fee: i128,
) {
    // Get the swap minter configuration for this token
    let config = token_controller::get_swap_minter_config(e, mint_token)
        .unwrap_or_else(|| panic_with_error!(e, TokenMessengerMinterError::SwapMinterConfigNotSet));

    let swap_minter_client = SwapMinterClient::new(e, &config.swap_minter);
    let allow_asset_client = TokenClient::new(e, &config.allow_asset);
    let this_contract = e.current_contract_address();

    // Approve the swap minter to spend the allowance asset for the total amount (amount + fee)
    let total_amount = amount
        .checked_add(fee)
        .unwrap_or_else(|| panic_with_error!(e, TokenMessengerMinterError::AmountOverflow));
    allow_asset_client.approve(
        &this_contract,
        &config.swap_minter,
        &total_amount,
        &e.ledger().sequence(),
    );

    // Mint tokens to recipient via swap_mint
    swap_minter_client.swap_mint(mint_recipient, &amount, &this_contract);

    // If there's a fee, mint it to the fee recipient
    if fee > 0 {
        let fee_recipient_addr = simple_role::try_get_role(e, fee_recipient::FEE_RECIPIENT)
            .unwrap_or_else(|| panic_with_error!(e, TokenMessengerMinterError::FeeRecipientNotSet));
        swap_minter_client.swap_mint(&fee_recipient_addr, &fee, &this_contract);
    }
}
