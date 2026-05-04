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
//! Minimum Fee Controller Module.
//!
//! This module manages per-token minimum fees and the controller role that can
//! update those fees.

use soroban_sdk::{contracterror, contractevent, Address, Env};

mod storage;
#[cfg(test)]
mod test;

pub use storage::{get_min_fee, get_min_fee_amount, set_min_fee};

/// Role identifier for the minimum fee controller.
pub const MIN_FEE_CONTROLLER: &str = "min_fee_controller";

/// Trait describing the minimum fee controller role.
pub trait MinFeeController {
    /// Returns the minimum fee controller address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// * `Some(Address)` – The minimum fee controller address if set.
    /// * `None` – If the minimum fee controller has not been set.
    fn get_min_fee_controller(e: &Env) -> Option<Address>;

    /// Sets the minimum fee controller address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `new_min_fee_controller` - The address that will be authorized to set
    ///   minimum fees.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   contract owner fails.
    ///
    /// # Events
    ///
    /// * topics - `["min_fee_controller_set", new_min_fee_controller: Address]`
    /// * data - `[]`
    fn set_min_fee_controller(e: &Env, new_min_fee_controller: Address);

    /// Sets the minimum fee, in 1/MIN_FEE_MULTIPLIER units, for a burn token.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `burn_token` - The burn token (local token) whose minimum fee is being
    ///   configured.
    /// * `min_fee` - The minimum fee in fractional units (must be non-negative
    ///   and less than [`MIN_FEE_MULTIPLIER`]).
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   minimum fee controller fails.
    /// * [`MinFeeControllerError::MinFeeControllerNotSet`] – If the controller
    ///   is not set.
    /// * [`MinFeeControllerError::MinFeeNegative`] – If `min_fee` is negative.
    /// * [`MinFeeControllerError::MinFeeTooHigh`] – If `min_fee` is greater
    ///   than or equal to [`MIN_FEE_MULTIPLIER`].
    ///
    /// # Events
    ///
    /// * topics - `["min_fee_set", burn_token: Address]`
    /// * data - `[min_fee: i128]`
    fn set_min_fee(e: &Env, burn_token: Address, min_fee: i128);

    /// Returns the configured minimum fee for a burn token, or zero if not set.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `burn_token` - The burn token (local token) to query.
    fn get_min_fee(e: &Env, burn_token: Address) -> i128;

    /// Returns the minimum fee amount for a given burn token and amount.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `burn_token` - The burn token (local token) to query.
    /// * `amount` - The transfer amount.
    ///
    /// # Errors
    ///
    /// * [`MinFeeControllerError::AmountTooLow`] – If `amount <= 1` when a
    ///   non-zero min fee is configured.
    /// * [`MinFeeControllerError::MinFeeComputationOverflow`] – If the fee
    ///   computation overflows `i128`.
    fn get_min_fee_amount(e: &Env, burn_token: Address, amount: i128) -> i128;
}

// ################## CONSTANTS ##################

/// Minimum fee multiplier to match the EVM implementation (1 / 10_000_000 units).
pub const MIN_FEE_MULTIPLIER: i128 = 10_000_000;

// ################## ERRORS ##################

/// Errors for the minimum fee controller module.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MinFeeControllerError {
    /// The minimum fee controller has not been set.
    MinFeeControllerNotSet = 6200,
    /// The provided minimum fee is greater than or equal to MIN_FEE_MULTIPLIER.
    MinFeeTooHigh = 6201,
    /// The provided amount is too low to compute a minimum fee (must be > 1).
    AmountTooLow = 6202,
    /// The fee computation overflowed i128.
    MinFeeComputationOverflow = 6203,
    /// The provided minimum fee is negative.
    MinFeeNegative = 6204,
}

// ################## EVENTS ##################

/// Emitted when the minimum fee controller role is updated.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinFeeControllerSet {
    #[topic]
    pub new_min_fee_controller: Address,
}

/// Emits a `min_fee_controller_set` event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `new_min_fee_controller` - The new controller address.
pub fn emit_min_fee_controller_set(e: &Env, new_min_fee_controller: &Address) {
    MinFeeControllerSet {
        new_min_fee_controller: new_min_fee_controller.clone(),
    }
    .publish(e);
}

/// Emitted when the minimum fee is updated for a specific burn token.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinFeeSet {
    #[topic]
    pub burn_token: Address,
    pub min_fee: i128,
}

/// Emits a `min_fee_set` event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `burn_token` - The burn token whose fee was updated.
/// * `min_fee` - The new minimum fee value.
pub fn emit_min_fee_set(e: &Env, burn_token: &Address, min_fee: i128) {
    MinFeeSet {
        burn_token: burn_token.clone(),
        min_fee,
    }
    .publish(e);
}
