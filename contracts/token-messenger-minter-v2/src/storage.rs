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
use soroban_sdk::{contracttype, panic_with_error, Address, Env};

use crate::TokenMessengerMinterError;

/// Minimum finality threshold required for unfinalized messages.
/// Messages with finality_threshold_executed below this value will be rejected.
pub const TOKEN_MESSENGER_MIN_FINALITY_THRESHOLD: u32 = 500;

/// Storage keys for the TokenMessengerMinter contract.
#[contracttype]
pub enum TokenMessengerMinterStorageKey {
    /// The address of the local MessageTransmitter contract
    LocalMessageTransmitter,
    /// The version of the burn message format
    MessageBodyVersion,
}

/// Returns the local message transmitter address.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Returns
///
/// The address of the local MessageTransmitter contract.
///
/// # Errors
///
/// * [`TokenMessengerMinterError::LocalMessageTransmitterNotSet`] – If the local
///   message transmitter has not been set.
pub fn get_local_message_transmitter(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&TokenMessengerMinterStorageKey::LocalMessageTransmitter)
        .unwrap_or_else(|| {
            panic_with_error!(e, TokenMessengerMinterError::LocalMessageTransmitterNotSet)
        })
}

/// Returns the message body version.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Returns
///
/// The message body version.
///
/// # Errors
///
/// * [`TokenMessengerMinterError::MessageBodyVersionNotSet`] – If the message body
///   version has not been set.
pub fn get_message_body_version(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&TokenMessengerMinterStorageKey::MessageBodyVersion)
        .unwrap_or_else(|| {
            panic_with_error!(e, TokenMessengerMinterError::MessageBodyVersionNotSet)
        })
}
