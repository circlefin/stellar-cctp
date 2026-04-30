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
//! Hook data parsing for CCTP Forwarder operations.
//!
//! Hook data format (ABI-encoded style):
//! - bytes 0-23: Magic bytes "cctp-forward" (optional - set to 0 to opt out of forwarding by Circle.)
//! - bytes 24-27: Circle Hook Data Version ID (Set to 0 for this use-case)
//! - bytes 28-31: Length of Circle Hook Data (Set to length of forward_recipient for this use-case)
//! - bytes 32+: forward_recipient strkey (variable length, UTF-8 encoded)

#[cfg(test)]
#[path = "message_test.rs"]
mod test;

use cctp_utils::{bytes::read_u32, BurnMessageV2, MessageV2};
use soroban_sdk::{panic_with_error, Bytes, BytesN, Env, MuxedAddress};
use stellar_utils::address_to_bytes32;

use crate::{storage, CctpForwarderError};

/// Expected hook data version
const HOOK_VERSION: u32 = 0;

/// Byte offsets within the hook data
const VERSION_OFFSET: u32 = 24;
const LENGTH_OFFSET: u32 = 28;
const FORWARD_RECIPIENT_OFFSET: u32 = 32;

/// Minimum hook data length to read header (version + length fields)
const MIN_HEADER_LENGTH: u32 = FORWARD_RECIPIENT_OFFSET;

/// Parsed hook data for CCTP Forwarder mint and forward operations.
pub struct CctpForwarderHookData {
    /// The recipient to forward tokens to
    pub forward_recipient: MuxedAddress,
}

/// Parses and validates the hook data from a CCTP message.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `hook_data` - The unvalidated hook data bytes.
///
/// # Errors
///
/// * [`CctpForwarderError::HookDataTooShort`] – If the hook data is too short.
/// * [`CctpForwarderError::InvalidHookVersion`] – If the hook data version is unsupported.
/// * [`CctpForwarderError::InvalidForwardRecipient`] – If the strkey cannot be parsed.
fn validate_hook_data(e: &Env, hook_data: &Bytes) -> CctpForwarderHookData {
    // Validate minimum length to read header fields
    if hook_data.len() < MIN_HEADER_LENGTH {
        panic_with_error!(e, CctpForwarderError::HookDataTooShort);
    }

    // Read and validate version
    let hook_version = read_u32(hook_data, VERSION_OFFSET)
        .unwrap_or_else(|_| panic_with_error!(e, CctpForwarderError::HookDataTooShort));
    if hook_version != HOOK_VERSION {
        panic_with_error!(e, CctpForwarderError::InvalidHookVersion);
    }

    // Read forward_recipient length and validate hook data contains the full recipient
    let forward_recipient_length = read_u32(hook_data, LENGTH_OFFSET)
        .unwrap_or_else(|_| panic_with_error!(e, CctpForwarderError::HookDataTooShort));
    let min_hook_length = FORWARD_RECIPIENT_OFFSET
        .checked_add(forward_recipient_length)
        .unwrap_or_else(|| panic_with_error!(e, CctpForwarderError::HookDataTooShort));
    if hook_data.len() < min_hook_length {
        panic_with_error!(e, CctpForwarderError::HookDataTooShort);
    }

    let strkey_bytes = hook_data.slice(FORWARD_RECIPIENT_OFFSET..min_hook_length);
    let forward_recipient = MuxedAddress::from_string_bytes(&strkey_bytes);

    CctpForwarderHookData { forward_recipient }
}

/// Data extracted and validated from a CCTP message.
pub struct ValidatedMessageData {
    pub source_domain: u32,
    pub burn_token: BytesN<32>,
    pub forward_recipient: MuxedAddress,
}

