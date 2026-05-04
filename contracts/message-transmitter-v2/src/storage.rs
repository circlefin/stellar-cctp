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
use soroban_sdk::{contracttype, panic_with_error, Address, Bytes, BytesN, Env};

use crate::{MessageTransmitterError, ValidatedMessage};
use cctp_roles::attestable;
use cctp_utils::MessageV2;
use stellar_utils::storage::ttl::{
    get_and_extend_persistent_ttl, DEFAULT_EXTEND_AMOUNT, DEFAULT_TTL_THRESHOLD,
};
use stellar_utils::{address_to_bytes32, is_zero_bytes};

/// Finality threshold value indicating the message is fully finalized.
pub const FINALITY_THRESHOLD_FINALIZED: u32 = 2000;

/// Storage keys for the Message Transmitter contract.
#[contracttype]
pub enum MessageTransmitterStorageKey {
    /// The local domain identifier for this chain
    LocalDomain,
    /// The message format version
    Version,
    /// The maximum allowed message body size
    MaxMessageBodySize,
    /// Storage for used nonces, keyed by the nonce BytesN<32>
    UsedNonce(BytesN<32>),
}

/// Returns the local domain identifier for this chain.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Returns
///
/// The local domain identifier.
///
/// # Errors
///
/// * [`MessageTransmitterError::LocalDomainNotSet`] – If the local domain has not been set.
pub fn get_local_domain(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&MessageTransmitterStorageKey::LocalDomain)
        .unwrap_or_else(|| panic_with_error!(e, MessageTransmitterError::LocalDomainNotSet))
}

/// Returns the message format version.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Returns
///
/// The message format version.
///
/// # Errors
///
/// * [`MessageTransmitterError::VersionNotSet`] – If the version has not been set.
pub fn get_version(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&MessageTransmitterStorageKey::Version)
        .unwrap_or_else(|| panic_with_error!(e, MessageTransmitterError::VersionNotSet))
}

// ################## MAX MESSAGE BODY SIZE ##################

/// Sets the maximum allowed message body size.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `max_message_body_size` - The maximum allowed message body size.
///
/// # Events
///
/// * topics - `["max_message_body_size_updated"]`
/// * data - `[new_max_message_body_size: u32]`
///
/// # Notes
///
/// * IMPORTANT: This function lacks authorization checks. It is expected to call this function only in the constructor!
pub fn set_max_message_body_size_unchecked(e: &Env, max_message_body_size: u32) {
    e.storage().instance().set(
        &MessageTransmitterStorageKey::MaxMessageBodySize,
        &max_message_body_size,
    );
    crate::emit_max_message_body_size_updated(e, max_message_body_size);
}

/// Returns the maximum allowed message body size.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Returns
///
/// The maximum allowed message body size.
///
/// # Errors
///
/// * [`MessageTransmitterError::MaxMessageBodySizeNotSet`] – If the max message body size has not been set.
pub fn get_max_message_body_size(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&MessageTransmitterStorageKey::MaxMessageBodySize)
        .unwrap_or_else(|| panic_with_error!(e, MessageTransmitterError::MaxMessageBodySizeNotSet))
}

// ################## NONCE MANAGEMENT ##################

/// Checks if a nonce has been used.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `nonce` - The nonce to check.
///
/// # Returns
///
/// `true` if the nonce has been used, `false` otherwise.
pub fn is_nonce_used(e: &Env, nonce: &BytesN<32>) -> bool {
    let key = MessageTransmitterStorageKey::UsedNonce(nonce.clone());
    get_and_extend_persistent_ttl(e, &key, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT)
        .unwrap_or(false)
}

/// Marks a nonce as used.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `nonce` - The nonce to mark as used.
pub fn set_nonce_used(e: &Env, nonce: &BytesN<32>) {
    e.storage().persistent().set(
        &MessageTransmitterStorageKey::UsedNonce(nonce.clone()),
        &true,
    );
}

