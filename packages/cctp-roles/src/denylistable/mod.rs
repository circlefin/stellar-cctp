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
//! # Denylistable Contract Module
//!
//! This module provides functionality for managing a "denylister" role and maintaining
//! a denylist of addresses that are blocked from calling depositForBurn function.
//!
//! ## Usage
//!
//! Contracts implementing the `Denylistable` trait use `simple_role` directly:
//!
//! ```ignore
//! use cctp_roles::{denylistable, simple_role};
//!
//! impl Denylistable for MyContract {
//!     fn get_denylister(e: &Env) -> Option<Address> {
//!         simple_role::try_get_role(e, denylistable::DENYLISTER)
//!     }
//!
//!     fn update_denylister(e: &Env, denylister: Address) {
//!         simple_role::set_role_and_emit_with_previous(
//!             e,
//!             denylistable::DENYLISTER,
//!             &denylister,
//!             denylistable::emit_denylister_changed,
//!         );
//!     }
//!
//!     // ... other trait methods
//! }
//! ```

mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contracterror, contractevent, Address, Env};

pub use storage::{denylist, is_denylisted, require_not_denylisted, un_denylist};

/// Role identifier for the denylister.
pub const DENYLISTER: &str = "denylister";

/// A trait for contracts that implement the Denylistable role.
///
/// This trait provides a standardized interface for managing denylisted addresses
/// and the denylister role that controls the denylist.
pub trait Denylistable {
    /// Returns the current denylister address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// * `Some(Address)` if the denylister is set, `None` otherwise.
    fn get_denylister(e: &Env) -> Option<Address>;

    /// Updates the denylister address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `denylister` - The address that will be authorized to manage the denylist.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   contract owner fails.
    ///
    /// # Events
    ///
    /// * topics - `["denylister_changed", new_denylister: Address]`
    fn update_denylister(e: &Env, denylister: Address);

    /// Adds an address to the denylist.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `account` - The address to add to the denylist.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   denylister fails.
    /// * [`RoleError::RoleNotSet`] – Denylister is not set.
    ///
    /// # Events
    ///
    /// * topics - `["denylisted", account: Address]`
    fn denylist(e: &Env, account: Address);

    /// Removes an address from the denylist.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `account` - The address to remove from the denylist.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   denylister fails.
    /// * [`RoleError::RoleNotSet`] – Denylister is not set.
    ///
    /// # Events
    ///
    /// * topics - `["un_denylisted", account: Address]`
    fn un_denylist(e: &Env, account: Address);

    /// Checks if an address is on the denylist.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `account` - The address to check.
    ///
    /// # Returns
    ///
    /// * `true` if the account is denylisted, `false` otherwise.
    fn is_denylisted(e: &Env, account: Address) -> bool;
}

// ################## EVENTS ##################

/// Emitted when an address is added to the denylist.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Denylisted {
    #[topic]
    pub account: Address,
}

/// Emitted when an address is removed from the denylist.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnDenylisted {
    #[topic]
    pub account: Address,
}

/// Emitted when the denylister is updated.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DenylisterChanged {
    #[topic]
    pub old_denylister: Option<Address>,
    #[topic]
    pub new_denylister: Address,
}

/// Emits a Denylisted event
///
/// # Arguments
/// * `e` - The contract environment
/// * `account` - The address that was added to the denylist
pub fn emit_denylisted(e: &Env, account: &Address) {
    let event = Denylisted {
        account: account.clone(),
    };
    event.publish(e);
}

/// Emits an UnDenylisted event
///
/// # Arguments
/// * `e` - The contract environment
/// * `account` - The address that was removed from the denylist
pub fn emit_un_denylisted(e: &Env, account: &Address) {
    let event = UnDenylisted {
        account: account.clone(),
    };
    event.publish(e);
}

/// Emits a DenylisterChanged event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `old_denylister` - The previous denylister address (None if not previously set).
/// * `new_denylister` - The new denylister address.
pub fn emit_denylister_changed(e: &Env, old_denylister: Option<Address>, new_denylister: &Address) {
    let event = DenylisterChanged {
        old_denylister,
        new_denylister: new_denylister.clone(),
    };
    event.publish(e);
}

// ################## ERRORS ##################

/// Error codes for denylist operations
#[contracterror]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DenylistError {
    /// The account is on the denylist
    AccountDenylisted = 6100,
}
