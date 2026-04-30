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
//! Fee Recipient Module.
//!
//! This module manages the fee recipient address where minting fees are sent.

use soroban_sdk::{contractevent, Address, Env};

#[cfg(test)]
mod test;

/// Role identifier for the fee recipient.
pub const FEE_RECIPIENT: &str = "fee_recipient";

/// Trait describing the fee recipient role.
pub trait FeeRecipient {
    /// Returns the fee recipient address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// * `Some(Address)` – The fee recipient address if set.
    /// * `None` – If the fee recipient has not been set.
    fn get_fee_recipient(e: &Env) -> Option<Address>;

    /// Sets the fee recipient address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `new_fee_recipient` - The address that will receive minting fees.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   contract owner fails.
    ///
    /// # Events
    ///
    /// * topics - `["fee_recipient_set"]`
    /// * data - `[fee_recipient: Address]`
    fn set_fee_recipient(e: &Env, new_fee_recipient: Address);
}

// ################## EVENTS ##################

/// Emitted when the fee recipient address is updated.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeRecipientSet {
    pub fee_recipient: Address,
}

/// Emits a `fee_recipient_set` event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `new_fee_recipient` - The new fee recipient address.
pub fn emit_fee_recipient_set(e: &Env, new_fee_recipient: &Address) {
    FeeRecipientSet {
        fee_recipient: new_fee_recipient.clone(),
    }
    .publish(e);
}
