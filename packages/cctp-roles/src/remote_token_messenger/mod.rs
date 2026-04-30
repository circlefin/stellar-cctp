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
//! # Remote Token Messenger Module
//!
//! This module provides functionality to manage a registry of authorized TokenMessenger
//! contracts on remote domains. It allows the contract owner to add and remove remote
//! TokenMessenger addresses for specific domains.
//!
//! The `RemoteTokenMessenger` trait exposes methods for:
//! - Adding a remote TokenMessenger for a domain
//! - Removing a remote TokenMessenger for a domain
//! - Getting the remote TokenMessenger for a domain
//! - Checking if a TokenMessenger is registered for a domain
//! - Requiring a valid remote TokenMessenger (panics if invalid)
//!
//! Access control is enforced through the contract owner role, which has exclusive
//! rights to add and remove remote TokenMessenger addresses.

use soroban_sdk::{contracterror, contractevent, BytesN, Env};

mod storage;
#[cfg(test)]
mod test;

pub use storage::{
    add_remote_token_messenger, add_remote_token_messenger_unchecked, get_remote_token_messenger,
    remove_remote_token_messenger, require_remote_token_messenger,
};

/// A trait for managing remote TokenMessenger addresses.
///
/// Provides functions to add, remove, and query remote TokenMessenger
/// addresses for cross-chain messaging operations.
pub trait RemoteTokenMessenger {
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
    fn add_remote_token_messenger(e: &Env, domain: u32, token_messenger: BytesN<32>);

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
    fn remove_remote_token_messenger(e: &Env, domain: u32);

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
    fn get_remote_token_messenger(e: &Env, domain: u32) -> Option<BytesN<32>>;
}

// ################## ERRORS ##################

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RemoteTokenMessengerError {
    /// If a TokenMessenger is already set for the domain.
    TokenMessengerAlreadySet = 6400,
    /// If no TokenMessenger is set for the domain.
    NoTokenMessengerSet = 6401,
    /// If the provided TokenMessenger address is zero.
    ZeroAddress = 6402,
    /// If the remote TokenMessenger is invalid
    RemoteTokenMessengerNotRegistered = 6403,
}

// ################## EVENTS ##################

/// Emitted when a remote TokenMessenger is added.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteTokenMessengerAdded {
    pub domain: u32,
    pub token_messenger: BytesN<32>,
}

/// Emitted when a remote TokenMessenger is removed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteTokenMessengerRemoved {
    pub domain: u32,
    pub token_messenger: BytesN<32>,
}

/// Emits an event when a remote TokenMessenger is added.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `domain` - The identifier of the remote domain.
/// * `token_messenger` - The 32-byte address of the TokenMessenger.
pub fn emit_remote_token_messenger_added(e: &Env, domain: u32, token_messenger: &BytesN<32>) {
    RemoteTokenMessengerAdded {
        domain,
        token_messenger: token_messenger.clone(),
    }
    .publish(e);
}

/// Emits an event when a remote TokenMessenger is removed.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `domain` - The identifier of the remote domain.
/// * `token_messenger` - The 32-byte address of the TokenMessenger that was removed.
pub fn emit_remote_token_messenger_removed(e: &Env, domain: u32, token_messenger: &BytesN<32>) {
    RemoteTokenMessengerRemoved {
        domain,
        token_messenger: token_messenger.clone(),
    }
    .publish(e);
}
