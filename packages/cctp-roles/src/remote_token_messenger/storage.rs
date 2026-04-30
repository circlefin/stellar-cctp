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
use soroban_sdk::{contracttype, panic_with_error, BytesN, Env};
use stellar_macros::only_owner;
use stellar_utils::is_zero_bytes;
use stellar_utils::storage::ttl::{
    get_and_extend_persistent_ttl, DEFAULT_EXTEND_AMOUNT, DEFAULT_TTL_THRESHOLD,
};

use super::{
    emit_remote_token_messenger_added, emit_remote_token_messenger_removed,
    RemoteTokenMessengerError,
};

#[contracttype]
pub enum RemoteTokenMessengerStorageKey {
    RemoteTokenMessenger(u32),
}

/// Adds a remote TokenMessenger for a specific domain.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `domain` - The identifier of the remote domain.
/// * `token_messenger` - The 32-byte address of the TokenMessenger on the remote domain.
///
/// # Errors
///
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the contract owner fails.
/// * [`RemoteTokenMessengerError::TokenMessengerAlreadySet`] – If a TokenMessenger is already
///   set for the domain.
/// * [`RemoteTokenMessengerError::ZeroAddress`] – If the provided token_messenger is zero.
///
/// # Events
///
/// * topics - `["remote_token_messenger_added", domain: u32, token_messenger: BytesN<32>]`
/// * data - `[]`
#[only_owner]
pub fn add_remote_token_messenger(e: &Env, domain: u32, token_messenger: &BytesN<32>) {
    add_remote_token_messenger_unchecked(e, domain, token_messenger);
}

/// Adds a remote TokenMessenger for a specific domain without authorization checks.
///
/// This function is intended for use during contract initialization.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `domain` - The identifier of the remote domain.
/// * `token_messenger` - The 32-byte address of the TokenMessenger on the remote domain.
///
/// # Errors
///
/// * [`RemoteTokenMessengerError::TokenMessengerAlreadySet`] – If a TokenMessenger is already
///   set for the domain.
/// * [`RemoteTokenMessengerError::ZeroAddress`] – If the provided token_messenger is zero.
///
/// # Events
///
/// * topics - `["remote_token_messenger_added", domain: u32, token_messenger: BytesN<32>]`
/// * data - `[]`
pub fn add_remote_token_messenger_unchecked(e: &Env, domain: u32, token_messenger: &BytesN<32>) {
    // Validate token_messenger is not zero
    if is_zero_bytes(token_messenger) {
        panic_with_error!(e, RemoteTokenMessengerError::ZeroAddress);
    }

    let key = RemoteTokenMessengerStorageKey::RemoteTokenMessenger(domain);

    // Check if already set
    if e.storage().persistent().has(&key) {
        panic_with_error!(e, RemoteTokenMessengerError::TokenMessengerAlreadySet);
    }

    e.storage().persistent().set(&key, token_messenger);

    emit_remote_token_messenger_added(e, domain, token_messenger);
}

/// Removes the remote TokenMessenger for a specific domain.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `domain` - The identifier of the remote domain.
///
/// # Errors
///
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the contract owner fails.
/// * [`RemoteTokenMessengerError::NoTokenMessengerSet`] – If no TokenMessenger is set for
///   the domain.
///
/// # Events
///
/// * topics - `["remote_token_messenger_removed", domain: u32, token_messenger: BytesN<32>]`
/// * data - `[]`
#[only_owner]
pub fn remove_remote_token_messenger(e: &Env, domain: u32) {
    let key = RemoteTokenMessengerStorageKey::RemoteTokenMessenger(domain);

    // Get the existing token messenger to emit in the event
    let token_messenger: BytesN<32> =
        e.storage().persistent().get(&key).unwrap_or_else(|| {
            panic_with_error!(e, RemoteTokenMessengerError::NoTokenMessengerSet)
        });

    e.storage().persistent().remove(&key);

    emit_remote_token_messenger_removed(e, domain, &token_messenger);
}

/// Returns the remote TokenMessenger address for a specific domain if set.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `domain` - The identifier of the remote domain.
///
/// # Returns
///
/// The 32-byte TokenMessenger address if set, otherwise `None`.
pub fn get_remote_token_messenger(e: &Env, domain: u32) -> Option<BytesN<32>> {
    let key = RemoteTokenMessengerStorageKey::RemoteTokenMessenger(domain);
    get_and_extend_persistent_ttl(e, &key, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT)
}

/// Checks if the provided TokenMessenger is registered for the given domain.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `domain` - The identifier of the remote domain.
/// * `token_messenger` - The 32-byte address to check.
///
/// # Returns
///
/// `true` if the TokenMessenger is registered for the domain, `false` otherwise.
/// Returns `false` if the provided token_messenger is zero.
fn is_remote_token_messenger(e: &Env, domain: u32, token_messenger: &BytesN<32>) -> bool {
    if is_zero_bytes(token_messenger) {
        return false;
    }

    let key = RemoteTokenMessengerStorageKey::RemoteTokenMessenger(domain);
    match get_and_extend_persistent_ttl::<_, BytesN<32>>(
        e,
        &key,
        DEFAULT_TTL_THRESHOLD,
        DEFAULT_EXTEND_AMOUNT,
    ) {
        Some(stored) => stored == *token_messenger,
        None => false,
    }
}

/// Requires that the provided TokenMessenger is registered for the given domain.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `domain` - The identifier of the remote domain.
/// * `token_messenger` - The 32-byte address to validate.
///
/// # Errors
///
/// * [`RemoteTokenMessengerError::RemoteTokenMessengerNotRegistered`] – If the TokenMessenger
///   is not registered for the domain or is the zero address.
pub fn require_remote_token_messenger(e: &Env, domain: u32, token_messenger: &BytesN<32>) {
    if !is_remote_token_messenger(e, domain, token_messenger) {
        panic_with_error!(
            e,
            RemoteTokenMessengerError::RemoteTokenMessengerNotRegistered
        );
    }
}