/// Validates the CCTP message and burn message format.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `message` - The raw CCTP message bytes.
///
/// # Returns
///
/// Validated message data containing source domain, burn token, and forward recipient.
///
/// # Errors
///
/// * [`CctpForwarderError::InvalidMessageFormat`] – If the message format is invalid.
/// * [`CctpForwarderError::UnsupportedMessageVersion`] – If the message version is unsupported.
/// * [`CctpForwarderError::InvalidBurnMessageFormat`] – If the burn message format is invalid.
/// * [`CctpForwarderError::UnsupportedBurnMessageVersion`] – If the burn message version is unsupported.
/// * [`CctpForwarderError::InvalidRecipient`] – If the recipient is not the TokenMessengerMinter.
/// * [`CctpForwarderError::InvalidMintRecipient`] – If the mint recipient is not this contract.
/// * [`CctpForwarderError::HookDataTooShort`] – If the hook data is too short.
/// * [`CctpForwarderError::InvalidHookVersion`] – If the hook data version is unsupported.
/// * [`CctpForwarderError::InvalidForwardRecipient`] – If the forward recipient strkey is invalid.
pub fn validate_cctp_message(e: &Env, message: &Bytes) -> ValidatedMessageData {
    MessageV2::validate_format(message)
        .unwrap_or_else(|_| panic_with_error!(e, CctpForwarderError::InvalidMessageFormat));

    let expected_message_version = storage::get_expected_msg_version(e);
    let message_version = MessageV2::get_version(message)
        .unwrap_or_else(|_| panic_with_error!(e, CctpForwarderError::InvalidMessageFormat));
    if message_version != expected_message_version {
        panic_with_error!(e, CctpForwarderError::UnsupportedMessageVersion);
    }

    let message_body = MessageV2::get_message_body(message);

    BurnMessageV2::validate_format(&message_body)
        .unwrap_or_else(|_| panic_with_error!(e, CctpForwarderError::InvalidBurnMessageFormat));

    let expected_burn_message_version = storage::get_expected_burn_msg_version(e);
    let burn_message_version = BurnMessageV2::get_version(&message_body)
        .unwrap_or_else(|_| panic_with_error!(e, CctpForwarderError::InvalidBurnMessageFormat));
    if burn_message_version != expected_burn_message_version {
        panic_with_error!(e, CctpForwarderError::UnsupportedBurnMessageVersion);
    }

    let recipient = MessageV2::get_recipient(message)
        .unwrap_or_else(|_| panic_with_error!(e, CctpForwarderError::InvalidMessageFormat));
    let token_messenger_minter = storage::get_token_messenger_minter(e);
    let tmm_bytes32 = address_to_bytes32(&token_messenger_minter)
        .unwrap_or_else(|| panic_with_error!(e, CctpForwarderError::InvalidRecipient));
    if recipient != tmm_bytes32 {
        panic_with_error!(e, CctpForwarderError::InvalidRecipient);
    }

    let mint_recipient = BurnMessageV2::get_mint_recipient(&message_body)
        .unwrap_or_else(|_| panic_with_error!(e, CctpForwarderError::InvalidMintRecipient));
    let contract_address = e.current_contract_address();
    let contract_bytes32 = address_to_bytes32(&contract_address)
        .unwrap_or_else(|| panic_with_error!(e, CctpForwarderError::InvalidMintRecipient));
    if mint_recipient != contract_bytes32 {
        panic_with_error!(e, CctpForwarderError::InvalidMintRecipient);
    }

    let source_domain = MessageV2::get_source_domain(message)
        .unwrap_or_else(|_| panic_with_error!(e, CctpForwarderError::InvalidMessageFormat));
    let burn_token = BurnMessageV2::get_burn_token(&message_body)
        .unwrap_or_else(|_| panic_with_error!(e, CctpForwarderError::InvalidBurnMessageFormat));

    let hook_data_bytes = BurnMessageV2::get_hook_data(&message_body);
    let hook_data = validate_hook_data(e, &hook_data_bytes);

    ValidatedMessageData {
        source_domain,
        burn_token,
        forward_recipient: hook_data.forward_recipient,
    }
}