/// Validates a received message, including the attestation signatures as well
/// as the message contents.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `caller` - The address of the caller.
/// * `message` - The message bytes to validate.
/// * `attestation` - Concatenated 65-byte signature(s) of the message.
///
/// # Returns
///
/// A [`ValidatedMessage`] containing the parsed and validated message fields.
///
/// # Errors
///
/// * [`MessageTransmitterError::InvalidMessageFormat`] – Message is malformed or too short.
/// * [`MessageTransmitterError::InvalidDestinationDomain`] – Destination domain does not match local domain.
/// * [`MessageTransmitterError::InvalidDestinationCaller`] – Caller is not the authorized destination caller.
/// * [`MessageTransmitterError::InvalidMessageVersion`] – Message version does not match.
/// * [`MessageTransmitterError::NonceAlreadyUsed`] – Nonce has already been used.
pub fn validate_received_message(
    e: &Env,
    caller: &Address,
    message: &Bytes,
    attestation: &Bytes,
) -> ValidatedMessage {
    // Verify attestation signatures
    attestable::verify_attestation_signatures(e, message, attestation);

    // Validate message format
    MessageV2::validate_format(message)
        .unwrap_or_else(|_| panic_with_error!(e, MessageTransmitterError::InvalidMessageFormat));

    // Validate destination domain matches local domain
    let destination_domain = MessageV2::get_destination_domain(message)
        .unwrap_or_else(|_| panic_with_error!(e, MessageTransmitterError::InvalidMessageFormat));
    let local_domain = get_local_domain(e);
    if destination_domain != local_domain {
        panic_with_error!(e, MessageTransmitterError::InvalidDestinationDomain);
    }

    // Validate destination caller (if not zero, must match caller)
    let destination_caller = MessageV2::get_destination_caller(message)
        .unwrap_or_else(|_| panic_with_error!(e, MessageTransmitterError::InvalidMessageFormat));
    if !is_zero_bytes(&destination_caller) {
        let caller_bytes32 = address_to_bytes32(caller).unwrap_or_else(|| {
            panic_with_error!(e, MessageTransmitterError::AddressTypeNotRecognized)
        });
        if destination_caller != caller_bytes32 {
            panic_with_error!(e, MessageTransmitterError::InvalidDestinationCaller);
        }
    }

    // Validate version
    let message_version = MessageV2::get_version(message)
        .unwrap_or_else(|_| panic_with_error!(e, MessageTransmitterError::InvalidMessageFormat));
    let version = get_version(e);
    if message_version != version {
        panic_with_error!(e, MessageTransmitterError::InvalidMessageVersion);
    }

    // Validate nonce is not used
    let nonce = MessageV2::get_nonce(message)
        .unwrap_or_else(|_| panic_with_error!(e, MessageTransmitterError::InvalidMessageFormat));
    if is_nonce_used(e, &nonce) {
        panic_with_error!(e, MessageTransmitterError::NonceAlreadyUsed);
    }

    // Parse remaining fields
    let source_domain = MessageV2::get_source_domain(message)
        .unwrap_or_else(|_| panic_with_error!(e, MessageTransmitterError::InvalidMessageFormat));
    let sender = MessageV2::get_sender(message)
        .unwrap_or_else(|_| panic_with_error!(e, MessageTransmitterError::InvalidMessageFormat));
    let recipient = MessageV2::get_recipient(message)
        .unwrap_or_else(|_| panic_with_error!(e, MessageTransmitterError::InvalidMessageFormat));
    let finality_threshold_executed = MessageV2::get_finality_threshold_executed(message)
        .unwrap_or_else(|_| panic_with_error!(e, MessageTransmitterError::InvalidMessageFormat));
    let message_body = MessageV2::get_message_body(message);

    ValidatedMessage {
        nonce,
        source_domain,
        sender,
        recipient,
        finality_threshold_executed,
        message_body,
    }
}
